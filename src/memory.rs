use crate::{error::ResourcesExceeded, log};
use std::{
    fmt::{self, Debug, Formatter},
    mem::{self, MaybeUninit},
    slice,
};

pub struct Memory(Vec<MaybeUninit<u8>>);

#[derive(Debug)]
pub struct MemoryRemaining<'a>(&'a mut [MaybeUninit<u8>]);

impl Memory {
    /// Allocate memory.
    pub fn new(size: usize) -> Memory {
        log::write_line!(Info, "allocating {} MB", size >> 20);
        Self(vec![MaybeUninit::uninit(); size])
    }

    /// Get memory.
    pub fn into_remaining(&mut self) -> MemoryRemaining {
        MemoryRemaining(&mut self.0)
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
            .field("len", &self.0.len())
            .finish()
    }
}

impl MemoryRemaining<'_> {
    /// Allocate a slice with a given value.
    pub fn allocate_slice<T: Copy + Default>(
        &mut self,
        n: usize,
    ) -> Result<(&mut [T], MemoryRemaining), ResourcesExceeded> {
        let offset = self.0.as_ptr().align_offset(mem::align_of::<T>());
        let size = n * mem::size_of::<T>();
        if self.0.len() < offset + size {
            return Err(ResourcesExceeded::Memory);
        };
        let (_, tail) = self.0.split_at_mut(offset);
        let (slice, tail) = tail.split_at_mut(size);
        let p = slice.as_mut_ptr() as *mut T;
        for i in 0..n {
            unsafe {
                p.add(i).write(T::default());
            }
        }
        let slice = unsafe { slice::from_raw_parts_mut(p, n) };
        Ok((slice, MemoryRemaining(tail)))
    }
}
