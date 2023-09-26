use super::chacha::chacha20_block;
use std::time::SystemTime;

#[derive(Debug)]
pub struct RandomGenerator {
    chacha20_nonce: u64,
    chacha20_counter: u64,
    block: [u32; 16],
    block_index: usize,
    bits: u32,
    num_bits: u32,
    number: u32,
    number_range: u32,
}

static CHACHA_KEY: [u32; 8] = [
    0x061b0fa0, 0x31f6e1d8, 0x1976a540, 0x1393b186, 0xeabd9a35, 0x84e55cc7, 0xd6efbcb5, 0x583b337b,
];

impl RandomGenerator {
    pub fn with_nonce(nonce: u64) -> Self {
        Self {
            chacha20_nonce: nonce,
            chacha20_counter: 0,
            block: [0; 16],
            block_index: 16,
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
        Self::with_nonce(nonce)
    }

    /// Random number in 0..n.
    pub fn uniform(&mut self, n: u32) -> u32 {
        assert_ne!(n, 0);

        loop {
            // Refill number_range to >= 2^31.
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

    fn random_bits(&mut self, num_bits: u32) -> u32 {
        assert!(num_bits < 32);
        if num_bits <= self.num_bits {
            let bits = self.bits & ((1 << num_bits) - 1);
            self.bits >>= num_bits;
            self.num_bits -= num_bits;
            assert!(self.num_bits == 32 || self.bits < (1 << self.num_bits));
            bits
        } else {
            let more_bits = num_bits - self.num_bits;
            let new_bits = self.random_u32();
            let bits = self.bits | (new_bits & ((1 << more_bits) - 1)) << self.num_bits;
            self.bits = new_bits >> more_bits;
            self.num_bits = 32 - more_bits;
            assert!(self.num_bits == 32 || self.bits < (1 << self.num_bits));
            bits
        }
    }

    fn random_u32(&mut self) -> u32 {
        if self.block_index == 16 {
            self.block = chacha20_block(&CHACHA_KEY, self.chacha20_nonce, self.chacha20_counter);
            self.chacha20_counter += 1;
            self.block_index = 0;
        }
        let res = self.block[self.block_index];
        self.block_index += 1;
        res
    }
}
