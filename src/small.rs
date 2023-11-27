use crate::{error::InvalidInput, random::RandomGenerator};
use std::{
    hint::unreachable_unchecked,
    ops::{Index, IndexMut},
};

/// A number in range 0..L.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]

pub struct Small<const L: usize>(u8);

impl<const L: usize> Small<L> {
    /// Creates a new `Small` from a `u8`.
    ///
    /// # Panics
    ///
    /// Panics if `x >= L`.
    pub const fn new(x: u8) -> Self {
        assert!((x as usize) < L);
        Self(x)
    }

    /// Creates a new `Small` from a `u8` without checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `x < L`.
    pub const unsafe fn new_unchecked(x: u8) -> Self {
        Self(x)
    }

    /// A random `Small`.
    pub fn random(rng: &mut RandomGenerator) -> Self {
        assert!(L <= 256);
        Self(rng.uniform_usize(L) as u8)
    }

    /// Iterate all.
    pub fn all() -> impl Iterator<Item = Self> {
        assert!(L <= 256);
        // Safety: `i` is in range 0..L.
        (0..L).map(|i| unsafe { Self::new_unchecked(i as u8) })
    }
}

impl<const L: usize> From<Small<L>> for u8 {
    fn from(x: Small<L>) -> Self {
        if usize::from(x.0) >= L {
            unsafe { unreachable_unchecked() }
        }
        x.0
    }
}

impl<const L: usize> From<Small<L>> for usize {
    fn from(x: Small<L>) -> Self {
        u8::from(x).into()
    }
}

impl<const L: usize> TryFrom<u8> for Small<L> {
    type Error = InvalidInput;

    fn try_from(x: u8) -> Result<Self, InvalidInput> {
        if usize::from(x) < L {
            Ok(Self(x))
        } else {
            Err(InvalidInput)
        }
    }
}

impl<const L: usize> TryFrom<usize> for Small<L> {
    type Error = InvalidInput;

    fn try_from(x: usize) -> Result<Self, InvalidInput> {
        if x < L {
            Ok(Self(x as u8))
        } else {
            Err(InvalidInput)
        }
    }
}

impl<const L: usize> Default for Small<L> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T, const N: usize> Index<Small<N>> for [T; N] {
    type Output = T;

    fn index(&self, index: Small<N>) -> &T {
        let index = usize::from(index);
        // SAFETY: `index` is in range 0..N.
        unsafe { self.get_unchecked(index) }
    }
}

impl<T, const N: usize> IndexMut<Small<N>> for [T; N] {
    fn index_mut(&mut self, index: Small<N>) -> &mut T {
        let index = usize::from(index);
        // SAFETY: `index` is in range 0..N.
        unsafe { self.get_unchecked_mut(index) }
    }
}

macro_rules! impl_extend {
    ($a:literal < $b:literal) => {
        impl From<Small<$a>> for Small<$b> {
            fn from(x: Small<$a>) -> Self {
                // SAFETY: $a < $b.
                unsafe { Small::new_unchecked(x.0) }
            }
        }

        impl TryFrom<Small<$b>> for Small<$a> {
            type Error = InvalidInput;

            fn try_from(x: Small<$b>) -> Result<Self, InvalidInput> {
                if x.0 < $a {
                    Ok(unsafe { Small::new_unchecked(x.0) })
                } else {
                    Err(InvalidInput)
                }
            }
        }
    };
}

impl_extend!(3 < 4);
impl_extend!(9 < 16);
impl_extend!(15 < 16);

pub trait CartesianProduct<A, B> {
    fn combine(lhs: A, rhs: B) -> Self;
    fn split(self) -> (A, B);
}

macro_rules! impl_cartesian_product {
    ($n:literal = $a:literal x $b:literal) => {
        impl CartesianProduct<Small<$a>, Small<$b>> for Small<$n> {
            fn combine(a: Small<$a>, b: Small<$b>) -> Self {
                let a = u8::from(a);
                let b = u8::from(b);
                let n = a * $b + b;
                // SAFETY: $n = $a * $b.
                unsafe { Self::new_unchecked(n) }
            }

            fn split(self) -> (Small<$a>, Small<$b>) {
                let n = u8::from(self);
                let a = n / $b;
                let b = n % $b;
                // SAFETY: $n = $a * $b.
                unsafe { (Small::new_unchecked(a), Small::new_unchecked(b)) }
            }
        }
    };
}

impl_cartesian_product!(4 = 2 x 2);
impl_cartesian_product!(8 = 2 x 4);
impl_cartesian_product!(9 = 3 x 3);
impl_cartesian_product!(16 = 2 x 8);
impl_cartesian_product!(16 = 4 x 4);
impl_cartesian_product!(32 = 2 x 16);
impl_cartesian_product!(81 = 9 x 9);
impl_cartesian_product!(128 = 2 x 64);
impl_cartesian_product!(128 = 4 x 32);
impl_cartesian_product!(128 = 8 x 16);
impl_cartesian_product!(128 = 16 x 8);
impl_cartesian_product!(256 = 4 x 64);
impl_cartesian_product!(256 = 8 x 32);
impl_cartesian_product!(256 = 16 x 16);
impl_cartesian_product!(256 = 32 x 8);
