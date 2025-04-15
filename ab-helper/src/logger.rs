use std::sync::OnceLock;

use log::{
	Level,
	LevelFilter,
	Log,
	Metadata,
	Record,
};

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Debug)]
struct Logger {
	level: Option<Level>,
}

impl Log for Logger {
	fn enabled(&self, md: &Metadata) -> bool {
		self.level.is_some_and(|l| md.level() <= l)
	}

	fn log(&self, r: &Record) {
		if self.enabled(r.metadata())
			&& r.module_path().is_some_and(|m| {
				m == env!("CARGO_CRATE_NAME")
					|| m.starts_with(concat!(env!("CARGO_CRATE_NAME"), "::"))
			}) {
			match r.level() {
				Level::Error => eprintln!("e: {}", r.args()),
				Level::Warn => eprintln!("w: {}", r.args()),
				_ => eprintln!("{}", r.args()),
			}
		}
	}

	fn flush(&self) {}
}

pub fn init(level: Option<Level>) {
	LOGGER.set(Logger { level }).unwrap();
	log::set_logger(LOGGER.get().unwrap()).unwrap();
	log::set_max_level(LevelFilter::Trace);
}
