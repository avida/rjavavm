pub mod stack {
    use crate::loader::java_class::java_class::ConstantPoolInfoTable;
    use crate::vm::class::{Class, ClassPtr};
    use crate::vm::operand_stack::operand_stack::*;
    use crate::vm::types::types::*;

    pub struct StackFrame {
        operand_stack: OperandStack,
        local_variables: Vec<Type>,
        class: ClassPtr,
    }

    pub struct Stack {
        frames: Vec<StackFrame>,
    }
}
