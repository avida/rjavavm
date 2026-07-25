pub mod operand_stack {
    use crate::vm::types::types::*;
    pub struct OperandStack {
        stack: Vec<Type>,
    }
    impl OperandStack {
        pub fn new(size: usize) -> Self {
            let mut vec: Vec<Type> = Vec::with_capacity(size);

            Self { stack: vec }
        }
    }

}
