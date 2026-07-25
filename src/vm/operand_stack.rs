pub mod operand_stack {
    use crate::vm::types::types::*;
    pub struct OperandStack {
        stack: Vec<VarSlot>,
    }
    impl OperandStack {
        pub fn new(size: u16) -> Self {
            let mut vec: Vec<VarSlot> = Vec::with_capacity(size.into());
            Self { stack: vec }
        }
        pub fn push<V: StackValue>(&mut self, value: V) {
            for slot in value.to_slots() {
                self.stack.push(slot);
            }
        }
        pub fn pop<P: Popable>(&mut self) -> Option<P> {
            P::pop_from(&mut self.stack)
        }
    }

    pub trait StackValue {
        fn to_slots(self) -> Vec<VarSlot>;
    }

    impl StackValue for i32 {
        fn to_slots(self) -> Vec<VarSlot> {
            vec![self.to_be_bytes()]
        }
    }


    impl StackValue for f32 {
        fn to_slots(self) -> Vec<VarSlot> {
            vec![self.to_bits().to_be_bytes()]
        }
    }

    impl StackValue for i64 {
        fn to_slots(self) -> Vec<VarSlot> {
            let b = self.to_be_bytes();
            vec![[b[0], b[1], b[2], b[3]], [b[4], b[5], b[6], b[7]]]
        }
    }


    impl StackValue for f64 {
        fn to_slots(self) -> Vec<VarSlot> {
            let bits = self.to_bits();
            let b = bits.to_be_bytes();
            vec![[b[0], b[1], b[2], b[3]], [b[4], b[5], b[6], b[7]]]
        }
    }

    pub trait Popable: Sized {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self>;
    }

    impl Popable for i32 {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self> {
            stack.pop().map(|b| i32::from_be_bytes(b))
        }
    }

    

    impl Popable for f32 {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self> {
            stack.pop().map(|b| f32::from_bits(u32::from_be_bytes(b)))
        }
    }

    impl Popable for i64 {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self> {
            let low = stack.pop()?;
            let high = stack.pop()?;
            let bytes = [high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3]];
            Some(i64::from_be_bytes(bytes))
        }
    }

    

    impl Popable for f64 {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self> {
            let low = stack.pop()?;
            let high = stack.pop()?;
            let bytes = [high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3]];
            Some(f64::from_bits(u64::from_be_bytes(bytes)))
        }
    }

}
