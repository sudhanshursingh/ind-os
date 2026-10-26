//! Leveled Logging Framework & Boot-Stage Marker System for IndK
//! Implements SP-05 requirements for leveled logging and deterministic boot markers.

use core::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info  = 2,
    Warn  = 3,
    Error = 4,
}

static CURRENT_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Debug as u8);

/// Set the runtime maximum log level
#[allow(dead_code)]
pub fn set_log_level(level: LogLevel) {
    CURRENT_LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Retrieve the active runtime log level
#[allow(dead_code)]
pub fn get_log_level() -> LogLevel {
    match CURRENT_LOG_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Trace,
        1 => LogLevel::Debug,
        2 => LogLevel::Info,
        3 => LogLevel::Warn,
        _ => LogLevel::Error,
    }
}

/// Emit a boot-stage transition marker per Bible §8.7 standard
pub fn emit_stage(stage_id: &str, description: &str) {
    crate::println!("[STAGE:{}] {}", stage_id, description);
}

/// Dispatcher for leveled logging
#[doc(hidden)]
pub fn _log(level: LogLevel, module: &str, args: core::fmt::Arguments) {
    if (level as u8) >= CURRENT_LOG_LEVEL.load(Ordering::Relaxed) {
        let tag = match level {
            LogLevel::Trace => "[TRACE]",
            LogLevel::Debug => "[DEBUG]",
            LogLevel::Info  => "[INFO ]",
            LogLevel::Warn  => "[WARN ]",
            LogLevel::Error => "[ERROR]",
        };
        crate::println!("{} {}: {}", tag, module, args);
    }
}

/// Emit a boot stage transition marker
#[macro_export]
macro_rules! stage_marker {
    ($id:expr, $desc:expr) => {
        $crate::logger::emit_stage($id, $desc);
    };
}

/// Log a message at TRACE level
#[macro_export]
macro_rules! log_trace {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Trace, $module, format_args!($($arg)*));
    };
}

/// Log a message at DEBUG level
#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Debug, $module, format_args!($($arg)*));
    };
}

/// Log a message at INFO level
#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Info, $module, format_args!($($arg)*));
    };
}

/// Log a message at WARN level
#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Warn, $module, format_args!($($arg)*));
    };
}

/// Log a message at ERROR level
#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Error, $module, format_args!($($arg)*));
    };
}
