pub mod types {
    pub type VarSlot = [u8; 4];
    pub struct Reference {}
    pub enum Type {
        Bool(bool),
        Byte(u8),
        Char(u16),
        Short(u16),
        Int(u32),
        Long(u64),
        Float(f32),
        Double(f64),
        Reference(Reference),
        ReturnAddress(u32),
    }
}