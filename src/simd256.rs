use crate::{
    simd::Simd8x16,
    small::{CartesianProduct, Small},
};
use std::{
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

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

    pub fn and_not(self, rhs: Self) -> Self {
        Self([self.0[0].and_not(rhs.0[0]), self.0[1].and_not(rhs.0[1])])
    }

    pub fn set_bit(&mut self, i: Small<4>, j: Small<4>, bit: Small<16>) {
        let (i0, i1): (Small<2>, Small<2>) = i.split();
        let j1: Small<8> = Small::combine(i1, j);
        self.0[i0].set_bit(j1, bit);
    }

    pub fn clear_bit(&mut self, i: Small<4>, j: Small<4>, bit: Small<16>) {
        let (i0, i1): (Small<2>, Small<2>) = i.split();
        let j1: Small<8> = Small::combine(i1, j);
        self.0[i0].clear_bit(j1, bit);
    }
}

impl BitAnd for Simd4x4x16 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self([self.0[0] & rhs.0[0], self.0[1] & rhs.0[1]])
    }
}

impl BitAndAssign for Simd4x4x16 {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for Simd4x4x16 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self([self.0[0] | rhs.0[0], self.0[1] | rhs.0[1]])
    }
}

impl BitOrAssign for Simd4x4x16 {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
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
