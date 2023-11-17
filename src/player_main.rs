use std::time::{Duration, Instant};

use crate::{
    board::{Board, FilledBoard, FullMove, Move},
    endgame::Endgame,
    error::InvalidInput,
    fast_solver::FastSolver,
    log,
    player::Player,
    solver::generate_solutions,
};

pub struct PlayerMain {
    board: Board,
    solutions_generated: bool,
    solutions: Vec<FilledBoard>,
    endgame: Endgame,
}

impl PlayerMain {
    // 81 KiB
    const SOLUTION_LIMIT: usize = 1000;
    // 256 MiB
    const ENDGAME_MEMORY_LIMIT: usize = 256 << 20;

    pub fn new() -> Self {
        Self {
            board: Board::new(),
            solutions_generated: false,
            solutions: Vec::with_capacity(Self::SOLUTION_LIMIT),
            endgame: Endgame::new(Self::ENDGAME_MEMORY_LIMIT),
        }
    }

    fn make_move(&mut self, mov: Move) -> Result<(), InvalidInput> {
        self.board.make_move(mov)?;
        if self.solutions_generated {
            self.solutions
                .retain(|solution| solution.squares[mov.square] == mov.digit);
            log::write_line!(Info, "solutions: {}", self.solutions.len());
        }
        Ok(())
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
            if generate_solutions::<FastSolver>(
                &self.board,
                &mut self.solutions,
                Self::SOLUTION_LIMIT,
                start_time + time_left / 10,
            )
            .is_ok()
            {
                self.solutions_generated = true;
            }

            let t = Instant::now();
            let used_time = t.saturating_duration_since(start_time);
            log::write_line!(Info, "generate_solutions time {used_time:.3?}");
            time_left = time_left.saturating_sub(t - start_time);
            start_time = t;
        }

        if self.solutions_generated {
            let (result, mov) = self
                .endgame
                .solve(&self.solutions, start_time + time_left / 10);
            if let Ok(win) = result {
                log::write_line!(Info, "{}", if win { "win" } else { "lose" });
            }
            mov
        } else {
            self.choose_move_without_solutions(start_time + time_left / 10)
        }
    }
}
