use tracing::Level;
use tracing_appender::non_blocking;
use tracing_subscriber::{self, fmt::writer::MakeWriterExt};

pub fn init() {
    let log_file = tracing_appender::rolling::never(".", "kasuri.log");
    let (non_blocking_log_file, _guard) = non_blocking(log_file);
    let log_file_writer = non_blocking_log_file;
    let writer = log_file_writer.and(std::io::stdout);

    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_target(false)
        .with_thread_names(true)
        .with_max_level(Level::TRACE)
        .with_writer(writer)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::trace!("Logging initialized");
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
