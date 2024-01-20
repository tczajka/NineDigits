use std::time::Duration;

#[cfg(feature = "tomeks_computer")]
pub const GAME_TIME_LIMIT: Duration = Duration::from_millis(10_000);
#[cfg(not(feature = "tomeks_computer"))]
pub const GAME_TIME_LIMIT: Duration = Duration::from_millis(29_800);

// Time allocation.
pub const OPENING_MOVEGEN_TIME_FRACTION: f64 = 0.1;
pub const MIDGAME_MOVEGEN_TIME_FRACTION: f64 = 0.1;
pub const SOLUTION_GENERATE_TIME_FRACTION: f64 = 0.2;

pub const MIDGAME_DEFENSE_TIME_FRACTION: f64 = 0.2;
pub const MIDGAME_DEFENSE_EXTENDED_TIME_FRACTION: f64 = 0.25;
pub const ENDGAME_OFFENSE_TIME_FRACTION: f64 = 0.33;
pub const ENDGAME_OFFENSE_EXTENDED_TIME_FRACTION: f64 = 0.48;
pub const ENDGAME_DEFENSE_TIME_FRACTION: f64 = 0.2;
pub const ENDGAME_DEFENSE_EXTENDED_TIME_FRACTION: f64 = 0.30;

// Opening.
pub const OPENING_MAX_SQUARES: u8 = 14;

// Midgame.
pub const SOLUTION_GENERATE_CHECK_TIME_ITERS: u64 = 1024;
pub const SOLUTIONS_MIN: u32 = 2;
pub const SOLUTIONS_MAX: u32 = 100_000;
pub const MIDGAME_DEFENSE_SOLUTIONS_MAX: u32 = 90_000;
pub const MIDGAME_DEFENSE_DIFFICULTY_MAX: Option<u32> = Some(4500);

// Endgame.
pub const ENDGAME_CHECK_TIME_NODES: u64 = 1024;
pub const TRANSPOSITION_TABLE_MEMORY: usize = 512 << 20;
pub const ENDGAME_OFFENSE_DIFFICULTY_MAX: Option<u32> = Some(11000);
pub const ENDGAME_DEFENSE_DIFFICULTY_MAX: Option<u32> = Some(4000);
