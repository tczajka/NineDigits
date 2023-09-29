use std::mem::MaybeUninit;

// A queue that can hold up to `M` - 1 elements.
#[derive(Debug)]
pub struct Queue<T, const M: usize> {
    elems: [MaybeUninit<T>; M],
    head: usize,
    tail: usize,
}

impl<T, const M: usize> Queue<T, M> {
    pub fn new() -> Self {
        Self {
            // Safety: array of `MaybeUninit` doesn't need initialization.
            elems: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    /// Push an element to the queue.
    ///
    /// # Panics
    ///
    /// Panics if the queue is full.
    pub fn push(&mut self, elem: T) {
        self.elems[self.tail].write(elem);
        self.tail = Self::advance(self.tail);
        assert!(!self.is_empty())
    }

    /// Take an element from the queue.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        // Safety: `head` != `tail` because of the above check.
        let elem = unsafe { self.elems[self.head].assume_init_read() };
        self.head = Self::advance(self.head);
        Some(elem)
    }

    fn advance(index: usize) -> usize {
        let index = index + 1;
        if index == M {
            0
        } else {
            index
        }
    }
}

impl<T, const M: usize> Default for Queue<T, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const M: usize> Drop for Queue<T, M> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}
