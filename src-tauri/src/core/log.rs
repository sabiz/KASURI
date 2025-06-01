use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use std::sync::{LazyLock, Mutex};

static INSTANCE: LazyLock<Mutex<Logger>> = LazyLock::new(|| {
    Mutex::new(Logger {
        level: log::LevelFilter::Info,
    })
});

pub fn init_logger() -> () {
    let logger = INSTANCE.lock().unwrap();
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
        .level(logger.level)
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

pub struct Logger {
    pub level: log::LevelFilter,
}
