/// A number in range 0..L.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
