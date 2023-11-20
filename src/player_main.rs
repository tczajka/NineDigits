use std::time::{Duration, Instant};

use crate::{
    board::{Board, FullMove, Move},
    endgame::{generate_solutions, Endgame, Solution},
    error::InvalidInput,
    fast_solver::FastSolver,
    log,
    memory::MemoryBuffer,
    player::Player,
};

pub struct PlayerMain {
    board: Board,
    solutions_generated: bool,
    solutions: Vec<Solution>,
    endgame: Endgame,
    memory_buffer: MemoryBuffer,
}

impl PlayerMain {
    // 81 KiB
    const SOLUTION_LIMIT: usize = 1000;
    // 256 MiB
    const MEMORY_LIMIT: usize = 256 << 20;

    pub fn new() -> Self {
        Self {
            board: Board::new(),
            solutions_generated: false,
            solutions: Vec::with_capacity(Self::SOLUTION_LIMIT),
            endgame: Endgame::new(),
            memory_buffer: MemoryBuffer::new(Self::MEMORY_LIMIT),
        }
    }

    fn make_move(&mut self, mov: Move) -> Result<(), InvalidInput> {
        self.board.make_move(mov)?;
        if self.solutions_generated {
            self.solutions.retain(|solution| solution.matches(mov));
            log::write_line!(Info, "solutions: {}", self.solutions.len());
        } else {
            self.solutions.clear();
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
            let (result, mov) = self.endgame.solve_with_move(
                &self.solutions,
                start_time + time_left / 10,
                &mut self.memory_buffer.memory(),
            );
            if let Ok(win) = result {
                log::write_line!(Info, "{}", if win { "win" } else { "lose" });
            }
            mov
        } else {
            self.choose_move_without_solutions(start_time + time_left / 10)
        }
    }
}
