#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessFlags(pub u16);

impl AccessFlags {
    pub const ACC_PUBLIC: u16 = 0x0001;
    pub const ACC_PRIVATE: u16 = 0x0002;
    pub const ACC_PROTECTED: u16 = 0x0004;
    pub const ACC_STATIC: u16 = 0x0008;
    pub const ACC_FINAL: u16 = 0x0010;
    pub const ACC_SYNCHRONIZED: u16 = 0x0020; // methods only
    pub const ACC_VOLATILE: u16 = 0x0040; // fields only
    pub const ACC_TRANSIENT: u16 = 0x0080; // fields only
    pub const ACC_NATIVE: u16 = 0x0100; // methods only
    pub const ACC_INTERFACE: u16 = 0x0200; // classes only
    pub const ACC_ABSTRACT: u16 = 0x0400;
    pub const ACC_STRICT: u16 = 0x0800; // methods only
    pub const ACC_SYNTHETIC: u16 = 0x1000;
    pub const ACC_ANNOTATION: u16 = 0x2000; // classes only
    pub const ACC_ENUM: u16 = 0x4000;

    pub fn from_bits(bits: u16) -> Self {
        AccessFlags(bits)
    }

    pub fn bits(self) -> u16 {
        self.0
    }

    pub fn contains(self, flag: u16) -> bool {
        (self.0 & flag) != 0
    }

    pub fn is_public(self) -> bool {
        self.contains(Self::ACC_PUBLIC)
    }

    pub fn is_private(self) -> bool {
        self.contains(Self::ACC_PRIVATE)
    }

    pub fn is_protected(self) -> bool {
        self.contains(Self::ACC_PROTECTED)
    }

    pub fn is_static(self) -> bool {
        self.contains(Self::ACC_STATIC)
    }

    pub fn is_final(self) -> bool {
        self.contains(Self::ACC_FINAL)
    }

    pub fn is_synthetic(self) -> bool {
        self.contains(Self::ACC_SYNTHETIC)
    }

    // Method-specific helpers
    pub fn is_synchronized(self) -> bool {
        self.contains(Self::ACC_SYNCHRONIZED)
    }

    pub fn is_native(self) -> bool {
        self.contains(Self::ACC_NATIVE)
    }

    pub fn is_abstract(self) -> bool {
        self.contains(Self::ACC_ABSTRACT)
    }

    pub fn is_strict(self) -> bool {
        self.contains(Self::ACC_STRICT)
    }

    // Field-specific helpers
    pub fn is_volatile(self) -> bool {
        self.contains(Self::ACC_VOLATILE)
    }

    pub fn is_transient(self) -> bool {
        self.contains(Self::ACC_TRANSIENT)
    }

    pub fn is_enum(self) -> bool {
        self.contains(Self::ACC_ENUM)
    }
}

impl From<u16> for AccessFlags {
    fn from(v: u16) -> Self {
        AccessFlags::from_bits(v)
    }
}

impl From<AccessFlags> for u16 {
    fn from(f: AccessFlags) -> Self {
        f.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::AccessFlags;

    #[test]
    fn from_and_into_u16() {
        let raw: u16 = 0x0009; // ACC_PUBLIC | ACC_STATIC
        let f = AccessFlags::from(raw);
        assert_eq!(f.bits(), raw);
        let raw2: u16 = f.into();
        assert_eq!(raw2, raw);
    }

    #[test]
    fn flag_checks() {
        let f = AccessFlags::from(AccessFlags::ACC_PUBLIC | AccessFlags::ACC_STATIC);
        assert!(f.is_public());
        assert!(f.is_static());
        assert!(!f.is_private());
    }

    #[test]
    fn contains_combination() {
        let f = AccessFlags::from(AccessFlags::ACC_PUBLIC | AccessFlags::ACC_FINAL);
        assert!(f.contains(AccessFlags::ACC_PUBLIC));
        assert!(f.contains(AccessFlags::ACC_FINAL));
        assert!(!f.contains(AccessFlags::ACC_PROTECTED));
    }
}
