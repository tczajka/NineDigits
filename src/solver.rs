use super::{
    board::{Board, Coordinates, FilledBoard},
    small::Small,
};

#[derive(Debug)]
pub struct Solver {
    remaining: Vec<SearchState>,
}

impl Solver {
    fn new(board: &Board) -> Self {
        let mut state = SearchState {
            solved: *board,
            to_solve: Vec::new(),
            line_possibilities: [[[0x1ff; 3]; 3]; 2],
            box_possibilities: [[0x1ff; 3]; 3],
        };
        for sq_idx in Small::all() {
            let coord = Coordinates::from(sq_idx);
            match state.solved.squares[sq_idx].to_digit() {
                Some(digit) => {
                    let digit_bit = 1 << u8::from(Small::<9>::from(digit));
                    state.line_possibilities[0][coord.big[0]][coord.small[0]] &= !digit_bit;
                    state.line_possibilities[1][coord.big[1]][coord.small[1]] &= !digit_bit;
                    state.box_possibilities[coord.big[0]][coord.big[1]] &= !digit_bit;
                    state.solved.squares[sq_idx] = digit.into();
                }
                None => {
                    state.to_solve.push(coord);
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

#[derive(Clone, Debug)]
struct SearchState {
    solved: Board,
    to_solve: Vec<Coordinates>,
    line_possibilities: [[[u16; 3]; 3]; 2],
    box_possibilities: [[u16; 3]; 3],
}
