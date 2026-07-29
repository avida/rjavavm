pub mod errors {
    use std::error::Error;
    use std::fmt;
    #[derive(Debug)]
    pub enum RunTimeError {
        Other(String),
        UnknownInstruction(u8),
        Notimplemented(String),
    }
    impl fmt::Display for RunTimeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                RunTimeError::Other(message) => write!(f, "{}", message),
                        RunTimeError::UnknownInstruction(op) => {
                            write!(f, "Unknown instruction: 0x{:02x}", op)
                        },
                        RunTimeError::Notimplemented(msg) => write!(f, "Not implemented: {}", msg),
            }
        }
    }
    impl Error for RunTimeError {}
}
