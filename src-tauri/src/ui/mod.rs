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
pub enum MenuId {
    /// Exit application
    Exit,
    /// Reload application cache
    Reload,
    /// Open log directory
    OpenLogDir,
    /// Open settings
    Settings,
}

/// Converts MenuId to string for use in menu events
impl std::fmt::Display for MenuId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuId::Exit => write!(f, "exit"),
            MenuId::Reload => write!(f, "reload"),
            MenuId::OpenLogDir => write!(f, "open-log-dir"),
            MenuId::Settings => write!(f, "settings"),
        }
    }
}

/// Converts string to MenuId for parsing menu events
impl std::str::FromStr for MenuId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exit" => Ok(MenuId::Exit),
            "reload" => Ok(MenuId::Reload),
            "open-log-dir" => Ok(MenuId::OpenLogDir),
            "settings" => Ok(MenuId::Settings),
            _ => Err(()),
        }
    }
}
