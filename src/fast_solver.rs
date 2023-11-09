use crate::{
    board::{box_major_coordinates, Board, Coordinates, FilledBoard},
    digit::Digit,
    digit_box::DigitBox,
    digit_set::DigitSet,
    queue::Queue,
    small::{CartesianProduct, Small},
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
        for coord in box_major_coordinates() {
            if let Some(digit) = board.square(coord.into()).to_digit() {
                state.assert_digit(coord, digit);
            }
        }
        Self {
            remaining: vec![state],
        }
    }

    fn step(&mut self) -> SolverStep {
        let Some(mut state) = self.remaining.pop() else {
            return SolverStep::Done;
        };
        loop {
            if state.simplify().is_none() {
                return SolverStep::NoProgress;
            }
            if let Some(filled_board) = state.get_solution() {
                return SolverStep::Found(filled_board);
            }
            self.remaining.push(state.branch());
        }
    }
}

#[derive(Clone, Debug)]
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
    variables: [Variables4x4x9; 15],

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

        Self {
            #[rustfmt::skip]
            variables: [
                box_variables, box_variables, box_variables, band_variables,
                box_variables, box_variables, box_variables, band_variables,
                box_variables, box_variables, box_variables, band_variables,
                band_variables, band_variables, band_variables,
            ],

            processing_queue: Queue::new(),
            unprocessed: SmallSet::EMPTY,
        }
    }

    fn assert_digit(&mut self, coord: Coordinates, digit: Digit) {
        let box_index = encode_box_index(coord.big[0].into(), coord.big[1].into());
        self.variables[box_index]
            .asserted
            .set(coord.small[0].into(), coord.small[1].into(), digit);

        self.add_to_queue(box_index);
    }

    fn add_to_queue(&mut self, box_index: Small<15>) {
        if !self.unprocessed.contains(box_index) {
            self.processing_queue.push(box_index);
            self.unprocessed.insert(box_index);
        }
    }

    /// `None` if the state is inconsistent.
    fn simplify(&mut self) -> Option<()> {
        todo!()
    }

    fn branch(&mut self) -> Self {
        todo!()
    }

    // `None` if the search isn't finished.
    fn get_solution(&self) -> Option<FilledBoard> {
        todo!()
    }
}

/// Panics if y=x=3.
fn encode_box_index(y: Small<4>, x: Small<4>) -> Small<15> {
    let res: Small<16> = Small::combine(y, x);
    res.try_into().unwrap()
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
