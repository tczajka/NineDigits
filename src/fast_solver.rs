use crate::{
    board::{row_major_coordinates, Board, Coordinates},
    digit::Digit,
    digit_box::DigitBox,
    digit_set::DigitSet,
    queue::Queue,
    small::Small,
    small_set::SmallSet,
    solver::{Solver, SolverStep},
};

#[derive(Debug)]
pub struct FastSolver {
    remaining: Vec<SearchState>,
}

impl Solver for FastSolver {
    fn new(board: &Board) -> Self {
        let mut state = SearchState::initial();
        for coord in row_major_coordinates() {
            if let Some(digit) = board.square(coord.into()).to_digit() {
                state.assert_digit(coord, digit);
            }
        }
        Self {
            remaining: vec![state],
        }
    }

    fn step(&mut self) -> SolverStep {
        todo!()
    }
}

#[derive(Debug)]
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

    /// In queue: 4 * i + j.
    processing_queue: Queue<Small<15>, 16>,
    unprocessed: SmallSet<15, u16>,
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

            processing_queue: Queue::new(),
            unprocessed: SmallSet::EMPTY,
        }
    }

    fn assert_digit(&mut self, coord: Coordinates, digit: Digit) {
        self.variables[usize::from(coord.big[0])][usize::from(coord.big[1])]
            .asserted
            .set(coord.small[0].into(), coord.small[1].into(), digit);
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
