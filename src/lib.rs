#![warn(unsafe_op_in_unsafe_fn)]
#![allow(clippy::new_without_default)]

pub mod basic_solver; // submission::skip
pub mod bits;
pub mod board;
pub mod chacha;
pub mod codecup;
pub mod digit;
pub mod digit_box;
pub mod digit_set;
pub mod endgame;
pub mod error;
pub mod fast_solver;
pub mod log;
pub mod midgame;
pub mod permutation;
pub mod platform;
pub mod player;
pub mod player_main;
pub mod queue;
pub mod random;
pub mod settings;
pub mod simd128;
#[cfg(target_feature = "avx2")] // submission::skip
pub mod simd256; // submission::skip
#[cfg(not(target_feature = "avx2"))] // submission::skip
pub use simd256_emulated as simd256;
pub mod simd256_emulated;
pub mod small;
pub mod small_set;
pub mod solution_table;
pub mod solver;
pub mod symmetry;
pub mod transposition_table;
