use crate::small::Small;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Digit(Small<9>);

impl Digit {
    pub fn to_char(self) -> char {
        char::from(b'1' + u8::from(self.0))
    }

    pub fn try_from_char(c: char) -> Option<Self> {
        let val = c as u8;
        if (b'1'..=b'9').contains(&val) {
            let val = Small::<9>::new(val - b'1');
            Some(Self(val))
        } else {
            None
        }
    }
}

impl From<Small<9>> for Digit {
    fn from(val: Small<9>) -> Self {
        Self(val)
    }
}

impl From<Digit> for Small<9> {
    fn from(digit: Digit) -> Self {
        digit.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    pub fn to_char(self) -> char {
        match self.to_digit() {
            Some(digit) => digit.to_char(),
            None => '.',
        }
    }

    pub fn try_from_char(c: char) -> Option<Self> {
        if c == '.' {
            Some(Self::NONE)
        } else {
            let digit = Digit::try_from_char(c)?;
            Some(digit.into())
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
