use super::small::Small;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SquareSet(u128);

impl SquareSet {
    pub const EMPTY: SquareSet = SquareSet(0);
    pub const ALL: SquareSet = SquareSet((1u128 << 81) - 1);

    pub fn insert(&mut self, square: Small<81>) {
        self.0 |= 1 << u8::from(square);
    }

    pub fn remove(&mut self, square: Small<81>) {
        self.0 &= !(1 << u8::from(square));
    }
}
