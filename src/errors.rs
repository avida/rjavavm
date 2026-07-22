pub mod errors {
    use std::fmt;

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

    pub fn map_error(e: std::io::Error) -> ClassLoadError {
        ClassLoadError::InvalidFormat(format!("Error: {}", e))
    }

}
