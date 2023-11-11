use crate::{
    simd128::{Simd16x8, Simd2x64, Simd4x32, Simd8x16},
    small::{CartesianProduct, Small},
};
use std::{
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

macro_rules! define_simd_256 {
    ($simd:ident = [$elem:ident; $n:literal] = [$half:ident; 2]) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $simd([$half; 2]);

        impl $simd {
            pub fn zero() -> Self {
                Self([$half::zero(); 2])
            }

            pub fn is_all_zero(self) -> bool {
                self.0[0].is_all_zero() && self.0[1].is_all_zero()
            }

            pub fn and_not(self, rhs: Self) -> Self {
                Self([self.0[0].and_not(rhs.0[0]), self.0[1].and_not(rhs.0[1])])
            }

            pub fn set_bit(&mut self, i: Small<$n>, bit: Small<{ <$elem>::BITS as usize }>) {
                let (half, j): (Small<2>, Small<{ $n / 2 }>) = i.split();
                self.0[half].set_bit(j, bit);
            }

            pub fn clear_bit(&mut self, i: Small<$n>, bit: Small<{ <$elem>::BITS as usize }>) {
                let (half, j): (Small<2>, Small<{ $n / 2 }>) = i.split();
                self.0[half].clear_bit(j, bit);
            }
        }

        impl From<[$elem; $n]> for $simd {
            fn from(x: [$elem; $n]) -> Self {
                let x: [[$elem; $n / 2]; 2] = unsafe { mem::transmute(x) };
                Self([<$half>::from(x[0]), <$half>::from(x[1])])
            }
        }

        impl From<$simd> for [$elem; $n] {
            fn from(x: $simd) -> Self {
                let x: [[$elem; $n / 2]; 2] = [x.0[0].into(), x.0[1].into()];
                unsafe { mem::transmute(x) }
            }
        }

        impl BitAnd for $simd {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self {
                Self([self.0[0] & rhs.0[0], self.0[1] & rhs.0[1]])
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
                Self([self.0[0] | rhs.0[0], self.0[1] | rhs.0[1]])
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
                Self([self.0[0] ^ rhs.0[0], self.0[1] ^ rhs.0[1]])
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
                Self([x.0[0].into(), x.0[1].into()])
            }
        }
    };
}

macro_rules! define_all_simd_256 {
    () => {};
    ($simd:ident = $t1:tt = $t2:tt, $($simd2:ident = $u1:tt = $u2:tt,)*) => {
        define_simd_256!($simd = $t1 = $t2);
        $(
            convert_simd_256!($simd -> $simd2);
            convert_simd_256!($simd2 -> $simd);
        )*
        define_all_simd_256!($($simd2 = $u1 = $u2,)*);
    };
}

define_all_simd_256! {
    Simd32x8 = [u8; 32] = [Simd16x8; 2],
    Simd16x16 = [u16; 16] = [Simd8x16; 2],
    Simd8x32 = [u32; 8] = [Simd4x32; 2],
    Simd4x64 = [u64; 4] = [Simd2x64; 2],
}

impl Simd16x16 {
    pub fn popcount_9(self) -> Self {
        Self([self.0[0].popcount_9(), self.0[1].popcount_9()])
    }

    pub fn any_lt(self, other: Self) -> bool {
        self.0[0].any_lt(other.0[0]) || self.0[1].any_lt(other.0[1])
    }

    /// Returns 0xffff for equal values, 0 otherwise.
    pub fn masks_eq(self, other: Self) -> Self {
        Self([
            self.0[0].masks_eq(other.0[0]),
            self.0[1].masks_eq(other.0[1]),
        ])
    }

    /// mask contains 0xffff for entries to replace.
    pub fn replace(self, mask: Self, other: Self) -> Self {
        Self([
            self.0[0].replace(mask.0[0], other.0[0]),
            self.0[1].replace(mask.0[1], other.0[1]),
        ])
    }

    /// Rotate every 4 words by 1.
    pub fn rotate_words_1_mod_4(self) -> Self {
        Self([
            self.0[0].rotate_words_1_mod_4(),
            self.0[1].rotate_words_1_mod_4(),
        ])
    }

    /// Rotate words by 4.
    pub fn rotate_words_4(self) -> Self {
        Self([
            self.0[1].shift_words_minus_4_with_top(self.0[0]),
            self.0[0].shift_words_minus_4_with_top(self.0[1]),
        ])
    }
}
