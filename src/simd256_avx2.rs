use std::{
    arch::x86_64::{
        __m256i, _mm256_loadu_si256, _mm256_storeu_si256, _mm256_testz_si256, _mm256_xor_si256,
    },
    mem,
    ops::{BitXor, BitXorAssign},
};

#[derive(Clone, Copy, Debug)]
pub struct Simd4x4x16(__m256i);

impl PartialEq for Simd4x4x16 {
    fn eq(&self, rhs: &Self) -> bool {
        (*self ^ *rhs).is_all_zero()
    }
}

impl Eq for Simd4x4x16 {}

impl From<[[u16; 4]; 4]> for Simd4x4x16 {
    fn from(x: [[u16; 4]; 4]) -> Self {
        assert!(mem::size_of::<[[u16; 4]; 4]>() == 32);
        Self(unsafe { _mm256_loadu_si256(x.as_ptr() as *const __m256i) })
    }
}

impl From<Simd4x4x16> for [[u16; 4]; 4] {
    fn from(x: Simd4x4x16) -> Self {
        assert!(mem::size_of::<[[u16; 4]; 4]>() == 32);
        let mut output = [[0; 4]; 4];
        unsafe { _mm256_storeu_si256(output.as_mut_ptr() as *mut __m256i, x.0) };
        output
    }
}

impl Simd4x4x16 {
    pub fn is_all_zero(self) -> bool {
        unsafe { _mm256_testz_si256(self.0, self.0) != 0 }
    }

    pub fn set_bit(&mut self, i: u8, j: u8, bit: u8) {
        todo!()
    }
}

impl BitXor for Simd4x4x16 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Self(unsafe { _mm256_xor_si256(self.0, rhs.0) })
    }
}

impl BitXorAssign for Simd4x4x16 {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}
