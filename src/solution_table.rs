use crate::{
    board::{Board, Move},
    digit::Digit,
    error::ResourcesExceeded,
    fast_solver::FastSolver,
    random::RandomGenerator,
    settings,
    small::Small,
    solver::{Solver, SolverStep},
};
use std::{slice, time::Instant};

pub struct SolutionTable {
    original_squares: Vec<Small<81>>,
    /// Each solution is: ID_BYTES + square_infos.len().
    solutions: Vec<u8>,
    // Number of solutions. len * num_moves_per_square.len() == solutions.len().
    len: u32,
    /// Xor of solution IDs.
    hash: u64,
}

impl SolutionTable {
    const ID_BYTES: usize = 8;

    pub fn empty() -> Self {
        Self {
            original_squares: Vec::new(),
            solutions: Vec::new(),
            len: 0,
            hash: 0,
        }
    }

    pub fn with_capacity(original_squares: Vec<Small<81>>, max_solutions: u32) -> Self {
        let solution_len = Self::ID_BYTES + original_squares.len();
        Self {
            original_squares,
            solutions: Vec::with_capacity(usize::try_from(max_solutions).unwrap() * solution_len),
            len: 0,
            hash: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn num_squares(&self) -> u8 {
        self.original_squares.len() as u8
    }

    pub fn iter(&self) -> impl Iterator<Item = SolutionRef<'_>> {
        let slen = self.solution_len();
        self.solutions.chunks_exact(slen).map(SolutionRef)
    }

    pub fn append(&mut self, id: u64, digits: &[Digit]) {
        assert_eq!(digits.len(), usize::from(self.num_squares()));
        self.solutions.extend_from_slice(&id.to_le_bytes());
        // Safety: Digits are repr(u8).
        let digit_bytes =
            unsafe { slice::from_raw_parts(digits.as_ptr() as *const u8, digits.len()) };
        self.solutions.extend_from_slice(digit_bytes);
        self.len += 1;
        self.hash ^= id;
    }

    pub fn append_from(&mut self, other: SolutionRef) {
        assert_eq!(other.0.len(), self.solution_len());
        self.solutions.extend_from_slice(other.0);
        self.len += 1;
        self.hash ^= other.id();
    }

    // Generate solutions.
    // `ResourcesExceeded::Memory` if more than `max` solutions.
    // `ResourcesExceeded::Time` if deadline exceeded and at least `min` solutions.
    pub fn generate(
        board: &Board,
        min: u32,
        max: u32,
        deadline: Instant,
        rng: &mut RandomGenerator,
    ) -> (Result<(), ResourcesExceeded>, Self) {
        let original_squares = Small::all().collect();
        let mut table = Self::with_capacity(original_squares, max);
        let mut solver = FastSolver::new(board);
        let mut since_last_time_check: u64 = 0;
        let mut num_solutions = 0;
        loop {
            match solver.step() {
                SolverStep::Found(filled_board) => {
                    if num_solutions >= max {
                        return (Err(ResourcesExceeded::Memory), table);
                    }
                    let id = rng.random_bits_64();
                    table.append(id, &filled_board.squares);
                    num_solutions += 1;
                }
                SolverStep::NoProgress => {}
                SolverStep::Done => {
                    return (Ok(()), table);
                }
            }

            since_last_time_check += 1;
            if since_last_time_check >= settings::SOLUTION_GENERATE_CHECK_TIME_ITERS
                && num_solutions >= min
            {
                since_last_time_check = 0;
                if Instant::now() >= deadline {
                    return (Err(ResourcesExceeded::Time), table);
                }
            }
        }
    }

    pub fn filter(&self, capacity: u32, mov: Move) -> Self {
        let mut table = Self::with_capacity(self.original_squares.clone(), capacity);
        for solution in self.iter() {
            let digits = solution.digits();
            let digit = *unsafe { digits.get_unchecked(usize::from(mov.square)) };
            if digit == mov.digit {
                table.append_from(solution);
            }
        }
        table
    }

    pub fn move_tables(&self) -> Vec<SquareMoveTable> {
        let mut move_tables = vec![SquareMoveTable::default(); usize::from(self.num_squares())];

        for solution in self.iter() {
            let id = solution.id();
            let digits = solution.digits();
            for (&digit, move_table) in digits.iter().zip(move_tables.iter_mut()) {
                move_table.num_solutions[digit] += 1;
                move_table.hash[digit] ^= id;
            }
        }
        move_tables
    }

    pub fn compress_and_gen_moves(
        &self,
        move_tables: &[SquareMoveTable],
    ) -> (Self, Vec<EndgameMove>) {
        assert_eq!(move_tables.len(), usize::from(self.num_squares()));

        let total_solutions = self.len();
        let mut prev_squares = Vec::with_capacity(self.num_squares().into());
        let mut compressed_original_squares = Vec::with_capacity(self.num_squares().into());
        let mut moves = Vec::with_capacity(usize::from(self.num_squares()) * 9);

        for ((&original_square, move_table), prev_square) in self
            .original_squares
            .iter()
            .zip(move_tables.iter())
            .zip(0..)
        {
            let prev_square = unsafe { Small::<81>::new_unchecked(prev_square) };
            let prev_num_moves = moves.len();

            for ((digit, &num_solutions), &hash) in Digit::all()
                .zip(move_table.num_solutions.iter())
                .zip(move_table.hash.iter())
            {
                if num_solutions != 0 && num_solutions != total_solutions {
                    moves.push(EndgameMove {
                        mov: Move {
                            square: unsafe { Small::new_unchecked(prev_squares.len() as u8) },
                            digit,
                        },
                        num_solutions,
                        hash,
                    });
                }
            }
            if moves.len() != prev_num_moves {
                prev_squares.push(prev_square);
                compressed_original_squares.push(original_square);
            }
        }

        let mut compressed_table = Self::with_capacity(compressed_original_squares, self.len());
        let mut compressed_digits = vec![Digit::from(Small::new(0)); prev_squares.len()];
        for solution in self.iter() {
            let prev_digits = solution.digits();
            for (compressed_digit, &prev_square) in
                compressed_digits.iter_mut().zip(prev_squares.iter())
            {
                // Safety: prev_index is valid.
                *compressed_digit = *unsafe { prev_digits.get_unchecked(usize::from(prev_square)) };
            }
            compressed_table.append(solution.id(), &compressed_digits);
        }

        assert_eq!(compressed_table.len(), self.len());
        assert_eq!(compressed_table.hash(), self.hash());

        (compressed_table, moves)
    }

    fn solution_len(&self) -> usize {
        Self::ID_BYTES + usize::from(self.num_squares())
    }

    pub fn original_move(&self, mov: Move) -> Move {
        Move {
            square: self.original_squares[usize::from(mov.square)],
            digit: mov.digit,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SolutionRef<'a>(&'a [u8]);

impl<'a> SolutionRef<'a> {
    pub fn id(self) -> u64 {
        let res = unsafe { self.0.get_unchecked(..SolutionTable::ID_BYTES) };
        u64::from_le_bytes(res.try_into().unwrap())
    }

    pub fn digits(self) -> &'a [Digit] {
        let bytes = unsafe { self.0.get_unchecked(SolutionTable::ID_BYTES..) };
        // Safety: The solution is always valid digits, Digits is repr(u8).
        unsafe { slice::from_raw_parts::<'a, Digit>(bytes.as_ptr() as *const _, bytes.len()) }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SquareMoveTable {
    pub num_solutions: [u32; 9],
    pub hash: [u64; 9],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EndgameMove {
    pub mov: Move,
    pub num_solutions: u32,
    pub hash: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EndgameMoveNoHash {
    pub mov: Move,
    pub num_solutions: u32,
}
