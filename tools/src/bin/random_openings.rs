use clap::Parser;
use std::time::{Duration, Instant};
use sudoku_game::{
    board::Board, error::ResourcesExceeded, midgame, random::RandomGenerator,
    solution_table::SolutionTable,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    num_games: u32,

    /// If we have <= num_solutions after move, the move is no longer an opening move.
    #[arg(short, long)]
    max_solutions: u32,
}

fn main() {
    let args = Args::parse();
    let mut rng = RandomGenerator::with_time_nonce();

    let mut opening_square_counts = [0u32; 82];

    for game_number in 0..args.num_games {
        let squares = opening_length(&mut rng, args.max_solutions);
        eprintln!("Game {game_number} squares {squares}");
        opening_square_counts[usize::from(squares)] += 1;
    }

    for (squares, &count) in opening_square_counts.iter().enumerate() {
        if count > 0 {
            println!("{} squares: {}", squares, count);
        }
    }
}

fn opening_length(rng: &mut RandomGenerator, max_solutions: u32) -> u8 {
    let mut board = Board::new();
    let mut result = 0;

    loop {
        let (solution_gen_result, solutions) = SolutionTable::generate(
            &board,
            0,
            max_solutions,
            Instant::now() + Duration::from_secs(3600),
            rng,
        );
        match solution_gen_result {
            Ok(()) => {
                return result;
            }
            Err(ResourcesExceeded::Time) => {
                panic!("Time exceeded");
            }
            Err(ResourcesExceeded::Memory) => {
                // Continue opening.
            }
        }
        let moves = midgame::generate_moves(&mut board, &solutions);
        result = 81 - board.empty_squares().size();
        let mov = rng.choose(&moves).mov;
        board.make_move(mov).unwrap();
    }
}
