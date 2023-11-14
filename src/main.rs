use sudoku_game::{codecup, log}; // submission::skip

fn main() -> std::io::Result<()> {
    log::init(log::Level::Info);
    codecup::run_codecup_interaction()?;
    Ok(())
}
