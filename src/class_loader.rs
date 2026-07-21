pub mod class_loader {
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

        Ok(JavaClass {
            magic_number,
            minor_version,
            major_version,
            constant_pool_count,
        })
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
