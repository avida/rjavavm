pub mod stack {
    use crate::loader::java_class::java_class::ConstantPoolInfoTable;
    use crate::vm::class::{Class, ClassPtr, Method, MethodReference};
    use crate::vm::errors::errors::*;
    use crate::vm::operand_stack::operand_stack::OperandStack;
    use crate::vm::types::types::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    pub struct StackFrame {
        pub class: ClassPtr,
        pub method: Rc<Method>,
        pub operand_stack: OperandStack,
        local_variables: Vec<VarSlot>,
    }
    pub type StackFramePtr = Rc<RefCell<StackFrame>>;
    pub struct Stack {
        frames: Vec<StackFramePtr>,
        program_counter: usize,
        return_addresses: Vec<usize>,
    }
    impl StackFrame {
        fn make_local_variables() {}
        pub fn new(method_ref: MethodReference) -> Result<Self, RunTimeError> {
            let method = method_ref.method();
            let class = method_ref.class();
            let mut local_variables: Vec<VarSlot> = vec![];
            local_variables.resize(method.max_locals as usize, [0, 0, 0, 0]);
            Ok(Self {
                operand_stack: OperandStack::new(method.max_stack),
                local_variables,
                class: class.clone(),
                method: method.clone(),
            })
        }
        pub fn new_ptr(method_ref: MethodReference) -> Result<StackFramePtr, RunTimeError> {
            let frame = Self::new(method_ref)?;
            Ok(Rc::new(RefCell::new(frame)))
        }
        pub fn get_variable_value<T: LocalVariableValue>(
            &self,
            slot: u16,
        ) -> Result<T, RunTimeError> {
            let start = slot as usize;
            let end = start + T::slot_count();
            let slots = self.local_variables.get(start..end).ok_or_else(|| {
                RunTimeError::Other("Local variable slot out of range".to_string())
            })?;
            Ok(T::from_slots(slots))
        }
        pub fn set_variable_value<T: LocalVariableValue>(
            &mut self,
            slot: u16,
            value: T,
        ) -> Result<(), RunTimeError> {
            let start = slot as usize;
            let end = start + T::slot_count();
            let slots = self.local_variables.get_mut(start..end).ok_or_else(|| {
                RunTimeError::Other("Local variable slot out of range".to_string())
            })?;
            slots.copy_from_slice(&value.into_slots());
            Ok(())
        }
    }

    pub trait LocalVariableValue: Sized {
        fn slot_count() -> usize {
            1
        }
        fn from_slots(slots: &[VarSlot]) -> Self;
        fn into_slots(self) -> Vec<VarSlot>;
    }

    impl LocalVariableValue for bool {
        fn from_slots(slots: &[VarSlot]) -> Self {
            i32::from_be_bytes(slots[0]) != 0
        }
        fn into_slots(self) -> Vec<VarSlot> {
            vec![(self as i32).to_be_bytes()]
        }
    }

    impl LocalVariableValue for i8 {
        fn from_slots(slots: &[VarSlot]) -> Self {
            i32::from_be_bytes(slots[0]) as i8
        }
        fn into_slots(self) -> Vec<VarSlot> {
            vec![(self as i32).to_be_bytes()]
        }
    }

    impl LocalVariableValue for u16 {
        fn from_slots(slots: &[VarSlot]) -> Self {
            i32::from_be_bytes(slots[0]) as u16
        }
        fn into_slots(self) -> Vec<VarSlot> {
            vec![(self as i32).to_be_bytes()]
        }
    }

    impl LocalVariableValue for i32 {
        fn from_slots(slots: &[VarSlot]) -> Self {
            i32::from_be_bytes(slots[0])
        }
        fn into_slots(self) -> Vec<VarSlot> {
            vec![self.to_be_bytes()]
        }
    }

    impl LocalVariableValue for f32 {
        fn from_slots(slots: &[VarSlot]) -> Self {
            f32::from_bits(u32::from_be_bytes(slots[0]))
        }
        fn into_slots(self) -> Vec<VarSlot> {
            vec![self.to_bits().to_be_bytes()]
        }
    }

    impl LocalVariableValue for i64 {
        fn slot_count() -> usize {
            2
        }
        fn from_slots(slots: &[VarSlot]) -> Self {
            let high = slots[0];
            let low = slots[1];
            let bytes = [
                high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
            ];
            i64::from_be_bytes(bytes)
        }
        fn into_slots(self) -> Vec<VarSlot> {
            let b = self.to_be_bytes();
            vec![[b[0], b[1], b[2], b[3]], [b[4], b[5], b[6], b[7]]]
        }
    }

    impl LocalVariableValue for f64 {
        fn slot_count() -> usize {
            2
        }
        fn from_slots(slots: &[VarSlot]) -> Self {
            let high = slots[0];
            let low = slots[1];
            let bytes = [
                high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
            ];
            f64::from_bits(u64::from_be_bytes(bytes))
        }
        fn into_slots(self) -> Vec<VarSlot> {
            let b = self.to_bits().to_be_bytes();
            vec![[b[0], b[1], b[2], b[3]], [b[4], b[5], b[6], b[7]]]
        }
    }

    impl Stack {
        pub fn new() -> Self {
            Self {
                frames: Vec::new(),
                program_counter: 0,
                return_addresses: Vec::new(),
            }
        }

        pub fn push_frame(&mut self, frame: StackFramePtr) {
            self.frames.push(frame);
        }

        pub fn pop_frame(&mut self) -> Option<StackFramePtr> {
            self.frames.pop()
        }
        pub fn push_return_address(&mut self, addr: usize) {
            self.return_addresses.push(addr);
        }
        pub fn pop_return_address(&mut self) -> Option<usize> {
            self.return_addresses.pop()
        }
        pub fn top_frame(&self) -> Option<StackFramePtr> {
            if self.frames.is_empty() {
                return None;
            }
            Some(self.frames[self.frames.len() - 1].clone())
        }
        pub fn program_counter(&self) -> usize {
            self.program_counter
        }
        pub fn get_pc(&self) -> usize {
            self.program_counter
        }

        pub fn set_pc(&mut self, pc: usize) {
            self.program_counter = pc;
        }
        pub fn move_pc_next(&mut self) {
            self.increase_pc(1);
        }

        pub fn increase_pc(&mut self, delta: usize) {
            self.program_counter = self.program_counter.wrapping_add(delta);
        }

        pub fn decrease_pc(&mut self, delta: usize) {
            self.program_counter = self.program_counter.wrapping_sub(delta);
        }

        pub fn jump_pc(&mut self, addr: usize) {
            self.program_counter = addr;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::vm::AccessFlags;
        use std::collections::HashMap;

        fn dummy_class() -> ClassPtr {
            Rc::new(Class {
                constant_pool: Rc::new(vec![]),
                methods: vec![],
                fields: vec![],
                method_by_index: HashMap::new(),
                field_by_index: HashMap::new(),
                static_values: RefCell::new(HashMap::new()),
            })
        }

        fn make_frame(size: usize) -> StackFrame {
            StackFrame {
                operand_stack: OperandStack::new(0),
                local_variables: vec![[0, 0, 0, 0]; size],
                class: dummy_class(),
                method: Rc::new(Method {
                    name: "<dummy>".to_string(),
                    access_flags: AccessFlags::from(0u16),
                    max_stack: 0,
                    max_locals: 0,
                    descriptor: "".to_string(),
                    code: vec![],
                }),
            }
        }

        #[test]
        fn set_get_variable_value_bool() {
            let mut frame = make_frame(1);
            frame.set_variable_value(0, true).unwrap();
            let v: bool = frame.get_variable_value(0).unwrap();
            assert_eq!(v, true);
        }

        #[test]
        fn set_get_variable_value_i8() {
            let mut frame = make_frame(1);
            frame.set_variable_value(0, -5i8).unwrap();
            let v: i8 = frame.get_variable_value(0).unwrap();
            assert_eq!(v, -5);
        }

        #[test]
        fn set_get_variable_value_u16() {
            let mut frame = make_frame(1);
            frame.set_variable_value(0, 1234u16).unwrap();
            let v: u16 = frame.get_variable_value(0).unwrap();
            assert_eq!(v, 1234);
        }

        #[test]
        fn set_get_variable_value_i32() {
            let mut frame = make_frame(1);
            frame.set_variable_value(0, 42i32).unwrap();
            let v: i32 = frame.get_variable_value(0).unwrap();
            assert_eq!(v, 42);
        }

        #[test]
        fn set_get_variable_value_f32() {
            let f: f32 = 3.14;
            let mut frame = make_frame(1);
            frame.set_variable_value(0, f).unwrap();
            let v: f32 = frame.get_variable_value(0).unwrap();
            assert_eq!(v.to_bits(), f.to_bits());
        }

        #[test]
        fn set_get_variable_value_i64() {
            let n: i64 = 0x0102030405060708;
            let mut frame = make_frame(2);
            frame.set_variable_value(0, n).unwrap();
            let v: i64 = frame.get_variable_value(0).unwrap();
            assert_eq!(v, n);
        }

        #[test]
        fn set_get_variable_value_f64() {
            let d: f64 = -12.3456789;
            let mut frame = make_frame(2);
            frame.set_variable_value(0, d).unwrap();
            let v: f64 = frame.get_variable_value(0).unwrap();
            assert_eq!(v.to_bits(), d.to_bits());
        }

        #[test]
        fn set_get_variable_value_at_nonzero_slot() {
            let mut frame = make_frame(3);
            frame.set_variable_value(1, 99i32).unwrap();
            let v: i32 = frame.get_variable_value(1).unwrap();
            assert_eq!(v, 99);
        }

        #[test]
        fn get_variable_value_out_of_range() {
            let frame = make_frame(1);
            let result: Result<i32, RunTimeError> = frame.get_variable_value(5);
            assert!(result.is_err());
        }

        #[test]
        fn set_variable_value_out_of_range() {
            let mut frame = make_frame(1);
            let result = frame.set_variable_value(5, 42i32);
            assert!(result.is_err());
        }
    }
}
