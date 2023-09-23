pub use codecup::run_codecup_interaction;

mod board;
mod codecup;
mod digit;
mod platform;
mod small;

#[rustfmt::skip]
#[cfg(test)] mod tests;
