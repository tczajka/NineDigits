pub use codecup::run_codecup_interaction;

mod board;
mod codecup;
mod digit;
mod error;
mod platform;
mod player;
mod small;

#[rustfmt::skip]
#[cfg(test)] mod tests;
