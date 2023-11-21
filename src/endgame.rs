use crate::{
    board::{FullMove, Move},
    digit::{Digit, OptionalDigit},
    error::ResourcesExceeded,
    log,
    small::Small,
    solution_table::{MoveSummary, SolutionTable, SquareCompression},
};
use std::time::Instant;

#[derive(Copy, Clone, Debug)]
struct EndgameMove {
    square_index: u8,
    digit: Digit,
    summary: MoveSummary,
}

pub struct EndgameSolver;

impl EndgameSolver {
    pub fn new() -> Self {
        Self
    }

    pub fn solve_best_effort(
        &mut self,
        solutions: &SolutionTable,
        deadline: Instant,
    ) -> (Result<bool, ResourcesExceeded>, FullMove) {
        if solutions.is_empty() {
            log::write_line!(Always, "Error: no solutions!");
            // No good option, let's just claim victory.
            return (Ok(false), FullMove::ClaimUnique);
        }
        if solutions.len() == 1 {
            log::write_line!(Info, "Lucky win: opponent didn't claim.");
            return (Ok(true), FullMove::ClaimUnique);
        }

        let move_summaries = solutions.move_summaries();

        if let Some(mov) = self.check_quick_win_root(&move_summaries) {
            return (Ok(true), mov);
        }

        let (solutions, square_compressions) = solutions.compress(&move_summaries);

        let mut moves =
            Self::generate_moves(solutions.num_moves_per_square(), &square_compressions);
        moves.sort_by_key(|x| x.summary.num_solutions);

        let mut result = Ok(false);
        for mov in moves.iter() {
            if mov.summary.num_solutions < 4 {
                // Ignore: a losing move.
                // Immediate wins (1) are already handled by `check_quick_win_root`.
                continue;
            }
            let new_solutions = solutions.filter(
                mov.summary.num_solutions.try_into().unwrap(),
                mov.square_index.into(),
                mov.digit,
            );
            // TODO: Check smaller deadline here.
            match self.solve(&new_solutions, deadline) {
                Ok(true) => {
                    return (
                        Ok(true),
                        FullMove::Move(Self::uncompress_root_move(mov, &square_compressions)),
                    );
                }
                Ok(false) => {
                    // Ignore: a losing move.
                }
                Err(e) => {
                    result = Err(e);
                    log::write_line!(Info, "terminated: {e}");
                    break;
                }
            }
        }

        // Did not find a winning move.
        // Use the move with the most solutions.
        // TODO: Defense, try last move.
        (
            result,
            FullMove::Move(Self::uncompress_root_move(
                moves.last().unwrap(),
                &square_compressions,
            )),
        )
    }

    pub fn solve(
        &mut self,
        solutions: &SolutionTable,
        deadline: Instant,
    ) -> Result<bool, ResourcesExceeded> {
        // TODO: Check less often.
        if Instant::now() >= deadline {
            return Err(ResourcesExceeded::Time);
        }

        assert!(solutions.len() >= 2);
        let move_summaries = solutions.move_summaries();

        if self.check_quick_win(solutions.num_moves_per_square(), &move_summaries) {
            return Ok(true);
        }

        let (solutions, square_compressions) = solutions.compress(&move_summaries);

        let mut moves =
            Self::generate_moves(solutions.num_moves_per_square(), &square_compressions);
        moves.sort_by_key(|x| x.summary.num_solutions);

        for mov in moves.iter() {
            if mov.summary.num_solutions < 4 {
                // Ignore: a losing move.
                // Immediate wins (1) are already handled by `check_quick_win_root`.
                continue;
            }
            let new_solutions = solutions.filter(
                mov.summary.num_solutions.try_into().unwrap(),
                mov.square_index.into(),
                mov.digit,
            );
            if self.solve(&new_solutions, deadline)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn check_quick_win_root(&self, move_summaries: &[[MoveSummary; 9]]) -> Option<FullMove> {
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

        // TODO: Enhanced transposition cutoff.

        None
    }

    fn check_quick_win(
        &self,
        num_moves_per_square: &[u8],
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

        // TODO: Enhanced transposition cutoff.

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
