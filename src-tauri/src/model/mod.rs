pub mod application;

/// Simplified application data structure used for passing to the UI layer.
///
/// This structure contains only the fields needed for displaying
/// and selecting applications in the user interface.
#[derive(serde::Serialize)]
pub struct AppForView {
    /// Display name of the application
    pub name: String,
    /// Unique identifier for the application
    pub app_id: String,
    /// Path to the application icon
    pub icon_path: String,
}
