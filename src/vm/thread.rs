mod thread {
    use crate::vm::{class::ClassPtr, errors::errors::RunTimeError, stack::stack::Stack};
    struct Thread {
        pc: usize,
        stack: Stack,
    }

    impl Thread {
        fn run() {}
        fn invoke(class: ClassPtr, method_index: u32) -> Result<(), RunTimeError> {
            Ok(())
        }
    }
}
