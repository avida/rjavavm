mod thread {
    use crate::vm::{class::ClassPtr, errors::errors::RunTimeError, stack::stack::{Stack, StackFrame}};
    struct Thread {
        pc: usize,
        stack: Stack,
    }

    impl Thread {
        fn run() {}
        fn invoke(class: ClassPtr, method_index: u16) -> Result<(), RunTimeError> {
            let frame = StackFrame::new(class, method_index);
            Ok(())
        }
    }
}
