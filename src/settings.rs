use std::time::Duration;

#[cfg(feature = "tomeks_computer")]
pub const GAME_TIME_LIMIT: Duration = Duration::from_millis(10_000);
#[cfg(not(feature = "tomeks_computer"))]
pub const GAME_TIME_LIMIT: Duration = Duration::from_millis(29_700);

// Time allocation.
pub const SOLUTION_GENERATE_TIME_FRACTION: f64 = 0.2;
pub const MIDGAME_DEFENSE_TIME_FRACTION: f64 = 0.2;
pub const ENDGAME_OFFENSE_TIME_FRACTION: f64 = 0.2;
pub const ENDGAME_OFFENSE_EXTENDED_TIME_FRACTION: f64 = 0.25;
pub const ENDGAME_DEFENSE_TIME_FRACTION: f64 = 0.2;

// Opening.
pub const OPENING_MAX_SQUARES: u8 = 12;

// Midgame.
pub const SOLUTION_GENERATE_CHECK_TIME_ITERS: u64 = 1024;
pub const MIDGAME_RANDOMIZE_FRACTION: Option<f64> = None;
pub const SOLUTIONS_MIN: u32 = 100;
pub const SOLUTIONS_MAX: u32 = 70_000;
pub const MIDGAME_DEFENSE_SOLUTIONS_MAX: u32 = 40_000;

// Endgame.
pub const ENDGAME_CHECK_TIME_NODES: u64 = 1024;
pub const TRANSPOSITION_TABLE_MEMORY: usize = 512 << 20;
