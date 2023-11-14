use crate::board::{FullMove, Move};

pub trait Player {
    fn opponent_move(&mut self, mov: Move);
    fn choose_move(&mut self) -> FullMove;
}
