use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

pub trait Bits:
    Sized
    + Copy
    + Eq
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr<Output = Self>
    + BitOrAssign
{
    const BITS: u8;
    const ZERO: Self;

    fn single_bit(n: u8) -> Self;
    fn low_bits(n: u8) -> Self;
    fn count_ones(self) -> u8;
    fn trailing_zeros(self) -> u8;
}

macro_rules! impl_bits {
    ($t:ty) => {
        impl Bits for $t {
            const BITS: u8 = Self::BITS as u8;
            const ZERO: Self = 0;

            fn single_bit(n: u8) -> Self {
                1 << n
            }

            fn low_bits(n: u8) -> Self {
                let limit = <Self as Bits>::BITS;
                assert!(n <= limit);
                if n == limit {
                    Self::MAX
                } else {
                    (1 << n) - 1
                }
            }

            fn count_ones(self) -> u8 {
                self.count_ones() as u8
            }

            fn trailing_zeros(self) -> u8 {
                self.trailing_zeros() as u8
            }
        }
    };
}

impl_bits!(u8);
impl_bits!(u16);
impl_bits!(u32);
impl_bits!(u64);
impl_bits!(u128);
