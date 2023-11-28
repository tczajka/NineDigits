use crate::{
    board::{FullMove, Move},
    digit::Digit,
    log,
    random::RandomGenerator,
    settings,
    small::Small,
    solution_table::SolutionTable,
};
use std::cmp;

pub fn choose_move_best_effort(solutions: &SolutionTable, rng: &mut RandomGenerator) -> FullMove {
    let num_solutions: u32 = solutions.len().try_into().unwrap();
    assert!(num_solutions >= 2);

    let move_summaries = solutions.move_summaries();
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
