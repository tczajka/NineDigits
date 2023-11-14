use crate::{log, platform::platform_description, player::Player, player_main::PlayerMain};
use std::{
    io::{self, BufRead, Write},
    time::{Duration, Instant},
};

pub fn run_codecup_interaction() -> io::Result<()> {
    log::write_line!(Info, "platform: {}", platform_description());

    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut line = String::new();
    let mut time_used = Duration::ZERO;
    let mut player = None;

    loop {
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(());
        }
        let mut start_time = Instant::now();

        if player.is_none() {
            player = Some(PlayerMain::new());
            let t = Instant::now();
            time_used += t - start_time;
            start_time = t;
            log::write_line!(Info, "init {:?}", time_used);
        }

        let command = line.trim();
        log::write_line!(Info, "recv {command}");

        if command.is_empty() {
            continue;
        } else if command == "Quit" {
            break;
        } else if command == "Start" {
        } else {
            match command.parse() {
                Ok(mov) => {
                    player.as_mut().unwrap().opponent_move(mov);
                }
                Err(_) => {
                    log::write_line!(Always, "Error: invalid command!");
                    continue;
                }
            }
        }

        let mov = player.as_mut().unwrap().choose_move();

        time_used += start_time.elapsed();
        log::write_line!(Info, "send {mov} time {time_used:?}");
        writeln!(output, "{mov}")?;
        output.flush()?;
    }
    Ok(())
}
