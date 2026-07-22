pub mod java_class {
    use std::fmt;
    use crate::attributes::attributes::*;

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
    pub enum ConstantPoolPFieldInfo {
        ClassInfo {
            name_index: u16,
        },
        Utf8Info {
            length: u16,
            bytes: Vec<u8>,
        },
        NameAndType {
            name_index: u16,
            descriptor_index: u16,
        },

        MethodRef(RefFieldInfo),
        InterfaceMethodRef(RefFieldInfo),
        FieldRef(RefFieldInfo),
        String {
            string_index: u16,
        },
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

    impl fmt::Display for ConstantPoolPFieldInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ConstantPoolPFieldInfo::ClassInfo { name_index } => {
                    write!(f, "ClassInfo name_index={}", name_index)
                }
                ConstantPoolPFieldInfo::Utf8Info { length, bytes } => {
                    let text = String::from_utf8_lossy(bytes);
                    write!(f, "Utf8Info length={} text={}", length, text)
                }
                ConstantPoolPFieldInfo::MethodRef(r) => write!(f, "MethodRef {}", r),
                ConstantPoolPFieldInfo::InterfaceMethodRef(r) => {
                    write!(f, "InterfaceMethodRef {}", r)
                }
                ConstantPoolPFieldInfo::FieldRef(r) => write!(f, "FieldRef {}", r),
                ConstantPoolPFieldInfo::NameAndType { name_index, descriptor_index } => {
                    write!(f, "NameAndType name_index={} descriptor_index={}", name_index, descriptor_index)
                }
                ConstantPoolPFieldInfo::String { string_index } => {
                    write!(f, "String string_index={}", string_index)
                }
                // _ => write!(f, "Unimplemented"),
            }
        }
    }
    #[derive(Debug)]
    pub struct ConstantPoolInfo {
        pub tag: ConstantPoolTag,
        pub info: ConstantPoolPFieldInfo,
    }

    #[derive(Debug)]
    pub struct FieldInfo {
        pub access_flags: u16,
        pub name_index: u16,
        pub descriptor_index: u16,
        pub attributes_count: u16,
        pub attributes: Vec<AttributeInfo>,
    }
    #[derive(Debug)]
    pub struct MethodInfo {
        pub access_flags: u16,
        pub name_index: u16,
        pub descriptor_index: u16,
        pub attributes_count: u16,
        pub attributes: Vec<AttributeInfo>,
    }
    impl fmt::Display for FieldInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(
                f,
                "access=0x{:04x} name_index={} descriptor_index={} attrs={}",
                self.access_flags, self.name_index, self.descriptor_index, self.attributes_count
            )?;
            if !self.attributes.is_empty() {
                writeln!(f, "    Attributes:")?;
                for (i, a) in self.attributes.iter().enumerate() {
                    writeln!(f, "      #{}: {}", i + 1, a)?;
                }
            }
            Ok(())
        }
    }

    impl fmt::Display for MethodInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(
                f,
                "access=0x{:04x} name_index={} descriptor_index={} attrs={}",
                self.access_flags, self.name_index, self.descriptor_index, self.attributes_count
            )?;
            if !self.attributes.is_empty() {
                writeln!(f, "    Attributes:")?;
                for (i, a) in self.attributes.iter().enumerate() {
                    writeln!(f, "      #{}: {}", i + 1, a)?;
                }
            }
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct JavaClass {
        pub magic_number: u32,
        pub minor_version: u16,
        pub major_version: u16,
        pub constant_pool_count: u16,
        pub constant_pool: Vec<ConstantPoolInfo>,
        pub access_flags: u16,
        pub this_class: u16,
        pub super_class: u16,
        pub interface_count: u16,
        pub interfaces: Vec<u16>,
        pub fields_count: u16,
        pub field_info: Vec<FieldInfo>,
        pub methods_count: u16,
        pub methods: Vec<MethodInfo>,
        pub attributes_count: u16,
        pub attributes_info: Vec<AttributeInfo>,
    }
    impl Default for JavaClass {
        fn default() -> Self {
            JavaClass {
                magic_number: 0,
                minor_version: 0,
                major_version: 0,
                constant_pool_count: 0,
                constant_pool: vec![],
                access_flags: 0,
                this_class: 0,
                super_class: 0,
                interface_count: 0,
                interfaces: vec![],
                fields_count: 0,
                field_info: vec![],
                methods_count: 0,
                methods: vec![],
                attributes_count: 0,
                attributes_info: vec![],
            }
        }
    }
    impl fmt::Display for JavaClass {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "Magic number: 0x{:08x}", self.magic_number)?;
            writeln!(f, "Major version: {}", self.major_version)?;
            writeln!(f, "Minor version: {}", self.minor_version)?;
            writeln!(f, "Constant pool count: {}", self.constant_pool_count)?;
            writeln!(f, "Access flags: 0x{:04x}", self.access_flags)?;
            writeln!(f, "This class index: {}", self.this_class)?;
            writeln!(f, "Super class index: {}", self.super_class)?;
            writeln!(f, "Interface count: {}", self.interface_count)?;
            if !self.interfaces.is_empty() {
                writeln!(f, "Interfaces: {:?}", self.interfaces)?;
            }
            writeln!(f, "Fields count: {}", self.fields_count)?;
            writeln!(f, "Methods count: {}", self.methods_count)?;
            writeln!(f, "Attributes count: {}", self.attributes_count)?;

            if !self.field_info.is_empty() {
                writeln!(f, "Fields:")?;
                for (i, field) in self.field_info.iter().enumerate() {
                    writeln!(f, "  #{}: {}", i + 1, field)?;
                }
            }

            if !self.methods.is_empty() {
                writeln!(f, "Methods:")?;
                for (i, m) in self.methods.iter().enumerate() {
                    writeln!(f, "  #{}: {}", i + 1, m)?;
                }
            }

            if !self.constant_pool.is_empty() {
                writeln!(f, "Constant Pool:")?;
                for (i, cp) in self.constant_pool.iter().enumerate() {
                    writeln!(f, "  #{}: {} => {}", i + 1, cp.tag, cp.info)?;
                }
            }

            if !self.attributes_info.is_empty() {
                writeln!(f, "Attributes:")?;
                for (i, a) in self.attributes_info.iter().enumerate() {
                    writeln!(f, "  #{}: {}", i + 1, a)?;
                }
            }

            Ok(())
        }
    }
}
