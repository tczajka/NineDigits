use std::time::Duration;

// Time allocation.
#[cfg(feature = "tomeks_computer")]
pub const GAME_TIME_LIMIT: Duration = Duration::from_millis(10_000);
#[cfg(not(feature = "tomeks_computer"))]
pub const GAME_TIME_LIMIT: Duration = Duration::from_millis(29_700);

pub const SOLUTION_GENERATE_TIME_FRACTION: f64 = 0.1;
pub const MIDGAME_DEFENSE_TIME_FRACTION: f64 = 0.1;
pub const ENDGAME_OFFENSE_TIME_FRACTION: f64 = 0.1;
pub const ENDGAME_DEFENSE_TIME_FRACTION: f64 = 0.1;

// Solution generation.
pub const SOLUTION_GENERATE_CHECK_TIME_ITERS: u64 = 1024;
pub const SOLUTIONS_MIN: usize = 100;
pub const SOLUTIONS_MAX: usize = 200_000;

// Midgame.
pub const MIDGAME_RANDOMIZE_FRACTION: Option<f64> = Some(0.9);
pub const MIDGAME_DEFENSE_SOLUTIONS_MAX: usize = 50_000;

// Endgame.
pub const ENDGAME_CHECK_TIME_NODES: u64 = 1024;
pub const TRANSPOSITION_TABLE_MEMORY: usize = 512 << 20;
