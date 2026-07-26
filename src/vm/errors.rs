pub mod errors {
    use std::error::Error;
    use std::ffi::os_str::Display;
    use std::fmt;
    #[derive(Debug)]
    pub enum RunTimeError {
        Other(String),
    }
    impl fmt::Display for RunTimeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                RunTimeError::Other(message) => write!(f, "{}", message),
            }
        }
    }
    impl Error for RunTimeError {}
}
