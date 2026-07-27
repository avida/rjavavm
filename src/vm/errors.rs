pub mod errors {
    use std::error::Error;
    use std::ffi::os_str::Display;
    use std::fmt;
    #[derive(Debug)]
    pub enum RunTimeError {
        Other(String),
        UnknownInstruction(u8),
    }
    impl fmt::Display for RunTimeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                RunTimeError::Other(message) => write!(f, "{}", message),
                RunTimeError::UnknownInstruction(op) => write!(f, "Unknown instruction: 0x{:02x}", op),
            }
        }
    }
    impl Error for RunTimeError {}
}
