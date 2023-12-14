use crate::{
    board::{FullMove, Move},
    digit::Digit,
    digit_set::DigitSet,
    error::ResourcesExceeded,
    log, settings,
    small::Small,
    solution_table::{EndgameMove, SolutionTable, SquareMoveTable},
    transposition_table::TranspositionTable,
};
use std::time::{Duration, Instant};

pub struct EndgameSolver {
    transposition_table: TranspositionTable,
    num_nodes: u64,
}

impl EndgameSolver {
    pub fn new(transposition_table_memory: usize) -> Self {
        Self {
            transposition_table: TranspositionTable::new(transposition_table_memory),
            num_nodes: 0,
        }
    }

    pub fn choose_move_best_effort(
        &mut self,
        solutions: &SolutionTable,
        mut start_time: Instant,
        mut time_left: Duration,
    ) -> FullMove {
        self.transposition_table.new_era();
        self.num_nodes = 0;

        if solutions.is_empty() {
            log::write_line!(Always, "Error: no solutions!");
            // No good option, let's just claim victory.
            return FullMove::ClaimUnique;
        }
        if solutions.len() == 1 {
            log::write_line!(Info, "Lucky win: opponent didn't claim.");
            return FullMove::ClaimUnique;
        }

        let move_tables = solutions.move_tables();

        if let Some(mov) = self.check_quick_win_root(solutions.len(), &move_tables) {
            log::write_line!(Info, "quick win");
            return mov;
        }

        let (solutions, mut moves) = solutions.compress_and_gen_moves(&move_tables);
        moves.sort_by_key(|x| x.num_solutions);
        let num_moves = moves.len();

        let offense_deadline =
            start_time + time_left.mul_f64(settings::ENDGAME_OFFENSE_TIME_FRACTION);
        let offense_deadline_extended =
            start_time + time_left.mul_f64(settings::ENDGAME_OFFENSE_EXTENDED_TIME_FRACTION);

        let mut offense_index = 0;
        let mut best_losing_move_index = num_moves - 1;
        let mut best_losing_move_difficulty = 0;

        while offense_index < num_moves {
            if Instant::now() > offense_deadline {
                log::write_line!(Info, "endgame offense {offense_index} / {num_moves}");
                break;
            }
            let mov = &moves[offense_index];
            if let Some(difficulty_max) = settings::ENDGAME_OFFENSE_DIFFICULTY_MAX {
                if mov.num_solutions > difficulty_max {
                    log::write_line!(
                        Info,
                        "endgame offense {offense_index} / {num_moves} {}",
                        ResourcesExceeded::Difficulty(mov.num_solutions)
                    );
                    break;
                }
            }
            match self.solve_after_move(&solutions, mov, None, offense_deadline_extended, None) {
                Ok(EndgameResult::Loss) => {
                    // Found a winning move.
                    log::write_line!(
                        Info,
                        "endgame win {offense_index} / {num_moves} difficulty={}",
                        mov.num_solutions
                    );
                    self.log_stats(start_time, Instant::now());
                    return FullMove::Move(solutions.original_move(mov.mov));
                }
                Ok(EndgameResult::Win(None)) => {}
                Ok(EndgameResult::Win(Some(opp_mov))) => {
                    if opp_mov.num_solutions > best_losing_move_difficulty {
                        best_losing_move_index = offense_index;
                        best_losing_move_difficulty = opp_mov.num_solutions;
                    }
                }
                Err(e) => {
                    log::write_line!(Info, "endgame offense {offense_index} / {num_moves} {e}");
                    break;
                }
            }
            offense_index += 1;
        }

        {
            let t = Instant::now();
            self.log_stats(start_time, t);
            time_left = time_left.saturating_sub(t.saturating_duration_since(start_time));
            start_time = t;
            self.num_nodes = 0;
        }

        let defense_start_time = start_time;
        for defense_index in (offense_index..num_moves).rev() {
            let mov = &moves[defense_index];
            let defense_deadline =
                start_time + time_left.mul_f64(settings::ENDGAME_DEFENSE_TIME_FRACTION);
            let defense_deadline_extended =
                start_time + time_left.mul_f64(settings::ENDGAME_DEFENSE_EXTENDED_TIME_FRACTION);

            match self.solve_after_move(
                &solutions,
                mov,
                Some(defense_deadline),
                defense_deadline_extended,
                settings::ENDGAME_DEFENSE_DIFFICULTY_MAX,
            ) {
                Ok(EndgameResult::Loss) => {
                    log::write_line!(
                        Info,
                        "endgame defense win! {defense_index} / {num_moves} difficulty = {}",
                        mov.num_solutions,
                    );
                    self.log_stats(defense_start_time, Instant::now());
                    return FullMove::Move(solutions.original_move(mov.mov));
                }
                Ok(EndgameResult::Win(maybe_move)) => {
                    // Panic. Reset time for next defensive move.
                    if let Some(opp_mov) = maybe_move {
                        if opp_mov.num_solutions > best_losing_move_difficulty {
                            best_losing_move_index = defense_index;
                            best_losing_move_difficulty = opp_mov.num_solutions;
                        }
                    }
                    let t = Instant::now();
                    time_left = time_left.saturating_sub(t.saturating_duration_since(start_time));
                    start_time = t;
                }
                Err(e) => {
                    log::write_line!(
                        Info,
                        "endgame defense safe: {defense_index} / {num_moves} num_solutions={} {e}",
                        mov.num_solutions,
                    );
                    self.log_stats(defense_start_time, Instant::now());
                    return FullMove::Move(solutions.original_move(mov.mov));
                }
            }
        }

        log::write_line!(
            Info,
            "endgame lost difficulty {best_losing_move_difficulty}"
        );
        self.log_stats(defense_start_time, Instant::now());
        FullMove::Move(solutions.original_move(moves[best_losing_move_index].mov))
    }

    pub fn solve(
        &mut self,
        solutions: &SolutionTable,
        deadline_toplevel: Option<Instant>,
        deadline: Instant,
        difficulty_max: Option<u32>,
    ) -> Result<EndgameResult, ResourcesExceeded> {
        let start_time = Instant::now();
        self.transposition_table.new_era();
        self.num_nodes = 0;

        if solutions.is_empty() {
            log::write_line!(Always, "Error: no solutions!");
            return Ok(EndgameResult::Loss);
        }
        if solutions.len() == 1 {
            return Ok(EndgameResult::Loss);
        }
        if solutions.len() < 4 {
            return Ok(EndgameResult::Win(None));
        }
        if let Some(result) = self.transposition_table.find(solutions.hash()) {
            return Ok(result);
        }

        let result =
            self.solve_recursive(solutions, deadline_toplevel, deadline, difficulty_max)?;
        self.log_stats(start_time, Instant::now());
        Ok(result)
    }

    fn solve_recursive(
        &mut self,
        solutions: &SolutionTable,
        deadline_toplevel: Option<Instant>,
        deadline_extended: Instant,
        difficulty_max: Option<u32>,
    ) -> Result<EndgameResult, ResourcesExceeded> {
        self.num_nodes += 1;

        if self.num_nodes % settings::ENDGAME_CHECK_TIME_NODES == 0
            && Instant::now() >= deadline_extended
        {
            return Err(ResourcesExceeded::Time);
        }

        let move_tables = solutions.move_tables();
        let mut result = self.check_quick_win(solutions, &move_tables);
        if matches!(result, EndgameResult::Loss) {
            let (solutions, mut moves) = solutions.compress_and_gen_moves(&move_tables);
            moves.sort_by_key(|x| x.num_solutions);

            let mut is_losing_move = [DigitSet::EMPTY; 81];

            for mov in moves.iter() {
                if let Some(difficulty_max) = difficulty_max {
                    if mov.num_solutions > difficulty_max {
                        Err(ResourcesExceeded::Difficulty(mov.num_solutions))?;
                    }
                }
                if let Some(deadline_toplevel) = deadline_toplevel {
                    if Instant::now() >= deadline_toplevel {
                        return Err(ResourcesExceeded::Time);
                    }
                }
                let orig_mov = solutions.original_move(mov.mov);
                if is_losing_move[orig_mov.square].contains(orig_mov.digit) {
                    continue;
                }
                match self.solve_after_move(&solutions, mov, None, deadline_extended, None)? {
                    EndgameResult::Loss => {
                        result = EndgameResult::Win(Some(EndgameMove {
                            mov: orig_mov,
                            ..*mov
                        }));
                        break;
                    }
                    EndgameResult::Win(None) => {}
                    EndgameResult::Win(Some(orig_opp_mov)) => {
                        // mov loses to opp_mov.
                        // Therefore by transposition opp_mov loses to mov, if mov response is allowed.
                        // But if mov isn't allowed (by Jelmer's rule), then num_solutions after we play opp_mov is the same as after the mov, opp_mov sequence.
                        // In the latter case setting this bit does no harm because we have already processed opp_move because it has fewer solutions.
                        is_losing_move[orig_opp_mov.mov.square].insert(orig_opp_mov.mov.digit);
                    }
                }
            }
        }
        self.transposition_table.insert(solutions.hash(), result);
        Ok(result)
    }

    fn solve_after_move(
        &mut self,
        solutions: &SolutionTable,
        mov: &EndgameMove,
        deadline_toplevel: Option<Instant>,
        deadline_extended: Instant,
        difficulty_max: Option<u32>,
    ) -> Result<EndgameResult, ResourcesExceeded> {
        if mov.num_solutions == 1 {
            return Ok(EndgameResult::Loss);
        }
        if mov.num_solutions < 4 {
            return Ok(EndgameResult::Win(None));
        }
        if let Some(result) = self.transposition_table.find(mov.hash) {
            return Ok(result);
        }
        assert_ne!(solutions.hash(), mov.hash);
        let new_solutions = solutions.filter(mov.num_solutions, mov.mov);
        assert_eq!(new_solutions.len(), mov.num_solutions);
        assert_eq!(new_solutions.hash(), mov.hash);

        self.solve_recursive(
            &new_solutions,
            deadline_toplevel,
            deadline_extended,
            difficulty_max,
        )
    }

    fn check_quick_win_root(
        &self,
        num_solutions: u32,
        move_tables: &[SquareMoveTable],
    ) -> Option<FullMove> {
        for (square, move_table) in Small::all().zip(move_tables.iter()) {
            for (digit, num_solutions) in Digit::all().zip(move_table.num_solutions) {
                if num_solutions == 1 {
                    return Some(FullMove::MoveClaimUnique(Move { square, digit }));
                }
            }
        }

        // Enhanced transposition cutoff.
        for (square, move_table) in Small::all().zip(move_tables.iter()) {
            for ((&move_num_solutions, &hash), digit) in move_table
                .num_solutions
                .iter()
                .zip(move_table.hash.iter())
                .zip(Digit::all())
            {
                if move_num_solutions >= 4
                    && move_num_solutions < num_solutions
                    && matches!(
                        self.transposition_table.find(hash),
                        Some(EndgameResult::Loss)
                    )
                {
                    return Some(FullMove::Move(Move { square, digit }));
                }
            }
        }

        None
    }

    fn check_quick_win(
        &self,
        solutions: &SolutionTable,
        move_tables: &[SquareMoveTable],
    ) -> EndgameResult {
        assert_eq!(move_tables.len(), usize::from(solutions.num_squares()));
        for ((square, &num_moves), move_table) in (0u8..)
            .zip(solutions.num_moves_per_square().iter())
            .zip(move_tables.iter())
        {
            for (digit, &num_solutions) in (0..num_moves).zip(
                unsafe {
                    move_table
                        .num_solutions
                        .get_unchecked(..usize::from(num_moves))
                }
                .iter(),
            ) {
                if num_solutions == 1 {
                    let digit = unsafe { Small::new_unchecked(digit) }.into();
                    return EndgameResult::Win(Some(EndgameMove {
                        mov: solutions.original_move(Move {
                            square: unsafe { Small::new_unchecked(square) },
                            digit,
                        }),
                        num_solutions,
                        hash: move_table.hash[digit],
                    }));
                }
            }
        }

        // Enhanced transposition cutoff.
        for ((square, &num_moves), move_table) in (0u8..)
            .zip(solutions.num_moves_per_square().iter())
            .zip(move_tables.iter())
        {
            for ((digit, &num_solutions), &hash) in (0..num_moves)
                .zip(
                    unsafe {
                        move_table
                            .num_solutions
                            .get_unchecked(..usize::from(num_moves))
                    }
                    .iter(),
                )
                .zip(unsafe { move_table.hash.get_unchecked(..usize::from(num_moves)) }.iter())
            {
                if num_solutions >= 4
                    && num_solutions < solutions.len()
                    && matches!(
                        self.transposition_table.find(hash),
                        Some(EndgameResult::Loss)
                    )
                {
                    let digit = unsafe { Small::new_unchecked(digit) }.into();
                    return EndgameResult::Win(Some(EndgameMove {
                        mov: solutions.original_move(Move {
                            square: unsafe { Small::new_unchecked(square) },
                            digit,
                        }),
                        num_solutions,
                        hash,
                    }));
                }
            }
        }
        EndgameResult::Loss
    }

    fn log_stats(&self, start_time: Instant, end_time: Instant) {
        let processing_time = end_time.saturating_duration_since(start_time);
        log::write_line!(
            Info,
            "nodes: {} time: {:.3?} knps: {:.1}",
            self.num_nodes,
            processing_time,
            self.num_nodes as f64 / processing_time.as_secs_f64() / 1000.0
        );
    }
}

/// Win stores the *original* move.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EndgameResult {
    Win(Option<EndgameMove>),
    Loss,
}
