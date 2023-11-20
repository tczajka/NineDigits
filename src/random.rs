use crate::{chacha::chacha20_block, log};
use std::time::SystemTime;

#[derive(Debug)]
pub struct RandomGenerator {
    chacha20_nonce: u64,
    chacha20_counter: u64,
    block: [u8; 64],
    block_index: usize,
    bits: u64,
    num_bits: u32,
    number: u64,
    number_range: u64,
}

static CHACHA_KEY: [u8; 32] = [
    0x06, 0x1b, 0x0f, 0xa0, 0x31, 0xf6, 0xe1, 0xd8, 0x19, 0x76, 0xa5, 0x40, 0x13, 0x93, 0xb1, 0x86,
    0xea, 0xbd, 0x9a, 0x35, 0x84, 0xe5, 0x5c, 0xc7, 0xd6, 0xef, 0xbc, 0xb5, 0x58, 0x3b, 0x33, 0x7b,
];

impl RandomGenerator {
    pub fn with_nonce(nonce: u64) -> Self {
        Self {
            chacha20_nonce: nonce,
            chacha20_counter: 0,
            block: [0u8; 64],
            block_index: 64,
            /// Always uniformly random in 0..2^num_bits.
            bits: 0,
            num_bits: 0,
            /// Always uniformly random in 0..number_range.
            number: 0,
            number_range: 1,
        }
    }

    pub fn with_time_nonce() -> Self {
        // 2^64 nanoseconds = 584 years.
        let nonce = SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_nanos() as u64;
        log::write_line!(Info, "RandomGenerator nonce {nonce}");
        Self::with_nonce(nonce)
    }

    /// Random number in 0..n.
    pub fn uniform_u64(&mut self, n: u64) -> u64 {
        assert_ne!(n, 0);

        loop {
            // Refill number_range to >= 2^63.
            let num_bits = self.number_range.leading_zeros();
            self.number_range <<= num_bits;
            self.number = self.number << num_bits | self.random_bits(num_bits);

            // Split number_range into num_groups * n + remainder.
            let num_groups = self.number_range / n;
            let group = self.number / n;
            let in_group = self.number % n;

            if group < num_groups {
                self.number = group;
                self.number_range = num_groups;
                return in_group;
            } else {
                self.number = in_group;
                self.number_range %= n;
            }
        }
    }

    /// Random number in 0..n.
    pub fn uniform_u8(&mut self, n: u8) -> u8 {
        self.uniform_u64(u64::from(n)) as u8
    }

    /// Random number in 0..n.
    pub fn uniform_usize(&mut self, n: usize) -> usize {
        self.uniform_u64(n.try_into().unwrap()) as usize
    }

    /// Choose uniformly at random.
    pub fn choose<'a, T>(&mut self, seq: &'a [T]) -> &'a T {
        let index = self.uniform_usize(seq.len());
        &seq[index]
    }

    fn random_bits(&mut self, num_bits: u32) -> u64 {
        assert!(num_bits < 64);
        if num_bits <= self.num_bits {
            let bits = self.bits & ((1 << num_bits) - 1);
            self.bits >>= num_bits;
            self.num_bits -= num_bits;
            assert!(self.num_bits == 64 || self.bits < (1 << self.num_bits));
            bits
        } else {
            let more_bits = num_bits - self.num_bits;
            let new_bits = self.random_bits_64();
            let bits = self.bits | (new_bits & ((1 << more_bits) - 1)) << self.num_bits;
            self.bits = new_bits >> more_bits;
            self.num_bits = 64 - more_bits;
            assert!(self.num_bits == 64 || self.bits < (1 << self.num_bits));
            bits
        }
    }

    pub fn random_bits_64(&mut self) -> u64 {
        if self.block_index == 64 {
            self.block = chacha20_block(&CHACHA_KEY, self.chacha20_nonce, self.chacha20_counter);
            self.chacha20_counter += 1;
            self.block_index = 0;
        }
        let res = u64::from_le_bytes(self.block[self.block_index..][..8].try_into().unwrap());
        self.block_index += 8;
        res
    }
}
