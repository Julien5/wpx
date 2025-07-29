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
