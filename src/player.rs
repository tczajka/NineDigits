use crate::board::{FullMove, Move};
use std::time::{Duration, Instant};

pub trait Player {
    fn opponent_move(&mut self, mov: Move);
    fn choose_move(&mut self, start_time: Instant, time_left: Duration) -> FullMove;
}
