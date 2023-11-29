use crate::{
    board::Board,
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
    /// Each square can only contain digits 0..square_moves[sq].
    num_moves_per_square: Vec<u8>,
    /// Each solution is: ID_BYTES + square_moves.len().
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
            num_moves_per_square: Vec::new(),
            solutions: Vec::new(),
            len: 0,
            hash: 0,
        }
    }

    pub fn with_capacity(num_moves_per_square: Vec<u8>, max_solutions: u32) -> Self {
        let solution_len = Self::ID_BYTES + num_moves_per_square.len();
        Self {
            num_moves_per_square,
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

    pub fn num_moves_per_square(&self) -> &[u8] {
        &self.num_moves_per_square
    }

    pub fn iter(&self) -> impl Iterator<Item = SolutionRef<'_>> {
        let slen = self.solution_len();
        self.solutions.chunks_exact(slen).map(SolutionRef)
    }

    pub fn append(&mut self, id: u64, digits: &[Digit]) {
        assert_eq!(digits.len(), self.num_moves_per_square.len());
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
        let mut table = Self::with_capacity(vec![9; 81], max);
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

    pub fn filter(&self, capacity: u32, square: usize, digit: Digit) -> Self {
        let mut table = Self::with_capacity(self.num_moves_per_square.clone(), capacity);
        for solution in self.iter() {
            if solution.digits()[square] == digit {
                table.append_from(solution);
            }
        }
        table
    }

    pub fn move_summaries(&self) -> Vec<[MoveSummary; 9]> {
        let mut summaries = vec![[MoveSummary::default(); 9]; self.num_moves_per_square.len()];

        for solution in self.iter() {
            let id = solution.id();
            let digits = solution.digits();
            for (square, &digit) in digits.iter().enumerate() {
                let summary = &mut summaries[square][digit];
                summary.num_solutions += 1;
                summary.hash ^= id;
            }
        }
        summaries
    }

    pub fn compress(&self, move_summaries: &[[MoveSummary; 9]]) -> (Self, Vec<SquareCompression>) {
        assert_eq!(self.num_moves_per_square.len(), move_summaries.len());

        let mut square_compressions = Vec::with_capacity(self.num_moves_per_square.len());
        let mut compressed_num_moves_per_square =
            Vec::with_capacity(self.num_moves_per_square.len());

        for (square_index, (&num_moves_sq, move_summaries_sq)) in self
            .num_moves_per_square
            .iter()
            .zip(move_summaries.iter())
            .enumerate()
        {
            let mut compressed_num_moves = 0;
            let mut square_compression = SquareCompression {
                prev_index: square_index,
                digit_map: [OptionalDigit::NONE; 9],
                move_summaries: [MoveSummary::default(); 9],
            };

            for (digit, summary) in (0..num_moves_sq).zip(move_summaries_sq.iter()) {
                let digit = Digit::from(unsafe { Small::new_unchecked(digit) });
                if summary.num_solutions != 0 {
                    let new_digit =
                        Digit::from(unsafe { Small::new_unchecked(compressed_num_moves) });
                    square_compression.digit_map[digit] = OptionalDigit::from(new_digit);
                    square_compression.move_summaries[new_digit] = *summary;
                    compressed_num_moves += 1;
                }
            }
            if compressed_num_moves >= 2 {
                square_compressions.push(square_compression);
                compressed_num_moves_per_square.push(compressed_num_moves);
            }
        }

        let mut compressed_table = Self::with_capacity(compressed_num_moves_per_square, self.len());
        let mut compressed_digits = vec![Digit::from(Small::new(0)); square_compressions.len()];
        for solution in self.iter() {
            let prev_digits = solution.digits();
            for (compressed_digit, square_compression) in
                compressed_digits.iter_mut().zip(square_compressions.iter())
            {
                let prev_digit = prev_digits[square_compression.prev_index];
                let opt_digit = square_compression.digit_map[prev_digit].to_digit();
                // Safety: digit_map contains prev_digit.
                *compressed_digit = unsafe { opt_digit.unwrap_unchecked() };
            }
            compressed_table.append(solution.id(), &compressed_digits);
        }

        (compressed_table, square_compressions)
    }

    fn solution_len(&self) -> usize {
        Self::ID_BYTES + self.num_moves_per_square.len()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SolutionRef<'a>(&'a [u8]);

impl<'a> SolutionRef<'a> {
    pub fn id(self) -> u64 {
        u64::from_le_bytes(self.0[..SolutionTable::ID_BYTES].try_into().unwrap())
    }

    pub fn digits(self) -> &'a [Digit] {
        let bytes = &self.0[SolutionTable::ID_BYTES..];
        // Safety: The solution is always valid digits, Digits is repr(u8).
        unsafe { slice::from_raw_parts::<'a, Digit>(bytes.as_ptr() as *const _, bytes.len()) }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MoveSummary {
    pub num_solutions: u32,
    pub hash: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct SquareCompression {
    pub prev_index: usize,
    // prev -> current
    pub digit_map: [OptionalDigit; 9],
    // current move summaries
    pub move_summaries: [MoveSummary; 9],
}
