use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use subprocess::{Popen, PopenConfig, Redirection};
use sudoku_game::{
    board::{Board, FullMove, Move},
    digit_set::DigitSet,
    fast_solver::FastSolver,
    solver::{Solver, SolverStep},
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value_t = 8)]
    threads: u32,

    #[arg(short, long)]
    games: u32,

    player0: PathBuf,
    player1: PathBuf,
}

struct MatchInfo {
    num_games: u32,
    games_played: u32,
    player_binaries: [PathBuf; 2],
    wins: [u32; 2],
    fails: [u32; 2],
    max_time: [Duration; 2],
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let match_info = Arc::new(Mutex::new(MatchInfo {
        num_games: args.games,
        games_played: 0,
        player_binaries: [args.player0, args.player1],
        wins: [0, 0],
        fails: [0, 0],
        max_time: [Duration::ZERO, Duration::ZERO],
    }));
    let join_handles: Vec<JoinHandle<()>> = (0..args.threads)
        .map(|_| {
            let match_info = match_info.clone();
            thread::spawn(move || run_matches(&match_info))
        })
        .collect();

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }

    {
        let match_info = match_info.lock().unwrap();
        println!(
            "{} vs {}",
            match_info.player_binaries[0].display(),
            match_info.player_binaries[1].display()
        );
        println!("Wins: {} : {}", match_info.wins[0], match_info.wins[1]);
        if match_info.fails != [0, 0] {
            println!("Fails: {} : {}", match_info.fails[0], match_info.fails[1]);
        }
        println!(
            "Time: {:.3?} : {:.3?}",
            match_info.max_time[0], match_info.max_time[1]
        );
    }

    Ok(())
}

fn run_matches(match_info: &Mutex<MatchInfo>) {
    loop {
        let game_number;
        let player_binaries;
        {
            let mut match_info = match_info.lock().unwrap();
            game_number = match_info.games_played;
            if game_number >= match_info.num_games {
                break;
            }
            match_info.games_played += 1;
            player_binaries = match_info.player_binaries.clone();
        }

        for side in 0..2 {
            let game_name = format!("{game_number}{}", ["a", "b"][side]);

            let log_files = [
                format!("/tmp/player{}.{}.log", ["A", "B"][side], game_name),
                format!("/tmp/player{}.{}.log", ["A", "B"][side ^ 1], game_name),
            ];

            let (winner, failure, times) = run_match(
                [&player_binaries[side], &player_binaries[side ^ 1]],
                log_files,
            );
            let real_winner = winner ^ side;
            let mut match_info = match_info.lock().unwrap();
            match_info.wins[real_winner] += 1;
            if failure {
                match_info.fails[real_winner ^ 1] += 1;
            }
            for i in 0..2 {
                match_info.max_time[i] = match_info.max_time[i].max(times[i ^ side]);
            }
            eprintln!(
                "Game {game_name}: {}{} {} - {}",
                ["A", "B"][real_winner],
                if failure { " failure" } else { "" },
                match_info.wins[0],
                match_info.wins[1],
            );
        }
    }
}

// Returns (winner, failure, times).
fn run_match(
    player_binaries: [&Path; 2],
    log_files: [String; 2],
    // game_name: &str,
) -> (usize, bool, [Duration; 2]) {
    let players: Vec<Popen> = (0..2)
        .map(|i| {
            let log_file = File::create(&log_files[i]).unwrap();
            Popen::create(
                &[player_binaries[i]],
                PopenConfig {
                    stdin: Redirection::Pipe,
                    stdout: Redirection::Pipe,
                    stderr: Redirection::File(log_file),
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect();

    let mut stdins: Vec<&File> = players.iter().map(|p| p.stdin.as_ref().unwrap()).collect();
    let mut stdouts: Vec<BufReader<&File>> = players
        .iter()
        .map(|p| BufReader::new(p.stdout.as_ref().unwrap()))
        .collect();

    let winner;
    let mut failure = false;
    let mut times = [Duration::ZERO, Duration::ZERO];

    let mut board = Board::new();
    let mut turn = 0;
    let mut prev_move: Option<Move> = None;

    loop {
        let start_time = Instant::now();
        match prev_move {
            None => writeln!(stdins[turn], "Start").unwrap(),
            Some(prev_move) => writeln!(stdins[turn], "{prev_move}").unwrap(),
        }
        stdins[turn].flush().unwrap();

        let mut line = String::new();
        stdouts[turn].read_line(&mut line).unwrap();
        times[turn] += start_time.elapsed();

        let Ok(full_move) = line.trim().parse() else {
            winner = Some(turn ^ 1);
            failure = true;
            break;
        };
        match full_move {
            FullMove::ClaimUnique => {
                if count_solutions(&board, 2, None) >= 2 {
                    winner = Some(turn ^ 1);
                    failure = true;
                    break;
                } else {
                    winner = Some(turn);
                    break;
                }
            }
            FullMove::Move(mov) => {
                if count_solutions(&board, 1, Some(mov)) == 0 {
                    winner = Some(turn ^ 1);
                    failure = true;
                    break;
                }
                let Ok(()) = board.make_move(mov) else {
                    winner = Some(turn ^ 1);
                    failure = true;
                    break;
                };
                if count_solutions(&board, 1, None) == 0 {
                    winner = Some(turn ^ 1);
                    failure = true;
                    break;
                }
                prev_move = Some(mov);
            }
            FullMove::MoveClaimUnique(mov) => {
                if count_solutions(&board, 1, Some(mov)) == 0 {
                    winner = Some(turn ^ 1);
                    failure = true;
                    break;
                }
                let Ok(()) = board.make_move(mov) else {
                    winner = Some(turn ^ 1);
                    failure = true;
                    break;
                };
                if count_solutions(&board, 2, None) == 1 {
                    winner = Some(turn);
                    break;
                } else {
                    winner = Some(turn ^ 1);
                    failure = true;
                    break;
                }
            }
        }

        turn ^= 1;
    }

    for mut stdin in stdins {
        writeln!(stdin, "Quit").unwrap();
        stdin.flush().unwrap();
    }

    for mut player in players {
        assert!(player.wait().unwrap().success());
    }

    (winner.unwrap(), failure, times)
}

fn count_solutions(board: &Board, max_solutions: u32, except: Option<Move>) -> u32 {
    let mut solutions = 0;
    let mut solver = FastSolver::new(board);
    if let Some(mov) = except {
        solver.remove_possibilities(mov.square, DigitSet::only(mov.digit));
    }
    while solutions < max_solutions {
        match solver.step() {
            SolverStep::NoProgress => {}
            SolverStep::Found(_) => {
                solutions += 1;
            }
            SolverStep::Done => break,
        }
    }
    solutions
}
