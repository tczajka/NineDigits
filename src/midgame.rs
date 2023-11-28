use crate::{
    board::{Board, FilledBoard, FullMove, Move},
    digit::Digit,
    digit_set::DigitSet,
    fast_solver::FastSolver,
    log,
    random::RandomGenerator,
    settings,
    small::Small,
    solution_table::SolutionTable,
    solver::{Solver, SolverStep},
};
use std::cmp;

pub fn choose_move_best_effort(
    partial_solutions: &SolutionTable,
    rng: &mut RandomGenerator,
) -> FullMove {
    let num_solutions: u32 = partial_solutions.len().try_into().unwrap();
    assert!(num_solutions >= 2);

    let move_summaries = partial_solutions.move_summaries();
    let mut move_candidates: Vec<(Move, u32)> = Vec::new();
    for (square, move_summaries_sq) in Small::<81>::all().zip(move_summaries.iter()) {
        for (digit, move_summary) in Digit::all().zip(move_summaries_sq.iter()) {
            if move_summary.num_solutions == 0 || move_summary.num_solutions == num_solutions {
                continue;
            }
            move_candidates.push((Move { square, digit }, move_summary.num_solutions));
        }
    }
    move_candidates.sort_by_key(|x| cmp::Reverse(x.1));
    let best_solutions = move_candidates[0].1;
    let min_solutions = ((best_solutions as f64 * settings::EARLY_GAME_MIN_SOLUTIONS_FRACTION)
        as u32)
        .clamp(2, best_solutions);
    while move_candidates.last().unwrap().1 < min_solutions {
        move_candidates.pop();
    }
    let (chosen_move, num_solutions) = *rng.choose(&move_candidates);
    log::write_line!(
        Info,
        "midgame candidates: {num_candidates} num_solutions: {num_solutions} best_solutions: {best_solutions}",
        num_candidates = move_candidates.len()
    );
    FullMove::Move(chosen_move)
}

/// Returns (normalized board, all possible moves)
pub fn generate_moves(
    board: &Board,
    partial_solutions: &SolutionTable,
) -> (Board, Vec<MidgameMove>) {
    let mut normalized_board = *board;
    let mut counts: Vec<[u32; 9]> = vec![[0; 9]; 81];
    let counts: &mut [[u32; 9]; 81] = counts.as_mut_slice().try_into().unwrap();
    for solution in partial_solutions.iter() {
        increment_counts(counts, solution.digits().try_into().unwrap());
    }
    for square in board.empty_squares() {
        let mut possible = DigitSet::EMPTY;
        for digit in Digit::all() {
            if counts[usize::from(square)][digit] == 0 {
                possible.insert(digit);
            }
        }
        while possible != DigitSet::all() {
            let Some(filled_board) = find_one_solution_except(&normalized_board, square, possible)
            else {
                break;
            };
            increment_counts(counts, &filled_board.squares);
            possible.insert(filled_board.squares[square]);
        }
        let first_digit = possible.smallest().unwrap();
        possible.remove(first_digit);
        if possible.is_empty() {
            normalized_board
                .make_move(Move {
                    square,
                    digit: first_digit,
                })
                .unwrap();
        }
    }
    let mut moves = Vec::with_capacity(81 * 9);
    for square in normalized_board.empty_squares() {
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
    (normalized_board, moves)
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

#[derive(Debug)]
pub struct MidgameMove {
    mov: Move,
    num_solutions_lower_bound: u32,
}
