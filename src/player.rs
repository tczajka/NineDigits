use super::board::{Board, FullMove, Move};

trait Player {
    fn opponent_move(&mut self, board: &Board, mov: Move);
    fn choose_move(&mut self, board: &Board) -> FullMove;
}
