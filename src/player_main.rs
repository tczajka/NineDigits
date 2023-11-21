use std::time::{Duration, Instant};

use crate::{
    board::{Board, FullMove, Move},
    endgame::EndgameSolver,
    error::InvalidInput,
    log,
    player::Player,
    random::RandomGenerator,
    solution_table::SolutionTable,
};

pub struct PlayerMain {
    board: Board,
    all_solutions_generated: bool,
    solutions: SolutionTable,
    endgame_solver: EndgameSolver,
    rng: RandomGenerator,
}

impl PlayerMain {
    // 81 KiB
    const SOLUTION_LIMIT: usize = 1000;

    pub fn new() -> Self {
        Self {
            board: Board::new(),
            all_solutions_generated: false,
            solutions: SolutionTable::empty(),
            endgame_solver: EndgameSolver::new(),
            rng: RandomGenerator::with_time_nonce(),
        }
    }

    fn make_move(&mut self, mov: Move) -> Result<(), InvalidInput> {
        self.board.make_move(mov)?;
        if self.all_solutions_generated {
            self.solutions =
                self.solutions
                    .filter(self.solutions.len(), mov.square.into(), mov.digit);
            log::write_line!(Info, "solutions: {}", self.solutions.len());
        } else {
            self.solutions = SolutionTable::empty();
        }
        Ok(())
    }

    fn choose_move_without_solutions(&mut self, deadline: Instant) -> FullMove {
        assert!(!self.all_solutions_generated);
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
        if !self.all_solutions_generated {
            let (res, solutions) = SolutionTable::generate(
                &self.board,
                Self::SOLUTION_LIMIT,
                start_time + time_left / 10,
                &mut self.rng,
            );
            self.solutions = solutions;
            if res.is_ok() {
                self.all_solutions_generated = true;
            }

            let t = Instant::now();
            let used_time = t.saturating_duration_since(start_time);
            time_left = time_left.saturating_sub(t - start_time);
            start_time = t;

            log::write_line!(
                Info,
                "solutions count={count} res={res:?} time={used_time:.3?}",
                count = self.solutions.len()
            );
        }

        if self.all_solutions_generated {
            let (result, mov) = self
                .endgame_solver
                .solve_best_effort(&self.solutions, start_time + time_left / 10);
            if let Ok(win) = result {
                log::write_line!(Info, "{}", if win { "win" } else { "lose" });
            }
            mov
        } else {
            self.choose_move_without_solutions(start_time + time_left / 10)
        }
    }
}
