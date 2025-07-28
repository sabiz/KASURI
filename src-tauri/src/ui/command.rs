use super::WINDOW_ID_MAIN;
use kasuri::Kasuri;
use kasuri::core::settings::Settings;
use kasuri::model::AppForView;
use std::sync::Mutex;
use tauri::{LogicalSize, Manager};

/// Tauri command for handling content size changes.
///
/// This function is called when the content size of the main window changes.
/// It updates the window size based on the new content height.
///
/// # Arguments
///
/// * `content_height` - The new content height
/// * `app_handle` - Tauri app handle for accessing the main window
/// * `app_state` - Tauri state containing the Kasuri instance
///
/// # Returns
///
/// None
#[tauri::command]
pub fn changed_content_size(
    content_height: u32,
    app_handle: tauri::AppHandle,
    app_state: tauri::State<'_, Mutex<Kasuri>>,
) {
    log::debug!("Content size changed: height={}", content_height);
    let window = app_handle
        .get_window(WINDOW_ID_MAIN)
        .expect("Failed to get main window");
    if let Err(e) = window.set_size(LogicalSize::new(
        app_state.lock().unwrap().settings.get_width(),
        content_height + 2,
    )) {
        log::error!("Failed to set window size: {}", e);
    }
}

/// Tauri command for closing the main window.
///
/// This function is called when the user wants to hide or close the main window.
/// It hides the window instead of closing it, allowing for a tray icon interaction.
///
/// # Arguments
/// * `app_handle` - Tauri app handle for accessing the main window
///
/// # Returns
///
/// None
#[tauri::command]
pub fn close_window(app_handle: tauri::AppHandle) {
    log::debug!("Closing window");
    let window = app_handle
        .get_window(WINDOW_ID_MAIN)
        .expect("Failed to get main window");
    if let Err(e) = window.hide() {
        log::error!("Failed to hide window: {}", e);
    }
}

/// Tauri command for searching applications based on user input.
///
/// This function is exposed to the frontend and allows the UI to search for applications
/// using a query string. It delegates to the Kasuri instance managed by Tauri's state.
///
/// # Arguments
///
/// * `query` - The search query string provided by the user
/// * `app_state` - Tauri state containing the Kasuri instance
///
/// # Returns
///
/// A vector of simplified application objects for display in the UI
#[tauri::command]
pub fn search_application(
    query: &str,
    app_state: tauri::State<'_, Mutex<Kasuri>>,
) -> Vec<AppForView> {
    log::debug!("Searching for application: {}", query);
    let kasuri = app_state.lock().unwrap();
    kasuri.handle_search_application(query)
}

/// Tauri command for launching an application.
///
/// This function is called when the user selects an application to launch.
/// It delegates to the Kasuri instance to handle the actual launching process.
///
/// # Arguments
///
/// * `app_id` - The unique identifier of the application to launch
/// * `app_state` - Tauri state containing the Kasuri instance
///
/// # Returns
///
/// None
#[tauri::command]
pub fn launch_application(app_id: String, app_state: tauri::State<'_, Mutex<Kasuri>>) {
    log::debug!("Launching application with ID: {}", app_id);
    let _ = app_state.lock().unwrap().handle_launch_application(&app_id);
}

/// Tauri command to retrieve the current settings of the application.
///
/// This function is exposed to the frontend and allows the UI to access
/// the current settings of the Kasuri application.
///
/// # Arguments
///
/// * `app_state` - Tauri state containing the Kasuri instance
///
/// # Returns
///
/// None
#[tauri::command]
pub fn get_settings(app_state: tauri::State<'_, Mutex<Kasuri>>) -> Settings {
    log::debug!("Retrieving settings");
    app_state.lock().unwrap().settings.clone()
}

/// Tauri command to retrieve the default settings of the application.
///
/// This function provides a way to access the default settings
/// for the Kasuri application, which can be useful for resetting or initializing settings.
/// # Arguments
/// * None
/// # Returns
/// * The default settings of the Kasuri application
#[tauri::command]
pub fn get_default_settings() -> Settings {
    log::debug!("Retrieving default settings");
    Settings::default()
}
