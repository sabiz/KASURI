use std::sync::Mutex;

use crate::core::kasuri::Kasuri;
use crate::core::kasuri::KasuriResult;
use crate::core::settings::Settings;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::HotKeyState;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconEvent;
use tauri::{App, LogicalSize, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::Shortcut;

/// Window ID
const WINDOW_ID: &str = "main";

/// Tray icon ID
const TRAY_ICON_ID: &str = "main";

/// Event name for window show action
///
/// This event is emitted when the main window is shown,
/// allowing the frontend to respond appropriately.
const EVENT_WINDOW_SHOW: &str = "window-show";

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

/// Initializes and runs the Kasuri application.
///
/// This function is the main entry point for the Kasuri application.
/// It loads settings, sets up the Tauri application with necessary plugins,
/// initializes the Kasuri controller, and launches the UI.
///
/// # Returns
///
/// Returns a `KasuriResult<()>` which is `Ok(())` if the application runs and exits normally,
/// or an error if initialization fails.
pub fn run() -> KasuriResult<()> {
    log::info!("Starting Kasuri application");
    let settings = Settings::load().map_err(|e| format!("Failed to load settings: {}", e))?;

    tauri::Builder::default()
        .plugin(get_plugin_log(&settings).build())
        .invoke_handler(tauri::generate_handler![
            search_application,
            changed_content_size,
            close_window,
            launch_application
        ])
        .setup(|app| {
            log::debug!("Setup started");
            log::debug!("Settings: {:#?}", settings);
            let _ = app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_shortcut(settings.get_shortcut_key().as_str())?
                    .with_handler(on_global_shortcut)
                    .build(),
            );
            let _ = app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec![]),
            ));
            if settings.get_auto_startup() {
                app.autolaunch().enable()?;
            } else if app.autolaunch().is_enabled().unwrap_or(false) {
                app.autolaunch().disable()?;
            }

            let mut kasuri = Kasuri::with_settings(settings)?;
            kasuri.init(app.app_handle())?;
            create_system_tray_menu(app)?;
            app.get_window(WINDOW_ID)
                .expect("Failed to get main window")
                .set_size(LogicalSize::new(*(&kasuri.settings.get_width()), 100))?;
            app.manage(Mutex::new(kasuri));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running kasuri");
    Ok(())
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
fn search_application(query: &str, app_state: tauri::State<'_, Mutex<Kasuri>>) -> Vec<AppForView> {
    log::debug!("Searching for application: {}", query);
    let kasuri = app_state.lock().unwrap();
    kasuri.handle_search_application(query)
}

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
fn changed_content_size(
    content_height: u32,
    app_handle: tauri::AppHandle,
    app_state: tauri::State<'_, Mutex<Kasuri>>,
) {
    log::debug!("Content size changed: height={}", content_height);
    let window = app_handle
        .get_window(WINDOW_ID)
        .expect("Failed to get main window");
    window
        .set_size(LogicalSize::new(
            app_state.lock().unwrap().settings.get_width(),
            content_height + 2,
        ))
        .unwrap();
}

/// Tauri command for closing the main window.
///
/// This function is called when the user wants to hide or close the main window.
/// It hides the window instead of closing it, allowing for a tray icon interaction.
///
/// # Arguments
///
/// * `app_handle` - Tauri app handle for accessing the main window
///
/// # Returns
///
/// None
#[tauri::command]
fn close_window(app_handle: tauri::AppHandle) {
    log::debug!("Closing window");
    let window = app_handle
        .get_window(WINDOW_ID)
        .expect("Failed to get main window");
    window.hide().unwrap();
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
fn launch_application(app_id: String, app_state: tauri::State<'_, Mutex<Kasuri>>) {
    log::debug!("Launching application with ID: {}", app_id);
    let _ = app_state.lock().unwrap().handle_launch_application(&app_id);
}

/// Handles global shortcut key events.
///
/// This function is called when a registered global shortcut is activated.
/// It toggles the visibility of the main application window based on the shortcut activation.
///
/// # Arguments
///
/// * `app` - The Tauri application handle
/// * `shortcut` - The shortcut that was activated
/// * `event` - The global hotkey event containing the state information
///
/// # Returns
///
/// None
fn on_global_shortcut(app: &AppHandle, shortcut: &Shortcut, event: GlobalHotKeyEvent) -> () {
    log::debug!(
        "Global shortcut triggered, key: {} state: {:?}",
        shortcut,
        event.state()
    );
    if event.state() != HotKeyState::Released {
        return;
    }
    let window = app
        .get_window(WINDOW_ID)
        .expect("Failed to get main window");
    if !window.is_visible().unwrap_or(true) {
        log::debug!("Window not visible, showing window");
        let _ = window.show();
        window.set_enabled(true).unwrap();
        window.set_focus().unwrap();
        app.emit(EVENT_WINDOW_SHOW, ()).unwrap();
    } else {
        log::debug!("Window visible, hiding window");
        let _ = window.hide();
    }
}

/// Configures and returns a Tauri log plugin builder based on application settings.
///
/// This function sets up logging for the application with appropriate levels, formatting,
/// and output targets based on the provided settings.
///
/// # Arguments
///
/// * `settings` - The application settings containing log configuration
///
/// # Returns
///
/// A configured Tauri log plugin builder
fn get_plugin_log(settings: &Settings) -> tauri_plugin_log::Builder {
    let log_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("logs");
    log::debug!("Log directory: {:?}", log_dir);
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    }

    tauri_plugin_log::Builder::new()
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Folder {
                path: log_dir,
                file_name: None,
            },
        ))
        .level(match settings.get_log_level().as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            _ => log::LevelFilter::Info,
        })
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} {}",
                tauri_plugin_log::TimezoneStrategy::UseLocal.get_now(),
                record.level(),
                message
            ))
        })
}

/// Creates and configures the system tray menu for the application.
///
/// Sets up the tray icon, menu items, and event handlers for tray interactions.
/// Currently includes an exit menu item and double-click behavior to show the main window.
///
/// # Arguments
///
/// * `app` - The Tauri app instance
///
/// # Returns
///
/// Returns a `KasuriResult<()>` indicating success or failure of the tray setup
fn create_system_tray_menu(app: &App) -> KasuriResult<()> {
    // See Tauri.toml for basic settings.
    let tray_icon_main = app.tray_by_id(TRAY_ICON_ID).unwrap();
    let item_exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let item_reload = MenuItem::with_id(app, "reload", "Reload", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&item_reload, &item_exit])?;
    tray_icon_main.set_menu(Some(menu))?;
    tray_icon_main.on_menu_event(|app, event| match event.id.as_ref() {
        "exit" => {
            log::debug!("Exit menu item clicked");
            app.exit(0);
        }
        "reload" => {
            log::debug!("Reload menu item clicked");
            app.state::<Mutex<Kasuri>>()
                .lock()
                .unwrap()
                .load_applications_to_cache(app)
                .expect("Failed to reload applications");
        }
        _ => {
            log::warn!("Unknown menu item clicked: {}", event.id.0);
        }
    });
    tray_icon_main.on_tray_icon_event(|tray_icon, event| match event {
        TrayIconEvent::DoubleClick {
            id: _,
            position: _,
            rect: _,
            button: _,
        } => {
            log::debug!("Tray icon double-clicked");
            if let Some(window) = tray_icon.app_handle().get_window(WINDOW_ID) {
                if !window.is_visible().unwrap_or(true) {
                    log::debug!("Showing window on tray icon double-click");
                    let _ = window.show();
                    window.set_focus().unwrap();
                    tray_icon.app_handle().emit(EVENT_WINDOW_SHOW, ()).unwrap();
                }
            }
        }
        _ => {}
    });
    Ok(())
}
