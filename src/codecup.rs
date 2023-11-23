use crate::{log, platform::platform_description, player::Player, player_main::PlayerMain};
use std::{
    io::{self, BufRead, Write},
    time::{Duration, Instant},
};

pub fn run_codecup_interaction() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut line = String::new();
    let mut time_used = Duration::ZERO;
    let mut player = None;

    let time_limit = Duration::from_millis(29_800);

    loop {
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(());
        }
        let mut start_time = Instant::now();

        if player.is_none() {
            log::write_line!(Info, "platform: {}", platform_description());
            player = Some(PlayerMain::new());
            let t = Instant::now();
            time_used += t.saturating_duration_since(start_time);
            start_time = t;
            log::write_line!(Info, "init {:.3?}", time_used);
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

        let mov = player
            .as_mut()
            .unwrap()
            .choose_move(start_time, time_limit.saturating_sub(time_used));

        time_used += start_time.elapsed();
        log::write_line!(Info, "send {mov} used {time_used:.3?}");
        writeln!(output, "{mov}")?;
        output.flush()?;
    }
    Ok(())
}
