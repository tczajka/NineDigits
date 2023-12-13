use crate::{
    board::{Board, Move},
    digit::{Digit, OptionalDigit},
    error::ResourcesExceeded,
    fast_solver::FastSolver,
    random::RandomGenerator,
    settings,
    small::Small,
    solver::{Solver, SolverStep},
};
use std::{slice, time::Instant};

pub struct SolutionTable {
    square_infos: Vec<SquareInfo>,
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
            square_infos: Vec::new(),
            solutions: Vec::new(),
            len: 0,
            hash: 0,
        }
    }

    fn with_capacity(square_infos: Vec<SquareInfo>, max_solutions: u32) -> Self {
        let solution_len = Self::ID_BYTES + square_infos.len();
        Self {
            square_infos,
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
        self.square_infos.len() as u8
    }

    pub fn num_moves(&self, square: u8) -> u8 {
        self.square_infos[usize::from(square)].num_moves
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
        let mut original_digits = [OptionalDigit::NONE; 9];
        for digit in Digit::all() {
            original_digits[digit] = digit.into();
        }
        let square_infos = Small::all()
            .map(|sq| SquareInfo {
                original_square: sq,
                num_moves: 9,
                original_digits,
            })
            .collect();
        let mut table = Self::with_capacity(square_infos, max);
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
        let mut table = Self::with_capacity(self.square_infos.clone(), capacity);
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

    pub fn compress(&self, move_tables: &[SquareMoveTable]) -> (Self, Vec<SquareCompression>) {
        assert_eq!(move_tables.len(), usize::from(self.num_squares()));

        let total_solutions = self.len();
        let mut square_compressions = Vec::with_capacity(self.num_squares().into());
        let mut compressed_square_infos = Vec::with_capacity(self.num_squares().into());

        for ((square_info, move_table), square) in
            self.square_infos.iter().zip(move_tables.iter()).zip(0..)
        {
            let mut square_compression = SquareCompression {
                prev_square: unsafe { Small::new_unchecked(square) },
                digit_map: [OptionalDigit::NONE; 9],
                num_solutions: [0; 9],
                hash: [0; 9],
            };
            let mut new_square_info = SquareInfo {
                original_square: square_info.original_square,
                num_moves: 0,
                original_digits: [OptionalDigit::NONE; 9],
            };

            for ((digit, &num_solutions), &hash) in (0..square_info.num_moves)
                .zip(move_table.num_solutions.iter())
                .zip(move_table.hash.iter())
            {
                let digit = Digit::from(unsafe { Small::new_unchecked(digit) });
                if num_solutions != 0 && num_solutions != total_solutions {
                    let new_digit =
                        Digit::from(unsafe { Small::new_unchecked(new_square_info.num_moves) });
                    square_compression.digit_map[digit] = OptionalDigit::from(new_digit);
                    square_compression.num_solutions[new_digit] = num_solutions;
                    square_compression.hash[new_digit] = hash;
                    new_square_info.original_digits[new_digit] = digit.into();
                    new_square_info.num_moves += 1;
                }
            }
            if new_square_info.num_moves != 0 {
                square_compressions.push(square_compression);
                compressed_square_infos.push(new_square_info);
            }
        }

        let mut compressed_table = Self::with_capacity(compressed_square_infos, self.len());
        let mut compressed_digits = vec![Digit::from(Small::new(0)); square_compressions.len()];
        for solution in self.iter() {
            let prev_digits = solution.digits();
            for (compressed_digit, square_compression) in
                compressed_digits.iter_mut().zip(square_compressions.iter())
            {
                // Safety: prev_index is valid.
                let prev_digit = *unsafe {
                    prev_digits.get_unchecked(usize::from(square_compression.prev_square))
                };
                let opt_digit = square_compression.digit_map[prev_digit].to_digit();
                // Safety: digit_map contains prev_digit.
                *compressed_digit = unsafe { opt_digit.unwrap_unchecked() };
            }
            compressed_table.append(solution.id(), &compressed_digits);
        }

        (compressed_table, square_compressions)
    }

    fn solution_len(&self) -> usize {
        Self::ID_BYTES + usize::from(self.num_squares())
    }

    pub fn generate_moves(&self, square_compressions: &[SquareCompression]) -> Vec<EndgameMove> {
        assert_eq!(square_compressions.len(), usize::from(self.num_squares()));
        let mut moves = Vec::with_capacity(
            self.square_infos
                .iter()
                .map(|&square_info| square_info.num_moves as usize)
                .sum(),
        );

        for ((square_info, square_compression), square) in self
            .square_infos
            .iter()
            .zip(square_compressions.iter())
            .zip(0..)
        {
            for ((digit, &num_solutions), &hash) in (0..square_info.num_moves)
                .zip(square_compression.num_solutions.iter())
                .zip(square_compression.hash.iter())
            {
                moves.push(EndgameMove {
                    mov: Move {
                        square: unsafe { Small::new_unchecked(square) },
                        // Safety: `digit < 9` because `num_moves_sq <= 9`.
                        digit: unsafe { Small::new_unchecked(digit) }.into(),
                    },
                    num_solutions,
                    hash,
                });
            }
        }
        moves
    }

    pub fn original_move(&self, mov: Move) -> Move {
        let square_info = &self.square_infos[usize::from(mov.square)];
        let digit = square_info.original_digits[mov.digit];
        Move {
            square: square_info.original_square,
            digit: digit.to_digit().unwrap(),
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

#[derive(Copy, Clone, Debug)]
pub struct SquareCompression {
    pub prev_square: Small<81>,
    // prev -> current
    pub digit_map: [OptionalDigit; 9],
    // current num_solutions
    pub num_solutions: [u32; 9],
    // current hashes
    pub hash: [u64; 9],
}

#[derive(Copy, Clone, Debug)]
pub struct EndgameMove {
    pub mov: Move,
    pub num_solutions: u32,
    pub hash: u64,
}

#[derive(Copy, Clone, Debug)]
struct SquareInfo {
    original_square: Small<81>,
    num_moves: u8,
    original_digits: [OptionalDigit; 9],
}
