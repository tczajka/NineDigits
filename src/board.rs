use crate::{
    digit::{Digit, OptionalDigit},
    error::InvalidInput,
    small::{CartesianProduct, Small},
    small_set::SmallSet,
};
use std::{
    fmt::{self, Display, Formatter},
    mem,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Coordinates {
    pub big: [Small<3>; 2],
    pub small: [Small<3>; 2],
}

pub fn row_major_coordinates() -> impl Iterator<Item = Coordinates> {
    Small::<3>::all().flat_map(|big0| {
        Small::<3>::all().flat_map(move |small0| {
            Small::<3>::all().flat_map(move |big1| {
                Small::<3>::all().map(move |small1| Coordinates {
                    big: [big0, big1],
                    small: [small0, small1],
                })
            })
        })
    })
}

pub fn box_major_coordinates() -> impl Iterator<Item = Coordinates> {
    Small::<3>::all().flat_map(|big0| {
        Small::<3>::all().flat_map(move |big1| {
            Small::<3>::all().flat_map(move |small0| {
                Small::<3>::all().map(move |small1| Coordinates {
                    big: [big0, big1],
                    small: [small0, small1],
                })
            })
        })
    })
}

impl From<Coordinates> for Small<81> {
    fn from(coords: Coordinates) -> Self {
        let big: Small<9> = Small::combine(coords.big[0], coords.big[1]);
        let small: Small<9> = Small::combine(coords.small[0], coords.small[1]);
        Small::combine(big, small)
    }
}

impl From<Small<81>> for Coordinates {
    fn from(val: Small<81>) -> Self {
        let (big, small) = val.split();
        let (big0, big1) = big.split();
        let (small0, small1) = small.split();
        Self {
            big: [big0, big1],
            small: [small0, small1],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    squares: [OptionalDigit; 81],
    empty: SquareSet,
}

pub type SquareSet = SmallSet<81, u128>;

impl Board {
    pub fn new() -> Self {
        Self {
            squares: [OptionalDigit::NONE; 81],
            empty: SquareSet::all(),
        }
    }

    pub fn square(&self, position: Small<81>) -> OptionalDigit {
        self.squares[position]
    }

    pub fn empty_squares(&self) -> SquareSet {
        self.empty
    }

    pub fn make_move(&mut self, mov: Move) -> Result<(), InvalidInput> {
        if !self.empty.contains(mov.square) {
            return Err(InvalidInput);
        }
        self.squares[mov.square] = mov.digit.into();
        self.empty.remove(mov.square);
        Ok(())
    }

    /// # Panics
    ///
    /// Panics if there are any empty equares.
    pub fn into_filled(self) -> Option<FilledBoard> {
        if self.empty != SquareSet::EMPTY {
            return None;
        }
        Some(FilledBoard {
            // Safety: None of the squares are `NONE` and the representation are all `u8`.
            squares: unsafe { mem::transmute(self.squares) },
        })
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for coord in row_major_coordinates() {
            write!(f, "{}", self.squares[Small::<81>::from(coord)])?;
        }
        Ok(())
    }
}

impl FromStr for Board {
    type Err = InvalidInput;

    fn from_str(s: &str) -> Result<Self, InvalidInput> {
        let mut squares = [OptionalDigit::NONE; 81];
        let mut empty = SquareSet::EMPTY;
        let mut coord_iter = row_major_coordinates();
        let mut char_iter = s.chars();
        loop {
            match (coord_iter.next(), char_iter.next()) {
                (Some(coord), Some(c)) => {
                    let position = Small::<81>::from(coord);
                    let odigit = OptionalDigit::try_from(c)?;
                    squares[position] = odigit;
                    if odigit == OptionalDigit::NONE {
                        empty.insert(position);
                    }
                }
                (None, None) => break,
                _ => return Err(InvalidInput),
            }
        }
        Ok(Self { squares, empty })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct FilledBoard {
    pub squares: [Digit; 81],
}

impl Display for FilledBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for coord in row_major_coordinates() {
            write!(f, "{}", self.squares[Small::<81>::from(coord)])?;
        }
        Ok(())
    }
}

impl FromStr for FilledBoard {
    type Err = InvalidInput;

    fn from_str(s: &str) -> Result<Self, InvalidInput> {
        let mut squares = [Digit::from(Small::new(0)); 81];
        let mut coord_iter = row_major_coordinates();
        let mut char_iter = s.chars();
        loop {
            match (coord_iter.next(), char_iter.next()) {
                (Some(coord), Some(c)) => {
                    squares[Small::<81>::from(coord)] = c.try_into()?;
                }
                (None, None) => break,
                _ => return Err(InvalidInput),
            }
        }
        Ok(Self { squares })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    pub square: Small<81>,
    pub digit: Digit,
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let coord = Coordinates::from(self.square);
        let row = char::from(b'A' + 3 * u8::from(coord.big[0]) + u8::from(coord.small[0]));
        let col = char::from(b'a' + 3 * u8::from(coord.big[1]) + u8::from(coord.small[1]));
        write!(f, "{}{}{}", row, col, self.digit)
    }
}

impl FromStr for Move {
    type Err = InvalidInput;

    fn from_str(s: &str) -> Result<Self, InvalidInput> {
        let mut chars = s.chars();
        let row = chars.next().ok_or(InvalidInput)?;
        let col = chars.next().ok_or(InvalidInput)?;
        let digit = chars.next().ok_or(InvalidInput)?;
        if chars.next().is_some() {
            return Err(InvalidInput);
        }

        let row = u8::try_from(row).map_err(|_| InvalidInput)?;
        if !(b'A'..=b'I').contains(&row) {
            return Err(InvalidInput);
        }
        let row = row - b'A';

        let col = u8::try_from(col).map_err(|_| InvalidInput)?;
        if !(b'a'..=b'i').contains(&col) {
            return Err(InvalidInput);
        }
        let col = col - b'a';

        let digit = Digit::try_from(digit)?;

        let big0 = Small::new(row / 3);
        let small0 = Small::new(row % 3);
        let big1 = Small::new(col / 3);
        let small1 = Small::new(col % 3);
        let coord = Coordinates {
            big: [big0, big1],
            small: [small0, small1],
        };
        Ok(Self {
            square: coord.into(),
            digit,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FullMove {
    Move(Move),
    MoveClaimUnique(Move),
    ClaimUnique,
}

impl Display for FullMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Move(mov) => write!(f, "{}", mov),
            Self::MoveClaimUnique(mov) => write!(f, "{}!", mov),
            Self::ClaimUnique => write!(f, "!"),
        }
    }
}

impl FromStr for FullMove {
    type Err = InvalidInput;

    fn from_str(s: &str) -> Result<Self, InvalidInput> {
        match s.strip_suffix('!') {
            Some("") => Ok(Self::ClaimUnique),
            Some(s) => Ok(Self::MoveClaimUnique(s.parse()?)),
            None => Ok(Self::Move(s.parse()?)),
        }
    }
}
