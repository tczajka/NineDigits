use super::{
    board::{Board, Coordinates, FilledBoard},
    small::Small,
    square_set::SquareSet,
};

#[derive(Debug)]
pub struct Solver {
    remaining: Vec<SearchState>,
}

impl Solver {
    fn new(board: &Board) -> Self {
        let mut state = SearchState {
            solved: *board,
            to_solve: SquareSet::EMPTY,
            line_possibilities: [[[0x1ff; 3]; 3]; 2],
            box_possibilities: [[0x1ff; 3]; 3],
        };
        for sq_idx in Small::<81>::all() {
            match state.solved.squares[sq_idx].to_digit() {
                Some(digit) => {
                    let coord = Coordinates::from(sq_idx);
                    let digit_bit = 1 << u8::from(Small::<9>::from(digit));
                    state.line_possibilities[0][coord.big[0]][coord.small[0]] &= !digit_bit;
                    state.line_possibilities[1][coord.big[1]][coord.small[1]] &= !digit_bit;
                    state.box_possibilities[coord.big[0]][coord.big[1]] &= !digit_bit;
                    state.solved.squares[sq_idx] = digit.into();
                }
                None => {
                    state.to_solve.insert(sq_idx);
                }
            }
        }
        Self {
            remaining: vec![state],
        }
    }

    fn step() -> SolverStep {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
enum SolverStep {
    Found(FilledBoard),
    NoProgress,
    Done,
}

#[derive(Clone, Copy, Debug)]
struct SearchState {
    solved: Board,
    to_solve: SquareSet,
    line_possibilities: [[[u16; 3]; 3]; 2],
    box_possibilities: [[u16; 3]; 3],
}
