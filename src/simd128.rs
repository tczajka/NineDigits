#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "sse2",
    target_feature = "ssse3",
    target_feature = "sse4.1",
)))]
compile_error!("simd module requires SSE4.1");

use std::{
    mem,
    ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};
#[rustfmt::skip]
use std::arch::x86_64::{
    __m128i,
    // SSE2
    _mm_add_epi16,
    _mm_add_epi32,
    _mm_and_si128,
    _mm_andnot_si128,
    _mm_blend_epi16,
    _mm_cmpeq_epi16,
    _mm_cmplt_epi16,
    _mm_cvtsi32_si128,
    _mm_loadu_si128,
    _mm_set1_epi16,
    _mm_set1_epi64x,
    _mm_setr_epi8,
    _mm_shuffle_epi32,
    _mm_slli_epi32,
    _mm_slli_epi64,
    _mm_srli_epi16,
    _mm_srli_epi32,
    _mm_or_si128,
    _mm_setzero_si128,
    _mm_srl_epi64,
    _mm_storeu_si128,
    _mm_unpackhi_epi64,
    _mm_unpacklo_epi64,
    _mm_xor_si128,
    // SSSE3
    _mm_alignr_epi8,
    _mm_shuffle_epi8,
    // SSE4.1
    _mm_blendv_epi8,
    _mm_test_all_zeros,
};
use crate::small::{CartesianProduct, Small};

macro_rules! define_simd_128 {
    ($simd:ident = [$elem:ident; $n:literal]) => {
        #[derive(Copy, Clone, Debug)]
        pub struct $simd(__m128i);

        impl $simd {
            pub fn zero() -> Self {
                Self(unsafe { _mm_setzero_si128() })
            }

            pub fn is_all_zero(self) -> bool {
                unsafe { _mm_test_all_zeros(self.0, self.0) != 0 }
            }

            pub fn and_not(self, rhs: Self) -> Self {
                Self(unsafe { _mm_andnot_si128(rhs.0, self.0) })
            }

            fn single_bit(i: Small<$n>, bit: Small<{ <$elem>::BITS as usize }>) -> Self {
                Self(single_bit_128(Small::<128>::combine(i, bit)))
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
        }

        impl From<[$elem; $n]> for $simd {
            fn from(x: [$elem; $n]) -> Self {
                assert!(mem::size_of::<[$elem; $n]>() == 16);
                Self(unsafe { _mm_loadu_si128(x.as_ptr() as *const __m128i) })
            }
        }

        impl From<$simd> for [$elem; $n] {
            fn from(x: $simd) -> Self {
                assert!(mem::size_of::<[$elem; $n]>() == 16);
                let mut output = [0; $n];
                unsafe { _mm_storeu_si128(output.as_mut_ptr() as *mut __m128i, x.0) };
                output
            }
        }

        impl PartialEq for $simd {
            fn eq(&self, rhs: &Self) -> bool {
                (*self ^ *rhs).is_all_zero()
            }
        }

        impl Eq for $simd {}

        impl BitAnd for $simd {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self {
                Self(unsafe { _mm_and_si128(self.0, rhs.0) })
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
                Self(unsafe { _mm_or_si128(self.0, rhs.0) })
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
                Self(unsafe { _mm_xor_si128(self.0, rhs.0) })
            }
        }

        impl BitXorAssign for $simd {
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs;
            }
        }
    };
}

macro_rules! convert_simd_128 {
    ($from:ident -> $to:ident) => {
        impl From<$from> for $to {
            fn from(x: $from) -> Self {
                Self(x.0)
            }
        }
    };
}

macro_rules! define_all_simd_128 {
    () => {};
    ($simd:ident = $t:tt, $($simd2:ident = $t2:tt,)*) => {
        define_simd_128!($simd = $t);
        $(
            convert_simd_128!($simd -> $simd2);
            convert_simd_128!($simd2 -> $simd);
        )*
        define_all_simd_128!($($simd2 = $t2,)*);
    };
}

define_all_simd_128! {
    Simd16x8 = [u8; 16],
    Simd8x16 = [u16; 8],
    Simd4x32 = [u32; 4],
    Simd2x64 = [u64; 2],
}

impl Simd8x16 {
    pub fn fill(x: u16) -> Self {
        Self(unsafe { _mm_set1_epi16(x as i16) })
    }

    /// Each element is replaced by popcount, under the assumption that inputs are 9-bit.
    pub fn popcount_9(self) -> Self {
        let res = unsafe {
            let popcount_4_table = _mm_setr_epi8(0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4);
            let shr_8_table =
                _mm_setr_epi8(1, -1, 3, -1, 5, -1, 7, -1, 9, -1, 11, -1, 13, -1, 15, -1);
            let mask_04 = _mm_set1_epi16(0b1111);

            let bits_04 = _mm_and_si128(self.0, mask_04);
            let sum_04 = _mm_shuffle_epi8(popcount_4_table, bits_04);
            let bits_48 = _mm_and_si128(_mm_srli_epi16::<4>(self.0), mask_04);
            let sum_48 = _mm_shuffle_epi8(popcount_4_table, bits_48);
            let bit_8 = _mm_shuffle_epi8(self.0, shr_8_table);
            _mm_add_epi16(_mm_add_epi16(sum_04, sum_48), bit_8)
        };
        Self(res)
    }

    pub fn any_lt(self, other: Self) -> bool {
        unsafe {
            let lt = _mm_cmplt_epi16(self.0, other.0);
            _mm_test_all_zeros(lt, lt) == 0
        }
    }

    /// Returns 0xffff for equal values, 0 otherwise.
    pub fn masks_eq(self, other: Self) -> Self {
        Self(unsafe { _mm_cmpeq_epi16(self.0, other.0) })
    }

    /// mask contains 0xffff for entries to replace.
    pub fn replace(self, mask: Self, other: Self) -> Self {
        Self(unsafe { _mm_blendv_epi8(self.0, other.0, mask.0) })
    }

    pub fn replace_top_4_words(self, other: Self) -> Self {
        Self(unsafe { _mm_blend_epi16::<0b11110000>(self.0, other.0) })
    }

    pub fn replace_words_3_mod_4(self, other: Self) -> Self {
        Self(unsafe { _mm_blend_epi16::<0b10001000>(self.0, other.0) })
    }

    /// [self[0..4], other[0..4]]
    pub fn replace_top_4_words_with_bottom(self, other: Self) -> Self {
        Self(unsafe { _mm_unpacklo_epi64(self.0, other.0) })
    }

    /// [other[4..8], self[4..8]]
    pub fn replace_bottom_4_words_with_top(self, other: Self) -> Self {
        Self(unsafe { _mm_unpackhi_epi64(other.0, self.0) })
    }

    /// Rotate every 4 words by 1.
    pub fn rotate_words_1_mod_4(self) -> Self {
        let res = unsafe {
            let shuffle_table = _mm_setr_epi8(6, 7, 0, 1, 2, 3, 4, 5, 14, 15, 8, 9, 10, 11, 12, 13);
            _mm_shuffle_epi8(self.0, shuffle_table)
        };
        Self(res)
    }

    /// Rotate first three of every 4 words by 1.
    pub fn rotate_first_3_words_1_mod_4(self) -> Self {
        let res = unsafe {
            let shuffle_table = _mm_setr_epi8(4, 5, 0, 1, 2, 3, 6, 7, 12, 13, 8, 9, 10, 11, 14, 15);
            _mm_shuffle_epi8(self.0, shuffle_table)
        };
        Self(res)
    }

    /// Shift [self, other] right by 4 words.
    pub fn shift_words_minus_4_with_top(self, other: Self) -> Self {
        Self(unsafe { _mm_alignr_epi8::<8>(other.0, self.0) })
    }

    /// Move words 4*n+i to 4*n+3. Other words become zero.
    pub fn move_words_mod_4(self, from: Small<4>, to: Small<4>) -> Self {
        let res = unsafe {
            // Shift right by from words
            let shift = _mm_cvtsi32_si128(16 * i32::from(u8::from(from)));
            let a = _mm_srl_epi64(self.0, shift);
            // Shift left by 3 words.
            let a = _mm_slli_epi64::<48>(a);
            // Shift right by (3-to) words.
            let shift = _mm_cvtsi32_si128(16 * i32::from(3 - u8::from(to)));
            _mm_srl_epi64(a, shift)
        };
        Self(res)
    }
}

impl Simd4x32 {
    pub fn rotate_bits_7(self) -> Self {
        Self(unsafe { _mm_or_si128(_mm_slli_epi32::<7>(self.0), _mm_srli_epi32::<25>(self.0)) })
    }

    pub fn rotate_bits_8(self) -> Self {
        let from = Simd16x8::from([3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14]);
        Self(unsafe { _mm_shuffle_epi8(self.0, from.0) })
    }

    pub fn rotate_bits_12(self) -> Self {
        Self(unsafe { _mm_or_si128(_mm_slli_epi32::<12>(self.0), _mm_srli_epi32::<20>(self.0)) })
    }

    pub fn rotate_bits_16(self) -> Self {
        let from = Simd16x8::from([2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13]);
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

impl Simd2x64 {
    pub fn fill(x: u64) -> Self {
        Self(unsafe { _mm_set1_epi64x(x as i64) })
    }
}

fn single_bit_128(bit: Small<128>) -> __m128i {
    let (half, b): (Small<2>, Small<64>) = bit.split();
    Simd2x64::zero().insert(half, 1 << u8::from(b)).0
}
