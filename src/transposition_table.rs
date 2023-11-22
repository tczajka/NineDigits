use crate::log;
use std::mem;

pub struct TranspositionTable {
    table: Vec<Bucket>,
    index_mask: usize,
    era: u8,
}

impl TranspositionTable {
    pub fn new(memory: usize) -> Self {
        assert_eq!(mem::size_of::<Bucket>(), 64);
        let num_buckets = (memory / (2 * mem::size_of::<Bucket>()) + 1).next_power_of_two();
        log::write_line!(
            Info,
            "transposition table {} MiB",
            (num_buckets * mem::size_of::<Bucket>()) >> 20
        );
        Self {
            table: vec![Bucket::new(); num_buckets],
            index_mask: num_buckets - 1,
            era: 1,
        }
    }

    pub fn new_era(&mut self) {
        self.era = self.era.wrapping_add(1);
    }

    pub fn find(&self, hash: u64) -> Option<bool> {
        let bucket = &self.table[(hash as usize) & self.index_mask];
        for entry in &bucket.entries {
            if entry.hash == hash {
                return Some(entry.result);
            }
        }
        None
    }

    pub fn insert(&mut self, hash: u64, num_solutions: u32, result: bool) {
        let bucket = &mut self.table[(hash as usize) & self.index_mask];
        let best_entry = bucket
            .entries
            .iter_mut()
            .min_by_key(|entry| {
                (
                    entry.hash != hash,
                    entry.era == self.era,
                    entry.num_solutions,
                )
            })
            .unwrap();
        best_entry.hash = hash;
        best_entry.num_solutions = num_solutions;
        best_entry.result = result;
        best_entry.era = self.era;
    }
}

#[derive(Copy, Clone, Debug)]
struct Entry {
    hash: u64,
    num_solutions: u32,
    result: bool,
    era: u8,
}

impl Entry {
    fn new() -> Self {
        Self {
            hash: 0,
            num_solutions: 0,
            result: false,
            era: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(align(64))]
struct Bucket {
    entries: [Entry; 4],
}

impl Bucket {
    fn new() -> Self {
        Self {
            entries: [Entry::new(); 4],
        }
    }
}
