pub use codecup::run_codecup_interaction;

mod board;
mod codecup;
mod digit;
mod digit_set;
mod error;
mod platform;
mod player;
mod small;
mod solver;

#[rustfmt::skip]
#[cfg(test)] mod tests;
