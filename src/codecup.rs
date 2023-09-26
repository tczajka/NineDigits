use crate::{log, platform::platform_description, random::RandomGenerator};

pub fn run_codecup_interaction() {
    log::write_line!(Info, "platform: {}", platform_description());
    let mut _rng = RandomGenerator::with_time_nonce();
}
