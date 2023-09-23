use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InvalidInput;

impl Display for InvalidInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid input")
    }
}
