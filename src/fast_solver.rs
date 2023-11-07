use crate::{
    board::Board,
    digit_box::DigitBox,
    digit_set::DigitSet,
    solver::{Solver, SolverStep},
};

#[derive(Debug)]
pub struct FastSolver {
    remaining: Vec<SearchState>,
}

impl Solver for FastSolver {
    fn new(board: &Board) -> Self {
        let mut state = SearchState::initial();
        /*
        for square in board.squares() {
            let digit = board[square];
            if let Some(digit) = digit {
                state.variables[square.i][square.j].asserted[digit] = true;
            }
        }*/
        Self {
            remaining: vec![state],
        }
    }

    fn step(&mut self) -> SolverStep {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
struct SearchState {
    /// variables[i][j][y][x][d]
    /// At most one of the four coordinates can be 3.
    /// variables[i][j][y][x]: x_ijyx
    /// variables[i][j][y][3]: h_ijy (horizontal triad)
    /// variables[i][j][3][x]: v_ijx (vertical triad)
    /// variables[i][3][y][x]: h_ixy (horizontal triad)
    /// variables[3][j][y][x]: v_jyx (vertical triad)
    /// variables[i][j]: box
    /// variables[i][3]: horizontal band
    /// variables[3][j]: vertical band
    variables: [[Variables4x4x9; 4]; 4],
}

impl SearchState {
    fn initial() -> Self {
        let all = DigitSet::all();

        let box_all = DigitBox::from([
            [all; 4],
            [all; 4],
            [all; 4],
            [all, all, all, DigitSet::EMPTY],
        ]);

        let band_all = DigitBox::from([
            [all, all, all, DigitSet::EMPTY],
            [all, all, all, DigitSet::EMPTY],
            [all, all, all, DigitSet::EMPTY],
            [DigitSet::EMPTY; 4],
        ]);

        let box_variables = Variables4x4x9::initial(box_all);
        let band_variables = Variables4x4x9::initial(band_all);
        let empty_variables = Variables4x4x9::initial(DigitBox::empty());

        Self {
            #[rustfmt::skip]
            variables: [
                [box_variables, box_variables, box_variables, band_variables],
                [box_variables, box_variables, box_variables, band_variables],
                [box_variables, box_variables, box_variables, band_variables],
                [band_variables, band_variables, band_variables, empty_variables],
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Variables4x4x9 {
    asserted: DigitBox,
    possible: DigitBox,
    asserted_processed: DigitBox,
    possible_processed: DigitBox,
}

impl Variables4x4x9 {
    fn initial(possible: DigitBox) -> Self {
        Self {
            asserted: DigitBox::empty(),
            possible,
            asserted_processed: DigitBox::empty(),
            possible_processed: possible,
        }
    }
}
