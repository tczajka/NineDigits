use crate::{error::ResourcesExceeded, log};
use std::{
    mem::{self, MaybeUninit},
    slice,
};

pub struct MemoryBuffer(Vec<MaybeUninit<u8>>);

#[derive(Debug)]
pub struct Memory<'a>(&'a mut [MaybeUninit<u8>]);

impl MemoryBuffer {
    /// Allocate memory.
    pub fn new(size: usize) -> MemoryBuffer {
        log::write_line!(Info, "allocating {} MB", size >> 20);
        Self(vec![MaybeUninit::uninit(); size])
    }

    /// Get memory.
    pub fn into_memory(&mut self) -> Memory {
        Memory(&mut self.0)
    }
}

impl Memory<'_> {
    /// Allocate a slice with a given value.
    pub fn allocate_slice<T: Copy + Default>(
        &mut self,
        n: usize,
    ) -> Result<(&mut [T], Memory), ResourcesExceeded> {
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
        Ok((slice, Memory(tail)))
    }
}
