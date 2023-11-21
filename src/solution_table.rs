use crate::{
    board::Board,
    digit::Digit,
    error::ResourcesExceeded,
    fast_solver::FastSolver,
    random::RandomGenerator,
    solver::{Solver, SolverStep},
};
use std::{slice, time::Instant};

pub struct SolutionTable {
    /// Each square can only contain digits 0..square_moves[sq].
    square_moves: Vec<u8>,
    /// Each solution is: ID_BYTES + square_moves.len().
    solutions: Vec<u8>,
}

impl SolutionTable {
    const ID_BYTES: usize = 8;

    pub fn empty() -> Self {
        Self {
            square_moves: Vec::new(),
            solutions: Vec::new(),
        }
    }

    pub fn with_capacity(square_moves: Vec<u8>, max_solutions: usize) -> Self {
        let solution_len = Self::ID_BYTES + square_moves.len();
        Self {
            square_moves,
            solutions: Vec::with_capacity(max_solutions * solution_len),
        }
    }

    pub fn append(&mut self, id: u64, digits: &[Digit]) {
        assert_eq!(digits.len(), self.square_moves.len());
        self.solutions.extend_from_slice(&id.to_le_bytes());
        // Safety: Digits are repr(u8).
        let digit_bytes =
            unsafe { slice::from_raw_parts(digits.as_ptr() as *const u8, digits.len()) };
        self.solutions.extend_from_slice(digit_bytes);
    }

    pub fn append_from(&mut self, other: SolutionRef) {
        assert_eq!(other.0.len(), self.solution_len());
        self.solutions.extend_from_slice(other.0);
    }

    pub fn is_empty(&self) -> bool {
        self.solutions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.solutions.len() / self.solution_len()
    }

    pub fn iter(&self) -> impl Iterator<Item = SolutionRef<'_>> {
        let len = self.solution_len();
        self.solutions.chunks_exact(len).map(SolutionRef)
    }

    pub fn generate(
        board: &Board,
        limit: usize,
        deadline: Instant,
        rng: &mut RandomGenerator,
    ) -> (Result<(), ResourcesExceeded>, Self) {
        const CHECK_TIME_ITERS: u64 = 1024;

        let mut table = Self {
            square_moves: vec![9; 81],
            solutions: Vec::with_capacity(limit * (Self::ID_BYTES + 81)),
        };

        let mut solver = FastSolver::new(board);
        let mut since_last_time_check: u64 = 0;
        let mut num_solutions = 0;
        loop {
            match solver.step() {
                SolverStep::Found(filled_board) => {
                    if num_solutions >= limit {
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
            if since_last_time_check >= CHECK_TIME_ITERS {
                since_last_time_check = 0;
                if Instant::now() >= deadline {
                    return (Err(ResourcesExceeded::Time), table);
                }
            }
        }
    }

    pub fn filter(&self, capacity: usize, square: usize, digit: Digit) -> Self {
        let mut table = Self::with_capacity(self.square_moves.clone(), capacity);
        for solution in self.iter() {
            if solution.digits()[square] == digit {
                table.append_from(solution);
            }
        }
        table
    }

    fn solution_len(&self) -> usize {
        Self::ID_BYTES + self.square_moves.len()
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
