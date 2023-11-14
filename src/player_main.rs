use crate::{
    board::{FullMove, Move},
    player::Player,
};

pub struct PlayerMain {}

impl PlayerMain {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for PlayerMain {
    fn opponent_move(&mut self, mov: Move) {
        unimplemented!()
    }

    fn choose_move(&mut self) -> FullMove {
        unimplemented!()
    }
}
