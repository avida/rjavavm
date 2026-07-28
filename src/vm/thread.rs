pub mod thread {
    use crate::vm::byte_code::byte_code;
    use crate::vm::{
        class::ClassPtr,
        errors::errors::RunTimeError,
        runtime::Runtime,
        stack::stack::{Stack, StackFrame},
    };

    pub struct Thread {
        pc: usize,
        stack: Stack,
    }

    impl Thread {
        pub fn run(&mut self) -> Result<(), RunTimeError> {
            println!("Running thread");
            let current_frame = self
                .stack
                .top_frame()
                .ok_or(RunTimeError::Other("Stack is empty".to_string()))?;
            let class = current_frame.borrow_mut().class.clone();
            let method = current_frame.borrow_mut().method.clone();
            let pc = self.stack.get_pc();
            loop {
                let (next_op, args_len ) = byte_code::parse_op_at(&method.code, self.stack.get_pc())?;
                println!("Next op is {}, next_offset {}", next_op, next_op.args.len());
                self.stack.increase_pc(args_len + 1);
            }

            // let code_parsed = byte_code::parse(&method.code)?;
            // for c in code_parsed {
            //     println!("{}", c);
            // }
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
            }
        }
    }
}
