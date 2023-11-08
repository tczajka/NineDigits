use crate::{error::InvalidInput, random::RandomGenerator};
use std::ops::{Index, IndexMut};

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
    pub fn new_random(rng: &mut RandomGenerator) -> Self {
        assert!(L < 256);
        Self(rng.uniform_u8(L as u8))
    }

    /// Iterate all.
    pub fn all() -> impl Iterator<Item = Self> {
        // Safety: `i` is in range 0..L.
        (0..u8::try_from(L).unwrap()).map(|i| unsafe { Self::new_unchecked(i) })
    }
}

impl<const L: usize> From<Small<L>> for u8 {
    fn from(x: Small<L>) -> Self {
        x.0
    }
}

impl<const L: usize> From<Small<L>> for usize {
    fn from(x: Small<L>) -> Self {
        x.0.into()
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

macro_rules! impl_from {
    ($a:literal => $b:literal) => {
        impl From<Small<$a>> for Small<$b> {
            fn from(x: Small<$a>) -> Self {
                // SAFETY: $a < $b.
                unsafe { Small::new_unchecked(x.0) }
            }
        }
    };
}

impl_from!(3 => 4);
impl_from!(9 => 16);

macro_rules! impl_try_from {
    ($a:literal => $b:literal) => {
        impl TryFrom<Small<$a>> for Small<$b> {
            type Error = InvalidInput;

            fn try_from(x: Small<$a>) -> Result<Self, InvalidInput> {
                if x.0 < $b {
                    // SAFETY: $a < $b.
                    Ok(unsafe { Small::new_unchecked(x.0) })
                } else {
                    Err(InvalidInput)
                }
            }
        }
    };
}

impl_try_from!(16 => 15);

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
impl_cartesian_product!(16 = 4 x 4);
impl_cartesian_product!(81 = 9 x 9);
