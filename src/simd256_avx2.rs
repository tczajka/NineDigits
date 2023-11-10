use std::{
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};
#[rustfmt::skip]
use std::arch::x86_64::{
    __m256i,
    _mm_setr_epi8,
    _mm256_add_epi16,
    _mm256_and_si256,
    _mm256_andnot_si256,
    _mm256_blendv_epi8,
    _mm256_cmpeq_epi16,
    _mm256_cmpgt_epi16,
    _mm256_loadu_si256,
    _mm256_or_si256,
    _mm256_set1_epi16,
    _mm256_setr_m128i,
    _mm256_setzero_si256,
    _mm256_shuffle_epi8,
    _mm256_srli_epi16,
    _mm256_storeu_si256,
    _mm256_testz_si256,
    _mm256_xor_si256,
};
use crate::small::{CartesianProduct, Small};

// Include the emulated code just so it gets compiled.
#[path = "simd256.rs"]
mod simd256_noavx2;

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
            let lt = _mm256_cmpgt_epi16(other.0, self.0);
            _mm256_testz_si256(lt, lt) == 0
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
}

fn single_bit_256(bit: Small<256>) -> __m256i {
    let (i, b): (Small<4>, Small<64>) = bit.split();
    let mut val = [0u64; 4];
    val[i] = 1 << u8::from(b);
    Simd4x64::from(val).0
}
