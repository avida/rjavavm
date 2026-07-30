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
    }

    impl Thread {
        pub fn run(&mut self) -> Result<(), RunTimeError> {
            println!("Running thread");
            let current_frame = self
                .stack
                .top_frame()
                .ok_or(RunTimeError::Other("Stack is empty".to_string()))?;
            self.current_frame = Some(current_frame.clone());

            let class = current_frame.borrow_mut().class.clone();
            let method = current_frame.borrow_mut().method.clone();
            let pc = self.stack.get_pc();
            loop {
                let (next_op, args_len) =
                    byte_code::parse_op_at(&method.code, self.stack.get_pc())?;
                // println!("Next op is {}, next_offset {}", next_op, next_op.args.len());
                self.run_op(&next_op)?;
                self.stack.increase_pc(args_len + 1);
            }

            Ok(())
        }
        pub fn push_frame(&mut self, method_ref: MethodReference) -> Result<(), RunTimeError> {
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
            self.stack.push_frame(new_frame);
            Ok(())
        }
        pub fn run_op(&self, op: &byte_code::Op) -> Result<(), RunTimeError> {
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
                Instruction::Invokestatic => {
                    let param = bytes_to_short!(op.args);
                    println!("Executing {op}, {param}");
                    let frame_ref = self.current_frame.as_ref().unwrap().borrow();
                    let const_pool = &&frame_ref.class.constant_pool;
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
                                    Ok(_method_ref) => {
                                        println!("Resolved via MethodArea: {}", identifier);
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
                Instruction::Invokevirtual => {
                    let param = bytes_to_short!(op.args);
                    println!("Executing {op}, {param}");
                }
                Instruction::Iadd => {
                    println!("Executing {op}");
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
                }
                _ => {
                    return Err(RunTimeError::Notimplemented(format!(
                        "Instruction {}",
                        op.instruction
                    )));
                }
            }
            Ok(())
        }
        pub fn invoke(&mut self, method_ref: MethodReference) -> Result<(), RunTimeError> {
            let frame = StackFrame::new_ptr(method_ref)?;
            self.stack.push_frame(frame.clone());
            Ok(())
        }
        pub fn new(ma: &MethodAreaPtr) -> Self {
            Self {
                pc: 0,
                stack: Stack::new(),
                current_frame: None,
                method_area: ma.clone(),
            }
        }
    }
}
