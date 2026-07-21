pub mod java_class {
    use std::fmt;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum ConstantPoolTag {
        Utf8 = 1,
        Integer = 3,
        Float = 4,
        Long = 5,
        Double = 6,
        Class = 7,
        String = 8,
        Fieldref = 9,
        Methodref = 10,
        InterfaceMethodref = 11,
        NameAndType = 12,
        MethodHandle = 15,
        MethodType = 16,
        Dynamic = 17,
        InvokeDynamic = 18,
        Module = 19,
        Package = 20,
    }

    impl TryFrom<u8> for ConstantPoolTag {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                1 => Ok(ConstantPoolTag::Utf8),
                3 => Ok(ConstantPoolTag::Integer),
                4 => Ok(ConstantPoolTag::Float),
                5 => Ok(ConstantPoolTag::Long),
                6 => Ok(ConstantPoolTag::Double),
                7 => Ok(ConstantPoolTag::Class),
                8 => Ok(ConstantPoolTag::String),
                9 => Ok(ConstantPoolTag::Fieldref),
                10 => Ok(ConstantPoolTag::Methodref),
                11 => Ok(ConstantPoolTag::InterfaceMethodref),
                12 => Ok(ConstantPoolTag::NameAndType),
                15 => Ok(ConstantPoolTag::MethodHandle),
                16 => Ok(ConstantPoolTag::MethodType),
                17 => Ok(ConstantPoolTag::Dynamic),
                18 => Ok(ConstantPoolTag::InvokeDynamic),
                19 => Ok(ConstantPoolTag::Module),
                20 => Ok(ConstantPoolTag::Package),
                _ => Err(()),
            }
        }
    }

    impl fmt::Display for ConstantPoolTag {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                ConstantPoolTag::Utf8 => "Utf8",
                ConstantPoolTag::Integer => "Integer",
                ConstantPoolTag::Float => "Float",
                ConstantPoolTag::Long => "Long",
                ConstantPoolTag::Double => "Double",
                ConstantPoolTag::Class => "Class",
                ConstantPoolTag::String => "String",
                ConstantPoolTag::Fieldref => "Fieldref",
                ConstantPoolTag::Methodref => "Methodref",
                ConstantPoolTag::InterfaceMethodref => "InterfaceMethodref",
                ConstantPoolTag::NameAndType => "NameAndType",
                ConstantPoolTag::MethodHandle => "MethodHandle",
                ConstantPoolTag::MethodType => "MethodType",
                ConstantPoolTag::Dynamic => "Dynamic",
                ConstantPoolTag::InvokeDynamic => "InvokeDynamic",
                ConstantPoolTag::Module => "Module",
                ConstantPoolTag::Package => "Package",
            };
            write!(f, "{}", s)
        }
    }

    #[derive(Debug)]
    pub struct RefFieldInfo {
        pub class_index: u16,
        pub name_and_type_index: u16,
    }
    #[derive(Debug)]
    pub enum FieldInfo {
        ClassInfo { name_index: u16 },
        Utf8Info {
            length: u16,
            bytes: Vec<u8>,

        },
        NameAndType{
            name_index: u16,
            descriptor_index: u16
        },

        MethodRef(RefFieldInfo),
        InterfaceMethodRef(RefFieldInfo),
        FieldRef(RefFieldInfo),
    }

    impl fmt::Display for RefFieldInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "class_index: {}, name_and_type_index: {}",
                self.class_index, self.name_and_type_index
            )
        }
    }

    impl fmt::Display for FieldInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                FieldInfo::ClassInfo { name_index } => {
                    write!(f, "ClassInfo name_index={}", name_index)
                }
                FieldInfo::Utf8Info { length, bytes } => {
                    let text = String::from_utf8_lossy(bytes);
                    write!(f, "Utf8Info length={} text={}", length, text)
                }
                FieldInfo::MethodRef(r) => write!(f, "MethodRef {}", r),
                FieldInfo::InterfaceMethodRef(r) => write!(f, "InterfaceMethodRef {}", r),
                FieldInfo::FieldRef(r) => write!(f, "FieldRef {}", r),
                _ => write!(f, "Unimplemented")
            }
        }
    }
    #[derive(Debug)]
    pub struct ConstantPoolInfo {
        pub tag: ConstantPoolTag,
        pub info: FieldInfo,
    }
    #[derive(Debug)]
    pub struct JavaClass {
        pub magic_number: u32,
        pub minor_version: u16,
        pub major_version: u16,
        pub constant_pool_count: u16,
        pub constant_pool: Vec<ConstantPoolInfo>,
    }
    impl Default for JavaClass {
        fn default() -> Self {
            JavaClass {
                magic_number: 0,
                minor_version: 0,
                major_version: 0,
                constant_pool_count: 0,
                constant_pool: vec![],
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
