use crate::{
    board::{box_major_coordinates, Board, Coordinates, Move},
    digit::{Digit, OptionalDigit},
    permutation::{Permutation, ALL_PERMUTATIONS_2, ALL_PERMUTATIONS_3},
    random::RandomGenerator,
    small::Small,
};
use std::{
    cmp::{self, Ordering},
    collections::HashSet,
    iter,
};

/// First apply flip, then big, then small.
#[derive(Clone, Copy, Debug)]
pub struct Symmetry {
    pub flip: Permutation<2>,
    pub big: [Permutation<3>; 2],
    pub small: [[Permutation<3>; 3]; 2],
    pub digits: Permutation<9>,
}

impl Symmetry {
    pub fn identity() -> Self {
        Self {
            flip: Permutation::identity(),
            big: [Permutation::identity(); 2],
            small: [[Permutation::identity(); 3]; 2],
            digits: Permutation::identity(),
        }
    }

    pub fn random(rng: &mut RandomGenerator) -> Self {
        let flip = *rng.choose(&ALL_PERMUTATIONS_2);
        let big = [(); 2].map(|_| *rng.choose(&ALL_PERMUTATIONS_3));
        let small = [(); 2].map(|_| [(); 3].map(|_| *rng.choose(&ALL_PERMUTATIONS_3)));
        let mut digits = Permutation::identity();
        for i in Small::all() {
            digits.swap_forward(i, rng.uniform_usize(usize::from(i) + 1).try_into().unwrap());
        }
        Self {
            flip,
            big,
            small,
            digits,
        }
    }

    pub fn forward_coord_digit(
        &self,
        mut coord: Coordinates,
        mut digit: Digit,
    ) -> (Coordinates, Digit) {
        coord.big = self.flip.then_array(&coord.big);
        coord.small = self.flip.then_array(&coord.small);
        coord.big[0] = self.big[0].forward(coord.big[0]);
        coord.big[1] = self.big[1].forward(coord.big[1]);
        coord.small[0] = self.small[0][coord.big[0]].forward(coord.small[0]);
        coord.small[1] = self.small[1][coord.big[1]].forward(coord.small[1]);
        digit = self.digits.forward(digit.into()).into();
        (coord, digit)
    }

    pub fn backward_coord_digit(
        &self,
        mut coord: Coordinates,
        mut digit: Digit,
    ) -> (Coordinates, Digit) {
        digit = self.digits.backward(digit.into()).into();
        coord.small[1] = self.small[1][coord.big[1]].backward(coord.small[1]);
        coord.small[0] = self.small[0][coord.big[0]].backward(coord.small[0]);
        coord.big[1] = self.big[1].backward(coord.big[1]);
        coord.big[0] = self.big[0].backward(coord.big[0]);
        // flip is its own inverse
        coord.small = self.flip.then_array(&coord.small);
        coord.big = self.flip.then_array(&coord.big);
        (coord, digit)
    }

    pub fn forward_move(&self, mov: Move) -> Move {
        let (coord, digit) = self.forward_coord_digit(mov.square.into(), mov.digit);
        Move {
            square: coord.into(),
            digit,
        }
    }

    pub fn backward_move(&self, mov: Move) -> Move {
        let (coord, digit) = self.backward_coord_digit(mov.square.into(), mov.digit);
        Move {
            square: coord.into(),
            digit,
        }
    }

    pub fn forward_board(&self, board: &Board) -> Board {
        let mut new_board = Board::new();
        for coord in box_major_coordinates() {
            if let Some(digit) = board.square(coord.into()).to_digit() {
                let (new_coord, new_digit) = self.forward_coord_digit(coord, digit);
                new_board
                    .make_move(Move {
                        square: new_coord.into(),
                        digit: new_digit,
                    })
                    .unwrap();
            }
        }
        new_board
    }
}

pub fn normalize_board(board: &Board) -> (Board, Symmetry) {
    let mut possibilities: Vec<(Board, Symmetry)> = vec![(*board, Symmetry::identity())];

    // Set flip, big[0], big[1] to maximize box_counts.
    expand_possibilities(
        board,
        &mut possibilities,
        |_, &symmetry| {
            ALL_PERMUTATIONS_2.iter().flat_map(move |&flip| {
                ALL_PERMUTATIONS_3.iter().flat_map(move |&big0| {
                    ALL_PERMUTATIONS_3.iter().map(move |&big1| Symmetry {
                        flip,
                        big: [big0, big1],
                        ..symmetry
                    })
                })
            })
        },
        |b| cmp::Reverse(box_counts(b)),
    );

    // Set small[0][0], small[1][0] to maximize box_layout([0, 0]).
    expand_possibilities(
        board,
        &mut possibilities,
        |_, &symmetry| {
            ALL_PERMUTATIONS_3.iter().flat_map(move |&small00| {
                ALL_PERMUTATIONS_3.iter().map(move |&small10| {
                    let mut s = symmetry;
                    s.small[0][0] = small00;
                    s.small[1][0] = small10;
                    s
                })
            })
        },
        |b| cmp::Reverse(box_layout(b, [Small::new(0), Small::new(0)])),
    );

    for column in [Small::new(1), Small::new(2)] {
        // Set small[1][column] to maximize box_layout([0, column]).
        expand_possibilities(
            board,
            &mut possibilities,
            |_, &symmetry| {
                ALL_PERMUTATIONS_3.iter().map(move |&small| {
                    let mut s = symmetry;
                    s.small[1][column] = small;
                    s
                })
            },
            |b| cmp::Reverse(box_layout(b, [Small::new(0), column])),
        );
    }

    for row in [Small::new(1), Small::new(2)] {
        // Set small[0][row] to maximize box_layout([row, 1]) and box_layout([row, 2]).
        expand_possibilities(
            board,
            &mut possibilities,
            |_, &symmetry| {
                ALL_PERMUTATIONS_3.iter().map(move |&small| {
                    let mut s = symmetry;
                    s.small[0][row] = small;
                    s
                })
            },
            |b| {
                cmp::Reverse((
                    box_layout(b, [row, Small::new(1)]),
                    box_layout(b, [row, Small::new(2)]),
                ))
            },
        );
    }

    // Set digits to minimize the board.
    expand_possibilities(
        board,
        &mut possibilities,
        |b, &symmetry| {
            iter::once(Symmetry {
                digits: normalize_digits(b),
                ..symmetry
            })
        },
        |b| *b,
    );

    assert_eq!(possibilities.len(), 1);
    possibilities[0]
}

/// Expands the possibilties by applying expand_symmetry to each, and taking the smallest eval.
fn expand_possibilities<F, G, I, E>(
    board: &Board,
    possibilities: &mut Vec<(Board, Symmetry)>,
    expand_symmetry: F,
    eval: G,
) where
    F: Fn(&Board, &Symmetry) -> I,
    I: Iterator<Item = Symmetry>,
    G: Fn(&Board) -> E,
    E: Ord,
{
    let mut seen = HashSet::new();
    let mut new_possibilities = Vec::new();
    let mut best_val = None;

    for (sboard, symmetry) in &*possibilities {
        for new_symmetry in expand_symmetry(sboard, symmetry) {
            let new_board = new_symmetry.forward_board(board);
            let new_val = eval(&new_board);
            match best_val {
                None => {
                    best_val = Some(new_val);
                    new_possibilities.push((new_board, new_symmetry));
                    seen.insert(new_board);
                }
                Some(ref bval) => match new_val.cmp(bval) {
                    Ordering::Greater => {}
                    Ordering::Equal => {
                        if seen.insert(new_board) {
                            new_possibilities.push((new_board, new_symmetry));
                        }
                    }
                    Ordering::Less => {
                        best_val = Some(new_val);
                        new_possibilities.clear();
                        new_possibilities.push((new_board, new_symmetry));
                        seen.clear();
                        seen.insert(new_board);
                    }
                },
            }
        }
    }

    *possibilities = new_possibilities;
}

fn box_counts(board: &Board) -> [[u8; 3]; 3] {
    let mut counts = [[0; 3]; 3];
    for coord in box_major_coordinates() {
        if board.square(coord.into()) != OptionalDigit::NONE {
            counts[coord.big[0]][coord.big[1]] += 1;
        }
    }
    counts
}

fn box_layout(board: &Board, big: [Small<3>; 2]) -> [[bool; 3]; 3] {
    let mut layout = [[false; 3]; 3];
    for small0 in Small::all() {
        for small1 in Small::all() {
            let coord = Coordinates {
                big,
                small: [small0, small1],
            };
            layout[small0][small1] = board.square(coord.into()) != OptionalDigit::NONE;
        }
    }
    layout
}

fn normalize_digits(board: &Board) -> Permutation<9> {
    let mut perm = Permutation::identity();
    let mut next_digit: u8 = 0;

    for coord in box_major_coordinates() {
        if let Some(digit) = board.square(coord.into()).to_digit() {
            if u8::from(perm.forward(digit.into())) >= next_digit {
                // Digit not yet seen. Set forward(digit) to next_digit.
                let x = perm.backward(next_digit.try_into().unwrap());
                perm.swap_forward(digit.into(), x);
                next_digit += 1;
            }
        }
    }
    perm
}
