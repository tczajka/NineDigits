use std::{
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};
#[rustfmt::skip]
use std::arch::x86_64::{
    __m256i,
    _mm256_and_si256,
    _mm256_andnot_si256,
    _mm256_loadu_si256,
    _mm256_or_si256,
    _mm256_setzero_si256,
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

fn single_bit_256(bit: Small<256>) -> __m256i {
    let (i, b): (Small<4>, Small<64>) = bit.split();
    let mut val = [0u64; 4];
    val[i] = 1 << u8::from(b);
    Simd4x64::from(val).0
}
