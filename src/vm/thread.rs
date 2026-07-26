pub mod thread {
    use crate::vm::{
        class::ClassPtr,
        errors::errors::RunTimeError,
        stack::stack::{Stack, StackFrame},
    };

    pub struct Thread {
        pc: usize,
        stack: Stack,
    }

    impl Thread {
        fn run() -> Result<(), RunTimeError> {
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
