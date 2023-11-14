use crate::{bits::Bits, small::Small};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct SmallSet<const L: usize, T>(T);

impl<const L: usize, T: Bits> SmallSet<L, T> {
    pub const EMPTY: Self = Self(T::ZERO);

    pub fn all() -> Self {
        Self(T::low_bits(L as u8))
    }

    pub fn is_empty(self) -> bool {
        self.0 == T::ZERO
    }

    pub fn only(x: Small<L>) -> Self {
        Self(T::single_bit(u8::from(x)))
    }

    pub fn contains(self, x: Small<L>) -> bool {
        !(self & Self::only(x)).is_empty()
    }

    pub fn insert(&mut self, x: Small<L>) {
        *self |= Self::only(x);
    }

    pub fn remove(&mut self, x: Small<L>) {
        *self = self.and_not(Self::only(x));
    }

    pub fn and_not(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub fn size(self) -> u8 {
        self.0.count_ones()
    }

    pub fn smallest(self) -> Option<Small<L>> {
        if self == Self::EMPTY {
            return None;
        }
        let val: u8 = self.0.trailing_zeros();
        // SAFETY: `val` is in the range `0..L` because `self` is not empty.
        Some(unsafe { Small::<L>::new_unchecked(val) })
    }
}

impl<const L: usize, T: Bits> IntoIterator for SmallSet<L, T> {
    type Item = Small<L>;
    type IntoIter = SmallSetIterator<L, T>;

    fn into_iter(self) -> Self::IntoIter {
        SmallSetIterator::<L, T>(self)
    }
}

#[derive(Clone, Debug)]
pub struct SmallSetIterator<const L: usize, T>(SmallSet<L, T>);

impl<const L: usize, T: Bits> Iterator for SmallSetIterator<L, T> {
    type Item = Small<L>;

    fn next(&mut self) -> Option<Small<L>> {
        let res = self.0.smallest()?;
        self.0.remove(res);
        Some(res)
    }
}

impl<const L: usize, T: Bits> BitAnd for SmallSet<L, T> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl<const L: usize, T: Bits> BitAndAssign for SmallSet<L, T> {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl<const L: usize, T: Bits> BitOr for SmallSet<L, T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl<const L: usize, T: Bits> BitOrAssign for SmallSet<L, T> {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
