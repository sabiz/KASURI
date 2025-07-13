pub mod command;
pub mod event_handler;

/// Window ID
pub const WINDOW_ID: &str = "main";

/// Event name for window show action
///
/// This event is emitted when the main window is shown,
/// allowing the frontend to respond appropriately.
pub const EVENT_WINDOW_SHOW: &str = "window-show";

/// Menu item IDs
pub const MENU_ID_EXIT: &str = "exit";
pub const MENU_ID_RELOAD: &str = "reload";
pub const MENU_ID_OPEN_LOG_DIR: &str = "open_log_dir";

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
