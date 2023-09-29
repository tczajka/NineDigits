use std::ops::{Index, IndexMut};

use crate::random::RandomGenerator;

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
