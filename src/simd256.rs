use crate::simd::Simd8x16;
use std::{mem, ops::BitXor, ops::BitXorAssign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Simd4x4x16([Simd8x16; 2]);

impl From<[[u16; 4]; 4]> for Simd4x4x16 {
    fn from(x: [[u16; 4]; 4]) -> Self {
        let x: [[u16; 8]; 2] = unsafe { mem::transmute(x) };
        Self([Simd8x16::from(x[0]), Simd8x16::from(x[1])])
    }
}

impl From<Simd4x4x16> for [[u16; 4]; 4] {
    fn from(x: Simd4x4x16) -> Self {
        let x: [[u16; 8]; 2] = [x.0[0].into(), x.0[1].into()];
        unsafe { mem::transmute(x) }
    }
}

impl Simd4x4x16 {
    pub fn is_all_zero(self) -> bool {
        self.0[0].is_all_zero() && self.0[1].is_all_zero()
    }
}

impl BitXor for Simd4x4x16 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Self([self.0[0] ^ rhs.0[0], self.0[1] ^ rhs.0[1]])
    }
}

impl BitXorAssign for Simd4x4x16 {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}
