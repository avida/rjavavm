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

        Ok(JavaClass {
            magic_number,
            minor_version,
            major_version,
            constant_pool_count,
            constant_pool,
        })
    }

    fn map_error(_: std::io::Error) -> ClassLoadError {
        ClassLoadError::InvalidFormat("Error parsing constant fields".to_string())
    }
    fn parse_constant_pool_info(
        cursor: &mut Cursor<Vec<u8>>,
        tag: ConstantPoolTag,
    ) -> Result<FieldInfo, ClassLoadError> {
        macro_rules! read_two_bytes {
            () => {
                cursor.read_u16::<byteorder::BigEndian>().unwrap()
            };
        }
        match tag {
            ConstantPoolTag::Utf8 => {
                let length = read_two_bytes!();
                let mut bytes: Vec<u8> = vec![0u8; length as usize];
                cursor.read_exact(&mut bytes).map_err(map_error)?;
                Ok(FieldInfo::Utf8Info { length, bytes })
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
            ConstantPoolTag::Class => Ok(FieldInfo::ClassInfo {
                // name_index: cursor.read_u16::<byteorder::BigEndian>().unwrap(),
                name_index: read_two_bytes!(),
            }),
            ConstantPoolTag::String => {
                Err(ClassLoadError::Other("unimplemented: String".to_string()))
            }
            ConstantPoolTag::Fieldref => {
                Err(ClassLoadError::Other("unimplemented: Fieldref".to_string()))
            }
            ConstantPoolTag::Methodref => {
                // let mut buf = [0u8; std::mem::size_of::<RefFieldInfo>()];
                // cursor.read_exact(&mut buf).map_err(map_error)?;
                Ok(FieldInfo::MethodRef(RefFieldInfo {
                    class_index: read_two_bytes!(),
                    name_and_type_index: read_two_bytes!(),
                }))
            }
            ConstantPoolTag::InterfaceMethodref => Err(ClassLoadError::Other(
                "unimplemented: InterfaceMethodref".to_string(),
            )),
            ConstantPoolTag::NameAndType => Ok(FieldInfo::NameAndType {
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
            println!("next Tag is {}", next_tag);
            let tag = ConstantPoolTag::try_from(next_tag).unwrap();
            println!("Tag is {}", tag);
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
