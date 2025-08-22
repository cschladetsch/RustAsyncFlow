use tracing::{debug, error, info, trace, warn};

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

#[derive(Debug, Clone)]
pub struct Logger {
    pub verbosity: u8,
    pub prefix: String,
}

impl Logger {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            verbosity: 3,
            prefix: prefix.into(),
        }
    }

    pub fn error(&self, message: impl AsRef<str>) {
        error!("[{}] {}", self.prefix, message.as_ref());
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        warn!("[{}] {}", self.prefix, message.as_ref());
    }

    pub fn info(&self, message: impl AsRef<str>) {
        info!("[{}] {}", self.prefix, message.as_ref());
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        debug!("[{}] {}", self.prefix, message.as_ref());
    }

    pub fn trace(&self, message: impl AsRef<str>) {
        trace!("[{}] {}", self.prefix, message.as_ref());
    }

    pub fn verbose(&self, level: u8, message: impl AsRef<str>) {
        if level <= self.verbosity {
            match level {
                0 => self.error(message),
                1 => self.warn(message),
                2 => self.info(message),
                3 => self.debug(message),
                _ => self.trace(message),
            }
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new("AsyncFlow")
    }
}