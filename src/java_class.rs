pub mod java_class {
    use std::fmt;
    #[derive(Debug)]
    pub struct JavaClass {
        pub magic_number: u32,
        pub minor_version: u16,
        pub major_version: u16,
        pub constant_pool_count: u16,
    }
    impl Default for JavaClass {
        fn default() -> Self {
            JavaClass {
                magic_number: 0,
                minor_version: 0,
                major_version: 0,
                constant_pool_count: 0,
            }
        }
    }
    impl fmt::Display for JavaClass {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "Magic number: {:x}", self.magic_number)?;
            writeln!(f, "Major version: {:}", self.major_version)?;
            writeln!(f, "Minot version: {:}", self.minor_version)?;
            writeln!(f, "Constant pool count: {:}", self.constant_pool_count)?;
            Ok(())
        }
    }
}
