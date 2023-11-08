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
    _mm_add_epi32,
    _mm_and_si128,
    _mm_andnot_si128,
    _mm_loadu_si128,
    _mm_shuffle_epi32,
    _mm_slli_epi32,
    _mm_srli_epi32,
    _mm_or_si128,
    _mm_storeu_si128,
    _mm_xor_si128,
    // SSSE3
    _mm_shuffle_epi8,
    // SSE4.1
    _mm_test_all_zeros,
};

use crate::small::Small;

macro_rules! define_simd_128 {
    ($simd:ident = [$elem:ident; $n:literal]) => {
        #[derive(Copy, Clone, Debug)]
        pub struct $simd(__m128i);

        impl $simd {
            pub fn is_all_zero(self) -> bool {
                unsafe { _mm_test_all_zeros(self.0, self.0) != 0 }
            }

            pub fn and_not(self, rhs: Self) -> Self {
                Self(unsafe { _mm_andnot_si128(rhs.0, self.0) })
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
    pub fn set_bit(&mut self, i: Small<8>, bit: Small<16>) {
        todo!()
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
