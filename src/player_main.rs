use std::{
    cmp,
    time::{Duration, Instant},
};

use crate::{
    board::{Board, FullMove, Move},
    digit::Digit,
    endgame::EndgameSolver,
    error::InvalidInput,
    log,
    player::Player,
    random::RandomGenerator,
    settings,
    small::Small,
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

    fn midgame(&mut self) -> FullMove {
        assert!(!self.all_solutions_generated);
        let num_solutions: u32 = self.solutions.len().try_into().unwrap();
        assert!(num_solutions >= 2);

        let move_summaries = self.solutions.move_summaries();
        let mut move_candidates: Vec<(Move, u32)> = Vec::new();
        for (square, move_summaries_sq) in Small::<81>::all().zip(move_summaries.iter()) {
            for (digit, move_summary) in Digit::all().zip(move_summaries_sq.iter()) {
                if move_summary.num_solutions == 0 || move_summary.num_solutions == num_solutions {
                    continue;
                }
                move_candidates.push((Move { square, digit }, move_summary.num_solutions));
            }
        }
        move_candidates.sort_by_key(|x| cmp::Reverse(x.1));
        let best_solutions = move_candidates[0].1;
        let min_solutions = ((best_solutions as f64 * settings::EARLY_GAME_MIN_SOLUTIONS_FRACTION)
            as u32)
            .clamp(2, best_solutions);
        while move_candidates.last().unwrap().1 < min_solutions {
            move_candidates.pop();
        }
        let (chosen_move, num_solutions) =
            move_candidates[self.rng.uniform_usize(move_candidates.len())];
        log::write_line!(
            Info,
            "midgame candidates: {num_candidates} num_solutions: {num_solutions} best_solutions: {best_solutions}",
            num_candidates = move_candidates.len()
        );
        FullMove::Move(chosen_move)
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
                .solve_best_effort(&self.solutions, start_time, time_left)
        } else {
            self.midgame()
        };
        if let Some(mov) = mov.to_move() {
            self.make_move(mov).unwrap();
        }
        mov
    }
}
