use crate::{
    board::{Board, FilledBoard, FullMove, Move},
    digit::Digit,
    digit_set::DigitSet,
    endgame::EndgameSolver,
    fast_solver::FastSolver,
    log,
    random::RandomGenerator,
    settings,
    small::Small,
    solution_table::SolutionTable,
    solver::{Solver, SolverStep},
};
use std::time::{Duration, Instant};

pub fn choose_move_best_effort(
    board: &mut Board,
    partial_solutions: &SolutionTable,
    start_time: Instant,
    time_left: Duration,
    endgame_solver: &mut EndgameSolver,
    rng: &mut RandomGenerator,
) -> FullMove {
    assert!(partial_solutions.len() >= settings::SOLUTIONS_MIN);
    let mut moves = generate_moves(board, partial_solutions);
    let num_moves = moves.len();
    assert!(!moves.is_empty());
    moves.sort_by_key(|x| x.num_solutions_lower_bound);

    if let Some(fraction) = settings::MIDGAME_RANDOMIZE_FRACTION {
        let best_solutions = moves.last().unwrap().num_solutions_lower_bound;
        let min_solutions = ((best_solutions as f64 * fraction) as u32).clamp(2, best_solutions);
        let mut shuffle_from = num_moves - 1;
        while shuffle_from != 0
            && moves[shuffle_from - 1].num_solutions_lower_bound >= min_solutions
        {
            shuffle_from -= 1;
        }
        log::write_line!(Info, "shuffling {n} moves", n = num_moves - shuffle_from);
        rng.shuffle(&mut moves[shuffle_from..]);
    }

    // TODO: Defense.

    let mov = moves.last().unwrap();
    log::write_line!(
        Info,
        "midgame num_solutions >= {num_solutions}",
        num_solutions = mov.num_solutions_lower_bound,
    );
    FullMove::Move(mov.mov)
}

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
    mov: Move,
    num_solutions_lower_bound: u32,
}
