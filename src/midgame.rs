use crate::{
    board::{Board, FilledBoard, Move},
    digit::Digit,
    digit_set::DigitSet,
    fast_solver::FastSolver,
    small::Small,
    solution_table::SolutionTable,
    solver::{Solver, SolverStep},
};

/// Returns (normalized board, all possible moves)
pub fn generate_moves(board: &mut Board, partial_solutions: &SolutionTable) -> Vec<MidgameMove> {
    let mut counts: Vec<[u32; 9]> = vec![[0; 9]; 81];
    let counts: &mut [[u32; 9]; 81] = counts.as_mut_slice().try_into().unwrap();
    for solution in partial_solutions.iter() {
        increment_counts(counts, solution.digits().try_into().unwrap());
    }
    for square in board.empty_squares() {
        let mut possible = DigitSet::EMPTY;
        for digit in Digit::all() {
            if counts[square][digit] != 0 {
                possible.insert(digit);
            }
        }
        while possible != DigitSet::all() {
            let Some(filled_board) = find_one_solution_except(&*board, square, possible) else {
                break;
            };
            increment_counts(counts, &filled_board.squares);
            possible.insert(filled_board.squares[square]);
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
            if num_solutions_lower_bound != 0 {
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
) -> Option<FilledBoard> {
    let mut solver = FastSolver::new(board);
    solver.remove_possibilities(square, except);
    loop {
        match solver.step() {
            SolverStep::Found(filled_board) => return Some(filled_board),
            SolverStep::NoProgress => {}
            SolverStep::Done => return None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MidgameMove {
    pub mov: Move,
    pub num_solutions_lower_bound: u32,
}
