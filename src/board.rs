use std::mem;

use super::{
    digit::{Digit, OptionalDigit},
    small::Small,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Coordinates {
    pub big: [Small<3>; 2],
    pub small: [Small<3>; 2],
}

impl From<Coordinates> for Small<81> {
    fn from(coords: Coordinates) -> Self {
        let mut val = u8::from(coords.big[0]);
        val = 3 * val + u8::from(coords.big[1]);
        val = 3 * val + u8::from(coords.small[0]);
        val = 3 * val + u8::from(coords.small[1]);
        // SAFETY: Result smaller than 3^4 = 81.
        unsafe { Small::new_unchecked(val) }
    }
}

impl From<Small<81>> for Coordinates {
    fn from(val: Small<81>) -> Self {
        let mut val = u8::from(val);
        // SAFETY: val % 3 < 3.
        let small1 = unsafe { Small::new_unchecked(val % 3) };
        val /= 3;
        // SAFETY: val % 3 < 3.
        let small0 = unsafe { Small::new_unchecked(val % 3) };
        val /= 3;
        // SAFETY: val % 3 < 3.
        let big1 = unsafe { Small::new_unchecked(val % 3) };
        val /= 3;
        // SAFETY: val < 3^4 / 3^3 = 3.
        let big0 = unsafe { Small::new_unchecked(val) };
        Self {
            big: [big0, big1],
            small: [small0, small1],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub squares: [OptionalDigit; 81],
}

impl Board {
    pub const EMPTY: Self = Self {
        squares: [OptionalDigit::NONE; 81],
    };

    /// # Safety
    ///
    /// None of the squares must be `NONE`.
    pub unsafe fn to_filled(&self) -> FilledBoard {
        FilledBoard {
            // Safety: None of the squares are `NONE` and the representation are all `u8`.
            squares: unsafe { mem::transmute(self.squares) },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FilledBoard {
    pub squares: [Digit; 81],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    position: Small<81>,
    digit: Digit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FullMove {
    Move(Move),
    MoveClaimUnique(Move),
    ClaimUnique,
}
