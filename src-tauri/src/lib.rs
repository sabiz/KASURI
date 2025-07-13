mod core;
mod model;
mod repositories;
mod service;
mod ui;
/// Represents a Result type for Kasuri operations.
///
/// This type alias simplifies error handling throughout the application by
/// wrapping any error type in a Box<dyn std::error::Error>.
pub type KasuriResult<T> = Result<T, Box<dyn std::error::Error>>;
