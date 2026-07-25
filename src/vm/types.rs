pub mod types {
    pub struct Reference {}
    pub enum Type {
        Bool,
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