//! All structures involved in error management. It combines a list a Rust standard library
//! error types, used crates error types and a specific one to the application.
//! Use `map_err` method to report errors with context (see examples in tests).
use std::clone::Clone;
use std::{fmt, io, num};
use serde_json;
use rusqlite::Error;

/// A specific custom `Result` for all functions
pub type AppResult<T> = Result<T, AppError>;

/// Error kind specific to an application error, different from standard errors.
#[derive(Debug, PartialEq)]
pub enum AppCustomErrorKind {
    RepositoryStructure,
    RepositoryMetadata,
    RepositorySign,
    PhantomCloneError
}

impl fmt::Display for AppCustomErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppCustomErrorKind::RepositoryStructure => {
                write!(f, "a repository structure operation issue")
            }
            AppCustomErrorKind::RepositoryMetadata => {
                write!(f, "repository metadata operation issue")
            }
            AppCustomErrorKind::RepositorySign => {
                write!(f, "repository sign issue")
            }
            AppCustomErrorKind::PhantomCloneError => {
                write!(f, "no error")
            }
        }
    }
}

/// A specific error type combining all possible error types in the app.
#[derive(Debug)]
pub enum InternalError {
    Io(io::Error),
    Parse(num::ParseIntError),
    Json(serde_json::Error),
    SystemTime(std::time::SystemTimeError),
    Utf8(std::str::Utf8Error),
    Db(rusqlite::Error),
    Custom(AppCustomErrorKind),
}

/// To simplify definition of all error conversions.
macro_rules! from_error {
    ($e:path, $f:path) => {
        impl From<$e> for InternalError {
            fn from(err: $e) -> InternalError {
                $f(err)
            }
        }
    };
}

from_error!(io::Error, InternalError::Io);
from_error!(serde_json::Error, InternalError::Json);
from_error!(std::time::SystemTimeError, InternalError::SystemTime);
from_error!(num::ParseIntError, InternalError::Parse);
from_error!(std::str::Utf8Error, InternalError::Utf8);
from_error!(rusqlite::Error, InternalError::Db);

/// Custom error which will be used for all errors conversions and throughout the code.
#[derive(Debug)]
pub struct AppError {
    pub error_kind: InternalError,
    pub msg: String,
}

impl AppError {
    /// A simple and convenient creation of a new application error
    pub fn new_custom(kind: AppCustomErrorKind, msg: &str) -> Self {
        AppError {
            error_kind: InternalError::Custom(kind),
            msg: msg.to_string(),
        }
    }

    /// Convert from an internal error
    pub fn from_error<T: Into<InternalError>>(err: T, msg: &str) -> Self {
        AppError {
            error_kind: err.into(),
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.error_kind {
            InternalError::Io(ref err) => write!(f, "I/O error: {} ({})", self.msg, err),
            InternalError::Parse(ref err) => write!(f, "conversion error: {} ({})", self.msg, err),
            InternalError::Json(ref err) => write!(f, "JSON error: {} ({})", self.msg, err),
            InternalError::Utf8(ref err) => {
                write!(f, "Utf8 conversion error: {} ({})", self.msg, err)
            }
            InternalError::SystemTime(ref err) => {
                write!(f, "system time error: {} ({})", self.msg, err)
            }
            InternalError::Db(ref err) => {
                write!(f, "database error: {} ({})", self.msg, err)
            }
            InternalError::Custom(ref err) => write!(f, "custom error: {} ({})", self.msg, err),
        }
    }
}


impl Clone for AppError {
    fn clone(&self) -> Self {
        AppError::new_custom(AppCustomErrorKind::PhantomCloneError, "fake clone error")
    }
}

/// To simplify definition of all error conversions.
#[macro_export]
macro_rules! context {
    ($err:ident, $fmt:expr, $($arg:tt)*) => {
        AppError::from_error(
            $err,
            &format!($fmt, $($arg)*)
        )

    };
}
