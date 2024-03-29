use clap::{Parser, ValueEnum};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use sudoku_game::{
    basic_solver::BasicSolver,
    board::Board,
    fast_solver::FastSolver,
    solver::{Solver, SolverStep},
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    input: Vec<PathBuf>,

    #[arg(short, long)]
    output: PathBuf,

    #[arg(short, long)]
    solver: Vec<SolverType>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SolverType {
    Basic,
    Fast,
}

impl Display for SolverType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SolverType::Basic => write!(f, "basic"),
            SolverType::Fast => write!(f, "fast"),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    for solver_type in args.solver {
        println!("{solver_type}:");
        for input_file_name in &args.input {
            match solver_type {
                SolverType::Basic => {
                    run_benchmark::<BasicSolver>(input_file_name, &args.output)?;
                }
                SolverType::Fast => {
                    run_benchmark::<FastSolver>(input_file_name, &args.output)?;
                }
            };
        }
    }
    Ok(())
}

fn run_benchmark<S: Solver>(
    input_file_name: &Path,
    output_file_name: &Path,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_file_name)?;
    let buf_reader = BufReader::new(input_file);
    let output_file = File::create(output_file_name)?;
    let mut buf_writer = BufWriter::new(output_file);

    let start = Instant::now();
    let mut num_puzzles: u32 = 0;
    let mut total_solutions: u64 = 0;
    let mut min_solutions: u64 = u64::MAX;
    let mut max_solutions: u64 = 0;
    let mut num_no_progress: u64 = 0;

    for line in buf_reader.lines() {
        let line = line?;
        let board: Board = line.parse()?;
        num_puzzles += 1;

        let mut num_solutions = 0;
        let mut solver = S::new(&board);
        loop {
            match solver.step() {
                SolverStep::Found(filled_board) => {
                    num_solutions += 1;
                    writeln!(buf_writer, "{}", filled_board)?;
                }
                SolverStep::NoProgress => {
                    num_no_progress += 1;
                }
                SolverStep::Done => {
                    break;
                }
            }
        }
        total_solutions += num_solutions;
        min_solutions = min_solutions.min(num_solutions);
        max_solutions = max_solutions.max(num_solutions);
    }
    let elapsed = start.elapsed();
    let avg_solutions = total_solutions as f64 / num_puzzles as f64;
    //let avg_time = elapsed.as_secs_f64() * 1e6 / num_puzzles as f64;
    let avg_time = elapsed / num_puzzles;
    let avg_no_progress = num_no_progress as f64 / num_puzzles as f64;
    println!(
        "  {}  puzzles: {num_puzzles}  sols: {avg_solutions:.3} ({min_solutions}-{max_solutions})  time: {avg_time:.3?}  no_progress: {avg_no_progress:.3}",
        input_file_name.display()
    );
    Ok(())
}
