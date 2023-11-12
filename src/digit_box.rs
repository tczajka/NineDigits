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

    /// Rotate right by 1.
    pub fn rotate_right(self) -> Self {
        Self(self.0.rotate_words_1_mod_4())
    }

    /// Rotate down by 1.
    pub fn rotate_down(self) -> Self {
        Self(self.0.rotate_words_4())
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
        for y in 0..4 {
            for x in 0..4 {
                write!(f, "{}", a[y][x])?;
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

    /// mask contains 0xffff for entries to pick.
    pub fn pick(self, mask: Box4x4x16) -> Self {
        Self(self.0 & mask)
    }

    /// mask contains 0xffff for entries to replace.
    pub fn replace(self, mask: Box4x4x16, other: DigitBox) -> Self {
        Self(self.0.replace(mask, other.0))
    }

    /// Rotate right by 1.
    pub fn rotate_right(self) -> Self {
        Self(self.0.rotate_right())
    }

    /// Rotate right by 1.
    pub fn rotate_down(self) -> Self {
        Self(self.0.rotate_down())
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
        for y in 0..4 {
            for x in 0..4 {
                write!(f, "{}", a[y][x])?;
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
        for y in 0..4 {
            let boxes: Vec<&str> = lines[y].split('|').collect();
            if boxes.len() != 4 {
                return Err(InvalidInput);
            }
            for x in 0..4 {
                a[y][x] = boxes[x].parse()?;
            }
        }
        Ok(a.into())
    }
}
