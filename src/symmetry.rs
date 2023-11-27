use crate::{
    board::{Coordinates, Move},
    digit::Digit,
    permutation::{Permutation, ALL_PERMUTATIONS_2, ALL_PERMUTATIONS_3},
    random::RandomGenerator,
    small::Small,
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
}
