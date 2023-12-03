use crate::{
    board::{FullMove, Move},
    digit::{Digit, OptionalDigit},
    error::ResourcesExceeded,
    log, settings,
    small::Small,
    solution_table::{MoveSummary, SolutionTable, SquareCompression},
    transposition_table::TranspositionTable,
};
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug)]
struct EndgameMove {
    square_index: u8,
    digit: Digit,
    summary: MoveSummary,
}

pub struct EndgameSolver {
    transposition_table: TranspositionTable,
    num_nodes: u64,
}

impl EndgameSolver {
    pub fn new(transposition_table_memory: usize) -> Self {
        Self {
            transposition_table: TranspositionTable::new(transposition_table_memory),
            num_nodes: 0,
        }
    }

    pub fn choose_move_best_effort(
        &mut self,
        solutions: &SolutionTable,
        mut start_time: Instant,
        mut time_left: Duration,
    ) -> FullMove {
        self.transposition_table.new_era();
        self.num_nodes = 0;

        if solutions.is_empty() {
            log::write_line!(Always, "Error: no solutions!");
            // No good option, let's just claim victory.
            return FullMove::ClaimUnique;
        }
        if solutions.len() == 1 {
            log::write_line!(Info, "Lucky win: opponent didn't claim.");
            return FullMove::ClaimUnique;
        }

        let move_summaries = solutions.move_summaries();

        if let Some(mov) = self.check_quick_win_root(solutions.len(), &move_summaries) {
            log::write_line!(Info, "quick win");
            return mov;
        }

        let (solutions, square_compressions) = solutions.compress(&move_summaries);

        let mut moves =
            Self::generate_moves(solutions.num_moves_per_square(), &square_compressions);
        moves.sort_by_key(|x| x.summary.num_solutions);
        let num_moves = moves.len();

        let offense_deadline =
            start_time + time_left.mul_f64(settings::ENDGAME_OFFENSE_TIME_FRACTION);
        let offense_deadline_extended =
            start_time + time_left.mul_f64(settings::ENDGAME_OFFENSE_EXTENDED_TIME_FRACTION);

        let mut offense_index = 0;
        let mut best_losing_move_index = num_moves - 1;
        let mut best_losing_move_difficulty = 0;

        while offense_index < num_moves {
            if Instant::now() > offense_deadline {
                log::write_line!(Info, "endgame offense {offense_index} / {num_moves}");
                break;
            }
            let mov = &moves[offense_index];
            match self.solve_after_move(&solutions, mov, None, offense_deadline_extended) {
                Ok(EndgameResult::Loss) => {
                    // Found a winning move.
                    log::write_line!(Info, "endgame win {offense_index} / {num_moves}");
                    self.log_stats(start_time, Instant::now());
                    return FullMove::Move(Self::uncompress_root_move(mov, &square_compressions));
                }
                Ok(EndgameResult::Win { difficulty }) => {
                    if difficulty > best_losing_move_difficulty {
                        best_losing_move_index = offense_index;
                        best_losing_move_difficulty = difficulty;
                    }
                }
                Err(e) => {
                    log::write_line!(Info, "endgame offense {offense_index} / {num_moves} {e}");
                    break;
                }
            }
            offense_index += 1;
        }

        {
            let t = Instant::now();
            self.log_stats(start_time, t);
            time_left = time_left.saturating_sub(t.saturating_duration_since(start_time));
            start_time = t;
            self.num_nodes = 0;
        }

        let defense_start_time = start_time;
        for defense_index in (offense_index..num_moves).rev() {
            let mov = &moves[defense_index];
            let defense_deadline =
                start_time + time_left.mul_f64(settings::ENDGAME_DEFENSE_TIME_FRACTION);
            let defense_deadline_extended =
                start_time + time_left.mul_f64(settings::ENDGAME_DEFENSE_EXTENDED_TIME_FRACTION);

            match self.solve_after_move(
                &solutions,
                mov,
                Some(defense_deadline),
                defense_deadline_extended,
            ) {
                Ok(EndgameResult::Loss) => {
                    log::write_line!(Info, "endgame defense win! {defense_index} / {num_moves}",);
                    self.log_stats(defense_start_time, Instant::now());
                    return FullMove::Move(Self::uncompress_root_move(mov, &square_compressions));
                }
                Ok(EndgameResult::Win { difficulty }) => {
                    // Panic. Reset time for next defensive move.
                    if defense_index == 0 {
                        log::write_line!(Info, "PANIC");
                    }
                    if difficulty > best_losing_move_difficulty {
                        best_losing_move_index = defense_index;
                        best_losing_move_difficulty = difficulty;
                    }
                    let t = Instant::now();
                    time_left = time_left.saturating_sub(t.saturating_duration_since(start_time));
                    start_time = t;
                }
                Err(e) => {
                    log::write_line!(
                        Info,
                        "endgame defense safe: {defense_index} / {num_moves} {e}",
                    );
                    self.log_stats(defense_start_time, Instant::now());
                    return FullMove::Move(Self::uncompress_root_move(mov, &square_compressions));
                }
            }
        }

        log::write_line!(
            Info,
            "endgame lost difficulty {best_losing_move_difficulty}"
        );
        self.log_stats(defense_start_time, Instant::now());
        FullMove::Move(Self::uncompress_root_move(
            &moves[best_losing_move_index],
            &square_compressions,
        ))
    }

    pub fn solve(
        &mut self,
        solutions: &SolutionTable,
        deadline: Instant,
        deadline_extended: Instant,
    ) -> Result<EndgameResult, ResourcesExceeded> {
        let start_time = Instant::now();
        self.transposition_table.new_era();
        self.num_nodes = 0;

        if solutions.is_empty() {
            log::write_line!(Always, "Error: no solutions!");
            return Ok(EndgameResult::Loss);
        }
        if solutions.len() == 1 {
            return Ok(EndgameResult::Loss);
        }
        if solutions.len() < 4 {
            return Ok(EndgameResult::Win { difficulty: 0 });
        }
        if let Some((result, difficulty)) = self.transposition_table.find(solutions.hash()) {
            return Ok(if result {
                EndgameResult::Win { difficulty }
            } else {
                EndgameResult::Loss
            });
        }

        let result = self.solve_recursive(solutions, Some(deadline), deadline_extended)?;
        self.log_stats(start_time, Instant::now());
        Ok(result)
    }

    fn solve_recursive(
        &mut self,
        solutions: &SolutionTable,
        deadline_toplevel: Option<Instant>,
        deadline_extended: Instant,
    ) -> Result<EndgameResult, ResourcesExceeded> {
        self.num_nodes += 1;

        if self.num_nodes % settings::ENDGAME_CHECK_TIME_NODES == 0
            && Instant::now() >= deadline_extended
        {
            return Err(ResourcesExceeded::Time);
        }

        let move_summaries = solutions.move_summaries();
        let result = if let EndgameResult::Win { difficulty } = self.check_quick_win(
            solutions.num_moves_per_square(),
            solutions.len(),
            &move_summaries,
        ) {
            EndgameResult::Win { difficulty }
        } else {
            let (solutions, square_compressions) = solutions.compress(&move_summaries);

            let mut moves =
                Self::generate_moves(solutions.num_moves_per_square(), &square_compressions);
            moves.sort_by_key(|x| x.summary.num_solutions);
            let mut result = EndgameResult::Loss;

            for mov in moves.iter() {
                if let Some(deadline_toplevel) = deadline_toplevel {
                    if Instant::now() >= deadline_toplevel {
                        return Err(ResourcesExceeded::Time);
                    }
                }
                if let EndgameResult::Loss =
                    self.solve_after_move(&solutions, mov, None, deadline_extended)?
                {
                    result = EndgameResult::Win {
                        difficulty: mov.summary.num_solutions,
                    };
                    break;
                }
            }
            result
        };
        match result {
            EndgameResult::Win { difficulty } => {
                self.transposition_table
                    .insert(solutions.hash(), difficulty, true);
            }
            EndgameResult::Loss => {
                self.transposition_table
                    .insert(solutions.hash(), solutions.len(), false);
            }
        }
        Ok(result)
    }

    fn solve_after_move(
        &mut self,
        solutions: &SolutionTable,
        mov: &EndgameMove,
        deadline_toplevel: Option<Instant>,
        deadline_extended: Instant,
    ) -> Result<EndgameResult, ResourcesExceeded> {
        if mov.summary.num_solutions == 1 {
            return Ok(EndgameResult::Loss);
        }
        if mov.summary.num_solutions < 4 {
            return Ok(EndgameResult::Win { difficulty: 0 });
        }
        if let Some((result, difficulty)) = self.transposition_table.find(mov.summary.hash) {
            return Ok(if result {
                EndgameResult::Win { difficulty }
            } else {
                EndgameResult::Loss
            });
        }
        let new_solutions = solutions.filter(
            mov.summary.num_solutions,
            mov.square_index.into(),
            mov.digit,
        );
        assert_eq!(new_solutions.len(), mov.summary.num_solutions);
        assert_eq!(new_solutions.hash(), mov.summary.hash);
        self.solve_recursive(&new_solutions, deadline_toplevel, deadline_extended)
    }

    fn check_quick_win_root(
        &self,
        num_solutions: u32,
        move_summaries: &[[MoveSummary; 9]],
    ) -> Option<FullMove> {
        for (square, move_summaries_sq) in move_summaries.iter().enumerate() {
            for (digit, move_summary) in Digit::all().zip(move_summaries_sq) {
                if move_summary.num_solutions == 1 {
                    return Some(FullMove::MoveClaimUnique(Move {
                        square: square.try_into().unwrap(),
                        digit,
                    }));
                }
            }
        }

        // Enhanced transposition cutoff.
        for (square, move_summaries_sq) in move_summaries.iter().enumerate() {
            for (digit, move_summary) in Digit::all().zip(move_summaries_sq) {
                if move_summary.num_solutions >= 4
                    && move_summary.num_solutions < num_solutions
                    && matches!(
                        self.transposition_table.find(move_summary.hash),
                        Some((false, _))
                    )
                {
                    return Some(FullMove::Move(Move {
                        square: square.try_into().unwrap(),
                        digit,
                    }));
                }
            }
        }

        None
    }

    fn check_quick_win(
        &self,
        num_moves_per_square: &[u8],
        num_solutions: u32,
        move_summaries: &[[MoveSummary; 9]],
    ) -> EndgameResult {
        assert_eq!(num_moves_per_square.len(), move_summaries.len());
        for (&num_moves, move_summaries_sq) in
            num_moves_per_square.iter().zip(move_summaries.iter())
        {
            for move_summary in &move_summaries_sq[..usize::from(num_moves)] {
                if move_summary.num_solutions == 1 {
                    return EndgameResult::Win { difficulty: 1 };
                }
            }
        }

        // Enhanced transposition cutoff.
        for (&num_moves, move_summaries_sq) in
            num_moves_per_square.iter().zip(move_summaries.iter())
        {
            for move_summary in &move_summaries_sq[..usize::from(num_moves)] {
                if move_summary.num_solutions >= 4
                    && move_summary.num_solutions < num_solutions
                    && matches!(
                        self.transposition_table.find(move_summary.hash),
                        Some((false, _))
                    )
                {
                    return EndgameResult::Win {
                        difficulty: move_summary.num_solutions,
                    };
                }
            }
        }
        EndgameResult::Loss
    }

    fn uncompress_root_move(mov: &EndgameMove, square_compressions: &[SquareCompression]) -> Move {
        let square_compression = &square_compressions[usize::from(mov.square_index)];
        for (digit, &compressed_digit) in Digit::all().zip(square_compression.digit_map.iter()) {
            if compressed_digit == OptionalDigit::from(mov.digit) {
                return Move {
                    square: square_compression.prev_index.try_into().unwrap(),
                    digit,
                };
            }
        }
        unreachable!()
    }

    fn generate_moves(
        num_moves_per_square: &[u8],
        square_compressions: &[SquareCompression],
    ) -> Vec<EndgameMove> {
        assert_eq!(num_moves_per_square.len(), square_compressions.len());
        let mut moves = Vec::with_capacity(num_moves_per_square.iter().map(|&x| x as usize).sum());

        for ((&num_moves_sq, square_compression), square_index) in num_moves_per_square
            .iter()
            .zip(square_compressions.iter())
            .zip(0..)
        {
            for (digit, &move_summary) in
                (0..num_moves_sq).zip(square_compression.move_summaries.iter())
            {
                moves.push(EndgameMove {
                    square_index,
                    // Safety: `digit < 9` because `num_moves_sq <= 9`.
                    digit: unsafe { Small::new_unchecked(digit) }.into(),
                    summary: move_summary,
                });
            }
        }
        moves
    }

    fn log_stats(&self, start_time: Instant, end_time: Instant) {
        let processing_time = end_time.saturating_duration_since(start_time);
        log::write_line!(
            Info,
            "nodes: {} time: {:.3?} knps: {:.1}",
            self.num_nodes,
            processing_time,
            self.num_nodes as f64 / processing_time.as_secs_f64() / 1000.0
        );
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EndgameResult {
    Win { difficulty: u32 },
    Loss,
}
