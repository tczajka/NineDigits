use std::{
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};
#[rustfmt::skip]
use std::arch::x86_64::{
    // SSE2
    _mm_cvtsi32_si128,
    _mm_setr_epi8,
    // AVX2
    __m256i,
    _mm256_add_epi16,
    _mm256_and_si256,
    _mm256_andnot_si256,
    _mm256_blend_epi32,
    _mm256_blend_epi16,
    _mm256_blendv_epi8,
    _mm256_cmpeq_epi16,
    _mm256_cmpeq_epi8,
    _mm256_loadu_si256,
    _mm256_max_epu16,
    _mm256_movemask_epi8,
    _mm256_or_si256,
    _mm256_permute4x64_epi64,
    _mm256_set1_epi16,
    _mm256_set1_epi64x,
    _mm256_setr_m128i,
    _mm256_setzero_si256,
    _mm256_shuffle_epi8,
    _mm256_slli_epi64,
    _mm256_srl_epi64,
    _mm256_srli_epi16,
    _mm256_storeu_si256,
    _mm256_testc_si256,
    _mm256_testz_si256,
    _mm256_xor_si256,
};
use crate::small::{CartesianProduct, Small};

macro_rules! define_simd_256 {
    ($simd:ident = [$elem:ident; $n:literal]) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $simd(__m256i);

        impl PartialEq for $simd {
            fn eq(&self, rhs: &Self) -> bool {
                (*self ^ *rhs).is_all_zero()
            }
        }

        impl Eq for $simd {}

        impl $simd {
            pub fn zero() -> Self {
                Self(unsafe { _mm256_setzero_si256() })
            }

            pub fn is_all_zero(self) -> bool {
                unsafe { _mm256_testz_si256(self.0, self.0) != 0 }
            }

            pub fn and_not(self, rhs: Self) -> Self {
                Self(unsafe { _mm256_andnot_si256(rhs.0, self.0) })
            }

            fn single_bit(i: Small<$n>, bit: Small<{ <$elem>::BITS as usize }>) -> Self {
                Self(single_bit_256(Small::<256>::combine(i, bit)))
            }

            pub fn set_bit(&mut self, i: Small<$n>, bit: Small<{ <$elem>::BITS as usize }>) {
                *self |= Self::single_bit(i, bit);
            }

            pub fn clear_bit(&mut self, i: Small<$n>, bit: Small<{ <$elem>::BITS as usize }>) {
                *self = self.and_not(Self::single_bit(i, bit));
            }

            pub fn extract(self, index: Small<$n>) -> $elem {
                let a: [$elem; $n] = self.into();
                a[index]
            }

            pub fn insert(self, index: Small<$n>, val: $elem) -> Self {
                let mut a: [$elem; $n] = self.into();
                a[index] = val;
                a.into()
            }

            pub fn first_bit(self) -> Option<(Small<$n>, Small<{ <$elem>::BITS as usize }>)> {
                let bit = first_bit_256(self.0)?;
                Some(Small::split(bit))
            }
        }

        impl From<[$elem; $n]> for $simd {
            fn from(x: [$elem; $n]) -> Self {
                assert!(mem::size_of::<[$elem; $n]>() == 32);
                Self(unsafe { _mm256_loadu_si256(x.as_ptr() as *const __m256i) })
            }
        }

        impl From<$simd> for [$elem; $n] {
            fn from(x: $simd) -> Self {
                assert!(mem::size_of::<[$elem; $n]>() == 32);
                let mut output = [0; $n];
                unsafe { _mm256_storeu_si256(output.as_mut_ptr() as *mut __m256i, x.0) };
                output
            }
        }

        impl BitAnd for $simd {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self {
                Self(unsafe { _mm256_and_si256(self.0, rhs.0) })
            }
        }

        impl BitAndAssign for $simd {
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs;
            }
        }

        impl BitOr for $simd {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self {
                Self(unsafe { _mm256_or_si256(self.0, rhs.0) })
            }
        }

        impl BitOrAssign for $simd {
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs;
            }
        }

        impl BitXor for $simd {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self {
                Self(unsafe { _mm256_xor_si256(self.0, rhs.0) })
            }
        }

        impl BitXorAssign for $simd {
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs;
            }
        }
    };
}

macro_rules! convert_simd_256 {
    ($from:ident -> $to:ident) => {
        impl From<$from> for $to {
            fn from(x: $from) -> Self {
                Self(x.0)
            }
        }
    };
}

macro_rules! define_all_simd_256 {
    () => {};
    ($simd:ident = $t:tt, $($simd2:ident = $u:tt,)*) => {
        define_simd_256!($simd = $t);
        $(
            convert_simd_256!($simd -> $simd2);
            convert_simd_256!($simd2 -> $simd);
        )*
        define_all_simd_256!($($simd2 = $u,)*);
    };
}

define_all_simd_256! {
    Simd32x8 = [u8; 32],
    Simd16x16 = [u16; 16],
    Simd8x32 = [u32; 8],
    Simd4x64 = [u64; 4],
}

impl Simd16x16 {
    pub fn fill(x: u16) -> Self {
        Self(unsafe { _mm256_set1_epi16(x as i16) })
    }

    pub fn popcount_9(self) -> Self {
        let res = unsafe {
            let popcount_4_table_128 =
                _mm_setr_epi8(0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4);
            let popcount_4_table = _mm256_setr_m128i(popcount_4_table_128, popcount_4_table_128);
            let shr_8_table_128 =
                _mm_setr_epi8(1, -1, 3, -1, 5, -1, 7, -1, 9, -1, 11, -1, 13, -1, 15, -1);
            let shr_8_table = _mm256_setr_m128i(shr_8_table_128, shr_8_table_128);
            let mask_04 = _mm256_set1_epi16(0b1111);

            let bits_04 = _mm256_and_si256(self.0, mask_04);
            let sum_04 = _mm256_shuffle_epi8(popcount_4_table, bits_04);
            let bits_48 = _mm256_and_si256(_mm256_srli_epi16::<4>(self.0), mask_04);
            let sum_48 = _mm256_shuffle_epi8(popcount_4_table, bits_48);
            let bit_8 = _mm256_shuffle_epi8(self.0, shr_8_table);
            _mm256_add_epi16(_mm256_add_epi16(sum_04, sum_48), bit_8)
        };
        Self(res)
    }

    pub fn any_lt(self, other: Self) -> bool {
        unsafe {
            let max = _mm256_max_epu16(self.0, other.0);
            let ge = _mm256_cmpeq_epi16(self.0, max);
            let ones = _mm256_cmpeq_epi16(max, max);
            // Test that ge is all ones.
            _mm256_testc_si256(ge, ones) == 0
        }
    }

    /// Returns 0xffff for equal values, 0 otherwise.
    pub fn masks_eq(self, other: Self) -> Self {
        Self(unsafe { _mm256_cmpeq_epi16(self.0, other.0) })
    }

    /// mask contains 0xffff for entries to replace.
    pub fn replace(self, mask: Self, other: Self) -> Self {
        Self(unsafe { _mm256_blendv_epi8(self.0, other.0, mask.0) })
    }

    pub fn replace_top_4_words(self, other: Self) -> Self {
        Self(unsafe { _mm256_blend_epi32::<0b11_00_00_00>(self.0, other.0) })
    }

    pub fn replace_words_3_mod_4(self, other: Self) -> Self {
        Self(unsafe { _mm256_blend_epi16::<0b10001000>(self.0, other.0) })
    }

    /// Rotate every 4 words by 1.
    pub fn rotate_words_1_mod_4(self) -> Self {
        let res = unsafe {
            let shuffle_table_128 =
                _mm_setr_epi8(6, 7, 0, 1, 2, 3, 4, 5, 14, 15, 8, 9, 10, 11, 12, 13);
            let shuffle_table = _mm256_setr_m128i(shuffle_table_128, shuffle_table_128);
            _mm256_shuffle_epi8(self.0, shuffle_table)
        };
        Self(res)
    }

    /// Rotate words by 4.
    pub fn rotate_words_4(self) -> Self {
        Self(unsafe { _mm256_permute4x64_epi64::<0b10_01_00_11>(self.0) })
    }

    /// Rotate first three of every 4 words by 1.
    pub fn rotate_first_3_words_1_mod_4(self) -> Self {
        let res = unsafe {
            let shuffle_table_128 =
                _mm_setr_epi8(4, 5, 0, 1, 2, 3, 6, 7, 12, 13, 8, 9, 10, 11, 14, 15);
            let shuffle_table = _mm256_setr_m128i(shuffle_table_128, shuffle_table_128);
            _mm256_shuffle_epi8(self.0, shuffle_table)
        };
        Self(res)
    }

    /// Rotate first 12 words by 4.
    pub fn rotate_first_12_words_4(self) -> Self {
        Self(unsafe { _mm256_permute4x64_epi64::<0b11_01_00_10>(self.0) })
    }

    /// Move words [4*from..4*from+4] to [4*to..4*to+4]. Other words become zero.
    pub fn move_4_words(self, from: Small<4>, to: Small<4>) -> Self {
        let a = Simd4x64::from(self).extract(from);
        Simd4x64::zero().insert(to, a).into()
    }

    /// Move words 4*n+from to 4*n+to. Other words become zero.
    pub fn move_words_mod_4(self, from: Small<4>, to: Small<4>) -> Self {
        let res = unsafe {
            // Shift right by from words
            let shift = _mm_cvtsi32_si128(16 * i32::from(u8::from(from)));
            let a = _mm256_srl_epi64(self.0, shift);
            // Shift left by 3 words.
            let a = _mm256_slli_epi64::<48>(a);
            // Shift right by (3 - to) words.
            let shift = _mm_cvtsi32_si128(16 * i32::from(3 - u8::from(to)));
            _mm256_srl_epi64(a, shift)
        };
        Self(res)
    }
}

impl Simd4x64 {
    pub fn fill(x: u64) -> Self {
        Self(unsafe { _mm256_set1_epi64x(x as i64) })
    }
}

fn single_bit_256(bit: Small<256>) -> __m256i {
    let (i, b): (Small<4>, Small<64>) = bit.split();
    Simd4x64::zero().insert(i, 1 << u8::from(b)).0
}

fn first_bit_256(a: __m256i) -> Option<Small<256>> {
    let first_byte: Small<32> = unsafe {
        let zero_mask = _mm256_cmpeq_epi8(a, _mm256_setzero_si256());
        let zero_bytes: u32 = _mm256_movemask_epi8(zero_mask) as u32;
        if zero_bytes == 0xffffffff {
            return None;
        }
        // SAFETY: trailing_ones is in 0..32.
        Small::new_unchecked(zero_bytes.trailing_ones() as u8)
    };
    let byte = Simd32x8(a).extract(first_byte);
    // SAFETY: byte is non-zero, trailing_zeros is 0..8.
    let first_bit: Small<8> = unsafe { Small::new_unchecked(byte.trailing_zeros() as u8) };
    Some(Small::combine(first_byte, first_bit))
}
