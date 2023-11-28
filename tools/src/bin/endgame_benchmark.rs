use clap::Parser;
use std::{
    error::Error,
    fmt::{self, Display},
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use sudoku_game::{
    board::Board, endgame::EndgameSolver, random::RandomGenerator, solution_table::SolutionTable,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long, default_value_t = 512)]
    ttable_mb: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    run_benchmark(&args.input, args.ttable_mb << 20)?;
    Ok(())
}

struct EndgameStatistics {
    num_puzzles: u32,
    total_solutions: u64,
    total_time: Duration,
}

impl EndgameStatistics {
    fn new() -> Self {
        Self {
            num_puzzles: 0,
            total_solutions: 0,
            total_time: Duration::ZERO,
        }
    }
}

impl Display for EndgameStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "num_puzzles: {num_puzzles} avg solutions: {avg_solutions:.1} avg time: {avg_time:.3?}",
            num_puzzles = self.num_puzzles,
            avg_solutions = self.total_solutions as f64 / self.num_puzzles as f64,
            avg_time = self.total_time / self.num_puzzles,
        )
    }
}

fn run_benchmark(input_file_name: &Path, ttable_memory: usize) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_file_name)?;
    let buf_reader = BufReader::new(input_file);

    let mut rng = RandomGenerator::with_nonce(0);
    let mut endgame_solver = EndgameSolver::new(ttable_memory);

    let mut statistics_generate = EndgameStatistics::new();
    let mut statistics_win = EndgameStatistics::new();
    let mut statistics_lose = EndgameStatistics::new();

    for line in buf_reader.lines() {
        let line = line?;
        let board: Board = line.parse()?;

        eprintln!("Working on {}", statistics_generate.num_puzzles);
        let start_time = Instant::now();
        let (res, solutions) = SolutionTable::generate(
            &board,
            0,
            3000000,
            start_time + Duration::from_secs(3600),
            &mut rng,
        );
        res.unwrap();
        let generated_time = Instant::now();

        statistics_generate.num_puzzles += 1;
        statistics_generate.total_solutions += u64::try_from(solutions.len()).unwrap();
        statistics_generate.total_time += generated_time.saturating_duration_since(start_time);

        let win = endgame_solver
            .solve(&solutions, start_time, Duration::from_secs(24 * 3600))
            .unwrap();
        let endgame_duration = Instant::now().saturating_duration_since(generated_time);
        let statistics = if win {
            &mut statistics_win
        } else {
            &mut statistics_lose
        };
        statistics.num_puzzles += 1;
        statistics.total_solutions += u64::try_from(solutions.len()).unwrap();
        statistics.total_time += endgame_duration;
    }

    println!("generate: {statistics_generate}");
    if statistics_win.num_puzzles > 0 {
        println!("win:  {statistics_win}");
    }
    if statistics_lose.num_puzzles > 0 {
        println!("lose: {statistics_lose}");
    }

    Ok(())
}
