use super::{EVENT_WINDOW_SHOW, MenuId, WINDOW_ID_MAIN, WINDOW_ID_SETTINGS};
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::HotKeyState;
use kasuri::Kasuri;
use kasuri::core::log::get_log_directory;
use std::sync::Mutex;
use tauri::menu::MenuEvent;
use tauri::tray::TrayIcon;
use tauri::tray::TrayIconEvent;
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::Shortcut;
use tauri_plugin_opener::OpenerExt;

/// Handles global shortcut key events.
///
/// This function is called when a registered global shortcut is activated.
/// It toggles the visibility of the main application window based on the shortcut activation.
pub fn on_global_shortcut(app: &AppHandle, shortcut: &Shortcut, event: GlobalHotKeyEvent) -> () {
    log::debug!(
        "Global shortcut triggered, key: {} state: {:?}",
        shortcut,
        event.state()
    );
    if event.state() != HotKeyState::Released {
        return;
    }
    let window = app
        .get_window(WINDOW_ID_MAIN)
        .expect("Failed to get main window");
    if !window.is_visible().unwrap_or(true) {
        log::debug!("Window not visible, showing window");
        let _ = window.show();
        if let Err(e) = window.set_enabled(true) {
            log::error!("Failed to enable window: {}", e);
        }
        if let Err(e) = window.set_focus() {
            log::error!("Failed to focus window: {}", e);
        }
        if let Err(e) = app.emit(EVENT_WINDOW_SHOW, ()) {
            log::error!("Failed to emit window show event: {}", e);
        }
    } else {
        log::debug!("Window visible, hiding window");
        let _ = window.hide();
    }
}

/// Handles menu events for the application.
///
/// This function is called when a menu item is clicked.
/// It processes the menu event based on the item ID and performs the corresponding action.
/// Currently, it handles exit, reload, and open log directory actions.
pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref().parse::<MenuId>() {
        Err(_) => {
            log::warn!("Unknown menu item clicked: {}", event.id.as_ref());
            return;
        }
        Ok(menu_id) => {
            log::debug!("Menu item clicked: {}", menu_id);
            match menu_id {
                MenuId::Exit => {
                    app.exit(0);
                }
                MenuId::Reload => {
                    app.state::<Mutex<Kasuri>>()
                        .lock()
                        .unwrap()
                        .load_applications_to_cache(app)
                        .expect("Failed to reload applications");
                }
                MenuId::OpenLogDir => {
                    let log_dir = get_log_directory();
                    log::debug!("Opening log directory: {:?}", log_dir);
                    app.opener()
                        .open_path(log_dir.to_string_lossy(), None::<&str>)
                        .expect("Failed to open log directory");
                }
                MenuId::Settings => {
                    if app
                        .windows()
                        .iter()
                        .any(|(label, _)| label == WINDOW_ID_SETTINGS)
                    {
                        log::debug!("Settings window already exists, showing it");
                        if let Some(window) = app.get_window(WINDOW_ID_SETTINGS) {
                            if let Err(e) = window.show() {
                                log::error!("Failed to show settings window: {}", e);
                            }
                            if let Err(e) = window.set_focus() {
                                log::error!("Failed to focus settings window: {}", e);
                            }
                            return;
                        } else {
                            log::error!("Settings window should exist... but it was not found.");
                            unreachable!("Settings window should exist...");
                        }
                    }

                    let window_config = app
                        .config()
                        .app
                        .windows
                        .iter()
                        .filter(|w| w.label == WINDOW_ID_SETTINGS)
                        .next()
                        .expect("Settings window not found");

                    if let Ok(window_builder) =
                        WebviewWindowBuilder::from_config(app, window_config)
                    {
                        if let Ok(window) = window_builder.build() {
                            log::debug!("Settings window created successfully");
                            if let Err(e) = window.show() {
                                log::error!("Failed to show settings window: {}", e);
                            }
                        } else {
                            log::error!("Failed to build settings window");
                        }
                    } else {
                        log::error!("Failed to create settings window");
                    }
                }
            }
        }
    }
}

/// Handles tray icon events.
/// This function is called when a tray icon event occurs, such as a double-click.
/// It currently handles double-click events to show the main window if it is hidden.
pub fn on_tray_icon_event(tray_icon: &TrayIcon, event: TrayIconEvent) {
    match event {
        TrayIconEvent::DoubleClick {
            id: _,
            position: _,
            rect: _,
            button: _,
        } => {
            log::debug!("Tray icon double-clicked");
            if let Some(window) = tray_icon.app_handle().get_window(WINDOW_ID_MAIN) {
                if !window.is_visible().unwrap_or(true) {
                    log::debug!("Showing window on tray icon double-click");
                    if let Err(e) = window.show() {
                        log::error!("Failed to show window: {}", e);
                    }
                    if let Err(e) = window.set_focus() {
                        log::error!("Failed to focus window: {}", e);
                    }
                    if let Err(e) = tray_icon.app_handle().emit(EVENT_WINDOW_SHOW, ()) {
                        log::error!("Failed to emit window show event: {}", e);
                    }
                }
            } else {
                log::warn!("Main window not found on tray icon double-click");
            }
        }
        _ => {}
    }
}
