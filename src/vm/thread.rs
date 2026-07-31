pub mod thread {
    use std::ops::{Index, RemAssign};
    use std::rc::Rc;

    use crate::bytes_to_short;
    use crate::loader::java_class::java_class::{ConstantPoolInfo, ConstantPoolTag};
    use crate::vm::byte_code::byte_code::{self, Instruction};

    use crate::vm::class::MethodReference;
    use crate::vm::method_area::{MethodArea, MethodAreaPtr};
    use crate::vm::stack::stack::StackFramePtr;
    use crate::vm::{
        class::ClassPtr,
        errors::errors::RunTimeError,
        runtime::Runtime,
        stack::stack::{Stack, StackFrame},
    };

    pub struct Thread {
        pc: usize,
        stack: Stack,
        current_frame: Option<StackFramePtr>,
        method_area: MethodAreaPtr,
        reference_manager: crate::vm::reference_manager::ReferenceManagerPtr,
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
    enum RunResult {
        Invoke(MethodReference),
    }

    impl Thread {
        pub fn run(&mut self) -> Result<(), RunTimeError> {
            println!("Running thread");
            let (mut class, mut method) = set_current_frame!(self);
            loop {
                let (next_op, args_len) =
                    byte_code::parse_op_at(&method.code, self.stack.get_pc())?;
                match self.run_op(&next_op) {
                    Ok(Some(RunResult::Invoke(method_ref))) => {
                        println!("Invoke");
                        self.push_frame(&method_ref)?;
                        (class, method) = set_current_frame!(self);
                        continue;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        return Err(e);
                    }
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
        fn run_op(&mut self, op: &byte_code::Op) -> Result<Option<RunResult>, RunTimeError> {
            match op.instruction {
                Instruction::Sipush => {
                    let param = bytes_to_short!(op.args);
                    self.current_frame
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .operand_stack
                        .push(param);
                    println!("Executing {op}, {param}");
                }
                Instruction::Bipush => {
                    let param = op.args[0] as i8;
                    self.current_frame
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .operand_stack
                        .push(param);
                    println!("Executing {op}, {param}");
                }
                Instruction::Invokestatic => {
                    let param = bytes_to_short!(op.args);
                    println!("Executing {op}, {param}");
                    let frame_ref = self.current_frame.as_ref().unwrap().borrow();
                    let const_pool = &frame_ref.class.constant_pool;
                    let index = param as u16;

                    match const_pool.as_ref()[index as usize - 1].tag {
                        ConstantPoolTag::Methodref => {
                            let class = frame_ref.class.as_ref();
                            if let Some(_method) = class.get_method_by_index(index) {
                                // already resolved on class
                            } else {
                                println!("Method at {index} not resolved");
                                let identifier = class.resolve_ref_to_identifier(index).ok_or(
                                    RunTimeError::ResolveMethodError(format!(
                                        "Failed to resolve constant pool ref {}",
                                        index
                                    )),
                                )?;
                                println!("Resolved identifier: {identifier}");

                                let mut ma = self.method_area.lock().unwrap();
                                match ma.resolve(&identifier) {
                                    Ok(method_ref) => {
                                        println!("Resolved via MethodArea: {}", identifier);
                                        return Ok(Some(RunResult::Invoke(method_ref)));
                                    }
                                    Err(e) => return Err(e),
                                }
                            }

                            println!("Tag found");
                        }
                        _ => return Err(RunTimeError::Other("Unexpected tag".to_string())),
                    }
                }
                Instruction::Invokedynamic => {
                    let param = bytes_to_short!(op.args);
                    println!("Executing {op}, {param}");
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

                    let v2: i32 = self
                        .current_frame
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .operand_stack
                        .pop()
                        .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;
                    let v1: i32 = self
                        .current_frame
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .operand_stack
                        .pop()
                        .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;

                    let take_branch = match op.instruction {
                        Instruction::IfIcmpeq => v1 == v2,
                        Instruction::IfIcmpne => v1 != v2,
                        Instruction::IfIcmplt => v1 < v2,
                        Instruction::IfIcmpge => v1 >= v2,
                        Instruction::IfIcmpgt => v1 > v2,
                        Instruction::IfIcmple => v1 <= v2,
                        _ => false,
                    };

                    println!("Executing {op}, v1={} v2={} take_branch={}", v1, v2, take_branch);
                    if take_branch {
                        let target = (op.index as isize) + offset;
                        let new_pc = target - ((1 + arg_len) as isize);
                        self.stack.set_pc(new_pc as usize);
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

                    let v: i32 = self
                        .current_frame
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .operand_stack
                        .pop()
                        .ok_or(RunTimeError::Other("Operand stack underflow".to_string()))?;

                    let take_branch = match op.instruction {
                        Instruction::IfEq => v == 0,
                        Instruction::IfNe => v != 0,
                        Instruction::IfLt => v < 0,
                        Instruction::IfGe => v >= 0,
                        Instruction::IfGt => v > 0,
                        Instruction::IfLe => v <= 0,
                        _ => false,
                    };

                    println!("Executing {op}, v={} take_branch={}", v, take_branch);

                    if take_branch {
                        let target = (op.index as isize) + offset;
                        let new_pc = target - ((1 + arg_len) as isize);
                        self.stack.set_pc(new_pc as usize);
                    }
                }
                Instruction::Invokevirtual => {
                    let param = bytes_to_short!(op.args);
                    println!("Executing {op}, {param}");
                }
                Instruction::Iadd => {
                    println!("Executing {op}");
                }
                Instruction::Iload => {
                    let idx = op.args[0] as u16;
                    println!("Executing {op} {}", idx);
                    let val: i32 = self
                        .current_frame
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .get_variable_value(idx)?;
                    self.current_frame
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .operand_stack
                        .push(val);
                }
                Instruction::Iload0
                | Instruction::Iload1
                | Instruction::Iload2
                | Instruction::Iload3 => {
                    let pos = op.instruction - Instruction::Iload0;
                    let idx = pos as u16;
                    let val: i32 = self
                        .current_frame
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .get_variable_value(idx)?;
                    self.current_frame
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .operand_stack
                        .push(val);
                    println!("Executing {op} {idx}");
                }
                Instruction::Pop => {
                    println!("Executing {op}");
                }
                Instruction::Return => {
                    println!("Executing {op}");
                }
                Instruction::Astore0
                | Instruction::Astore1
                | Instruction::Astore2
                | Instruction::Astore3 => {
                    let pos = op.instruction - Instruction::Astore0;
                    println!("Executing {op} {pos}");
                }
                Instruction::Iconst0
                | Instruction::Iconst1
                | Instruction::Iconst2
                | Instruction::Iconst3 => {
                    let pos = op.instruction - Instruction::Iconst0;
                    println!("Executing {op} {pos}");
                }
                Instruction::Aload0
                | Instruction::Aload1
                | Instruction::Aload2
                | Instruction::Aload3 => {
                    let pos = op.instruction - Instruction::Aload0;
                    println!("Executing {op} {pos}");
                }
                Instruction::Getstatic => {
                    let param = bytes_to_short!(op.args);
                    println!("Executing {op}, {param}");
                    let frame_ref = self.current_frame.as_ref().unwrap().borrow();
                    let const_pool = &frame_ref.class.constant_pool;
                    let index = param as u16;

                    use crate::loader::java_class::java_class::ConstantPoolTag;

                    match const_pool.as_ref()[index as usize - 1].tag {
                        ConstantPoolTag::Fieldref => {
                            // resolve identifier from constant pool
                            let identifier = frame_ref
                                .class
                                .resolve_ref_to_identifier(index)
                                .ok_or(RunTimeError::ResolveMethodError(format!(
                                    "Failed to resolve constant pool ref {}",
                                    index
                                )))?;

                            // ensure class entries are registered in MethodArea
                            let class_name = identifier
                                .rfind('.')
                                .map(|pos| &identifier[..pos])
                                .ok_or(RunTimeError::Other(format!(
                                    "Invalid field identifier {}",
                                    identifier
                                )))?
                                .to_string();

                            let mut ma = self.method_area.lock().unwrap();
                            let class_ptr = ma.get_or_load_class(&class_name)?;
                            // populate resolved fields for that class
                            if ma.get_resolved_field(&identifier).is_none() {
                                ma.insert_resolved_for_class(&class_name, &class_ptr);
                            }

                            if let Some(_field_ref) = ma.get_resolved_field(&identifier) {
                                // allocate a reference for this field and push to operand stack
                                let mut rm = self.reference_manager.lock().unwrap();
                                let ref_u32 = rm.allocate_symbolic(identifier.clone());
                                drop(rm);
                                // push reference as i32 slot
                                drop(frame_ref);
                                let mut cf = self.current_frame.as_ref().unwrap().borrow_mut();
                                cf.operand_stack.push(ref_u32 as i32);
                            } else {
                                return Err(RunTimeError::Other(format!("Field {} not found", identifier)));
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
            let size = match &self.current_frame {
                Some(cf) => cf.borrow().operand_stack.len(),
                None => 0,
            };
            println!("Stack size: {}", size);
            Ok(None)
        }
        pub fn invoke(&mut self, method_ref: MethodReference) -> Result<(), RunTimeError> {
            let frame = StackFrame::new_ptr(method_ref)?;
            self.stack.push_frame(frame.clone());
            Ok(())
        }
        pub fn new(ma: &MethodAreaPtr, rm: &crate::vm::reference_manager::ReferenceManagerPtr) -> Self {
            Self {
                pc: 0,
                stack: Stack::new(),
                current_frame: None,
                method_area: ma.clone(),
                reference_manager: rm.clone(),
            }
        }
    }
}
