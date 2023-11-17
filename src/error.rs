use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InvalidInput;

impl Display for InvalidInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid input")
    }
}

impl Error for InvalidInput {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResourcesExceeded {
    Time,
    Memory,
}

impl Display for ResourcesExceeded {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ResourcesExceeded::Time => write!(f, "time exceeded"),
            ResourcesExceeded::Memory => write!(f, "memory exceeded"),
        }
    }
}

impl Error for ResourcesExceeded {}
