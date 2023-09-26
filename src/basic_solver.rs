use crate::{
    board::{Board, Coordinates, Move},
    digit::Digit,
    digit_set::DigitSet,
    small::Small,
    solver::{Solver, SolverStep},
    square_set::SquareSet,
};

#[derive(Debug)]
pub struct BasicSolver {
    remaining: Vec<SearchState>,
}

impl Solver for BasicSolver {
    fn new(board: &Board) -> Self {
        let mut state = SearchState::new();
        for sq_idx in Small::<81>::all() {
            if let Some(digit) = board.square(sq_idx).to_digit() {
                state.place_digit(sq_idx, digit);
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

        while state.board.empty_squares() != SquareSet::EMPTY {
            let mut progress = false;
            for sq_idx in state.board.empty_squares() {
                let possibilities = state.possibilities(sq_idx);
                let num = possibilities.size();
                match num {
                    0 => return SolverStep::NoProgress,
                    1 => {
                        let digit = possibilities.smallest().unwrap();
                        state.place_digit(sq_idx, digit);
                        progress = true;
                    }
                    _ => {}
                }
            }

            if progress {
                continue;
            }

            let mut sq_iter = state.board.empty_squares().into_iter();
            let mut branch_sq_idx = sq_iter.next().unwrap();
            let mut branch_possibilities = state.possibilities(branch_sq_idx);
            let mut num_branch_possibilities = branch_possibilities.size();

            for sq_idx in sq_iter {
                if num_branch_possibilities == 2 {
                    break;
                }
                let possibilities = state.possibilities(sq_idx);
                let num_possibilities = possibilities.size();
                if num_possibilities < num_branch_possibilities {
                    branch_sq_idx = sq_idx;
                    branch_possibilities = possibilities;
                    num_branch_possibilities = num_possibilities;
                }
            }

            for digit in branch_possibilities {
                let mut branch_state = state;
                branch_state.place_digit(branch_sq_idx, digit);
                self.remaining.push(branch_state);
            }
            state = self.remaining.pop().unwrap();
        }

        // Safety: `state` is fully solved.
        SolverStep::Found(state.board.into_filled())
    }
}

#[derive(Clone, Copy, Debug)]
struct SearchState {
    board: Board,
    line_possibilities: [[[DigitSet; 3]; 3]; 2],
    box_possibilities: [[DigitSet; 3]; 3],
}

impl SearchState {
    fn new() -> Self {
        SearchState {
            board: Board::EMPTY,
            line_possibilities: [[[DigitSet::ALL; 3]; 3]; 2],
            box_possibilities: [[DigitSet::ALL; 3]; 3],
        }
    }

    fn place_digit(&mut self, sq_idx: Small<81>, digit: Digit) {
        self.board.make_move(Move {
            position: sq_idx,
            digit,
        });
        let coord = Coordinates::from(sq_idx);
        self.line_possibilities[0][coord.big[0]][coord.small[0]].remove(digit);
        self.line_possibilities[1][coord.big[1]][coord.small[1]].remove(digit);
        self.box_possibilities[coord.big[0]][coord.big[1]].remove(digit);
    }

    fn possibilities(&self, sq_idx: Small<81>) -> DigitSet {
        let coord = Coordinates::from(sq_idx);
        self.line_possibilities[0][coord.big[0]][coord.small[0]]
            & self.line_possibilities[1][coord.big[1]][coord.small[1]]
            & self.box_possibilities[coord.big[0]][coord.big[1]]
    }
}
