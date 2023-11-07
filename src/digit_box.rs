use std::mem;

use crate::{digit_set::DigitSet, simd256::Simd4x4x16};

/// 4x4 box of `DigitSet`s.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DigitBox(Simd4x4x16);

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
