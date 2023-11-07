#![warn(unsafe_op_in_unsafe_fn)]

// TODO: Don't include in submission.
pub mod basic_solver;
pub mod board;
pub mod chacha;
pub mod codecup;
pub mod digit;
pub mod digit_box;
pub mod digit_set;
pub mod error;
pub mod fast_solver;
pub mod log;
pub mod platform;
pub mod player;
pub mod queue;
pub mod random;
pub mod simd;
#[cfg_attr(target_feature = "avx2", path = "simd256_avx2.rs")] // submission::skip
pub mod simd256;
pub mod small;
pub mod solver;
pub mod square_set;

#[path = "simd256.rs"] // submission::skip
pub mod simd256_noavx2; // submission::skip
