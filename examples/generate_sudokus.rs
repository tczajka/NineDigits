use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};
use sudoku_game::{
    board::{Board, Move},
    digit::Digit,
    fast_solver::FastSolver,
    random::RandomGenerator,
    small::Small,
    solver::{Solver, SolverStep},
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    num_sudokus: u64,

    #[arg(short, long)]
    max_solutions: u64,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut rng = RandomGenerator::with_nonce(10000);
    let output_file = File::create(args.output)?;
    let mut buf_writer = BufWriter::new(output_file);
    for i in 0..args.num_sudokus {
        eprintln!("Generating sudoku {i}");
        let board = generate(args.max_solutions, &mut rng);
        writeln!(buf_writer, "{}", board)?;
    }
    Ok(())
}

fn generate(max_solutions: u64, rng: &mut RandomGenerator) -> Board {
    let mut board = Board::new();
    loop {
        let mov = random_move(&board, rng);
        let mut board2 = board;
        board2.make_move(mov).unwrap();
        let num_solutions = count_solutions(&board2, max_solutions);
        if num_solutions == 0 {
            continue;
        }
        board = board2;
        if num_solutions <= max_solutions {
            break;
        }
    }
    board
}

fn random_move(board: &Board, rng: &mut RandomGenerator) -> Move {
    let squares: Vec<_> = board.empty_squares().into_iter().collect();
    let square = *rng.choose(&squares);
    let digit = Digit::from(Small::random(rng));
    Move { square, digit }
}

fn count_solutions(board: &Board, max_solutions: u64) -> u64 {
    let mut num_solutions = 0;
    let mut solver = FastSolver::new(board);
    loop {
        match solver.step() {
            SolverStep::Found(_) => {
                num_solutions += 1;
                if num_solutions > max_solutions {
                    break;
                }
            }
            SolverStep::NoProgress => {}
            SolverStep::Done => break,
        }
    }
    num_solutions
}
