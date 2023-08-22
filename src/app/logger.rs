use std::sync::{Arc, Mutex};

use log::{LevelFilter, set_max_level};

use crate::Result;

pub trait Writeable {
    fn write_line(&mut self, message: &str);
    fn flush(&mut self);
}

pub struct LetterLogger<W: Writeable + Send + 'static> {
    level: LevelFilter,
    writeable: Arc<Mutex<W>>
}

impl <W: Writeable + Send + 'static> LetterLogger<W> {
    pub fn init(log_level: LevelFilter, writeable: Arc<Mutex<W>>) -> Result<()> {
        set_max_level(log_level);
        log::set_boxed_logger(LetterLogger::new(log_level, writeable))?;

        Ok(())
    }

    pub fn new(log_level: LevelFilter, writeable: Arc<Mutex<W>>) -> Box<LetterLogger<W>> {
        Box::new(LetterLogger { level: log_level, writeable })
    }
}

impl <W: Writeable + Send + 'static> log::Log for LetterLogger<W> {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let target = if record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };

        let mut write_lock = self.writeable.lock().unwrap();

        write_lock.write_line(format!("{:<5}: [{}] {}", record.level(), target, record.args()).as_str());
    }

    fn flush(&self) {
        let _ = self.writeable.lock().unwrap().flush();
    }
}
