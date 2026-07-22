pub mod class_loader {

    use byteorder::BigEndian;
    use byteorder::ReadBytesExt;

    use crate::java_class::java_class::*;
    use std::fmt;
    use std::fs;
    use std::io::{Cursor, Read};
    #[derive(Debug)]
    pub enum ClassLoadError {
        Io(std::io::Error),
        NotFound(String),
        InvalidFormat(String),
        Other(String),
    }
    macro_rules! read_2_bytes {
        ($c: expr) => {
            $c.read_u16::<byteorder::BigEndian>().unwrap()
        };
    }
    macro_rules! read_4_bytes {
        ($c: expr) => {
            $c.read_u32::<byteorder::BigEndian>().unwrap()
        };
    }

    impl fmt::Display for ClassLoadError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ClassLoadError::Io(e) => write!(f, "I/O error while loading class: {}", e),
                ClassLoadError::NotFound(name) => write!(f, "Class not found: {}", name),
                ClassLoadError::InvalidFormat(reason) => {
                    write!(f, "Invalid class format: {}", reason)
                }
                ClassLoadError::Other(msg) => write!(f, "Class load error: {}", msg),
            }
        }
    }

    pub fn read(path: &str) -> Result<Vec<u8>, ClassLoadError> {
        fs::read(path)
            .and_then(|r| Ok(r))
            .map_err(|_| ClassLoadError::NotFound("File not found".to_string()))
    }

    pub fn parse(bytes: Vec<u8>) -> Result<JavaClass, ClassLoadError> {
        let mut cursor = Cursor::new(bytes);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).map_err(|e| {
            ClassLoadError::InvalidFormat(format!("Failed reading magic number: {}", e))
        })?;

        let magic_number = u32::from_be_bytes(buf);

        cursor.read_exact(&mut buf).map_err(|e| {
            ClassLoadError::InvalidFormat(format!("Failed reading version number: {}", e))
        })?;
        let (minor_version, major_version) = (
            u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            u16::from_be_bytes(buf[2..].try_into().unwrap()),
        );

        cursor.read_exact(&mut buf[..2]).map_err(|e| {
            ClassLoadError::InvalidFormat(format!("Failed reading constant pool count: {}", e))
        })?;
        let constant_pool_count = u16::from_be_bytes([buf[0], buf[1]]);
        let constant_pool = parse_constant_pool(&mut cursor, constant_pool_count)?;
        let (access_flags, this_class, super_class, interface_count) = (
            read_2_bytes!(cursor),
            read_2_bytes!(cursor),
            read_2_bytes!(cursor),
            read_2_bytes!(cursor),
        );

        let mut interfaces_u8: Vec<u8> = vec![];

        Vec::resize(&mut interfaces_u8, 2 * interface_count as usize, 0);

        cursor.read_exact(&mut interfaces_u8).map_err(|e| {
            ClassLoadError::InvalidFormat(format!("Failed reading interfaces: {}", e))
        })?;
        let fields_count = read_2_bytes!(cursor);

        let interfaces: Vec<u16> = interfaces_u8
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();

        let field_info = parse_field_info(&mut cursor, fields_count)?;
        let methods_count = read_2_bytes!(cursor);
        let methods = parse_method_info(&mut cursor, methods_count)?;
        let attributes_count = read_2_bytes!(cursor);
        let attributes_info = parse_method_attributes(&mut cursor, attributes_count)?;
        println!("att -> {}", attributes_info.len());

        Ok(JavaClass {
            magic_number,
            minor_version,
            major_version,
            constant_pool_count,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interface_count,
            interfaces,
            fields_count,
            field_info,
            methods_count,
            methods,
            attributes_count,
            attributes_info,
        })
    }

    fn map_error(_: std::io::Error) -> ClassLoadError {
        ClassLoadError::InvalidFormat("Error parsing constant fields".to_string())
    }

    fn parse_method_attributes(
        cursor: &mut Cursor<Vec<u8>>,
        attributes_count: u16,
    ) -> Result<Vec<AttributeInfo>, ClassLoadError> {
        let mut result: Vec<AttributeInfo> = Vec::new();
        for _ in 0..attributes_count {
            let (attribute_name_index, attribute_length) =
                (read_2_bytes!(cursor), read_4_bytes!(cursor));

            let mut info: Vec<u8> = Vec::new();
            info.resize(attribute_length as usize, 0);
            cursor.read_exact(&mut info).map_err(map_error)?;
            result.push(AttributeInfo {
                attribute_name_index,
                attribute_length,
                info,
            });
        }
        Ok(result)
    }
    fn parse_method_info(
        cursor: &mut Cursor<Vec<u8>>,
        methods_count: u16,
    ) -> Result<Vec<MethodInfo>, ClassLoadError> {
        let mut result: Vec<MethodInfo> = Vec::new();
        for _ in 0..methods_count {
            let (access_flags, name_index, descriptor_index, attributes_count) = (
                read_2_bytes!(cursor),
                read_2_bytes!(cursor),
                read_2_bytes!(cursor),
                read_2_bytes!(cursor),
            );
            let attributes = parse_method_attributes(cursor, attributes_count)?;
            result.push(MethodInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            });
        }
        Ok(result)
    }

    fn parse_field_info(
        cursor: &mut Cursor<Vec<u8>>,
        fields_count: u16,
    ) -> Result<Vec<FieldInfo>, ClassLoadError> {
        let mut result: Vec<FieldInfo> = Vec::new();
        for _ in 1..fields_count {
            todo!();
            // let (access_flags, name_index, descriptor_index, attribute_count) = (
            //     read_two_bytes!(cursor),
            //     read_two_bytes!(cursor),
            //     read_two_bytes!(cursor),
            //     read_two_bytes!(cursor),
            // );
        }
        Ok(result)
    }
    fn parse_constant_pool_info(
        cursor: &mut Cursor<Vec<u8>>,
        tag: ConstantPoolTag,
    ) -> Result<ConstantPoolPFieldInfo, ClassLoadError> {
        macro_rules! read_two_bytes {
            () => {
                cursor.read_u16::<byteorder::BigEndian>().unwrap()
            };
        }
        macro_rules! read_one_byte {
            () => {
                cursor.read_u8().unwrap()
            };
        }
        match tag {
            ConstantPoolTag::Utf8 => {
                let length = read_two_bytes!();
                let mut bytes: Vec<u8> = vec![0u8; length as usize];
                cursor.read_exact(&mut bytes).map_err(map_error)?;
                Ok(ConstantPoolPFieldInfo::Utf8Info { length, bytes })
            }
            ConstantPoolTag::Integer => {
                Err(ClassLoadError::Other("unimplemented: Integer".to_string()))
            }
            ConstantPoolTag::Float => {
                Err(ClassLoadError::Other("unimplemented: Float".to_string()))
            }
            ConstantPoolTag::Long => Err(ClassLoadError::Other("unimplemented: Long".to_string())),
            ConstantPoolTag::Double => {
                Err(ClassLoadError::Other("unimplemented: Double".to_string()))
            }
            ConstantPoolTag::Class => Ok(ConstantPoolPFieldInfo::ClassInfo {
                // name_index: cursor.read_u16::<byteorder::BigEndian>().unwrap(),
                name_index: read_two_bytes!(),
            }),
            ConstantPoolTag::String => Ok(ConstantPoolPFieldInfo::String {
                string_index: read_two_bytes!(),
            }),
            ConstantPoolTag::Fieldref => Ok(ConstantPoolPFieldInfo::FieldRef(RefFieldInfo {
                class_index: read_two_bytes!(),
                name_and_type_index: read_two_bytes!(),
            })),
            ConstantPoolTag::Methodref => Ok(ConstantPoolPFieldInfo::MethodRef(RefFieldInfo {
                class_index: read_two_bytes!(),
                name_and_type_index: read_two_bytes!(),
            })),
            ConstantPoolTag::InterfaceMethodref => Err(ClassLoadError::Other(
                "unimplemented: InterfaceMethodref".to_string(),
            )),
            ConstantPoolTag::NameAndType => Ok(ConstantPoolPFieldInfo::NameAndType {
                name_index: read_two_bytes!(),
                descriptor_index: read_two_bytes!(),
            }),
            ConstantPoolTag::MethodHandle => Err(ClassLoadError::Other(
                "unimplemented: MethodHandle".to_string(),
            )),
            ConstantPoolTag::MethodType => Err(ClassLoadError::Other(
                "unimplemented: MethodType".to_string(),
            )),
            ConstantPoolTag::Dynamic => {
                Err(ClassLoadError::Other("unimplemented: Dynamic".to_string()))
            }
            ConstantPoolTag::InvokeDynamic => Err(ClassLoadError::Other(
                "unimplemented: InvokeDynamic".to_string(),
            )),
            ConstantPoolTag::Module => {
                Err(ClassLoadError::Other("unimplemented: Module".to_string()))
            }
            ConstantPoolTag::Package => {
                Err(ClassLoadError::Other("unimplemented: Package".to_string()))
            }
        }
    }

    fn parse_constant_pool(
        cursor: &mut Cursor<Vec<u8>>,
        count: u16,
    ) -> Result<Vec<ConstantPoolInfo>, ClassLoadError> {
        let mut constant_pool: Vec<ConstantPoolInfo> = Vec::new();

        let mut next_tag: u8 = 0;
        for _ in 0..count - 1 {
            cursor
                .read_exact(std::slice::from_mut(&mut next_tag))
                .map_err(|e| {
                    ClassLoadError::InvalidFormat(format!(
                        "Failed reading constant pool info: {}",
                        e
                    ))
                })?;
            let tag = ConstantPoolTag::try_from(next_tag).unwrap();
            let info = parse_constant_pool_info(cursor, tag)
                .map_err(|e| ClassLoadError::InvalidFormat("Bad Field info".to_string()))?;
            constant_pool.push(ConstantPoolInfo { tag, info });
        }
        Ok(constant_pool)
    }

    pub fn load(name: &str) -> Result<JavaClass, ClassLoadError> {
        let data = read(name)?;
        parse(data)
    }
}

#[cfg(test)]
mod tests {
    use super::class_loader::*;
    #[test]
    fn test_load_error() {
        let err = load("file.class").unwrap_err();
        println!("Error: {err}");
        assert!(matches!(err, ClassLoadError::NotFound(_)));
    }
    #[test]
    fn test_load() {
        println!("cwd: {}", std::env::current_dir().unwrap().display());
        let res = read("test/Hello.class").unwrap();
        assert_eq!(Vec::len(&res), 420);
    }
    #[test]
    fn test_parse() {
        let j_class = load("test/Hello.class").unwrap();
        println!("Class:\n{j_class}");
        assert_eq!(j_class.magic_number, 0xcafebabe);
        assert_eq!(j_class.minor_version, 0);
        assert_eq!(j_class.major_version, 65);
        assert_eq!(j_class.constant_pool_count, 29);
    }
}
