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

    pub fn solve_best_effort(
        &mut self,
        solutions: &SolutionTable,
        start_time: Instant,
        time_left: Duration,
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

        let deadline = start_time + time_left;
        let mut result = Ok(false);
        let mut winning_move = None;
        for mov in moves.iter() {
            // TODO: Check smaller deadline here.
            match self.solve_move(&solutions, mov, deadline) {
                Ok(true) => {
                    result = Ok(true);
                    winning_move = Some(FullMove::Move(Self::uncompress_root_move(
                        mov,
                        &square_compressions,
                    )));
                    break;
                }
                Ok(false) => {
                    // Ignore: a losing move.
                }
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }

        let chosen_move = match winning_move {
            Some(mov) => mov,
            None => {
                // Did not find a winning move.
                // Use the move with the most solutions.
                // TODO: Defense, try last move.
                FullMove::Move(Self::uncompress_root_move(
                    moves.last().unwrap(),
                    &square_compressions,
                ))
            }
        };

        match result {
            Ok(true) => {
                log::write_line!(Info, "endgame win");
            }
            Ok(false) => {
                log::write_line!(Info, "endgame lose");
            }
            Err(e) => {
                log::write_line!(Info, "endgame {e}");
            }
        }

        let processing_time = Instant::now().saturating_duration_since(start_time);
        log::write_line!(
            Info,
            "nodes: {} time: {:.3?} knps: {:.1}",
            self.num_nodes,
            processing_time,
            self.num_nodes as f64 / processing_time.as_secs_f64() / 1000.0
        );

        chosen_move
    }

    pub fn solve(
        &mut self,
        solutions: &SolutionTable,
        start_time: Instant,
        time_left: Duration,
    ) -> Result<bool, ResourcesExceeded> {
        self.transposition_table.new_era();
        self.num_nodes = 0;

        if solutions.len() < 4 {
            return Ok(solutions.len() > 1);
        }
        if let Some(result) = self.transposition_table.find(solutions.hash()) {
            return Ok(result);
        }
        let res = self.solve_recursive(solutions, start_time + time_left);

        let processing_time = Instant::now().saturating_duration_since(start_time);
        log::write_line!(
            Info,
            "nodes: {} time: {:.3?} knps: {:.1}",
            self.num_nodes,
            processing_time,
            self.num_nodes as f64 / processing_time.as_secs_f64() / 1000.0
        );

        res
    }

    /// Already checked that there are at least 4 solutions and that this not in the transposition table.
    fn solve_recursive(
        &mut self,
        solutions: &SolutionTable,
        deadline: Instant,
    ) -> Result<bool, ResourcesExceeded> {
        self.num_nodes += 1;

        if self.num_nodes % settings::ENDGAME_CHECK_TIME_NODES == 0 && Instant::now() >= deadline {
            return Err(ResourcesExceeded::Time);
        }

        let move_summaries = solutions.move_summaries();

        let result = self.check_quick_win(
            solutions.num_moves_per_square(),
            solutions.len(),
            &move_summaries,
        ) || {
            let (solutions, square_compressions) = solutions.compress(&move_summaries);

            let mut moves =
                Self::generate_moves(solutions.num_moves_per_square(), &square_compressions);
            moves.sort_by_key(|x| x.summary.num_solutions);

            let mut result = false;
            for mov in moves.iter() {
                if self.solve_move(&solutions, mov, deadline)? {
                    result = true;
                    break;
                }
            }
            result
        };
        self.transposition_table
            .insert(solutions.hash(), solutions.len() as u32, result);
        Ok(result)
    }

    fn solve_move(
        &mut self,
        solutions: &SolutionTable,
        mov: &EndgameMove,
        deadline: Instant,
    ) -> Result<bool, ResourcesExceeded> {
        if mov.summary.num_solutions < 4 {
            return Ok(mov.summary.num_solutions == 1);
        }
        if let Some(result) = self.transposition_table.find(mov.summary.hash) {
            return Ok(!result);
        }
        let new_solutions = solutions.filter(
            mov.summary.num_solutions.try_into().unwrap(),
            mov.square_index.into(),
            mov.digit,
        );
        assert_eq!(new_solutions.len(), mov.summary.num_solutions as usize);
        assert_eq!(new_solutions.hash(), mov.summary.hash);
        let res = self.solve_recursive(&new_solutions, deadline)?;
        Ok(!res)
    }

    fn check_quick_win_root(
        &self,
        num_solutions: usize,
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
                    && move_summary.num_solutions < num_solutions as u32
                    && self.transposition_table.find(move_summary.hash) == Some(false)
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
        num_solutions: usize,
        move_summaries: &[[MoveSummary; 9]],
    ) -> bool {
        assert_eq!(num_moves_per_square.len(), move_summaries.len());
        for (&num_moves, move_summaries_sq) in
            num_moves_per_square.iter().zip(move_summaries.iter())
        {
            for move_summary in &move_summaries_sq[..usize::from(num_moves)] {
                if move_summary.num_solutions == 1 {
                    return true;
                }
            }
        }

        // Enhanced transposition cutoff.
        for (&num_moves, move_summaries_sq) in
            num_moves_per_square.iter().zip(move_summaries.iter())
        {
            for move_summary in &move_summaries_sq[..usize::from(num_moves)] {
                if move_summary.num_solutions >= 4
                    && move_summary.num_solutions < num_solutions as u32
                    && self.transposition_table.find(move_summary.hash) == Some(false)
                {
                    return true;
                }
            }
        }
        false
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
}
