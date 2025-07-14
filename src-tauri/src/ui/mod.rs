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
