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
        .apply()
        .expect("Failed to initialize logger");
}

pub struct Logger {
    pub level: log::LevelFilter,
}
