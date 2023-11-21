use crate::{
    board::FullMove, digit::Digit, error::ResourcesExceeded, log, solution_table::SolutionTable,
};
use std::time::Instant;

#[derive(Copy, Clone, Debug, Default)]
struct EndgameMove {
    empty_square_index: u8,
    digit: Digit,
    num_solutions: u32,
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

        todo!()
    }

    /*
    /// Returns a move even if can't solve.
    ///
    /// Panics if `memory` does not have enough space for depth 1 search.
    pub fn solve_with_move(
        &mut self,
        solutions: &[Solution],
        deadline: Instant,
        memory: &mut Memory,
    ) -> (Result<bool, ResourcesExceeded>, FullMove) {
        let num_solutions_table = Self::count_solutions_root(solutions);
        if let Some(mov) = self.check_quick_win_root(&num_solutions_table) {
            return (Ok(true), mov);
        }
        let empty_squares = Self::calc_empty_squares_root(&num_solutions_table);
        let (compressed_solutions, mut memory) =
            Self::compress_root_solutions(solutions, &empty_squares, &digit_compression, memory);
        let mut moves =
            Self::generate_root_moves(&empty_squares, &digit_compression, &num_solutions_table);
        let (res, mov) = self.final_solve_root_with_move(
            &empty_squares,
            compressed_solutions,
            &mut moves,
            deadline,
            &mut memory,
        );

        (
            res,
            FullMove::Move(Self::move_from_endgame_move(
                mov,
                &empty_squares,
                &digit_compression,
            )),
        )
    }

    // Some empty squares may have unique digits in solutions.
    fn solve(
        &self,
        old_empty_squares: &[EmptySquare],
        old_num_moves: usize, // total over old_empty_squares
        solutions: &[u8],
        deadline: Instant,
        memory: &mut Memory,
    ) -> Result<bool, ResourcesExceeded> {
        assert!(solutions.len() > old_empty_squares.len());

        let (num_solutions_table, mut memory) =
            Self::count_solutions(old_empty_squares, old_num_moves, solutions, memory)?;

        if self.check_quick_win(num_solutions_table) {
            return Ok(true);
        }

        // Overallocate empty_squares.
        let (empty_squares, mut memory) =
            memory.allocate_slice::<EmptySquare>(old_empty_squares.len())?;
        let (empty_squares, digit_compression, mut memory) = Self::plan_compression(
            old_empty_squares,
            num_solutions_table,
            empty_squares,
            &mut memory,
        )?;

        let (compressed_solutions, mut memory) = Self::compress_solutions(
            old_empty_squares,
            solutions,
            empty_squares,
            digit_compression,
            &mut memory,
        )?;

        let (moves, mut memory) = Self::generate_moves(
            old_empty_squares,
            old_num_moves,
            empty_squares,
            digit_compression,
            num_solutions_table,
            &mut memory,
        )?;

        todo!()
        /*
        let (moves, mut memory) =
            memory.allocate_slice::<CompressedMove>(position.num_moves.into())?;
        let mut num_moves = 0;

        let mut solution_counts_remaining: &[u32] = solution_counts;
        for (empty_square_index, empty_square) in position.empty_squares.iter().enumerate() {
            let (s_counts, tail) =
                solution_counts_remaining.split_at(empty_square.num_moves.into());
            solution_counts_remaining = tail;
            let mut new_empty_square = EmptySquare {
                prev_square_index: empty_square_index as u8,
                original: empty_square.original,
                move_index: num_moves,
                num_moves: 0,
            };

            // We are missing original digits here.
            for &solution_count in s_counts {
                if solution_count != 0 {
                    if solution_count == 1 {
                        return Ok(true);
                    }
                    moves[usize::from(num_moves)] = CompressedMove {
                        empty_square_index: num_empty_squares,
                        compressed_digit: new_empty_square.num_moves,
                        original_digit: Small::new(0).into(), // TODO: fix, store original digits!
                        solution_count,
                    };
                    new_empty_square.num_moves += 1;
                }
            }
            assert!(empty_square.num_moves != 0);
            if empty_square.num_moves != 1 {
                num_moves += u16::from(new_empty_square.num_moves);
                empty_squares[usize::from(num_empty_squares)] = new_empty_square;
                num_empty_squares += 1;
            }
        }
        assert!(solution_counts_remaining.is_empty());
        let empty_squares = &empty_squares[..usize::from(num_empty_squares)];
        let moves = &mut moves[..usize::from(num_moves)];

        let (compressed_solutions, mut memory) = memory.allocate_slice::<u8>(
            position.solutions.len() / position.empty_squares.len() * empty_squares.len(),
        )?;

        for (solution, compressed_solution) in position
            .solutions
            .chunks_exact(position.empty_squares.len())
            .zip(compressed_solutions.chunks_exact_mut(empty_squares.len()))
        {
            for (empty_square, new_digit) in
                empty_squares.iter().zip(compressed_solution.iter_mut())
            {
                *new_digit = solution[usize::from(empty_square.prev_square_index)];
            }
        }

        let compressed_position = EndgamePosition {
            empty_squares,
            num_moves: moves.len() as u16,
            solutions: compressed_solutions,
        };

        solve_compressed(compressed_position, moves, deadline, &mut memory)
        */
    }

    fn move_from_endgame_move(
        mov: EndgameMove,
        empty_squares: &[EmptySquare],
        digit_compression: &[[OptionalDigit; 9]],
    ) -> Move {
        let square = Small::new(empty_squares[usize::from(mov.empty_square_index)].square);
        let dc = &digit_compression[usize::from(mov.empty_square_index)];
        for (digit, &compressed_digit) in Digit::all().zip(dc.iter()) {
            if OptionalDigit::from(mov.digit) == compressed_digit {
                return Move { square, digit };
            }
        }
        unreachable!()
    }

    // num_solutions_table[square][digit]
    fn count_solutions_root(solutions: &[Solution]) -> Box<[[u32; 9]; 81]> {
        let mut num_solutions_table: Box<[[u32; 9]; 81]> = (0..81)
            .map(|_| [0; 9])
            .collect::<Box<[[u32; 9]]>>()
            .try_into()
            .unwrap();

        for solution in solutions {
            for (&digit, counts) in solution
                .filled_board
                .squares
                .iter()
                .zip(num_solutions_table.iter_mut())
            {
                counts[digit] += 1;
            }
        }

        num_solutions_table
    }

    fn count_solutions<'a>(
        empty_squares: &[EmptySquare],
        num_moves: usize,
        solutions: &[u8],
        memory: &'a mut Memory,
    ) -> Result<(&'a [[u32; 9]], Memory<'a>), ResourcesExceeded> {
        let (num_solutions_table, memory) = memory.allocate_slice::<[u32; 9]>(num_moves)?;
        let solution_len = empty_squares.len();

        for solution in solutions.chunks_exact(solution_len) {
            for ((digit, empty_square), counts) in solution
                .iter()
                .map(|&d| unsafe { Digit::from(Small::new_unchecked(d)) })
                .zip(empty_squares.iter())
                .zip(num_solutions_table.iter_mut())
            {
                counts[digit] += 1;
            }
        }
        Ok((num_solutions_table, memory))
    }

    fn check_quick_win_root(&self, num_solutions_table: &[[u32; 9]; 81]) -> Option<FullMove> {
        // TODO: Check transposition table.
        for (square, square_solutions_table) in Small::<81>::all().zip(num_solutions_table.iter()) {
            for (digit, &num_solutions) in Digit::all().zip(square_solutions_table) {
                if num_solutions == 1 {
                    return Some(FullMove::MoveClaimUnique(Move { square, digit }));
                }
            }
        }
        None
    }

    fn check_quick_win(&self, num_solutions_table: &[[u32; 9]]) -> bool {
        // TODO: Check transposition table.
        for square_solutions_table in num_solutions_table {
            for &num_solutions in square_solutions_table {
                if num_solutions == 1 {
                    return true;
                }
            }
        }
        false
    }

    /// Returns (empty_squares, digit_compression)
    fn plan_root_compression(
        num_solutions_table: &[[u32; 9]; 81],
    ) -> (Vec<EmptySquare>, Vec<[OptionalDigit; 9]>) {
        let mut empty_squares = Vec::with_capacity(81);
        let mut digit_compression = Vec::with_capacity(81);

        for (square, square_solutions_table) in Small::<81>::all().zip(num_solutions_table.iter()) {
            let mut num_moves = 0;
            let mut dc = [OptionalDigit::NONE; 9];
            for (digit, &num_solutions) in Digit::all().zip(square_solutions_table.iter()) {
                if num_solutions != 0 {
                    // Safety: `num_moves < 9`.
                    dc[digit] = Digit::from(unsafe { Small::new_unchecked(num_moves) }).into();
                    num_moves += 1;
                }
            }
            if num_moves != 1 {
                empty_squares.push(EmptySquare {
                    square: square.into(),
                    num_moves,
                });
                digit_compression.push(dc);
            }
        }
        (empty_squares, digit_compression)
    }

    /// Returns (empty_squares, digit_compression)
    fn plan_compression<'a, 'b>(
        old_empty_squares: &[EmptySquare],
        num_solutions_table: &[[u32; 9]],
        empty_squares: &'a mut [EmptySquare],
        memory: &'b mut Memory,
    ) -> Result<(&'a mut [EmptySquare], &'b [[OptionalDigit; 9]], Memory<'b>), ResourcesExceeded>
    {
        let mut num_empty_squares = 0;
        // Overallocated.
        let (digit_compression, memory) = memory.allocate_slice(old_empty_squares.len())?;

        for ((old_empty_square_index, old_empty_square), square_solutions_table) in (0..)
            .zip(old_empty_squares.iter())
            .zip(num_solutions_table.iter())
        {
            let old_num_moves = old_empty_square.num_moves;
            let mut dc = [OptionalDigit::NONE; 9];

            let mut num_moves = 0;
            for (digit, &num_solutions) in (0..old_num_moves).zip(square_solutions_table.iter()) {
                if num_solutions != 0 {
                    dc[usize::from(digit)] =
                        Digit::from(unsafe { Small::new_unchecked(num_moves) }).into();
                    num_moves += 1;
                }
            }

            if num_moves != 1 {
                empty_squares[num_empty_squares] = EmptySquare {
                    square: old_empty_square_index,
                    num_moves,
                };
                digit_compression[num_empty_squares] = dc;
                num_empty_squares += 1;
            }
        }

        Ok((
            &mut empty_squares[..num_empty_squares],
            &digit_compression[..num_empty_squares],
            memory,
        ))
    }

    fn compress_root_solutions<'a>(
        solutions: &[Solution],
        empty_squares: &[EmptySquare],
        digit_compression: &[[u8; 9]],
        memory: &'a mut Memory,
    ) -> (&'a mut [u8], Memory<'a>) {
        let compressed_solution_len = empty_squares.len();
        let (compressed_solutions, memory) = memory
            .allocate_slice::<u8>(solutions.len() * compressed_solution_len)
            .expect("out of memory");

        for (solution, compressed_solution) in solutions
            .iter()
            .zip(compressed_solutions.chunks_exact_mut(compressed_solution_len))
        {
            for ((empty_square, dc), compressed_digit) in empty_squares
                .iter()
                .zip(digit_compression.iter())
                .zip(compressed_solution.iter_mut())
            {
                let digit = solution.filled_board.squares[usize::from(empty_square.square)];
                *compressed_digit = dc[digit];
            }
        }

        (compressed_solutions, memory)
    }

    fn compress_solutions<'a>(
        old_empty_squares: &[EmptySquare],
        solutions: &[u8],
        empty_squares: &[EmptySquare],
        digit_compression: &[u8],
        memory: &'a mut Memory,
    ) -> Result<(&'a mut [u8], Memory<'a>), ResourcesExceeded> {
        let old_solution_len = old_empty_squares.len();
        let compressed_solution_len = empty_squares.len();

        let (compressed_solutions, mut memory) = memory
            .allocate_slice::<u8>(solutions.len() / old_solution_len * compressed_solution_len)?;

        let mut digit_compression_remaining = digit_compression;

        for (solution, compressed_solution) in solutions
            .chunks_exact(old_solution_len)
            .zip(compressed_solutions.chunks_exact_mut(compressed_solution_len))
        {
            for (empty_square, compressed_digit) in
                empty_squares.iter().zip(compressed_solution.iter_mut())
            {
                let old_num_moves = old_empty_squares[usize::from(empty_square.square)].num_moves;
                let (dc, tail) = digit_compression_remaining.split_at(old_num_moves.into());
                digit_compression_remaining = tail;
                let digit = solution[usize::from(empty_square.square)];
                *compressed_digit = dc[usize::from(digit)];
            }
        }
        assert!(digit_compression_remaining.is_empty());

        Ok((compressed_solutions, memory))
    }

    fn generate_root_moves(
        empty_squares: &[EmptySquare],
        digit_compression: &[[u8; 9]],
        num_solutions_table: &[[u32; 9]; 81],
    ) -> Vec<EndgameMove> {
        let mut moves = Vec::with_capacity(empty_squares.len() * 9);

        for (empty_square_index, (empty_square, dc)) in empty_squares
            .iter()
            .zip(digit_compression.iter())
            .enumerate()
        {
            for (digit, &num_solutions) in
                Digit::all().zip(num_solutions_table[usize::from(empty_square.square)].iter())
            {
                if num_solutions != 0 {
                    moves.push(EndgameMove {
                        empty_square_index: empty_square_index as u8,
                        digit: dc[digit],
                        num_solutions,
                    });
                }
            }
        }
        moves
    }

    fn generate_moves<'a>(
        old_empty_squares: &[EmptySquare],
        old_num_moves: usize,
        empty_squares: &[EmptySquare],
        digit_compression: &[u8],
        num_solutions_table: &[u32],
        memory: &'a mut Memory,
    ) -> Result<(&'a mut EndgameMove, Memory<'a>), ResourcesExceeded> {
        let (moves, mut memory) = memory.allocate_slice::<EndgameMove>(old_num_moves)?;
        let mut num_moves = 0;

        let mut digit_compression_remaining = digit_compression;
        let mut num_solutions_table_remaining = num_solutions_table;
        for (empty_square_index, empty_square) in empty_squares.iter().enumerate() {
            let old_num_moves = old_empty_squares[usize::from(empty_square.square)].num_moves;
            let (dc, tail) = digit_compression_remaining.split_at(old_num_moves.into());
            digit_compression_remaining = tail;
            for (digit, &num_solutions) in (0..old_num_moves).zip(num_solutions_table) {
                if num_solutions != 0 {
                    moves[usize::from(num_moves)] = EndgameMove {
                        empty_square_index: empty_square_index as u8,
                        digit: dc[usize::from(digit)],
                        num_solutions,
                    };
                    num_moves += 1;
                }
            }
        }

        assert!(digit_compression_remaining.is_empty());
        todo!()
    }

    fn final_solve_root_with_move(
        &self,
        empty_squares: &[EmptySquare],
        solutions: &[u8],
        moves: &mut [EndgameMove],
        deadline: Instant,
        memory: &mut Memory,
    ) -> (Result<bool, ResourcesExceeded>, EndgameMove) {
        moves.sort_by_key(|x| x.num_solutions);

        let mut result = Ok(false);

        for &mov in moves.iter() {
            if mov.num_solutions < 4 {
                // Ignore: a losing move.
                // Immediate wins (1) are already handled by `check_quick_win_root`.
            } else {
                match self.solve_move(
                    empty_squares,
                    moves.len(),
                    solutions,
                    mov,
                    deadline,
                    &mut *memory,
                ) {
                    Ok(true) => {
                        return (Ok(true), mov);
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
        }

        // Did not find a winning move.
        // Use the move with the most solutions.
        // TODO: Defense, try last move.
        (result, *moves.last().unwrap())
    }

    fn solve_move(
        &self,
        empty_squares: &[EmptySquare],
        num_moves: usize, // total over empty squares
        solutions: &[u8],
        mov: EndgameMove,
        deadline: Instant,
        memory: &mut Memory,
    ) -> Result<bool, ResourcesExceeded> {
        let solution_len = empty_squares.len();
        let (filtered_solutions, mut memory) =
            memory.allocate_slice::<u8>(mov.num_solutions as usize * solution_len)?;

        for (solution, new_solution) in solutions
            .chunks_exact(solution_len)
            .filter(|solution| solution[usize::from(mov.empty_square_index)] == mov.digit)
            .zip(filtered_solutions.chunks_exact_mut(solution_len))
        {
            new_solution.copy_from_slice(solution);
        }

        self.solve(
            empty_squares,
            num_moves,
            filtered_solutions,
            deadline,
            &mut memory,
        )
    }
    */
}
