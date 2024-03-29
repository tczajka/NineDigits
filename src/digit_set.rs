use crate::{
    digit::Digit,
    error::InvalidInput,
    small_set::{SmallSet, SmallSetIterator},
};
use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
    str::FromStr,
};

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct DigitSet(SmallSet<9, u16>);

impl DigitSet {
    pub const EMPTY: Self = Self(SmallSet::EMPTY);

    pub fn all() -> Self {
        Self(SmallSet::all())
    }

    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    pub fn only(digit: Digit) -> Self {
        Self(SmallSet::only(digit.into()))
    }

    pub fn contains(self, digit: Digit) -> bool {
        self.0.contains(digit.into())
    }

    pub fn insert(&mut self, digit: Digit) {
        self.0.insert(digit.into())
    }

    pub fn remove(&mut self, digit: Digit) {
        self.0.remove(digit.into())
    }

    pub fn and_not(self, other: Self) -> Self {
        Self(self.0.and_not(other.0))
    }

    pub fn size(self) -> u8 {
        self.0.size()
    }

    pub fn smallest(self) -> Option<Digit> {
        self.0.smallest().map(|x| x.into())
    }
}

impl IntoIterator for DigitSet {
    type Item = Digit;
    type IntoIter = DigitSetIterator;

    fn into_iter(self) -> DigitSetIterator {
        DigitSetIterator(self.0.into_iter())
    }
}

#[derive(Clone, Debug)]
pub struct DigitSetIterator(SmallSetIterator<9, u16>);

impl Iterator for DigitSetIterator {
    type Item = Digit;

    fn next(&mut self) -> Option<Digit> {
        self.0.next().map(|x| x.into())
    }
}

impl Display for DigitSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for digit in *self {
            write!(f, "{digit}")?;
        }
        Ok(())
    }
}

impl Debug for DigitSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

impl FromStr for DigitSet {
    type Err = InvalidInput;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut set = Self::EMPTY;
        for c in s.chars() {
            let digit = Digit::try_from(c)?;
            if set.contains(digit) {
                return Err(InvalidInput);
            }
            set.insert(digit);
        }
        Ok(set)
    }
}

impl BitAnd for DigitSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for DigitSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for DigitSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DigitSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
