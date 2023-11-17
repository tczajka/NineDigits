use crate::{error::InvalidInput, small::Small};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Digit(Small<9>);

impl Digit {
    pub fn all() -> impl Iterator<Item = Self> {
        Small::all().map(Self)
    }
}

impl From<Digit> for Small<9> {
    fn from(digit: Digit) -> Self {
        digit.0
    }
}

impl From<Small<9>> for Digit {
    fn from(val: Small<9>) -> Self {
        Self(val)
    }
}

impl From<Digit> for char {
    fn from(digit: Digit) -> char {
        char::from(b'1' + u8::from(digit.0))
    }
}

impl TryFrom<char> for Digit {
    type Error = InvalidInput;

    fn try_from(c: char) -> Result<Digit, InvalidInput> {
        let val = u8::try_from(c).map_err(|_| InvalidInput)?;
        if (b'1'..=b'9').contains(&val) {
            let val = Small::<9>::new(val - b'1');
            Ok(Self(val))
        } else {
            Err(InvalidInput)
        }
    }
}

impl Display for Digit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = char::from(*self);
        write!(f, "{c}")
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]

pub struct OptionalDigit(Small<10>);

impl OptionalDigit {
    pub const NONE: Self = Self(Small::new(9));

    pub fn to_digit(self) -> Option<Digit> {
        if self == Self::NONE {
            None
        } else {
            let val = u8::from(self.0);
            // SAFETY: `val < 9` because it's not NONE.
            let val = unsafe { Small::<9>::new_unchecked(val) };
            Some(Digit(val))
        }
    }
}

impl From<Digit> for OptionalDigit {
    fn from(digit: Digit) -> Self {
        let val = u8::from(digit.0);
        // SAFETY: val < 9 < 10.
        let val = unsafe { Small::<10>::new_unchecked(val) };
        Self(val)
    }
}

impl From<OptionalDigit> for char {
    fn from(digit: OptionalDigit) -> char {
        match digit.to_digit() {
            Some(digit) => digit.into(),
            None => '0',
        }
    }
}

impl TryFrom<char> for OptionalDigit {
    type Error = InvalidInput;

    fn try_from(c: char) -> Result<Self, InvalidInput> {
        if c == '0' || c == '.' {
            Ok(Self::NONE)
        } else {
            let digit = Digit::try_from(c)?;
            Ok(digit.into())
        }
    }
}

impl Display for OptionalDigit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = char::from(*self);
        write!(f, "{c}")
    }
}
