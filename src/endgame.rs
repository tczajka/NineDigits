use std::time::Instant;

use crate::{
    board::{FilledBoard, FullMove},
    log,
    memory::Memory,
};

#[derive(Debug)]
pub struct Endgame {
    memory: Memory,
}

impl Endgame {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            memory: Memory::new(memory_limit),
        }
    }

    pub fn choose_move_best_effort(
        &mut self,
        solutions: &[FilledBoard],
        deadline: Instant,
    ) -> FullMove {
        if solutions.is_empty() {
            log::write_line!(Always, "Error: invalid board!");
            return FullMove::ClaimUnique;
        }
        if solutions.len() == 1 {
            log::write_line!(Info, "Lucky win: opponent didn't claim.");
            return FullMove::ClaimUnique;
        }
        todo!()
    }
}
