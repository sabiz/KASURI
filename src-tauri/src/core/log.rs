use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use std::sync::{LazyLock, Mutex};

static INSTANCE: LazyLock<Mutex<Logger>> = LazyLock::new(|| {
    Mutex::new(Logger {
        level: log::LevelFilter::Info,
    })
});

/// Initializes the logger for the KASURI application.
/// This function sets up a global logger that writes logs to both the console and a rolling file.
/// The log files are stored in a `logs` directory next to the executable.
/// The logger supports log rotation, keeping up to 5 old log files, each with a maximum size of 10 MB.
/// The log messages are formatted with a timestamp, log level, and message content.
/// The log level can be dynamically changed at runtime.
/// # Panics
/// Panics if the log directory cannot be created or the rolling file appender cannot be initialized.
/// Panics if logger initialization fails.
pub fn init_logger() -> () {
    let top_dispatch = fern::Dispatch::new();
    let console_dispatch = fern::Dispatch::new().chain(std::io::stdout());

    let log_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    }
    let log_file = BasicRollingFileAppender::new(
        log_dir.join("KASURI.log"),
        RollingConditionBasic::new().max_size(
            10 * 1024 * 1024, // 10 MB
        ),
        5, // Keep 5 old log files
    )
    .expect("Failed to create rolling file appender");

    let file_dispatch =
        fern::Dispatch::new().chain(Box::new(log_file) as Box<dyn std::io::Write + Send>);

    top_dispatch
        .filter(|metadata| metadata.level() <= INSTANCE.lock().unwrap().level)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f%::z"),
                record.level(),
                message
            ))
        })
        .chain(console_dispatch)
        .chain(file_dispatch)
        .apply()
        .expect("Failed to initialize logger");
}

struct Logger {
    level: log::LevelFilter,
}

/// Sets the log level from a string representation.
/// This function allows you to change the log level dynamically at runtime.
/// # Arguments
/// * `level`: A string representing the log level. Valid values are "error", "warn", "info", "debug".
/// # If an invalid value is provided, it defaults to "info".
pub fn set_log_level_str(level: &str) {
    let level = match level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        _ => log::LevelFilter::Info, // Default to Info if invalid
    };
    set_log_level(level);
}

/// Sets the log level from a `log::LevelFilter`.
/// This function allows you to change the log level dynamically at runtime.
/// # Arguments
/// * `level`: The desired log level as a `log::LevelFilter`.
pub fn set_log_level(level: log::LevelFilter) {
    let mut logger = INSTANCE.lock().unwrap();
    logger.level = level;
}
