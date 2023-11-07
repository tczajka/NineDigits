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
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "sse2",
    target_feature = "ssse3",
    target_feature = "sse4.1",
))]
pub mod simd;
pub mod small;
pub mod solver;
pub mod square_set;
