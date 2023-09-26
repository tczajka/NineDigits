use std::{
    fmt,
    io::{BufWriter, Stderr, Write},
    sync::Mutex,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Level {
    Verbose,
    Info,
    Always,
}

#[derive(Debug)]
struct Logger {
    level: Level,
    writer: BufWriter<Stderr>,
}

static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

pub fn init(level: Level) {
    let writer = BufWriter::new(std::io::stderr());
    let logger = Logger { level, writer };
    *(LOGGER.lock().unwrap()) = Some(logger);
}

pub fn write_line_to_log(level: Level, message: fmt::Arguments) {
    let mut guard = LOGGER.lock().unwrap();
    let Some(logger) = &mut *guard else {
        return;
    };
    if level < logger.level {
        return;
    }
    writeln!(logger.writer, "{message}").unwrap();
    logger.writer.flush().unwrap();
}

macro_rules! write_line {
    ($level:ident, $($arg:tt)*) => {
        $crate::log::write_line_to_log($crate::log::Level::$level, format_args!($($arg)*))
    };
}

pub(crate) use write_line;
