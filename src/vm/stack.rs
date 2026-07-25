pub mod stack {
    use crate::loader::java_class::java_class::ConstantPoolInfoTable;
    use crate::vm::class::{Class, ClassPtr};
    use crate::vm::errors::errors::*;
    use crate::vm::operand_stack::operand_stack::OperandStack;
    use crate::vm::types::types::*;

    pub struct StackFrame {
        operand_stack: OperandStack,
        local_variables: Vec<VarSlot>,
        class: ClassPtr,
    }
    impl StackFrame {
        fn make_local_variables() {}
        pub fn new(class: ClassPtr, method_index: u16) -> Result<Self, RunTimeError> {
            let method = class
                .get_method_by_index(method_index)
                .ok_or_else(|| RunTimeError {
                    message: "Failed to fetch method".to_string(),
                })?;
            let mut local_variables: Vec<VarSlot> = vec![];
            local_variables.resize(method.max_locals as usize, [0, 0, 0, 0]);
            Ok(Self {
                operand_stack: OperandStack::new(method.max_stack),
                local_variables,
                class,
            })
        }
    }

    pub struct Stack {
        frames: Vec<StackFrame>,
    }
}
