use crate::*;

/// Msgpackin Error Type
pub enum Error {
    /// Unspecified other error type
    EOther(String),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EOther(s) => {
                f.write_str("EOther: ")?;
                f.write_str(s)
            }
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl StdError for Error {}

/// Msgpackin Result Type
pub type Result<T> = result::Result<T, Error>;
