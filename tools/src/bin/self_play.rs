use clap::Parser;
use std::time::Instant;
use sudoku_game::{board::FullMove, player::Player, player_main::PlayerMain, settings};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    games: u32,
}

fn main() {
    let args = Args::parse();
    let mut wins: [u32; 2] = [0, 0];
    for game_num in 0..args.games {
        eprintln!("Game {game_num} / {}", args.games);
        let winner = play_game();
        wins[winner] += 1;
    }
    println!("Wins: {} : {}", wins[0], wins[1]);
}

fn play_game() -> usize {
    let mut players = [PlayerMain::new(), PlayerMain::new()];
    let mut time_left = [settings::GAME_TIME_LIMIT; 2];
    let mut turn = 0;
    loop {
        let start_time = Instant::now();
        let fmov = players[turn].choose_move(start_time, time_left[turn]);
        let elapsed = start_time.elapsed();
        time_left[turn] = time_left[turn].saturating_sub(elapsed);
        match fmov {
            FullMove::Move(mov) => {
                players[turn ^ 1].opponent_move(mov);
                turn = 1 - turn;
            }
            FullMove::MoveClaimUnique(_) => return turn,
            FullMove::ClaimUnique => return turn,
        }
    }
}
