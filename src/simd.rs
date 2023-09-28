use std::ops::{Add, AddAssign, BitXor, BitXorAssign};

#[rustfmt::skip]
use std::arch::x86_64::{
    __m128i,
    // SSE2
    _mm_add_epi32,
    _mm_loadu_si128,
    _mm_shuffle_epi32,
    _mm_slli_epi32,
    _mm_srli_epi32,
    _mm_or_si128,
    _mm_storeu_si128,
    _mm_xor_si128,
    // SSSE3
    _mm_shuffle_epi8,
};

#[derive(Clone, Copy, Debug)]
pub struct Simd4x32(__m128i);

impl Simd4x32 {
    pub fn from_le_bytes(x: [u8; 16]) -> Self {
        Self(unsafe { _mm_loadu_si128(x.as_ptr() as *const __m128i) })
    }

    pub fn from_le_u64(x: [u64; 2]) -> Self {
        Self(unsafe { _mm_loadu_si128(x.as_ptr() as *const __m128i) })
    }

    pub fn rotate_bits_7(self) -> Self {
        Self(unsafe { _mm_or_si128(_mm_slli_epi32::<7>(self.0), _mm_srli_epi32::<25>(self.0)) })
    }

    pub fn rotate_bits_8(self) -> Self {
        let from = Self::from_le_bytes([3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14]);
        Self(unsafe { _mm_shuffle_epi8(self.0, from.0) })
    }

    pub fn rotate_bits_12(self) -> Self {
        Self(unsafe { _mm_or_si128(_mm_slli_epi32::<12>(self.0), _mm_srli_epi32::<20>(self.0)) })
    }

    pub fn rotate_bits_16(self) -> Self {
        let from = Self::from_le_bytes([2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13]);
        Self(unsafe { _mm_shuffle_epi8(self.0, from.0) })
    }

    pub fn rotate_words_1(self) -> Self {
        Self(unsafe { _mm_shuffle_epi32::<0b10_01_00_11>(self.0) })
    }

    pub fn rotate_words_2(self) -> Self {
        Self(unsafe { _mm_shuffle_epi32::<0b01_00_11_10>(self.0) })
    }

    pub fn rotate_words_3(self) -> Self {
        Self(unsafe { _mm_shuffle_epi32::<0b00_11_10_01>(self.0) })
    }
}

impl From<[u32; 4]> for Simd4x32 {
    fn from(x: [u32; 4]) -> Self {
        Self(unsafe { _mm_loadu_si128(x.as_ptr() as *const __m128i) })
    }
}

impl From<Simd4x32> for [u32; 4] {
    fn from(x: Simd4x32) -> Self {
        let mut output = [0u32; 4];
        unsafe { _mm_storeu_si128(output.as_mut_ptr() as *mut __m128i, x.0) };
        output
    }
}

impl Add for Simd4x32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(unsafe { _mm_add_epi32(self.0, rhs.0) })
    }
}

impl AddAssign for Simd4x32 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl BitXor for Simd4x32 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Self(unsafe { _mm_xor_si128(self.0, rhs.0) })
    }
}

impl BitXorAssign for Simd4x32 {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}
