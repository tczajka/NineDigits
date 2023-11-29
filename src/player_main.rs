use std::time::{Duration, Instant};

use crate::{
    board::{Board, FullMove, Move},
    endgame::EndgameSolver,
    error::InvalidInput,
    log, midgame,
    player::Player,
    random::RandomGenerator,
    settings,
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
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            all_solutions_generated: false,
            solutions: SolutionTable::empty(),
            endgame_solver: EndgameSolver::new(settings::TRANSPOSITION_TABLE_MEMORY),
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
                settings::SOLUTIONS_MIN,
                settings::SOLUTIONS_MAX,
                start_time + time_left.mul_f64(settings::SOLUTION_GENERATE_TIME_FRACTION),
                &mut self.rng,
            );
            self.solutions = solutions;

            let t = Instant::now();
            let used_time = t.saturating_duration_since(start_time);
            time_left = time_left.saturating_sub(t - start_time);
            start_time = t;
            let ksps = self.solutions.len() as f64 / used_time.as_secs_f64() / 1000.0;

            match res {
                Ok(()) => {
                    self.all_solutions_generated = true;
                    log::write_line!(
                        Info,
                        "All solutions generated count={count} time={used_time:.3?} ksps={ksps:.1} ",
                        count = self.solutions.len(),
                    );
                }
                Err(e) => {
                    log::write_line!(
                        Info,
                        "solutions count={count} time={used_time:.3?} ksps={ksps:.1} {e}",
                        count = self.solutions.len(),
                    );
                }
            }
        }

        let mov = if self.all_solutions_generated {
            self.endgame_solver
                .choose_move_best_effort(&self.solutions, start_time, time_left)
        } else {
            midgame::choose_move_best_effort(
                &mut self.board,
                &self.solutions,
                start_time,
                time_left,
                &mut self.endgame_solver,
                &mut self.rng,
            )
        };
        if let Some(mov) = mov.to_move() {
            self.make_move(mov).unwrap();
        }
        mov
    }
}
