use std::time::{Duration, Instant};

use crate::{
    board::{Board, FullMove, Move},
    endgame::{EndgameResult, EndgameSolver},
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

    fn choose_opening_move(&mut self, start_time: Instant) -> Option<Move> {
        let moves = midgame::generate_moves(&mut self.board, &SolutionTable::empty());
        log::write_line!(
            Info,
            "opening movegen time {:.3?}",
            Instant::now().saturating_duration_since(start_time)
        );
        if 81 - self.board.empty_squares().size() <= settings::OPENING_MAX_SQUARES {
            Some(self.rng.choose(&moves).mov)
        } else {
            None
        }
    }

    fn midgame_choose_move_best_effort(
        &mut self,
        mut start_time: Instant,
        mut time_left: Duration,
    ) -> FullMove {
        assert!(self.solutions.len() >= settings::SOLUTIONS_MIN);
        let mut moves = midgame::generate_moves(&mut self.board, &self.solutions);
        let num_moves = moves.len();
        assert!(!moves.is_empty());
        moves.sort_by_key(|x| x.num_solutions_lower_bound);

        {
            let t = Instant::now();
            let used_time = t.saturating_duration_since(start_time);
            time_left = time_left.saturating_sub(used_time);
            start_time = t;
            log::write_line!(Info, "midgame movegen time {used_time:.3?}",);
        }

        let mut best_losing_move_index = num_moves - 1;
        let mut best_losing_move_difficulty = 0;

        for (defense_index, mov) in moves.iter().enumerate().rev() {
            if mov.num_solutions_lower_bound <= settings::MIDGAME_DEFENSE_SOLUTIONS_MAX {
                let defense_deadline =
                    start_time + time_left.mul_f64(settings::MIDGAME_DEFENSE_TIME_FRACTION);
                let defense_deadline_extended = start_time
                    + time_left.mul_f64(settings::MIDGAME_DEFENSE_EXTENDED_TIME_FRACTION);
                let mut new_board = self.board;
                new_board.make_move(mov.mov).unwrap();
                let (solgen_result, solutions) = SolutionTable::generate(
                    &new_board,
                    0,
                    settings::MIDGAME_DEFENSE_SOLUTIONS_MAX,
                    defense_deadline,
                    &mut self.rng,
                );
                if let Err(e) = solgen_result {
                    log::write_line!(
                        Info,
                        "midgame defense safe {defense_index} / {num_moves} num_solutions >= {num_solutions} {e}",
                        num_solutions = solutions.len(),
                    );
                    return FullMove::Move(mov.mov);
                }
                log::write_line!(
                    Info,
                    "midgame defense {defense_index} / {num_moves} num_solutions = {num_solutions}",
                    num_solutions = solutions.len()
                );
                match self.endgame_solver.solve(
                    &solutions,
                    Some(defense_deadline),
                    defense_deadline_extended,
                    settings::MIDGAME_DEFENSE_DIFFICULTY_MAX,
                ) {
                    Ok(EndgameResult::Loss) => {
                        self.solutions = solutions;
                        self.all_solutions_generated = true;
                        log::write_line!(Info, "win!");
                        return FullMove::Move(mov.mov);
                    }
                    Ok(EndgameResult::Win { difficulty }) => {
                        log::write_line!(Info, "midgame PANIC");
                        if difficulty > best_losing_move_difficulty {
                            best_losing_move_index = defense_index;
                            best_losing_move_difficulty = difficulty;
                        }
                        let t = Instant::now();
                        time_left =
                            time_left.saturating_sub(t.saturating_duration_since(start_time));
                        start_time = t;
                    }
                    Err(_) => {
                        log::write_line!(Info, "safe");
                        self.solutions = solutions;
                        self.all_solutions_generated = true;
                        return FullMove::Move(mov.mov);
                    }
                }
            } else {
                log::write_line!(
                    Info,
                    "midgame num_solutions >= {num_solutions}",
                    num_solutions = mov.num_solutions_lower_bound,
                );
                return FullMove::Move(mov.mov);
            }
        }
        log::write_line!(
            Info,
            "midgame lost difficulty={best_losing_move_difficulty}"
        );
        FullMove::Move(moves[best_losing_move_index].mov)
    }
}

impl Player for PlayerMain {
    fn opponent_move(&mut self, mov: Move) {
        match self.board.make_move(mov) {
            Ok(()) => {
                if self.all_solutions_generated {
                    self.solutions =
                        self.solutions
                            .filter(self.solutions.len(), mov.square.into(), mov.digit);
                    log::write_line!(Info, "opp move solutions: {}", self.solutions.len());
                } else {
                    self.solutions = SolutionTable::empty();
                }
            }
            Err(InvalidInput) => {
                log::write_line!(Always, "Invalid opp move: {mov}");
            }
        }
    }

    fn choose_move(&mut self, mut start_time: Instant, mut time_left: Duration) -> FullMove {
        if 81 - self.board.empty_squares().size() <= settings::OPENING_MAX_SQUARES {
            if let Some(mov) = self.choose_opening_move(start_time) {
                self.board.make_move(mov).unwrap();
                return FullMove::Move(mov);
            }
        }
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

        if self.all_solutions_generated {
            let mov =
                self.endgame_solver
                    .choose_move_best_effort(&self.solutions, start_time, time_left);
            if let Some(mov) = mov.to_move() {
                self.board.make_move(mov).unwrap();
                self.solutions =
                    self.solutions
                        .filter(self.solutions.len(), mov.square.into(), mov.digit);
            }
            mov
        } else {
            let mov = self.midgame_choose_move_best_effort(start_time, time_left);
            if let Some(mov) = mov.to_move() {
                self.board.make_move(mov).unwrap();
                // if all_solutions_generated, solutions already updated to next move by midgame_choose_move_best_effort
                if !self.all_solutions_generated {
                    self.solutions = SolutionTable::empty();
                }
            }
            mov
        }
    }
}
