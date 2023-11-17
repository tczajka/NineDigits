use crate::{
    board::{Board, FilledBoard, FullMove, Move},
    digit::Digit,
    error::ResourcesExceeded,
    log,
    memory::{Memory, MemoryBuffer},
    small::Small,
};
use std::time::Instant;

pub struct Endgame {
    memory_buffer: MemoryBuffer,
}

#[derive(Copy, Clone)]
struct EndgamePosition<'a> {
    board: &'a Board,
    // Corresponds to board.empty_squares().
    empty_squares: &'a [EmptySquare],
    // TODO: We need to store the actual moves rather than num_moves.
    num_moves: u16,
    solutions: &'a [u8],
}

impl EndgamePosition<'_> {
    fn decode_move(&self, cmov: CompressedMove) -> Move {
        Move {
            square: self.empty_squares[usize::from(cmov.empty_square_index)].original,
            digit: cmov.original_digit,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct EmptySquare {
    prev_square_index: u8,
    original: Small<81>,
    move_index: u16,
    num_moves: u8,
}

#[derive(Copy, Clone, Debug, Default)]
struct CompressedMove {
    empty_square_index: u8,
    compressed_digit: u8,
    original_digit: Digit,
    solution_count: u32,
}

impl Endgame {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            memory_buffer: MemoryBuffer::new(memory_limit),
        }
    }

    pub fn solve(
        &mut self,
        solutions: &[FilledBoard],
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

        let mut memory = self.memory_buffer.into_memory();

        solve_root_uncompressed(solutions, deadline, &mut memory)
    }
}

fn solve_root_uncompressed(
    solutions: &[FilledBoard],
    deadline: Instant,
    memory: &mut Memory,
) -> (Result<bool, ResourcesExceeded>, FullMove) {
    assert!(solutions.len() > 1);
    let mut solution_counts: Vec<[u32; 9]> = vec![[0; 9]; 81];

    for solution in solutions {
        for (digit, counts) in solution.squares.iter().zip(solution_counts.iter_mut()) {
            counts[Small::from(*digit)] += 1;
        }
    }

    let mut board = Board::new();
    let mut empty_squares = Vec::with_capacity(81);
    let mut moves = Vec::with_capacity(81 * 9);

    for (square, s_counts) in Small::<81>::all().zip(solution_counts.iter()) {
        let mut empty_square = EmptySquare {
            prev_square_index: u8::from(square),
            original: square,
            move_index: moves.len() as u16,
            num_moves: 0,
        };
        for (digit, &solution_count) in Digit::all().zip(s_counts) {
            if solution_count != 0 {
                if solution_count == 1 {
                    return (Ok(true), FullMove::MoveClaimUnique(Move { square, digit }));
                }
                moves.push(CompressedMove {
                    empty_square_index: empty_squares.len() as u8,
                    compressed_digit: empty_square.num_moves,
                    original_digit: digit,
                    solution_count,
                });
                empty_square.num_moves += 1;
            }
        }
        assert!(empty_square.num_moves != 0);
        if empty_square.num_moves == 1 {
            let mov = moves.pop().unwrap();
            board
                .make_move(Move {
                    square,
                    digit: mov.original_digit,
                })
                .unwrap();
        } else {
            empty_squares.push(empty_square);
        }
    }

    let (compressed_solutions, mut memory) = memory
        .allocate_slice::<u8>(solutions.len() * empty_squares.len())
        .expect("out of memory");

    for (solution, compressed_solution) in solutions
        .iter()
        .zip(compressed_solutions.chunks_exact_mut(empty_squares.len()))
    {
        for (empty_square, new_digit) in empty_squares.iter().zip(compressed_solution.iter_mut()) {
            let digit = solution.squares[empty_square.original];
            *new_digit = u8::from(Small::from(digit));
        }
    }

    let position = EndgamePosition {
        board: &board,
        empty_squares: &empty_squares,
        num_moves: moves.len() as u16,
        solutions: compressed_solutions,
    };

    solve_root_compressed(position, &mut moves, deadline, &mut memory)
}

fn solve_root_compressed(
    position: EndgamePosition,
    moves: &mut [CompressedMove],
    deadline: Instant,
    memory: &mut Memory,
) -> (Result<bool, ResourcesExceeded>, FullMove) {
    moves.sort_by_key(|x| x.solution_count);

    let mut result = Ok(false);

    for &mov in moves.iter() {
        if mov.solution_count == 1 {
            return (
                Ok(true),
                FullMove::MoveClaimUnique(position.decode_move(mov)),
            );
        } else if mov.solution_count < 4 {
            // Ignore: a losing move.
        } else {
            match solve_move(position, mov, deadline, &mut *memory) {
                Ok(true) => {
                    return (Ok(true), FullMove::Move(position.decode_move(mov)));
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
    let &mov = moves.last().unwrap();
    (result, FullMove::Move(position.decode_move(mov)))
}

fn solve_move(
    position: EndgamePosition,
    mov: CompressedMove,
    deadline: Instant,
    memory: &mut Memory,
) -> Result<bool, ResourcesExceeded> {
    let n = position.empty_squares.len();
    let (new_solutions, mut memory) =
        memory.allocate_slice::<u8>(mov.solution_count as usize * n)?;

    for (solution, new_solution) in position
        .solutions
        .chunks_exact(n)
        .filter(|solution| solution[usize::from(mov.empty_square_index)] == mov.compressed_digit)
        .zip(new_solutions.chunks_exact_mut(n))
    {
        new_solution.copy_from_slice(solution);
    }

    let new_position = EndgamePosition {
        board: position.board,
        empty_squares: position.empty_squares,
        num_moves: position.num_moves,
        solutions: new_solutions,
    };

    solve_uncompressed(new_position, deadline, &mut memory)
}

fn solve_uncompressed(
    position: EndgamePosition,
    deadline: Instant,
    memory: &mut Memory,
) -> Result<bool, ResourcesExceeded> {
    assert!(position.solutions.len() > position.empty_squares.len());

    let (solution_counts, mut memory) = memory.allocate_slice::<u32>(position.num_moves.into())?;

    let mut solution_counts_remaining = &mut solution_counts[..];
    for solution in position
        .solutions
        .chunks_exact(position.empty_squares.len())
    {
        for (digit, empty_square) in solution.iter().zip(position.empty_squares.iter()) {
            let (counts, tail) =
                solution_counts_remaining.split_at_mut(empty_square.num_moves.into());
            solution_counts_remaining = tail;
            counts[usize::from(*digit)] += 1;
        }
    }
    assert!(solution_counts_remaining.is_empty());

    let mut board = *position.board;

    // Overallocate empty_squares and moves.
    let (empty_squares, mut memory) =
        memory.allocate_slice::<EmptySquare>(position.empty_squares.len())?;
    let mut num_empty_squares = 0;
    let (moves, mut memory) = memory.allocate_slice::<CompressedMove>(position.num_moves.into())?;
    let mut num_moves = 0;

    let mut solution_counts_remaining: &[u32] = solution_counts;
    for (empty_square_index, empty_square) in position.empty_squares.iter().enumerate() {
        let (s_counts, tail) = solution_counts_remaining.split_at(empty_square.num_moves.into());
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
                num_moves += 1;
                new_empty_square.num_moves += 1;
            }
        }
        assert!(empty_square.num_moves != 0);
        if empty_square.num_moves == 1 {
            num_moves -= 1;
            let mov = moves[usize::from(num_moves)];
            board
                .make_move(Move {
                    square: empty_square.original,
                    digit: mov.original_digit,
                })
                .unwrap();
        } else {
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
        for (empty_square, new_digit) in empty_squares.iter().zip(compressed_solution.iter_mut()) {
            *new_digit = solution[usize::from(empty_square.prev_square_index)];
        }
    }

    let compressed_position = EndgamePosition {
        board: &board,
        empty_squares,
        num_moves: moves.len() as u16,
        solutions: compressed_solutions,
    };

    solve_compressed(compressed_position, moves, deadline, &mut memory)
}

fn solve_compressed(
    position: EndgamePosition,
    moves: &mut [CompressedMove],
    deadline: Instant,
    memory: &mut Memory,
) -> Result<bool, ResourcesExceeded> {
    moves.sort_by_key(|x| x.solution_count);

    for &mov in moves.iter() {
        if mov.solution_count == 1 {
            // TODO: Impossible -- remove?
            return Ok(true);
        } else if mov.solution_count < 4 {
            // Ignore: a losing move.
        } else if solve_move(position, mov, deadline, &mut *memory)? {
            return Ok(true);
        }
    }
    Ok(false)
}
