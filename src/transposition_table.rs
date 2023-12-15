use crate::{endgame::EndgameResult, log};
use std::mem;

pub struct TranspositionTable {
    table: Vec<Bucket>,
    index_mask: usize,
    era: u8,
}

impl TranspositionTable {
    pub fn new(memory: usize) -> Self {
        let num_buckets = (memory / (2 * mem::size_of::<Bucket>()) + 1).next_power_of_two();
        log::write_line!(
            Info,
            "transposition table {} MiB",
            (num_buckets * mem::size_of::<Bucket>()) >> 20
        );
        Self {
            table: vec![Bucket::new(); num_buckets],
            era: 1,
            index_mask: num_buckets - 1,
        }
    }

    pub fn new_era(&mut self) {
        self.era = self.era.wrapping_add(1);
    }

    pub fn find(&self, hash: u64) -> Option<EndgameResult> {
        // Safety: index_mask guarantees the index is in range.
        let bucket = unsafe { self.table.get_unchecked((hash as usize) & self.index_mask) };
        for entry in &bucket.entries {
            if entry.hash == hash {
                return Some(entry.result);
            }
        }
        None
    }

    pub fn insert(&mut self, hash: u64, result: EndgameResult) {
        // Safety: index_mask guarantees the index is in range.
        let bucket = unsafe {
            self.table
                .get_unchecked_mut((hash as usize) & self.index_mask)
        };
        let best_entry = bucket
            .entries
            .iter_mut()
            .min_by_key(|entry| (entry.hash != hash, entry.era == self.era))
            .unwrap();
        if best_entry.hash == hash {
            best_entry.era = self.era;
            // Don't overwrite with less complete result.
            if !matches!(result, EndgameResult::Win(None)) {
                best_entry.result = result;
            }
        } else {
            best_entry.hash = hash;
            best_entry.result = result;
            best_entry.era = self.era;
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Entry {
    hash: u64,
    era: u8,
    result: EndgameResult,
}

impl Entry {
    fn new() -> Self {
        Self {
            hash: 0,
            era: 0,
            result: EndgameResult::Loss,
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
