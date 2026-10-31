//! Error type shared across gcal-core.

use std::fmt;

/// Domain error of the calendar core.
#[derive(Debug)]
pub enum Error {
    Sql(rusqlite::Error),
    Rrule(String),
    Validation(String),
    NotFound(String),
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Sql(e) => write!(f, "database error: {e}"),
            Error::Rrule(s) => write!(f, "recurrence error: {s}"),
            Error::Validation(s) => write!(f, "validation error: {s}"),
            Error::NotFound(s) => write!(f, "not found: {s}"),
            Error::Internal(s) => write!(f, "internal error: {s}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Sql(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
