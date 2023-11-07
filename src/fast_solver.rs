use crate::{
    board::Board,
    solver::{Solver, SolverStep},
};

#[derive(Debug)]
pub struct FastSolver {
    remaining: Vec<SearchState>,
}

impl Solver for FastSolver {
    fn new(board: &Board) -> Self {
        todo!()
    }

    fn step(&mut self) -> SolverStep {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
struct SearchState {
    /// variables[i][j][y][x][d]
    /// At most one of the four coordinates can be 3.
    /// If i, j, y, x < 3: x_ijyxd
    /// If x == 3: h_ijyd  (horizontal triad)
    /// If y == 3: v_ijxd  (vertical triad)
    /// If i == 3: v_yjxd  (copy of column vertical triads)
    /// If j == 3: h_ixyd  (copy of row horizontal triads)
    variables: [[Variables4x4x9; 4]; 4],
}

#[derive(Clone, Copy, Debug)]
struct Variables4x4x9 {
    asserted: DigitBox,
    possible: DigitBox,
    asserted_processed: DigitBox,
    possible_processed: DigitBox,
}
