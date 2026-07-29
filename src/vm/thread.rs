pub mod thread {
    use std::ops::RemAssign;
    use std::rc::Rc;

    use crate::bytes_to_short;
    use crate::vm::byte_code::byte_code::{self, Instruction};

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

            // let code_parsed = byte_code::parse(&method.code)?;
            // for c in code_parsed {
            //     println!("{}", c);
            // }
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
        pub fn invoke(&mut self, class: ClassPtr, method_index: u16) -> Result<(), RunTimeError> {
            let frame = StackFrame::new_ptr(class, method_index)?;
            self.stack.push_frame(frame.clone());
            Ok(())
        }
        pub fn new() -> Self {
            Self {
                pc: 0,
                stack: Stack::new(),
                current_frame: None,
            }
        }
    }
}
