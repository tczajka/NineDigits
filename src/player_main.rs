use std::time::{Duration, Instant};

use crate::{
    board::{Board, FilledBoard, FullMove, Move},
    error::InvalidInput,
    fast_solver::FastSolver,
    log,
    player::Player,
    solver::{Solver, SolverStep},
};

#[derive(Debug)]
pub struct PlayerMain {
    board: Board,
    solutions_generated: bool,
    /// 81 KiB
    solutions: Vec<FilledBoard>,
}

impl PlayerMain {
    const SOLUTION_LIMIT: usize = 1000;
    const SOLVER_CHECK_TIME_ITERS: u64 = 1024;

    pub fn new() -> Self {
        Self {
            board: Board::new(),
            solutions_generated: false,
            solutions: Vec::with_capacity(Self::SOLUTION_LIMIT),
        }
    }

    fn make_move(&mut self, mov: Move) -> Result<(), InvalidInput> {
        self.board.make_move(mov)?;
        if self.solutions_generated {
            self.solutions
                .retain(|solution| solution.squares[mov.square] == mov.digit);
        }
        Ok(())
    }

    fn try_generate_solutions(&mut self, deadline: Instant) {
        self.solutions.clear();
        let mut solver = FastSolver::new(&self.board);
        let mut since_last_time_check: u64 = 0;
        loop {
            match solver.step() {
                SolverStep::Found(solution) => {
                    if self.solutions.len() >= Self::SOLUTION_LIMIT {
                        log::write_line!(Info, "too many solutions");
                        break;
                    }
                    self.solutions.push(solution);
                }
                SolverStep::NoProgress => {}
                SolverStep::Done => {
                    self.solutions_generated = true;
                    log::write_line!(Info, "Generated {} solutions!", self.solutions.len());
                    break;
                }
            }

            since_last_time_check += 1;
            if since_last_time_check >= Self::SOLVER_CHECK_TIME_ITERS {
                since_last_time_check = 0;
                if Instant::now() >= deadline {
                    log::write_line!(
                        Info,
                        "time limit exceeded, {} solutions found",
                        self.solutions.len()
                    );
                    break;
                }
            }
        }
    }

    fn choose_move_with_solutions(&mut self, deadline: Instant) -> FullMove {
        assert!(self.solutions_generated);
        log::write_line!(Info, "solutions: {}", self.solutions.len());
        if self.solutions.is_empty() {
            log::write_line!(Always, "Error: invalid board!");
            return FullMove::ClaimUnique;
        }
        if self.solutions.len() == 1 {
            log::write_line!(Info, "Lucky win: opponent didn't claim.");
            return FullMove::ClaimUnique;
        }
        todo!()
    }

    fn choose_move_without_solutions(&mut self, deadline: Instant) -> FullMove {
        assert!(!self.solutions_generated);
        todo!()
    }
}

impl Player for PlayerMain {
    fn opponent_move(&mut self, mov: Move) {
        self.make_move(mov).unwrap_or_else(|_| {
            log::write_line!(Always, "Invalid opp move: {mov}");
        });
    }

    fn choose_move(&mut self, mut start_time: Instant, mut time_left: Duration) -> FullMove {
        if !self.solutions_generated {
            self.try_generate_solutions(start_time + time_left / 10);

            let t = Instant::now();
            let used_time = t.saturating_duration_since(start_time);
            log::write_line!(Info, "generate_solutions time {used_time:?}");
            time_left = time_left.saturating_sub(t - start_time);
            start_time = t;
        }

        if self.solutions_generated {
            self.choose_move_with_solutions(start_time + time_left / 10)
        } else {
            self.choose_move_without_solutions(start_time + time_left / 10)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ResourcesExceeded {
    Time,
    Memory,
}
