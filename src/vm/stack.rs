pub mod stack {
    use crate::loader::java_class::java_class::ConstantPoolInfoTable;
    use crate::vm::class::{Class, ClassPtr};
    use crate::vm::errors::errors::*;
    use crate::vm::operand_stack::operand_stack::OperandStack;
    use crate::vm::types::types::*;

    pub struct StackFrame {
        operand_stack: OperandStack,
        local_variables: Vec<Type>,
        class: ClassPtr,
    }
    impl StackFrame {
        fn new(class: ClassPtr, method_index: u32) -> Result<Self, RunTimeError> {
            // let method = class.constant_pool.
            Ok(Self {
                operand_stack: OperandStack::new(10),
                local_variables: vec![],
                class,
            })
        }
    }

    pub struct Stack {
        frames: Vec<StackFrame>,
    }
}
