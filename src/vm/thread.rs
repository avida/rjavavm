pub mod thread {
    use std::ops::{Index, RemAssign};
    use std::rc::Rc;

    use crate::bytes_to_short;
    use crate::loader::java_class::java_class::{ConstantPoolInfo, ConstantPoolTag};
    use crate::vm::byte_code::byte_code::{self, Instruction};

    use crate::vm::class::MethodReference;
    use crate::vm::heap::HeapPtr;
    use crate::vm::method_area::{MethodArea, MethodAreaPtr};
    use crate::vm::reference_manager::ReferenceManagerPtr;
    use crate::vm::stack::stack::StackFramePtr;
    use crate::vm::types::types::Type;
    use crate::vm::{
        class::ClassPtr,
        errors::errors::RunTimeError,
        runtime::Runtime,
        stack::stack::{Stack, StackFrame},
    };

    pub struct Thread {
        pc: usize,
        pub stack: Stack,
        current_frame: Option<StackFramePtr>,
        method_area: MethodAreaPtr,
        reference_manager: ReferenceManagerPtr,
        heap: HeapPtr,
        trace_ops: bool,
    }
    macro_rules! set_current_frame {
        ($self:ident) => {{
            let current_frame = $self
                .stack
                .top_frame()
                .ok_or(RunTimeError::Other("Stack is empty".to_string()))?;
            $self.current_frame = Some(current_frame.clone());
            let class = current_frame.borrow_mut().class.clone();
            let method = current_frame.borrow_mut().method.clone();
            (class, method)
        }};
    }
    macro_rules! pop_current_frame_operand {
        ($frame:expr, $ty:ty) => {{
            let value: $ty = $frame
                .borrow_mut()
                .operand_stack
                .pop()
                .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
            value
        }};
    }
    enum RunResult {
        Invoke(MethodReference),
        Return,
        Jump(u32),
        AReturn(u32),
        DReturn(f64),
    }

    impl Thread {
        fn trace(&self, message: impl AsRef<str>) {
            if self.trace_ops {
                println!("{}", message.as_ref());
            }
        }

        fn current_frame(&self) -> Result<StackFramePtr, RunTimeError> {
            self.current_frame
                .as_ref()
                .cloned()
                .ok_or(RunTimeError::Other("No current frame".to_string()))
        }

        fn finish_current_frame(&mut self) -> Result<bool, RunTimeError> {
            self.stack
                .pop_frame()
                .ok_or(RunTimeError::Other("Stack is empty".to_string()))?;

            if let Some(return_pc) = self.stack.pop_return_address() {
                self.stack.set_pc(return_pc);
            } else {
                self.current_frame = None;
                return Ok(false);
            }

            if let Some(frame) = self.stack.top_frame() {
                self.current_frame = Some(frame);
                Ok(true)
            } else {
                self.current_frame = None;
                Ok(false)
            }
        }

        pub fn run(&mut self) -> Result<(), RunTimeError> {
            let (mut class, mut method) = set_current_frame!(self);
            loop {
                // if program counter has reached or passed the end of the code
                // treat it as method completion (implicit return)
                if self.stack.get_pc() >= method.code.len() {
                    // reached end of code for current method; stop executing but
                    // keep the frame on the stack so callers (tests) can inspect it
                    break;
                }

                let (next_op, args_len) =
                    byte_code::parse_op_at(&method.code, self.stack.get_pc())?;
                match self.run_op(&next_op) {
                    Ok(Some(RunResult::Invoke(method_ref))) => {
                        let ret_pc = self.stack.get_pc();
                        // advance past the invoking instruction and its args when returning
                        self.push_frame_with_return(&method_ref, ret_pc + args_len + 1)?;
                        (class, method) = set_current_frame!(self);
                        continue;
                    }
                    Ok(Some(RunResult::Return)) => {
                        // print number of frames currently on the stack
                        // println!("Stack: {} ", self.stack);
                        if self.finish_current_frame()? {
                            (class, method) = set_current_frame!(self);
                            continue;
                        }
                        break;
                    }
                    Ok(Some(RunResult::Jump(target))) => {
                        self.stack.set_pc(target as usize);
                        continue;
                    }
                    Ok(Some(RunResult::AReturn(reference))) => {
                        if self.finish_current_frame()? {
                            let current_frame = self.current_frame()?;
                            current_frame
                                .borrow_mut()
                                .operand_stack
                                .push(reference as i32);
                            (class, method) = set_current_frame!(self);
                            continue;
                        }
                        break;
                    }
                    Ok(Some(RunResult::DReturn(value))) => {
                        if self.finish_current_frame()? {
                            let current_frame = self.current_frame()?;
                            current_frame.borrow_mut().operand_stack.push(value);
                            (class, method) = set_current_frame!(self);
                            continue;
                        }
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => return Err(e),
                }
                self.stack.increase_pc(args_len + 1);
            }

            Ok(())
        }
        fn push_frame(&mut self, method_ref: &MethodReference) -> Result<(), RunTimeError> {
            // determine parameter types from method descriptor
            let method = method_ref.method();

            let params = MethodArea::parse_params(&method.descriptor);

            // compute slot starts
            let mut starts: Vec<usize> = Vec::new();
            let mut acc = 0usize;
            for p in &params {
                starts.push(acc);
                let slotc = if p == "J" || p == "D" { 2 } else { 1 };
                acc += slotc;
            }

            // create new frame
            let new_frame = crate::vm::stack::stack::StackFrame::new_ptr(method_ref.clone())?;

            // pop parameters from current frame operand stack and set into new frame locals
            let current_frame = self
                .stack
                .top_frame()
                .ok_or(RunTimeError::Other("No current frame".to_string()))?;

            // pop parameters in reverse order
            for i in (0..params.len()).rev() {
                let p = &params[i];
                let start = starts[i];
                let mut nf = new_frame.borrow_mut();
                let mut cf = current_frame.borrow_mut();
                match p.as_str() {
                    "I" | "B" | "S" | "C" | "Z" => {
                        let v: i32 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    "J" => {
                        let v: i64 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    "F" => {
                        let v: f32 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    "D" => {
                        let v: f64 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    _ => {
                        // object/array types - treat as reference slot (stored in one slot)
                        // We don't have a Reference type implemented yet; store raw i32 slot
                        let v: i32 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                }
            }

            // save return address (current pc) and set pc to 0 for new frame
            let ret_pc = self.stack.get_pc();
            self.stack.push_return_address(ret_pc);
            self.stack.set_pc(0);

            // push new frame
            self.stack.push_frame(new_frame.clone());
            // update current frame to the newly pushed frame
            self.current_frame = Some(new_frame.clone());

            Ok(())
        }

        fn push_frame_with_return(
            &mut self,
            method_ref: &MethodReference,
            return_addr: usize,
        ) -> Result<(), RunTimeError> {
            // determine parameter types from method descriptor
            let method = method_ref.method();

            let params = MethodArea::parse_params(&method.descriptor);

            // compute slot starts
            let mut starts: Vec<usize> = Vec::new();
            let mut acc = 0usize;
            for p in &params {
                starts.push(acc);
                let slotc = if p == "J" || p == "D" { 2 } else { 1 };
                acc += slotc;
            }

            // create new frame
            let new_frame = crate::vm::stack::stack::StackFrame::new_ptr(method_ref.clone())?;

            // pop parameters from current frame operand stack and set into new frame locals
            let current_frame = self
                .stack
                .top_frame()
                .ok_or(RunTimeError::Other("No current frame".to_string()))?;

            // pop parameters in reverse order
            for i in (0..params.len()).rev() {
                let p = &params[i];
                let start = starts[i];
                let mut nf = new_frame.borrow_mut();
                let mut cf = current_frame.borrow_mut();
                match p.as_str() {
                    "I" | "B" | "S" | "C" | "Z" => {
                        let v: i32 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    "J" => {
                        let v: i64 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    "F" => {
                        let v: f32 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    "D" => {
                        let v: f64 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                    _ => {
                        let v: i32 = cf
                            .operand_stack
                            .pop()
                            .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                        nf.set_variable_value(start as u16, v)?;
                    }
                }
            }

            // save provided return address and set pc to 0 for new frame
            self.stack.push_return_address(return_addr);
            self.stack.set_pc(0);

            // push new frame
            self.stack.push_frame(new_frame.clone());
            // update current frame to the newly pushed frame
            self.current_frame = Some(new_frame.clone());

            Ok(())
        }

        /// Push a new frame for `method_ref` and allow the caller to initialize
        /// the frame's local variables before execution. This is useful for
        /// tests that need to invoke methods directly with prepared arguments.
        pub fn push_frame_with_setup<F>(
            &mut self,
            method_ref: &MethodReference,
            setup: F,
        ) -> Result<(), RunTimeError>
        where
            F: FnOnce(&mut crate::vm::stack::stack::StackFrame) -> Result<(), RunTimeError>,
        {
            let new_frame = crate::vm::stack::stack::StackFrame::new_ptr(method_ref.clone())?;

            {
                let mut nf = new_frame.borrow_mut();
                setup(&mut nf)?;
            }

            // set pc to 0 for the new frame and push it on the stack
            self.stack.push_frame(new_frame.clone());
            self.current_frame = Some(new_frame.clone());
            self.stack.set_pc(0);

            Ok(())
        }
        fn run_op(&mut self, op: &byte_code::Op) -> Result<Option<RunResult>, RunTimeError> {
            match op.instruction {
                Instruction::Sipush => {
                    let param = bytes_to_short!(op.args);
                    self.current_frame()?.borrow_mut().operand_stack.push(param);
                    self.trace(format!("Executing {op}, {param}"));
                }
                Instruction::Bipush => {
                    let param = op.args[0] as i8;
                    self.current_frame()?.borrow_mut().operand_stack.push(param);
                    self.trace(format!("Executing {op}, {param}"));
                }
                Instruction::Invokestatic => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                    let current_frame = self.current_frame()?;
                    let frame_ref = current_frame.borrow();
                    let const_pool = &frame_ref.class.constant_pool;
                    let index = param as u16;

                    match const_pool.as_ref()[index as usize - 1].tag {
                        ConstantPoolTag::Methodref => {
                            let class = frame_ref.class.as_ref();
                            if let Some(_method) = class.get_method_by_index(index) {
                                // already resolved on class
                            } else {
                                let identifier = class
                                    .resolve_method_ref_to_identifier(index)
                                    .ok_or(RunTimeError::ResolveMethodError(format!(
                                        "Failed to resolve constant pool ref {}",
                                        index
                                    )))?;
                                self.trace(format!("Resolved identifier: {identifier}"));

                                let mut ma = self.method_area.lock().map_err(|_| {
                                    RunTimeError::Other("Method area lock poisoned".to_string())
                                })?;
                                match ma.resolve(&identifier) {
                                    Ok(method_ref) => {
                                        return Ok(Some(RunResult::Invoke(method_ref)));
                                    }
                                    Err(e) => return Err(e),
                                }
                            }

                            self.trace("Tag found");
                        }
                        _ => return Err(RunTimeError::Other("Unexpected tag".to_string())),
                    }
                }
                Instruction::Invokespecial => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                    let current_frame = self.current_frame()?;
                    let frame_ref = current_frame.borrow();
                    let const_pool = &frame_ref.class.constant_pool;
                    let index = param as u16;

                    match const_pool.as_ref()[index as usize - 1].tag {
                        ConstantPoolTag::Methodref => {
                            let class = frame_ref.class.as_ref();
                            if let Some(_method) = class.get_method_by_index(index) {
                                // already resolved on class
                            } else {
                                let identifier = class
                                    .resolve_method_ref_to_identifier(index)
                                    .ok_or(RunTimeError::ResolveMethodError(format!(
                                        "Failed to resolve constant pool ref {}",
                                        index
                                    )))?;
                                self.trace(format!("Resolved identifier: {identifier}"));

                                let mut ma = self.method_area.lock().map_err(|_| {
                                    RunTimeError::Other("Method area lock poisoned".to_string())
                                })?;
                                match ma.resolve(&identifier) {
                                    Ok(method_ref) => {
                                        return Ok(Some(RunResult::Invoke(method_ref)));
                                    }
                                    Err(e) => return Err(e),
                                }
                            }

                            self.trace("Tag found");
                        }
                        _ => return Err(RunTimeError::Other("Unexpected tag".to_string())),
                    }
                }
                Instruction::Invokedynamic => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                }
                Instruction::New => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                    let current_frame = self.current_frame()?;
                    let class_name = current_frame
                        .borrow()
                        .class
                        .get_class_name(param as u16)
                        .ok_or(RunTimeError::Other(
                            "Failed to resolve class for new".to_string(),
                        ))?;

                    let mut rm = self.reference_manager.lock().map_err(|_| {
                        RunTimeError::Other("Reference manager lock poisoned".to_string())
                    })?;
                    let (ref_id, heap_id) = rm.allocate_new();
                    drop(rm);

                    let mut heap = self
                        .heap
                        .lock()
                        .map_err(|_| RunTimeError::Other("Heap lock poisoned".to_string()))?;
                    heap.allocate_object_with_id(heap_id, class_name.clone());
                    drop(heap);

                    current_frame.borrow_mut().operand_stack.push(ref_id as i32);
                }
                Instruction::IfIcmpeq
                | Instruction::IfIcmpne
                | Instruction::IfIcmplt
                | Instruction::IfIcmpge
                | Instruction::IfIcmpgt
                | Instruction::IfIcmple => {
                    // branch offset is a signed short relative to the opcode index
                    let offset = bytes_to_short!(op.args) as isize;
                    let arg_len = op.args.len();

                    let current_frame = self.current_frame()?;
                    let v2: i32 = pop_current_frame_operand!(current_frame, i32);
                    let v1: i32 = pop_current_frame_operand!(current_frame, i32);

                    let take_branch = match op.instruction {
                        Instruction::IfIcmpeq => v1 == v2,
                        Instruction::IfIcmpne => v1 != v2,
                        Instruction::IfIcmplt => v1 < v2,
                        Instruction::IfIcmpge => v1 >= v2,
                        Instruction::IfIcmpgt => v1 > v2,
                        Instruction::IfIcmple => v1 <= v2,
                        _ => false,
                    };
                    self.trace(format!(
                        "Executing {op}, v1={v1} v2={v2} take_branch={take_branch}"
                    ));
                    if take_branch {
                        let target = (op.index as isize) + offset;
                        return Ok(Some(RunResult::Jump(target as u32)));
                    }
                }
                Instruction::IfEq
                | Instruction::IfNe
                | Instruction::IfLt
                | Instruction::IfGe
                | Instruction::IfGt
                | Instruction::IfLe => {
                    let offset = bytes_to_short!(op.args) as isize;
                    let arg_len = op.args.len();

                    let current_frame = self.current_frame()?;
                    let v: i32 = pop_current_frame_operand!(current_frame, i32);

                    let take_branch = match op.instruction {
                        Instruction::IfEq => v == 0,
                        Instruction::IfNe => v != 0,
                        Instruction::IfLt => v < 0,
                        Instruction::IfGe => v >= 0,
                        Instruction::IfGt => v > 0,
                        Instruction::IfLe => v <= 0,
                        _ => false,
                    };
                    self.trace(format!("Executing {op}, v={v} take_branch={take_branch}"));
                    if take_branch {
                        let target = (op.index as isize) + offset;
                        let new_pc = target - ((1 + arg_len) as isize);
                        self.stack.set_pc(new_pc as usize);
                    }
                }
                Instruction::Invokevirtual => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                }
                Instruction::Dup => {
                    self.trace(format!("Executing {op}"));
                    let current_frame = self.current_frame()?;
                    current_frame
                        .borrow_mut()
                        .operand_stack
                        .dup()
                        .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                }
                Instruction::Iadd => {
                    let current_frame = self.current_frame()?;
                    let rhs: i32 = pop_current_frame_operand!(current_frame, i32);
                    let lhs: i32 = pop_current_frame_operand!(current_frame, i32);
                    let result = lhs.wrapping_add(rhs);
                    current_frame.borrow_mut().operand_stack.push(result);
                    self.trace(format!("Executing {op}"));
                }
                Instruction::Iload => {
                    let idx = op.args[0] as u16;
                    let current_frame = self.current_frame()?;
                    let val: i32 = current_frame.borrow().get_variable_value(idx)?;
                    current_frame.borrow_mut().operand_stack.push(val);
                    self.trace(format!("Executing {op} {idx}"));
                }
                Instruction::Dload => {
                    let idx = op.args[0] as u16;
                    let current_frame = self.current_frame()?;
                    let val: f64 = current_frame.borrow().get_variable_value(idx)?;
                    current_frame.borrow_mut().operand_stack.push(val);
                    self.trace(format!("Executing {op} {idx}"));
                }
                Instruction::Iload0
                | Instruction::Iload1
                | Instruction::Iload2
                | Instruction::Iload3 => {
                    let pos = op.instruction - Instruction::Iload0;
                    let idx = pos as u16;
                    let current_frame = self.current_frame()?;
                    let val: i32 = current_frame.borrow().get_variable_value(idx)?;
                    current_frame.borrow_mut().operand_stack.push(val);
                    self.trace(format!("Executing {op} {idx}"));
                }
                // dload_0..3
                Instruction::Dload0
                | Instruction::Dload1
                | Instruction::Dload2
                | Instruction::Dload3 => {
                    let pos = op.instruction - Instruction::Dload0;
                    let idx = pos as u16;
                    self.trace(format!("Executing {op} {idx}"));
                    let current_frame = self.current_frame()?;
                    let val: f64 = current_frame.borrow().get_variable_value(idx)?;
                    current_frame.borrow_mut().operand_stack.push(val);
                }
                Instruction::Aload => {
                    let idx = op.args[0] as u16;
                    self.trace(format!("Executing {op} {idx}"));
                    let current_frame = self.current_frame()?;
                    // load reference slot (stored as i32)
                    let val: i32 = current_frame.borrow().get_variable_value(idx)?;
                    current_frame.borrow_mut().operand_stack.push(val);
                }
                Instruction::Aaload => {
                    self.trace(format!("Executing {op}"));
                }
                Instruction::Pop => {
                    self.trace(format!("Executing {op}"));
                }
                Instruction::Return => {
                    self.trace(format!("Executing {op}"));
                    return Ok(Some(RunResult::Return));
                }
                Instruction::Areturn => {
                    let current_frame = self.current_frame()?;
                    let reference: i32 = pop_current_frame_operand!(current_frame, i32);
                    self.trace(format!("Executing {op}, ref={reference}"));
                    return Ok(Some(RunResult::AReturn(reference as u32)));
                }
                Instruction::Astore => {
                    let idx = op.args[0] as u16;
                    self.trace(format!("Executing {op} {idx}"));
                }
                Instruction::Aastore => {
                    self.trace(format!("Executing {op}"));
                }
                Instruction::Astore => {
                    let idx = op.args[0] as u16;
                    self.trace(format!("Executing {op} {idx}"));
                    let current_frame = self.current_frame()?;
                    let val: i32 = current_frame
                        .borrow_mut()
                        .operand_stack
                        .pop()
                        .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                    current_frame.borrow_mut().set_variable_value(idx, val)?;
                }
                Instruction::Astore0
                | Instruction::Astore1
                | Instruction::Astore2
                | Instruction::Astore3 => {
                    let pos = op.instruction - Instruction::Astore0;
                    let idx = pos as u16;
                    self.trace(format!("Executing {op} {idx}"));
                    let current_frame = self.current_frame()?;
                    let val: i32 = current_frame
                        .borrow_mut()
                        .operand_stack
                        .pop()
                        .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                    current_frame.borrow_mut().set_variable_value(idx, val)?;
                }
                Instruction::Iconst0
                | Instruction::Iconst1
                | Instruction::Iconst2
                | Instruction::Iconst3 => {
                    let pos = op.instruction - Instruction::Iconst0;
                    self.trace(format!("Executing {op} {pos}"));
                }
                Instruction::Aload0
                | Instruction::Aload1
                | Instruction::Aload2
                | Instruction::Aload3 => {
                    let pos = op.instruction - Instruction::Aload0;
                    let idx = pos as u16;
                    self.trace(format!("Executing {op} {idx}"));
                    let current_frame = self.current_frame()?;
                    let val: i32 = current_frame.borrow().get_variable_value(idx)?;
                    current_frame.borrow_mut().operand_stack.push(val);
                }
                Instruction::Getstatic => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                    let current_frame = self.current_frame()?;
                    let frame_ref = current_frame.borrow();
                    let const_pool = &frame_ref.class.constant_pool;
                    let index = param as u16;

                    match const_pool.as_ref()[index as usize - 1].tag {
                        ConstantPoolTag::Fieldref => {
                            // resolve identifier from constant pool
                            let identifier = frame_ref
                                .class
                                .resolve_field_ref_to_identifier(index)
                                .ok_or(RunTimeError::ResolveMethodError(format!(
                                    "Failed to resolve constant pool ref {}",
                                    index
                                )))?;
                            self.trace(format!("Field ref identifier: {identifier}"));

                            // ensure class entries are registered in MethodArea
                            let class_name = crate::class_name_from_identifier!(identifier)
                                .ok_or(RunTimeError::Other(format!(
                                    "Invalid field identifier {}",
                                    identifier
                                )))?
                                .to_string();

                            let mut ma = self.method_area.lock().map_err(|_| {
                                RunTimeError::Other("Method area lock poisoned".to_string())
                            })?;
                            let class_ptr = ma.get_or_load_class(&class_name)?;
                            // populate resolved fields for that class
                            if ma.get_resolved_field(&identifier).is_none() {
                                ma.insert_resolved_for_class(&class_name, &class_ptr);
                            }

                            // try to read a static value from the class static storage
                            if let Some(val) = class_ptr.get_static_by_identifier(&identifier) {
                                // push the stored value onto the operand stack
                                drop(frame_ref);
                                let mut cf = current_frame.borrow_mut();
                                match val {
                                    Type::Int(i) => cf.operand_stack.push(i as i32),
                                    Type::Long(l) => cf.operand_stack.push(l as i64),
                                    Type::Float(f) => cf.operand_stack.push(f as f32),
                                    Type::Double(d) => cf.operand_stack.push(d as f64),
                                    Type::Reference(r) => cf.operand_stack.push(r as i32),
                                    Type::Null => cf.operand_stack.push(0i32),
                                    Type::Byte(b) => cf.operand_stack.push(b as i32),
                                    Type::Short(s) => cf.operand_stack.push(s as i32),
                                    Type::Char(c) => cf.operand_stack.push(c as i32),
                                    Type::Bool(bv) => {
                                        cf.operand_stack.push(if bv { 1i32 } else { 0i32 })
                                    }
                                    _ => {
                                        // fallback: allocate symbolic reference like before
                                        let mut rm =
                                            self.reference_manager.lock().map_err(|_| {
                                                RunTimeError::Other(
                                                    "Reference manager lock poisoned".to_string(),
                                                )
                                            })?;
                                        let ref_u32 = rm.allocate_symbolic(identifier.clone());
                                        drop(rm);
                                        cf.operand_stack.push(ref_u32 as i32);
                                    }
                                }
                            } else {
                                // no static value set yet; fall back allocate symbolic reference
                                if let Some(_field_ref) = ma.get_resolved_field(&identifier) {
                                    let mut rm = self.reference_manager.lock().map_err(|_| {
                                        RunTimeError::Other(
                                            "Reference manager lock poisoned".to_string(),
                                        )
                                    })?;
                                    let ref_u32 = rm.allocate_symbolic(identifier.clone());
                                    drop(rm);
                                    drop(frame_ref);
                                    let mut cf = current_frame.borrow_mut();
                                    cf.operand_stack.push(ref_u32 as i32);
                                } else {
                                    return Err(RunTimeError::Other(format!(
                                        "Field {} not found",
                                        identifier
                                    )));
                                }
                            }
                        }
                        _ => return Err(RunTimeError::Other("Unexpected tag".to_string())),
                    }
                }
                Instruction::Putstatic => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                    let current_frame = self.current_frame()?;
                    let frame_ref = current_frame.borrow();
                    let const_pool = &frame_ref.class.constant_pool;
                    let index = param as u16;

                    match const_pool.as_ref()[index as usize - 1].tag {
                        ConstantPoolTag::Fieldref => {
                            let identifier = frame_ref
                                .class
                                .resolve_field_ref_to_identifier(index)
                                .ok_or(RunTimeError::ResolveMethodError(format!(
                                    "Failed to resolve constant pool ref {}",
                                    index
                                )))?;
                            self.trace(format!("Putstatic identifier: {identifier}"));

                            let class_name = crate::class_name_from_identifier!(identifier)
                                .ok_or(RunTimeError::Other(format!(
                                    "Invalid field identifier {}",
                                    identifier
                                )))?
                                .to_string();

                            let mut ma = self.method_area.lock().map_err(|_| {
                                RunTimeError::Other("Method area lock poisoned".to_string())
                            })?;
                            let class_ptr = ma.get_or_load_class(&class_name)?;
                            if ma.get_resolved_field(&identifier).is_none() {
                                ma.insert_resolved_for_class(&class_name, &class_ptr);
                            }

                            if let Some(field_ref) = ma.get_resolved_field(&identifier) {
                                // Pop value from operand stack according to descriptor
                                drop(frame_ref);
                                let mut cf = current_frame.borrow_mut();
                                let desc = field_ref.field().descriptor.clone();
                                let ch = desc.chars().next().unwrap_or('V');
                                match ch {
                                    'I' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr.put_static_by_identifier(
                                            &identifier,
                                            Type::Int(v as u32),
                                        );
                                    }
                                    'J' => {
                                        let v: i64 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr.put_static_by_identifier(
                                            &identifier,
                                            Type::Long(v as u64),
                                        );
                                    }
                                    'F' => {
                                        let v: f32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr
                                            .put_static_by_identifier(&identifier, Type::Float(v));
                                    }
                                    'D' => {
                                        let v: f64 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr
                                            .put_static_by_identifier(&identifier, Type::Double(v));
                                    }
                                    'L' | '[' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr.put_static_by_identifier(
                                            &identifier,
                                            Type::Reference(v as u32),
                                        );
                                    }
                                    'S' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr.put_static_by_identifier(
                                            &identifier,
                                            Type::Short(v as u16),
                                        );
                                    }
                                    'B' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr.put_static_by_identifier(
                                            &identifier,
                                            Type::Byte(v as u8),
                                        );
                                    }
                                    'C' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr.put_static_by_identifier(
                                            &identifier,
                                            Type::Char(v as u16),
                                        );
                                    }
                                    'Z' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        class_ptr.put_static_by_identifier(
                                            &identifier,
                                            Type::Bool(v != 0),
                                        );
                                    }
                                    _ => {
                                        return Err(RunTimeError::Other(
                                            "Unsupported field type for putstatic".to_string(),
                                        ));
                                    }
                                }
                            } else {
                                return Err(RunTimeError::Other(format!(
                                    "Field {} not found",
                                    identifier
                                )));
                            }
                        }
                        _ => return Err(RunTimeError::Other("Unexpected tag".to_string())),
                    }
                }
                Instruction::Putfield => {
                    let param = bytes_to_short!(op.args);
                    self.trace(format!("Executing {op}, {param}"));
                    let current_frame = self.current_frame()?;
                    let frame_ref = current_frame.borrow();
                    let const_pool = &frame_ref.class.constant_pool;
                    let index = param as u16;

                    match const_pool.as_ref()[index as usize - 1].tag {
                        ConstantPoolTag::Fieldref => {
                            let identifier = frame_ref
                                .class
                                .resolve_field_ref_to_identifier(index)
                                .ok_or(RunTimeError::ResolveMethodError(format!(
                                    "Failed to resolve constant pool ref {}",
                                    index
                                )))?;
                            self.trace(format!("Putfield identifier: {identifier}"));

                            // class name part
                            let class_name = crate::class_name_from_identifier!(identifier)
                                .ok_or(RunTimeError::Other(format!(
                                    "Invalid field identifier {}",
                                    identifier
                                )))?
                                .to_string();

                            let mut ma = self.method_area.lock().map_err(|_| {
                                RunTimeError::Other("Method area lock poisoned".to_string())
                            })?;
                            let class_ptr = ma.get_or_load_class(&class_name)?;
                            if ma.get_resolved_field(&identifier).is_none() {
                                ma.insert_resolved_for_class(&class_name, &class_ptr);
                            }

                            if let Some(field_ref) = ma.get_resolved_field(&identifier) {
                                // pop value then object reference
                                drop(frame_ref);
                                let mut cf = current_frame.borrow_mut();
                                let desc = field_ref.field().descriptor.clone();
                                let ch = desc.chars().next().unwrap_or('V');
                                // value first
                                let value_type = match ch {
                                    'I' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Int(v as u32)
                                    }
                                    'J' => {
                                        let v: i64 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Long(v as u64)
                                    }
                                    'F' => {
                                        let v: f32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Float(v)
                                    }
                                    'D' => {
                                        let v: f64 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Double(v)
                                    }
                                    'L' | '[' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Reference(v as u32)
                                    }
                                    'S' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Short(v as u16)
                                    }
                                    'B' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Byte(v as u8)
                                    }
                                    'C' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Char(v as u16)
                                    }
                                    'Z' => {
                                        let v: i32 = cf
                                            .operand_stack
                                            .pop()
                                            .ok_or(RunTimeError::StackUnderflow)?;
                                        Type::Bool(v != 0)
                                    }
                                    _ => {
                                        return Err(RunTimeError::Other(
                                            "Unsupported field type for putfield".to_string(),
                                        ));
                                    }
                                };

                                // now object reference
                                let obj_ref: i32 =
                                    cf.operand_stack.pop().ok_or(RunTimeError::StackUnderflow)?;
                                if obj_ref == 0 {
                                    return Err(RunTimeError::Other(
                                        "NullPointerException".to_string(),
                                    ));
                                }

                                let mut rm = self.reference_manager.lock().map_err(|_| {
                                    RunTimeError::Other(
                                        "Reference manager lock poisoned".to_string(),
                                    )
                                })?;
                                let heap_id = rm.resolve_heap(obj_ref as u32).ok_or(
                                    RunTimeError::Other(format!("Invalid object reference: {}", obj_ref)),
                                )?;
                                drop(rm);

                                let mut heap = self.heap.lock().map_err(|_| {
                                    RunTimeError::Other("Heap lock poisoned".to_string())
                                })?;
                                if let Some(obj) = heap.get_object_mut(heap_id) {
                                    obj.set_field(field_ref.field().name.clone(), value_type);
                                } else {
                                    return Err(RunTimeError::Other(
                                        "Heap object not found".to_string(),
                                    ));
                                }
                            } else {
                                return Err(RunTimeError::Other(format!(
                                    "Field {} not found",
                                    identifier
                                )));
                            }
                        }
                        _ => return Err(RunTimeError::Other("Unexpected tag".to_string())),
                    }
                }
                _ => {
                    return Err(RunTimeError::Notimplemented(format!(
                        "Instruction {}",
                        op.instruction
                    )));
                }
            }
            if self.trace_ops {
                let stack_size = self
                    .current_frame
                    .as_ref()
                    .map(|cf| cf.borrow().operand_stack.len())
                    .unwrap_or(0);
                // self.trace(format!("Stack size: {stack_size}"));
            }
            Ok(None)
        }
        pub fn invoke(&mut self, method_ref: MethodReference) -> Result<(), RunTimeError> {
            let frame = StackFrame::new_ptr(method_ref)?;
            self.stack.push_frame(frame.clone());
            Ok(())
        }
        pub fn new(
            ma: &MethodAreaPtr,
            rm: &crate::vm::reference_manager::ReferenceManagerPtr,
            heap: &crate::vm::heap::HeapPtr,
            trace_ops: bool,
        ) -> Self {
            Self {
                pc: 0,
                stack: Stack::new(),
                current_frame: None,
                method_area: ma.clone(),
                reference_manager: rm.clone(),
                heap: heap.clone(),
                trace_ops,
            }
        }
    }
}
