use crate::*;

/// Msgpackin Error Type
pub enum Error {
    /// InvalidUtf8 data
    EInvalidUtf8,

    /// Decode Error
    EDecode {
        /// What was expected during decode
        expected: String,

        /// What was received during decode
        got: String,
    },

    /// (`feature = "std"`)
    /// std::io::Error
    #[cfg(feature = "std")]
    EStdIo(std::io::Error),

    /// Unspecified other error type
    EOther(String),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EInvalidUtf8 => f.write_str("EInvalidUtf8"),
            Error::EDecode { expected, got } => {
                write!(f, "EDecode(expected: {}, got: {})", expected, got)
            }
            #[cfg(feature = "std")]
            Error::EStdIo(e) => {
                write!(f, "EStdIo({:?})", e)
            }
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

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::EStdIo(e)
    }
}

impl StdError for Error {}

/// Msgpackin Result Type
pub type Result<T> = result::Result<T, Error>;
