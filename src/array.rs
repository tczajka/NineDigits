use super::small::Small;
use std::ops::{Index, IndexMut};

pub struct Array<T, const N: usize>([T; N]);

impl<T: Copy, const N: usize> Array<T, N> {
    /// Creates a new `Array` from a `T`.
    pub const fn new(x: T) -> Self {
        Self([x; N])
    }
}

impl<T, const N: usize> Index<Small<N>> for Array<T, N> {
    type Output = T;

    fn index(&self, index: Small<N>) -> &T {
        let index = usize::from(index);
        // SAFETY: `index` is in range 0..N.
        unsafe { self.0.get_unchecked(index) }
    }
}

impl<T, const N: usize> IndexMut<Small<N>> for Array<T, N> {
    fn index_mut(&mut self, index: Small<N>) -> &mut T {
        let index = usize::from(index);
        // SAFETY: `index` is in range 0..N.
        unsafe { self.0.get_unchecked_mut(index) }
    }
}
