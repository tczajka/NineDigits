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
mod square_set;

#[rustfmt::skip]
#[cfg(test)] mod tests;
