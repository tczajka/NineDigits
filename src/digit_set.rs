use super::{digit::Digit, small::Small};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DigitSet(u16);

impl DigitSet {
    pub const EMPTY: DigitSet = DigitSet(0);
    pub const ALL: DigitSet = DigitSet(0x1ff);

    pub fn insert(&mut self, digit: Digit) {
        self.0 |= 1 << u8::from(Small::<9>::from(digit));
    }

    pub fn remove(&mut self, digit: Digit) {
        self.0 &= !(1 << u8::from(Small::<9>::from(digit)));
    }

    pub fn smallest(self) -> Option<Digit> {
        if self == DigitSet::EMPTY {
            return None;
        }
        let val = self.0.trailing_zeros() as u8;
        // SAFETY: `res` is in the range `0..9` because `self` is not empty.
        let small = unsafe { Small::<9>::new_unchecked(val) };
        Some(small.into())
    }
}

impl IntoIterator for DigitSet {
    type Item = Digit;
    type IntoIter = DigitSetIterator;

    fn into_iter(self) -> DigitSetIterator {
        DigitSetIterator(self)
    }
}

#[derive(Clone, Debug)]
pub struct DigitSetIterator(DigitSet);

impl Iterator for DigitSetIterator {
    type Item = Digit;

    fn next(&mut self) -> Option<Digit> {
        let res = self.0.smallest()?;
        self.0.remove(res);
        Some(res)
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
