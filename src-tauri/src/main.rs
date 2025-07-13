// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod model;
mod repositories;
mod service;
mod ui;

use crate::core::kasuri::Kasuri;
use crate::core::log::set_log_level_str;
use crate::core::settings::Settings;
use crate::ui::WINDOW_ID;
use crate::ui::command::{
    changed_content_size, close_window, launch_application, search_application,
};
use crate::ui::event_handler::{on_global_shortcut, on_menu_event, on_tray_icon_event};
use crate::ui::{MENU_ID_EXIT, MENU_ID_OPEN_LOG_DIR, MENU_ID_RELOAD};
use kasuri::KasuriResult;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::{App, LogicalSize, Manager};
use tauri_plugin_autostart::ManagerExt;

/// Tray icon ID
const TRAY_ICON_ID: &str = "main";

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
fn run() -> KasuriResult<()> {
    log::info!("Starting Kasuri application");
    let settings = Settings::load().map_err(|e| format!("Failed to load settings: {}", e))?;
    set_log_level_str(settings.get_log_level().as_str());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            search_application,
            changed_content_size,
            close_window,
            launch_application
        ])
        .setup(move |app| {
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
                if let Err(e) = app.autolaunch().enable() {
                    log::error!("Failed to enable autolaunch: {}", e);
                } else {
                    log::debug!("Autolaunch enabled successfully");
                }
            } else if app.autolaunch().is_enabled().unwrap_or(false) {
                if let Err(e) = app.autolaunch().disable() {
                    log::error!("Failed to disable autolaunch: {}", e);
                } else {
                    log::debug!("Autolaunch disabled successfully");
                }
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
    let item_exit = MenuItem::with_id(app, MENU_ID_EXIT, "Exit", true, None::<&str>)?;
    let item_reload = MenuItem::with_id(app, MENU_ID_RELOAD, "Reload", true, None::<&str>)?;
    let item_open_log_dir = MenuItem::with_id(
        app,
        MENU_ID_OPEN_LOG_DIR,
        "Open Log Directory",
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&item_reload, &item_open_log_dir, &item_exit])?;
    tray_icon_main.set_menu(Some(menu))?;
    tray_icon_main.on_menu_event(on_menu_event);
    tray_icon_main.on_tray_icon_event(on_tray_icon_event);
    Ok(())
}

/// Main function to start the Kasuri application.
fn main() {
    core::log::init_logger();
    if let Err(e) = run() {
        log::error!("Kasuri error: {}", e);
        std::process::exit(1);
    }
}
