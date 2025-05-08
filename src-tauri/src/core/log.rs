use std::sync::{Mutex, OnceLock};
use tracing::Level;
use tracing_appender::non_blocking;
use tracing_subscriber::{
    self, Layer, filter::LevelFilter, layer::SubscriberExt, registry::Registry, reload,
};

use super::kasuri::KasuriResult;

type ReloadHandle = reload::Handle<LevelFilter, Registry>;

static FILE_LOG_RELOADER: OnceLock<Mutex<ReloadHandle>> = OnceLock::new();
static STDOUT_LOG_RELOADER: OnceLock<Mutex<ReloadHandle>> = OnceLock::new();

pub fn init() {
    let log_file = tracing_appender::rolling::never(".", "kasuri.log");
    let (log_file_writer, _guard) = non_blocking(log_file);

    let (file_level_filter, file_reload_handle) = reload::Layer::new(LevelFilter::DEBUG);
    let (stdout_level_filter, stdout_reload_handle) = reload::Layer::new(LevelFilter::INFO);

    let _ = FILE_LOG_RELOADER.get_or_init(|| Mutex::new(file_reload_handle));
    let _ = STDOUT_LOG_RELOADER.get_or_init(|| Mutex::new(stdout_reload_handle));

    let file_fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(false)
        .with_thread_names(true)
        .with_writer(log_file_writer)
        .with_filter(file_level_filter);

    let stdout_fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_thread_names(true)
        .with_writer(std::io::stdout)
        .with_filter(stdout_level_filter);

    let combined_layer = file_fmt_layer.and_then(stdout_fmt_layer);

    let subscriber = tracing_subscriber::registry().with(combined_layer);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::trace!("Logging initialized with separate file and stdout filters");
}

/// Function to dynamically change the file output log level
///
/// # Arguments
///
/// * `level` - The new log level
///
/// # Examples
///
/// ```
/// use tracing::Level;
/// set_file_log_level(Level::DEBUG);
/// ```
pub fn set_file_log_level(level: &String) -> KasuriResult<()> {
    match FILE_LOG_RELOADER.get() {
        Some(mutex) => match mutex.lock() {
            Ok(handle) => handle
                .modify(|filter| {
                    *filter = LevelFilter::from_level(log_level_string_to_level(level));
                })
                .map_err(|e| format!("Failed to modify file log level: {}", e).into()),
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire lock on file logger.".to_string(),
            ))),
        },
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "File logger has not been initialized. Call init() first.".to_string(),
        ))),
    }
}

/// Function to dynamically change the standard output log level
///
/// # Arguments
///
/// * `level` - The new log level
///
/// # Examples
///
/// ```
/// use tracing::Level;
/// set_stdout_log_level(Level::DEBUG);
/// ```
pub fn set_stdout_log_level(level: &String) -> KasuriResult<()> {
    match STDOUT_LOG_RELOADER.get() {
        Some(mutex) => match mutex.lock() {
            Ok(handle) => handle
                .modify(|filter| {
                    *filter = LevelFilter::from_level(log_level_string_to_level(level));
                })
                .map_err(|e| format!("Failed to modify stdout log level: {}", e).into()),
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire lock on stdout logger.".to_string(),
            ))),
        },
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Stdout logger has not been initialized. Call init() first.".to_string(),
        ))),
    }
}

fn log_level_string_to_level(level: &String) -> Level {
    match level.to_lowercase().as_str() {
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        _ => Level::INFO, // Default to INFO if the level is not recognized
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}
