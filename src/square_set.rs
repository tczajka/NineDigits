use super::small::Small;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SquareSet(u128);

impl SquareSet {
    pub const EMPTY: SquareSet = SquareSet(0);
    pub const ALL: SquareSet = SquareSet((1u128 << 81) - 1);

    pub fn contains(self, square: Small<81>) -> bool {
        self.0 & 1 << u8::from(square) != 0
    }

    pub fn insert(&mut self, square: Small<81>) {
        self.0 |= 1 << u8::from(square);
    }

    pub fn remove(&mut self, square: Small<81>) {
        self.0 &= !(1 << u8::from(square));
    }

    pub fn smallest(self) -> Option<Small<81>> {
        if self == SquareSet::EMPTY {
            return None;
        }
        let val = self.0.trailing_zeros() as u8;
        // SAFETY: `res` is in the range `0..81` because `self` is not empty.
        let small = unsafe { Small::<81>::new_unchecked(val) };
        Some(small)
    }
}

impl IntoIterator for SquareSet {
    type Item = Small<81>;
    type IntoIter = SquareSetIterator;

    fn into_iter(self) -> SquareSetIterator {
        SquareSetIterator(self)
    }
}

#[derive(Clone, Debug)]
pub struct SquareSetIterator(SquareSet);

impl Iterator for SquareSetIterator {
    type Item = Small<81>;

    fn next(&mut self) -> Option<Small<81>> {
        let res = self.0.smallest()?;
        self.0.remove(res);
        Some(res)
    }
}
