pub mod types {
    pub type VarSlot = [u8; 4];
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SlotType {
        Bool,
        Byte,
        Char,
        Short,
        Int,
        Long,
        Float,
        Double,
        Reference,
        ReturnAddress,
    }
    #[derive(Debug, Clone, PartialEq)]
    pub enum Type {
        Bool(bool),
        Byte(u8),
        Char(u16),
        Short(u16),
        Int(u32),
        Long(u64),
        Float(f32),
        Double(f64),
        Reference(u32),
        Null,
        ReturnAddress(u32),
    }
}
