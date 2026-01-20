use std::{fmt, usize};

#[derive(Debug, Clone)]
pub enum Error {
    GPXNotFound,
    GPXInvalid,
    GPXHasNoSegment,
    MissingElevation { index: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GPXNotFound => write!(f, "GPX file not found"),
            Error::GPXInvalid => write!(f, "GPX file is invalid"),
            Error::MissingElevation { index } => {
                write!(f, "{}", format!("missing elevation at index {}", index))
            }
            Error::GPXHasNoSegment => write!(f, "GPX file has no segment"),
        }
    }
}

impl std::error::Error for Error {}

pub type GenericError = Box<dyn std::error::Error>;
pub type GenericResult<T> = Result<T, GenericError>;

#[derive(Debug)]
pub struct StringError(pub String);
// Implement Display so the error can be printed
impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for StringError {}
