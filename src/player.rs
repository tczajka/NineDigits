use super::board::{Board, Move};

trait Player {
    fn opponent_move(&mut self, board: &Board, mov: Move);
    fn choose_move(&mut self, board: &Board) -> FullMove;
}

pub enum FullMove {
    Move(Move),
    LastMove(Move),
    ClaimUnique,
}
