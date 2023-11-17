use crate::{
    board::{Board, FilledBoard, FullMove, Move},
    digit::Digit,
    error::ResourcesExceeded,
    log,
    memory::{Memory, MemoryRemaining},
    small::Small,
};
use std::time::Instant;

#[derive(Debug)]
pub struct Endgame {
    memory: Memory,
}

struct EndgamePosition<'a> {
    board: Board,
    // Corresponds to board.empty_squares().
    empty_squares: &'a [Small<81>],
    solutions: &'a [u8],
}

impl EndgamePosition<'_> {
    fn decode_move(&self, cmov: CompressedMove) -> Move {
        Move {
            square: self.empty_squares[usize::from(cmov.empty_square_index)],
            digit: cmov.original_digit,
        }
    }
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
            memory: Memory::new(memory_limit),
        }
    }

    pub fn solve_best_effort(
        &mut self,
        solutions: &[FilledBoard],
        deadline: Instant,
    ) -> (Option<bool>, FullMove) {
        if solutions.is_empty() {
            log::write_line!(Always, "Error: no solutions!");
            // No good option, let's just claim victory.
            return (None, FullMove::ClaimUnique);
        }
        if solutions.len() == 1 {
            log::write_line!(Info, "Lucky win: opponent didn't claim.");
            return (Some(true), FullMove::ClaimUnique);
        }

        let mut solution_counts: Vec<[u32; 9]> = vec![[0; 9]; 81];

        for solution in solutions {
            for (square, digit) in solution.squares.iter().enumerate() {
                solution_counts[square][Small::<9>::from(*digit)] += 1;
            }
        }

        let mut board = Board::new();
        let mut empty_squares = Vec::with_capacity(81);
        let mut moves = Vec::with_capacity(81 * 9);

        for square in Small::<81>::all() {
            let mut num_moves = 0;
            for digit in Digit::all() {
                let solution_count = solution_counts[usize::from(square)][Small::from(digit)];
                if solution_count != 0 {
                    moves.push(CompressedMove {
                        empty_square_index: empty_squares.len() as u8,
                        compressed_digit: num_moves,
                        original_digit: digit,
                        solution_count,
                    });
                    num_moves += 1;
                };
            }
            assert!(num_moves != 0);
            if num_moves == 1 {
                let mov = moves.pop().unwrap();
                board
                    .make_move(Move {
                        square,
                        digit: mov.original_digit,
                    })
                    .unwrap();
            } else {
                empty_squares.push(square);
            }
        }

        let mut memory = self.memory.into_remaining();
        let (compressed_solutions, mut memory) = memory
            .allocate_slice::<u8>(solutions.len() * empty_squares.len())
            .expect("out of memory");

        let mut compressed_solutions_next = 0;
        for solution in solutions {
            for &square in empty_squares.iter() {
                let digit = solution.squares[square];
                compressed_solutions[compressed_solutions_next] = u8::from(Small::from(digit));
                compressed_solutions_next += 1;
            }
        }
        assert_eq!(compressed_solutions_next, compressed_solutions.len());

        let position = EndgamePosition {
            board,
            empty_squares: &empty_squares,
            solutions: &compressed_solutions,
        };

        moves.sort_by_key(|x| x.solution_count);

        let mut result = Some(false);

        for &mov in moves.iter() {
            if mov.solution_count == 1 {
                return (
                    Some(true),
                    FullMove::MoveClaimUnique(position.decode_move(mov)),
                );
            } else if mov.solution_count < 4 {
                // Ignore: a losing move.
            } else {
                match solve_move(&position, mov, deadline, &mut memory) {
                    Ok(true) => {
                        return (Some(true), FullMove::Move(position.decode_move(mov)));
                    }
                    Ok(false) => {
                        // Ignore: a losing move.
                    }
                    Err(e) => {
                        result = None;
                        log::write_line!(Info, "terminated: {e}");
                        break;
                    }
                }
            }
        }

        let &mov = moves.last().unwrap();
        (result, FullMove::Move(position.decode_move(mov)))
    }
}

fn solve_move(
    position: &EndgamePosition,
    mov: CompressedMove,
    deadline: Instant,
    memory: &mut MemoryRemaining,
) -> Result<bool, ResourcesExceeded> {
    todo!()
}
