use std::time::Instant;

use crate::{
    board::{Board, FilledBoard},
    error::ResourcesExceeded,
    log,
};

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

pub fn generate_solutions<S: Solver>(
    board: &Board,
    solutions: &mut Vec<FilledBoard>,
    limit: usize,
    deadline: Instant,
) -> Result<(), ResourcesExceeded> {
    const CHECK_TIME_ITERS: u64 = 1024;

    solutions.clear();
    let mut solver = S::new(board);
    let mut since_last_time_check: u64 = 0;
    loop {
        match solver.step() {
            SolverStep::Found(solution) => {
                if solutions.len() >= limit {
                    log::write_line!(Info, "solutions > {}!", solutions.len());
                    return Err(ResourcesExceeded::Memory);
                }
                solutions.push(solution);
            }
            SolverStep::NoProgress => {}
            SolverStep::Done => {
                log::write_line!(Info, "Generated {} solutions!", solutions.len());
                return Ok(());
            }
        }

        since_last_time_check += 1;
        if since_last_time_check >= CHECK_TIME_ITERS {
            since_last_time_check = 0;
            if Instant::now() >= deadline {
                log::write_line!(
                    Info,
                    "time limit exceeded, {} solutions found",
                    solutions.len()
                );
                return Err(ResourcesExceeded::Time);
            }
        }
    }
}
