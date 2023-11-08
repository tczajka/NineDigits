use std::mem;

use crate::{digit::Digit, digit_set::DigitSet, simd256::Simd4x4x16, small::Small};

/// 4x4 box of `DigitSet`s.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DigitBox(Simd4x4x16);

impl DigitBox {
    pub fn empty() -> Self {
        Self::from([[DigitSet::EMPTY; 4]; 4])
    }

    pub fn set(&mut self, y: Small<4>, x: Small<4>, digit: Digit) {
        self.0.set_bit(y, x, Small::<9>::from(digit).into());
    }
}

impl From<[[DigitSet; 4]; 4]> for DigitBox {
    fn from(x: [[DigitSet; 4]; 4]) -> Self {
        let x: [[u16; 4]; 4] = unsafe { mem::transmute(x) };
        Self(Simd4x4x16::from(x))
    }
}

impl From<DigitBox> for [[DigitSet; 4]; 4] {
    fn from(x: DigitBox) -> Self {
        let x: [[u16; 4]; 4] = x.0.into();
        unsafe { mem::transmute(x) }
    }
}
