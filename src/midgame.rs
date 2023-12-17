use std::time::Instant;

use crate::{
    board::{Board, FilledBoard, Move},
    digit::Digit,
    digit_set::DigitSet,
    error::ResourcesExceeded,
    fast_solver::FastSolver,
    log, settings,
    small::Small,
    solution_table::SolutionTable,
    solver::{Solver, SolverStep},
};

/// Returns (normalized board, all possible moves)
pub fn generate_moves(
    board: &mut Board,
    partial_solutions: &SolutionTable,
    deadline: Instant,
) -> Vec<MidgameMove> {
    let mut counts: Vec<[u32; 9]> = vec![[0; 9]; 81];
    let counts: &mut [[u32; 9]; 81] = counts.as_mut_slice().try_into().unwrap();
    for solution in partial_solutions.iter() {
        increment_counts(counts, solution.digits().try_into().unwrap());
    }
    let mut num_solutions = partial_solutions.len();
    'outer: for square in board.empty_squares() {
        let mut possible = DigitSet::EMPTY;
        for digit in Digit::all() {
            if counts[square][digit] != 0 {
                possible.insert(digit);
            }
        }
        while possible != DigitSet::all() {
            match find_one_solution_except(&*board, square, possible, deadline) {
                Ok(Some(filled_board)) => {
                    num_solutions += 1;
                    increment_counts(counts, &filled_board.squares);
                    possible.insert(filled_board.squares[square]);
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    log::write_line!(Info, "midgame::generate_moves error: {e}");
                    break 'outer;
                }
            }
        }
        let first_digit = possible.smallest().unwrap();
        possible.remove(first_digit);
        if possible.is_empty() {
            board
                .make_move(Move {
                    square,
                    digit: first_digit,
                })
                .unwrap();
        }
    }
    let mut moves = Vec::with_capacity(81 * 9);
    for square in board.empty_squares() {
        for digit in Digit::all() {
            let num_solutions_lower_bound = counts[square][digit];
            if num_solutions_lower_bound != 0 && num_solutions_lower_bound != num_solutions {
                moves.push(MidgameMove {
                    mov: Move { square, digit },
                    num_solutions_lower_bound,
                });
            }
        }
    }
    moves
}

fn increment_counts(counts: &mut [[u32; 9]; 81], solution: &[Digit; 81]) {
    for (square, &digit) in Small::<81>::all().zip(solution.iter()) {
        counts[square][digit] += 1;
    }
}

fn find_one_solution_except(
    board: &Board,
    square: Small<81>,
    except: DigitSet,
    deadline: Instant,
) -> Result<Option<FilledBoard>, ResourcesExceeded> {
    let mut solver = FastSolver::new(board);
    solver.remove_possibilities(square, except);
    let mut iters: u64 = 0;
    loop {
        iters += 1;
        if iters == settings::SOLUTION_GENERATE_CHECK_TIME_ITERS {
            iters = 0;
            if Instant::now() >= deadline {
                return Err(ResourcesExceeded::Time);
            }
        }
        match solver.step() {
            SolverStep::Found(filled_board) => return Ok(Some(filled_board)),
            SolverStep::NoProgress => {}
            SolverStep::Done => return Ok(None),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MidgameMove {
    pub mov: Move,
    pub num_solutions_lower_bound: u32,
}
