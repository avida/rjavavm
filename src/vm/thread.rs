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
        fn run() {}
        fn invoke(class: ClassPtr, method_index: u16) -> Result<(), RunTimeError> {
            let frame = StackFrame::new(class, method_index);
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
