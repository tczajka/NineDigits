#![warn(unsafe_op_in_unsafe_fn)]

pub use {
    basic_solver::BasicSolver,
    board::{Board, FilledBoard},
    codecup::run_codecup_interaction,
    solver::{Solver, SolverStep},
};

mod basic_solver;
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
