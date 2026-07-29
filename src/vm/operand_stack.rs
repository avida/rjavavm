pub mod operand_stack {
    use crate::vm::types::types::*;
    use std::fmt;
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
        pub fn len(&self) -> usize {
            self.stack.len()
        }
    }

    impl fmt::Display for OperandStack {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "OperandStack(len={}) [", self.stack.len())?;
            for (i, slot) in self.stack.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                let v = u32::from_be_bytes(*slot);
                write!(f, "0x{:08x}", v)?;
            }
            write!(f, "]")
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

    impl StackValue for i8 {
        fn to_slots(self) -> Vec<VarSlot> {
            // sign-extend to 32-bit and store as one slot
            let v = (self as i32).to_be_bytes();
            vec![v]
        }
    }

    impl StackValue for i16 {
        fn to_slots(self) -> Vec<VarSlot> {
            // sign-extend to 32-bit and store as one slot
            let v = (self as i32).to_be_bytes();
            vec![v]
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

    impl Popable for i8 {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self> {
            stack.pop().map(|b| i32::from_be_bytes(b) as i8)
        }
    }

    impl Popable for i16 {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self> {
            stack.pop().map(|b| i32::from_be_bytes(b) as i16)
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
            let bytes = [
                high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
            ];
            Some(i64::from_be_bytes(bytes))
        }
    }

    impl Popable for f64 {
        fn pop_from(stack: &mut Vec<VarSlot>) -> Option<Self> {
            let low = stack.pop()?;
            let high = stack.pop()?;
            let bytes = [
                high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
            ];
            Some(f64::from_bits(u64::from_be_bytes(bytes)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::operand_stack::operand_stack::OperandStack;

    #[test]
    fn push_pop_i32() {
        let mut s = OperandStack::new(8);
        s.push(0i32);
        s.push(123i32);
        let v: i32 = s.pop().unwrap();
        assert_eq!(v, 123);
        let v2: i32 = s.pop().unwrap();
        assert_eq!(v2, 0);
        let none: Option<i32> = s.pop();
        assert!(none.is_none());
    }

    #[test]
    fn push_pop_i8() {
        let mut s = OperandStack::new(8);
        s.push(0i8);
        s.push(-5i8);
        let v: i8 = s.pop().unwrap();
        assert_eq!(v, -5);
        let v2: i8 = s.pop().unwrap();
        assert_eq!(v2, 0);
        let none: Option<i8> = s.pop();
        assert!(none.is_none());
    }

    #[test]
    fn push_pop_i64() {
        let mut s = OperandStack::new(8);
        s.push(0i64);
        let n: i64 = 0x0102030405060708i64;
        s.push(n);
        let v: i64 = s.pop().unwrap();
        assert_eq!(v, n);
        let v2: i64 = s.pop().unwrap();
        assert_eq!(v2, 0);
    }

    #[test]
    fn push_pop_f32_f64() {
        let mut s = OperandStack::new(8);
        let f: f32 = 3.14;
        let d: f64 = -12.3456789;
        s.push(f);
        s.push(d);
        let dd: f64 = s.pop().unwrap();
        let ff: f32 = s.pop().unwrap();
        assert_eq!(ff.to_bits(), f.to_bits());
        assert_eq!(dd.to_bits(), d.to_bits());
    }
}
