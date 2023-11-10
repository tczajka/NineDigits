use std::mem;

use crate::{
    digit::Digit,
    digit_set::DigitSet,
    simd256::Simd16x16,
    small::{CartesianProduct, Small},
};

/// 4x4 box of `u16`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Box4x4x16(Simd16x16);

impl Box4x4x16 {
    pub fn zero() -> Self {
        Self(Simd16x16::zero())
    }

    pub fn set_bit(&mut self, y: Small<4>, x: Small<4>, bit: Small<16>) {
        self.0.set_bit(Small::combine(y, x), bit);
    }

    pub fn clear_bit(&mut self, y: Small<4>, x: Small<4>, bit: Small<16>) {
        self.0.clear_bit(Small::combine(y, x), bit);
    }
}

impl From<[[u16; 4]; 4]> for Box4x4x16 {
    fn from(x: [[u16; 4]; 4]) -> Self {
        let x: [u16; 16] = unsafe { mem::transmute(x) };
        Self(x.into())
    }
}

impl From<Box4x4x16> for [[u16; 4]; 4] {
    fn from(x: Box4x4x16) -> Self {
        let x: [u16; 16] = x.0.into();
        unsafe { mem::transmute(x) }
    }
}

/// 4x4 box of `DigitSet`s.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DigitBox(Box4x4x16);

impl DigitBox {
    pub fn empty() -> Self {
        Self(Box4x4x16::zero())
    }

    pub fn set(&mut self, y: Small<4>, x: Small<4>, digit: Digit) {
        self.0.set_bit(y, x, Small::<9>::from(digit).into());
    }

    pub fn clear(&mut self, y: Small<4>, x: Small<4>, digit: Digit) {
        self.0.clear_bit(y, x, Small::<9>::from(digit).into());
    }

    pub fn counts(self) -> Box4x4x16 {
        Box4x4x16(self.0 .0.popcount_9())
    }
}

impl From<[[DigitSet; 4]; 4]> for DigitBox {
    fn from(x: [[DigitSet; 4]; 4]) -> Self {
        // SAFETY: DigitSet is repr(transparent) over u16.
        let x: [[u16; 4]; 4] = unsafe { mem::transmute(x) };
        Self(x.into())
    }
}

impl From<DigitBox> for [[DigitSet; 4]; 4] {
    fn from(x: DigitBox) -> Self {
        let x: [[u16; 4]; 4] = x.0.into();
        // SAFETY: DigitSet is repr(transparent) over u16.
        unsafe { mem::transmute(x) }
    }
}
