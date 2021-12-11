use std::fmt::Debug;

lazy_static::lazy_static! {
    pub static ref LOGGER: Logger = Logger {};
}

pub struct Logger;

impl log::Log for LOGGER {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn flush(&self) {}

    fn log(&self, record: &log::Record) {
        println!(
            "[{}] [{} > {:?}] {}",
            chrono::Local::now().format("%H:%M:%S"),
            record.metadata().target(),
            record.level(),
            record.args()
        );
    }
}
