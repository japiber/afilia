use std::{error, fmt};

#[derive(Debug, Clone)]
pub struct AfiliaError;

impl error::Error for AfiliaError {}

impl fmt::Display for AfiliaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Afilia generic error")
    }
}