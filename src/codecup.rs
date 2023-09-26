use crate::{log, platform::platform_description};

pub fn run_codecup_interaction() {
    log::write_line!(Info, "platform: {}", platform_description());
}
