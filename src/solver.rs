use super::board::{Board, FilledBoard};

pub trait Solver {
    fn new(board: &Board) -> Self;
    fn step(&mut self) -> SolverStep;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SolverStep {
    Found(FilledBoard),
    NoProgress,
    Done,
}
