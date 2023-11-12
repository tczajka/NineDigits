use crate::{
    digit::Digit,
    digit_set::DigitSet,
    error::InvalidInput,
    simd256::{Simd16x16, Simd4x64},
    small::{CartesianProduct, Small},
};
use std::{
    fmt::{self, Debug, Display, Formatter},
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
    str::FromStr,
};

/// 4x4 box of `u16`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Box4x4x16(Simd16x16);

impl Box4x4x16 {
    pub fn zero() -> Self {
        Self(Simd16x16::zero())
    }

    pub fn fill(x: u16) -> Self {
        Self(Simd16x16::fill(x))
    }

    pub fn all_bits() -> Self {
        Self::fill(0xffff)
    }

    pub fn fill_rows(row: [u16; 4]) -> Self {
        let val: u64 = unsafe { mem::transmute(row) };
        Self(Simd4x64::fill(val).into())
    }

    pub fn is_all_zero(self) -> bool {
        self.0.is_all_zero()
    }

    pub fn set_bit(&mut self, y: Small<4>, x: Small<4>, bit: Small<16>) {
        self.0.set_bit(Small::combine(y, x), bit);
    }

    pub fn clear_bit(&mut self, y: Small<4>, x: Small<4>, bit: Small<16>) {
        self.0.clear_bit(Small::combine(y, x), bit);
    }

    pub fn and_not(self, other: Self) -> Self {
        Self(self.0.and_not(other.0))
    }

    pub fn any_lt(self, other: Self) -> bool {
        self.0.any_lt(other.0)
    }

    pub fn any_gt(self, other: Self) -> bool {
        other.any_lt(self)
    }

    /// Returns 0xffff for equal values, 0 otherwise.
    pub fn masks_eq(self, other: Self) -> Self {
        Self(self.0.masks_eq(other.0))
    }

    /// mask contains 0xffff for entries to replace.
    pub fn replace(self, mask: Self, other: Self) -> Self {
        Self(self.0.replace(mask.0, other.0))
    }

    pub fn replace_last_row(self, other: Self) -> Self {
        Self(self.0.replace_top_4_words(other.0))
    }

    pub fn replace_last_column(self, other: Self) -> Self {
        Self(self.0.replace_words_3_mod_4(other.0))
    }

    /// Rotate right by 1.
    pub fn rotate_right(self) -> Self {
        Self(self.0.rotate_words_1_mod_4())
    }

    /// Rotate down by 1.
    pub fn rotate_down(self) -> Self {
        Self(self.0.rotate_words_4())
    }

    /// Rotate first three columns right by 1.
    pub fn rotate_first_3_right(self) -> Self {
        Self(self.0.rotate_first_3_words_1_mod_4())
    }

    /// Rotate first three rows down by 1.
    pub fn rotate_first_3_down(self) -> Self {
        Self(self.0.rotate_first_12_words_4())
    }

    /// Move a row to another row. Other rows become zero.
    pub fn move_row(self, from: Small<4>, to: Small<4>) -> Self {
        Self(self.0.move_4_words(from, to))
    }

    /// Move a column to another column. Other columns become zero.
    pub fn move_column(self, from: Small<4>, to: Small<4>) -> Self {
        Self(self.0.move_words_mod_4(from, to))
    }
}

impl From<[[u16; 4]; 4]> for Box4x4x16 {
    fn from(x: [[u16; 4]; 4]) -> Self {
        let x: [u16; 16] = unsafe { mem::transmute(x) };
        Self(x.into())
    }
}

impl From<Box4x4x16> for [[u16; 4]; 4] {
    fn from(x: Box4x4x16) -> Self {
        let x: [u16; 16] = x.0.into();
        unsafe { mem::transmute(x) }
    }
}

impl BitAnd for Box4x4x16 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Box4x4x16 {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Box4x4x16 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Box4x4x16 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Box4x4x16 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Box4x4x16 {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Display for Box4x4x16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let a: [[u16; 4]; 4] = (*self).into();
        for row in a {
            for (x, val) in row.into_iter().enumerate() {
                write!(f, "{}", val)?;
                if x < 3 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Debug for Box4x4x16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

/// 4x4 box of `DigitSet`s.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct DigitBox(Box4x4x16);

impl DigitBox {
    pub fn empty() -> Self {
        Self(Box4x4x16::zero())
    }

    #[rustfmt::skip]
    pub fn all3x3() -> Self {
        Self::from([
            [DigitSet::all(), DigitSet::all(), DigitSet::all(), DigitSet::EMPTY],
            [DigitSet::all(), DigitSet::all(), DigitSet::all(), DigitSet::EMPTY],
            [DigitSet::all(), DigitSet::all(), DigitSet::all(), DigitSet::EMPTY],
            [DigitSet::EMPTY, DigitSet::EMPTY, DigitSet::EMPTY, DigitSet::EMPTY],
        ])
    }

    pub fn fill(x: DigitSet) -> Self {
        let val: u16 = unsafe { mem::transmute(x) };
        Self(Box4x4x16::fill(val))
    }

    pub fn fill_rows(row: [DigitSet; 4]) -> Self {
        let val: [u16; 4] = unsafe { mem::transmute(row) };
        Self(Box4x4x16::fill_rows(val))
    }

    pub fn is_all_empty(self) -> bool {
        self.0.is_all_zero()
    }

    pub fn set(&mut self, y: Small<4>, x: Small<4>, digit: Digit) {
        self.0.set_bit(y, x, Small::<9>::from(digit).into());
    }

    pub fn clear(&mut self, y: Small<4>, x: Small<4>, digit: Digit) {
        self.0.clear_bit(y, x, Small::<9>::from(digit).into());
    }

    pub fn and_not_bits(self, other: Box4x4x16) -> Self {
        Self(self.0.and_not(other))
    }

    pub fn and_not(self, other: Self) -> Self {
        Self(self.0.and_not(other.0))
    }

    pub fn counts(self) -> Box4x4x16 {
        Box4x4x16(self.0 .0.popcount_9())
    }

    /// Returns 0xffff for equal values, 0 otherwise.
    pub fn masks_eq(self, other: Self) -> Box4x4x16 {
        self.0.masks_eq(other.0)
    }

    pub fn and_bits(self, mask: Box4x4x16) -> Self {
        Self(self.0 & mask)
    }

    /// mask contains 0xffff for entries to replace.
    pub fn replace(self, mask: Box4x4x16, other: DigitBox) -> Self {
        Self(self.0.replace(mask, other.0))
    }

    pub fn replace_last_row(self, other: Self) -> Self {
        Self(self.0.replace_last_row(other.0))
    }

    pub fn replace_last_column(self, other: Self) -> Self {
        Self(self.0.replace_last_column(other.0))
    }

    /// Rotate right by 1.
    pub fn rotate_right(self) -> Self {
        Self(self.0.rotate_right())
    }

    /// Rotate right by 1.
    pub fn rotate_down(self) -> Self {
        Self(self.0.rotate_down())
    }

    /// Rotate first three columns right by 1.
    pub fn rotate_first_3_right(self) -> Self {
        Self(self.0.rotate_first_3_right())
    }

    /// Rotate first three rows down by 1.
    pub fn rotate_first_3_down(self) -> Self {
        Self(self.0.rotate_first_3_down())
    }

    /// Move a row to another row. Other rows become empty.
    pub fn move_row(self, from: Small<4>, to: Small<4>) -> Self {
        Self(self.0.move_row(from, to))
    }

    /// Move a column to another column. Other columns become empty.
    pub fn move_column(self, from: Small<4>, to: Small<4>) -> Self {
        Self(self.0.move_column(from, to))
    }
}

impl From<[[DigitSet; 4]; 4]> for DigitBox {
    fn from(x: [[DigitSet; 4]; 4]) -> Self {
        // SAFETY: DigitSet is repr(transparent) over u16.
        let x: [[u16; 4]; 4] = unsafe { mem::transmute(x) };
        Self(x.into())
    }
}

impl From<DigitBox> for [[DigitSet; 4]; 4] {
    fn from(x: DigitBox) -> Self {
        let x: [[u16; 4]; 4] = x.0.into();
        // SAFETY: DigitSet is repr(transparent) over u16.
        unsafe { mem::transmute(x) }
    }
}

impl From<DigitBox> for Box4x4x16 {
    fn from(x: DigitBox) -> Self {
        x.0
    }
}

impl BitAnd for DigitBox {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for DigitBox {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for DigitBox {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DigitBox {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for DigitBox {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for DigitBox {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Display for DigitBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let a: [[DigitSet; 4]; 4] = (*self).into();
        for row in a {
            for (x, val) in row.into_iter().enumerate() {
                write!(f, "{}", val)?;
                if x < 3 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Debug for DigitBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

impl FromStr for DigitBox {
    type Err = InvalidInput;

    fn from_str(s: &str) -> Result<Self, InvalidInput> {
        let mut a = [[DigitSet::EMPTY; 4]; 4];
        let lines: Vec<&str> = s.lines().collect();
        if lines.len() != 4 {
            return Err(InvalidInput);
        }
        for (y, line) in lines.into_iter().enumerate() {
            let boxes: Vec<&str> = line.split('|').collect();
            if boxes.len() != 4 {
                return Err(InvalidInput);
            }
            for (x, entry) in boxes.into_iter().enumerate() {
                a[y][x] = entry.parse()?;
            }
        }
        Ok(a.into())
    }
}
