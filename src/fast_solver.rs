//! variables[i][j][y][x][d]
//! At most one of the four coordinates can be 3.
//! variables[i][j][y][x]: x_ijyxd
//! variables[i][j][y][3]: h_ijyd (horizontal triad, inverted)
//! variables[i][j][3][x]: v_ijxd (vertical triad, inverted)
//! variables[i][3][y][x]: h_ixyd (horizontal triad, inverted, copied)
//! variables[3][j][y][x]: v_jyxd (vertical triad, inverted, copied)
//! variables[i][j]: box
//! variables[i][3]: horizontal band
//! variables[3][j]: vertical band
//!
//! Constraints:
//! A (one digit per square):
//! sum_d x_ijyx = 1
//!
//! B (three digits per triad):
//! sum_d h_ijyd = 6
//! sum_d v_ijyd = 6
//!
//! C (triad definitions)
//! sum_x x_ijyxd + h_ijyd = 1
//! sum_y x_ijyxd + v_ijyd = 1
//!
//! D (one digit per box)
//! sum_x v_jyxd = 2
//! sum_y h_ixyd = 2
//!
//! E (one digit per row/column)
//! sum_i h_ijyd = 2
//! sum_j v_ijyd = 2
//!
//! Once a constraint is tight, `possible` and `asserted` will be set tight for that sum.
//! If `asserted` is not a subset of `possible`, one of those sums will become out of bands.
//! So no need to separately check whether `asserted` is a subset of `possible`.

use crate::{
    board::{box_major_coordinates, Board, Coordinates, FilledBoard, Move},
    digit::Digit,
    digit_box::{Box4x4x16, DigitBox},
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
            if state.simplify().is_err() {
                return SolverStep::NoProgress;
            }
            if let Some(filled_board) = state.get_solution() {
                return SolverStep::Found(filled_board);
            }
            self.remaining.push(state.branch());
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum VariableBigCoord {
    Box(Small<3>, Small<3>),
    HBand(Small<3>),
    VBand(Small<3>),
}

impl VariableBigCoord {
    fn encode(self) -> Small<15> {
        let (i, j): (Small<4>, Small<4>) = match self {
            Self::Box(i, j) => (i.into(), j.into()),
            Self::HBand(i) => (i.into(), Small::<4>::new(3)),
            Self::VBand(j) => (Small::<4>::new(3), j.into()),
        };
        Small::<16>::combine(i, j).try_into().unwrap()
    }
}

#[derive(Clone, Debug)]
struct ProcessingQueue {
    queue: Queue<VariableBigCoord, 16>,
    unprocessed: SmallSet<15, u16>,
}

impl ProcessingQueue {
    fn empty() -> Self {
        Self {
            queue: Queue::empty(),
            unprocessed: SmallSet::EMPTY,
        }
    }

    fn push(&mut self, big_coord: VariableBigCoord) {
        let box_index = big_coord.encode();
        if !self.unprocessed.contains(box_index) {
            self.queue.push(big_coord);
            self.unprocessed.insert(box_index);
        }
    }

    fn pop(&mut self) -> Option<VariableBigCoord> {
        let big_coord = self.queue.pop()?;
        self.unprocessed.remove(big_coord.encode());
        Some(big_coord)
    }
}

#[derive(Clone, Debug)]
struct SearchState {
    variables: [Variables4x4x9; 15],
    queue: ProcessingQueue,
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

            queue: ProcessingQueue::empty(),
        }
    }

    fn assert_digit(&mut self, coord: Coordinates, digit: Digit) {
        let big_coord = VariableBigCoord::Box(coord.big[0], coord.big[1]);
        self.variables[big_coord.encode()].asserted.set(
            coord.small[0].into(),
            coord.small[1].into(),
            digit,
        );
        self.queue.push(big_coord);
    }

    /// `Err` if the state is inconsistent.
    fn simplify(&mut self) -> Result<(), ()> {
        while let Some(big_coord) = self.queue.pop() {
            match big_coord {
                VariableBigCoord::Box(big0, big1) => self.simplify_box(big0, big1)?,
                VariableBigCoord::HBand(big0) => self.simplify_hband(big0)?,
                VariableBigCoord::VBand(big1) => self.simplify_vband(big1)?,
            }
        }
        Ok(())
    }

    /// Simplify a regular box.
    fn simplify_box(&mut self, big0: Small<3>, big1: Small<3>) -> Result<(), ()> {
        let box_index = VariableBigCoord::Box(big0, big1).encode();

        if self.variables[box_index].process_box()? {
            {
                let hband_coord = VariableBigCoord::HBand(big0);
                let hband_index = hband_coord.encode();
                let (variables0, variables1) = self.variables.split_at_mut(hband_index.into());
                variables0[usize::from(box_index)].propagate_to_hband(&mut variables1[0], big1);
                self.queue.push(hband_coord);
            }
            {
                let vband_coord = VariableBigCoord::VBand(big1);
                let vband_index = vband_coord.encode();
                let (variables0, variables1) = self.variables.split_at_mut(vband_index.into());
                variables0[usize::from(box_index)].propagate_to_vband(&mut variables1[0], big0);
                self.queue.push(vband_coord);
            }
        }

        Ok(())
    }

    /// Simplify an hband box.
    fn simplify_hband(&mut self, big0: Small<3>) -> Result<(), ()> {
        let hband_index = VariableBigCoord::HBand(big0).encode();
        let (variables0, variables1) = self.variables.split_at_mut(hband_index.into());
        if variables1[0].process_hband()? {
            for big1 in Small::<3>::all() {
                let box_coord = VariableBigCoord::Box(big0, big1);
                let box_index = box_coord.encode();
                variables0[usize::from(box_index)].propagate_from_hband(&variables1[0], big1);
                self.queue.push(box_coord);
            }
        }
        Ok(())
    }

    /// Simplify a vband box.
    fn simplify_vband(&mut self, big1: Small<3>) -> Result<(), ()> {
        let vband_index = VariableBigCoord::VBand(big1).encode();
        let (variables0, variables1) = self.variables.split_at_mut(vband_index.into());
        if variables1[0].process_vband()? {
            for big0 in Small::<3>::all() {
                let box_coord = VariableBigCoord::Box(big0, big1);
                let box_index = box_coord.encode();
                variables0[usize::from(box_index)].propagate_from_vband(&variables1[0], big0);
                self.queue.push(box_coord);
            }
        }
        Ok(())
    }

    /// Branch and return a second state.
    fn branch(&mut self) -> Self {
        todo!()
    }

    // `None` if the search isn't finished.
    fn get_solution(&self) -> Option<FilledBoard> {
        for variables in &self.variables {
            if variables.asserted != variables.possible {
                return None;
            }
        }
        let mut board = Board::empty();
        for big0 in Small::<3>::all() {
            for big1 in Small::<3>::all() {
                let big_coord = VariableBigCoord::Box(big0, big1);
                let digit_sets: [[DigitSet; 4]; 4] =
                    self.variables[big_coord.encode()].asserted.into();
                for small0 in Small::<3>::all() {
                    for small1 in Small::<3>::all() {
                        let coord = Coordinates {
                            big: [big0, big1],
                            small: [small0, small1],
                        };
                        let square = coord.into();
                        let mut digit_set =
                            digit_sets[Small::<4>::from(small0)][Small::<4>::from(small1)];
                        let digit = digit_set.smallest().unwrap();
                        digit_set.remove(digit);
                        assert_eq!(digit_set, DigitSet::EMPTY);
                        board.make_move(Move { square, digit });
                    }
                }
            }
        }
        Some(board.into_filled())
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

impl Variables4x4x9 {
    // Returns whether something changed.
    fn process_box(&mut self) -> Result<bool, ()> {
        let mut changed = false;
        loop {
            if self.asserted_processed != self.asserted {
                self.process_box_asserted()?;
                changed = true;
            } else if changed {
                // We don't break on the first iteration.
                // On subsequent iterations `changed` is true and we break.
                break;
            }

            if self.possible_processed == self.possible {
                break;
            }
            self.process_box_possible()?;
            changed = true;
        }
        Ok(changed)
    }

    /// Process `self.asserted` and update `self.possible` for a regular box.
    fn process_box_asserted(&mut self) -> Result<(), ()> {
        self.process_box_asserted_squares()?;
        self.process_box_asserted_horizontal()?;
        self.process_box_asserted_vertical()?;
        self.asserted_processed = self.asserted;
        Ok(())
    }

    /// Equation A and B: sum in each square is 1 or 6.
    fn process_box_asserted_squares(&mut self) -> Result<(), ()> {
        let counts_target =
            Box4x4x16::from([[1, 1, 1, 6], [1, 1, 1, 6], [1, 1, 1, 6], [6, 6, 6, 0]]);
        let counts = self.asserted.counts();
        if counts.any_gt(counts_target) {
            return Err(());
        }
        let fixed = counts.masks_eq(counts_target);
        let impossible = fixed.and_not(self.asserted.into());
        self.possible = self.possible.and_not_bits(impossible);
        Ok(())
    }

    /// Equation C and D horizontal: sum in row is 1 or 2.
    fn process_box_asserted_horizontal(&mut self) -> Result<(), ()> {
        let mut rot = self.asserted.rotate_right();
        let mut ge_2 = self.asserted & rot;
        let mut ge_1 = self.asserted | rot;
        rot = rot.rotate_right();
        let mut ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;
        rot = rot.rotate_right();
        ge_3 |= rot & ge_2;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;

        if !(ge_2.replace_last_row(ge_3)).is_all_empty() {
            return Err(());
        }
        let fixed = ge_1.replace_last_row(ge_2);
        let impossible = fixed.and_not(self.asserted);
        self.possible = self.possible.and_not(impossible);
        Ok(())
    }

    /// Equation C and D vertical: sum in column is 1 or 2.
    fn process_box_asserted_vertical(&mut self) -> Result<(), ()> {
        let mut rot = self.asserted.rotate_down();
        let mut ge_2 = self.asserted & rot;
        let mut ge_1 = self.asserted | rot;
        rot = rot.rotate_down();
        let mut ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;
        rot = rot.rotate_down();
        ge_3 |= rot & ge_2;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;

        if !(ge_2.replace_last_column(ge_3)).is_all_empty() {
            return Err(());
        }
        let fixed = ge_1.replace_last_row(ge_2);
        let impossible = fixed.and_not(self.asserted);
        self.possible = self.possible.and_not(impossible);
        Ok(())
    }

    /// Process `self.possible` and update `self.asserted` for a regular box.
    fn process_box_possible(&mut self) -> Result<(), ()> {
        self.process_box_possible_squares()?;
        self.process_box_possible_horizontal()?;
        self.process_box_possible_vertical()?;
        self.possible_processed = self.possible;
        Ok(())
    }

    /// Equation A and B: sum in each square is 1 or 6.
    fn process_box_possible_squares(&mut self) -> Result<(), ()> {
        // Equations A and B.
        let counts_target =
            Box4x4x16::from([[1, 1, 1, 6], [1, 1, 1, 6], [1, 1, 1, 6], [6, 6, 6, 0]]);

        let counts = self.possible.counts();
        if counts.any_lt(counts_target) {
            return Err(());
        }
        let fixed = counts.masks_eq(counts_target);
        let required = self.possible.and_bits(fixed);
        self.asserted |= required;

        Ok(())
    }

    /// Equation C and D horizontal: sum in row is 1 or 2.
    fn process_box_possible_horizontal(&mut self) -> Result<(), ()> {
        let mut rot = self.possible.rotate_right();
        let mut ge_2 = self.possible & rot;
        let mut ge_1 = self.asserted | rot;
        rot = rot.rotate_right();
        let mut ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;
        rot = rot.rotate_right();
        ge_3 |= rot & ge_2;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;

        let all = DigitBox::fill(DigitSet::all());
        if ge_1.replace_last_row(ge_2) != all {
            return Err(());
        }
        let not_fixed = ge_2.replace_last_row(ge_3);
        let required = self.possible.and_not(not_fixed);
        self.asserted |= required;
        Ok(())
    }

    /// Equation C and D vertical: sum in column is 1 or 2.
    fn process_box_possible_vertical(&mut self) -> Result<(), ()> {
        let mut rot = self.possible.rotate_down();
        let mut ge_2 = self.possible & rot;
        let mut ge_1 = self.asserted | rot;
        rot = rot.rotate_down();
        let mut ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;
        rot = rot.rotate_down();
        ge_3 |= rot & ge_2;
        ge_2 |= rot & ge_1;
        ge_1 |= rot;

        let all = DigitBox::fill(DigitSet::all());

        if ge_1.replace_last_column(ge_2) != all {
            return Err(());
        }
        let not_fixed = ge_2.replace_last_row(ge_3);
        let required = self.possible.and_not(not_fixed);
        self.asserted |= required;
        Ok(())
    }

    // Returns whether something changed.
    fn process_hband(&mut self) -> Result<bool, ()> {
        let mut changed = false;
        loop {
            if self.asserted_processed != self.asserted {
                self.process_hband_asserted()?;
                changed = true;
            } else if changed {
                // We don't break on the first iteration.
                // On subsequent iterations `changed` is true and we break.
                break;
            }

            if self.possible_processed == self.possible {
                break;
            }
            self.process_hband_possible()?;
            changed = true;
        }
        Ok(changed)
    }

    // Returns whether something changed.
    fn process_vband(&mut self) -> Result<bool, ()> {
        let mut changed = false;
        loop {
            if self.asserted_processed != self.asserted {
                self.process_vband_asserted()?;
                changed = true;
            } else if changed {
                // We don't break on the first iteration.
                // On subsequent iterations `changed` is true and we break.
                break;
            }

            if self.possible_processed == self.possible {
                break;
            }
            self.process_vband_possible()?;
            changed = true;
        }
        Ok(changed)
    }

    /// Equation E, horizontal: sum in row is 2.
    fn process_hband_asserted(&mut self) -> Result<(), ()> {
        let mut rot = self.asserted.rotate_first_3_right();
        let mut ge_2 = self.asserted & rot;
        let ge_1 = self.asserted | rot;
        rot = rot.rotate_first_3_right();
        let ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        // ge_1 |= rot;

        if !ge_3.is_all_empty() {
            return Err(());
        }
        let impossible = ge_2.and_not(self.asserted);
        self.possible = self.possible.and_not(impossible);

        self.asserted_processed = self.asserted;
        Ok(())
    }

    /// Equation E, horizontal: sum in row is 2.
    fn process_hband_possible(&mut self) -> Result<(), ()> {
        let mut rot = self.possible.rotate_first_3_right();
        let mut ge_2 = self.possible & rot;
        let ge_1 = self.possible | rot;
        rot = rot.rotate_first_3_right();
        let ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        // ge_1 |= rot;

        let all = DigitBox::all3x3();
        if ge_2 != all {
            return Err(());
        }
        let required = self.possible.and_not(ge_3);
        self.asserted |= required;

        self.possible_processed = self.possible;
        Ok(())
    }

    /// Equation E, vertical: sum in column is 2.
    fn process_vband_asserted(&mut self) -> Result<(), ()> {
        let mut rot = self.asserted.rotate_first_3_down();
        let mut ge_2 = self.asserted & rot;
        let ge_1 = self.asserted | rot;
        rot = rot.rotate_first_3_down();
        let ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        // ge_1 |= rot;

        if !ge_3.is_all_empty() {
            return Err(());
        }
        let impossible = ge_2.and_not(self.asserted);
        self.possible = self.possible.and_not(impossible);

        self.asserted_processed = self.asserted;
        Ok(())
    }

    /// Equation E, vertical: sum in column is 2.
    fn process_vband_possible(&mut self) -> Result<(), ()> {
        let mut rot = self.possible.rotate_first_3_down();
        let mut ge_2 = self.possible & rot;
        let ge_1 = self.possible | rot;
        rot = rot.rotate_first_3_down();
        let ge_3 = ge_2 & rot;
        ge_2 |= rot & ge_1;
        // ge_1 |= rot;

        let all = DigitBox::all3x3();
        if ge_2 != all {
            return Err(());
        }
        let required = self.possible.and_not(ge_3);
        self.asserted |= required;

        self.possible_processed = self.possible;
        Ok(())
    }

    fn propagate_from_hband(&mut self, hband: &Self, big1: Small<3>) {
        self.asserted |= hband.asserted.move_column(big1.into(), Small::new(3));
        let impossible = Box4x4x16::all_bits()
            .and_not(hband.possible.into())
            .move_column(big1.into(), Small::new(3));
        self.possible = self.possible.and_not_bits(impossible);
    }

    fn propagate_from_vband(&mut self, vband: &Self, big0: Small<3>) {
        self.asserted |= vband.asserted.move_row(big0.into(), Small::new(3));
        let impossible = Box4x4x16::all_bits()
            .and_not(vband.possible.into())
            .move_row(big0.into(), Small::new(3));
        self.possible = self.possible.and_not_bits(impossible);
    }

    fn propagate_to_hband(&self, hband: &mut Self, big1: Small<3>) {
        hband.asserted |= self.asserted.move_column(Small::new(3), big1.into());
        let impossible = Box4x4x16::all_bits()
            .and_not(self.possible.into())
            .move_column(Small::new(3), big1.into());
        hband.possible = hband.possible.and_not_bits(impossible);
    }

    fn propagate_to_vband(&self, vband: &mut Self, big0: Small<3>) {
        vband.asserted |= self.asserted.move_row(Small::new(3), big0.into());
        let impossible = Box4x4x16::all_bits()
            .and_not(self.possible.into())
            .move_row(Small::new(3), big0.into());
        vband.possible = vband.possible.and_not_bits(impossible);
    }
}
