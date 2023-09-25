use super::{digit::Digit, small::Small};

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
}
