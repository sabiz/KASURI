//! # Kasuri Core Module
//!
//! This module contains the main application logic for Kasuri, an application launcher.
//! It handles application search, Tauri integration, system tray functionality,
//! and other core features.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconEvent;
use tauri::{App, Manager};
// use crate::core::log::{self, set_file_log_level, set_stdout_log_level};
use crate::core::settings::{
    SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP, Settings,
};
use crate::model::application::Application;
use crate::repositories::application_repository::ApplicationRepository;
use crate::repositories::kasuri_repository::KasuriRepository;
use crate::repositories::repository_initializer::RepositoryInitializer;
use crate::service::fuzzy_sorter::FuzzySorter;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a Result type for Kasuri operations.
///
/// This type alias simplifies error handling throughout the application by
/// wrapping any error type in a Box<dyn std::error::Error>.
pub type KasuriResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Maximum number of search results to display to the user.
const SEARCH_RESULT_LIMIT: usize = 6;

/// Main application controller for Kasuri.
///
/// This struct handles application lifecycle, search functionality,
/// and acts as the central coordinator between various components.
struct Kasuri {
    /// Application settings loaded from configuration file.
    settings: Settings,
    /// Repository for application data access.
    application_repository: ApplicationRepository,
    /// Repository for Kasuri's internal data.
    kasuri_repository: KasuriRepository,
    /// Service for fuzzy searching and sorting applications.
    fuzzy_sorter: FuzzySorter,
    /// In-memory cache of available applications.
    app_cache: Option<Vec<Application>>,
}

/// Simplified application data structure used for passing to the UI layer.
///
/// This structure contains only the fields needed for displaying
/// and selecting applications in the user interface.
#[derive(serde::Serialize)]
struct AppForView {
    /// Display name of the application
    name: String,
    /// Unique identifier for the application
    app_id: String,
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
    let settings = Settings::load().map_err(|e| format!("Failed to load settings: {}", e))?;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(get_plugin_log(&settings).build())
        .invoke_handler(tauri::generate_handler![search_application])
        .setup(|app| {
            log::debug!("Setup started");
            log::debug!("Settings: {:#?}", settings);
            let mut kasuri = Kasuri::with_settings(settings)?;
            kasuri.init()?;
            app.manage(kasuri);
            create_system_tray_menu(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running kasuri");
    Ok(())
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
    tauri_plugin_log::Builder::new()
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::LogDir {
                file_name: Some("logs".to_string()), // log to %APPDATA%\Local\jp.sabiz.kasuri\logs
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
    let tray_icon_main = app.tray_by_id("main").unwrap();
    let item_exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&item_exit])?;
    tray_icon_main.set_menu(Some(menu))?;
    tray_icon_main.on_menu_event(|app, event| match event.id.as_ref() {
        "exit" => {
            log::debug!("Exit menu item clicked");
            app.exit(0);
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
            if let Some(window) = tray_icon.app_handle().get_window("main") {
                if !window.is_visible().unwrap_or(true) {
                    let _ = window.show();
                }
            }
        }
        _ => {}
    });
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
fn search_application(query: &str, app_state: tauri::State<'_, Kasuri>) -> Vec<AppForView> {
    log::debug!("Searching for application: {}", query);
    let kasuri = app_state.inner();
    kasuri.handle_search_application(query)
}

impl Kasuri {
    /// Creates a new Kasuri instance with the provided settings.
    ///
    /// Initializes the application with custom settings instead of loading from the file.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings to use for initialization
    ///
    /// # Returns
    ///
    /// A `KasuriResult<Self>` containing the initialized Kasuri instance or an error
    pub fn with_settings(settings: Settings) -> KasuriResult<Self> {
        let repository_initializer = RepositoryInitializer::new();
        let repositories = repository_initializer.get_repositories()?;
        let application_repository = repositories.application_repository;
        let kasuri_repository = repositories.kasuri_repository;
        Ok(Self {
            settings,
            application_repository,
            kasuri_repository,
            fuzzy_sorter: FuzzySorter::new(),
            app_cache: None,
        })
    }

    /// Initializes the Kasuri instance by loading applications into the cache.
    ///
    /// This method should be called after creating a Kasuri instance and before using it.
    ///
    /// # Returns
    ///
    /// A `KasuriResult<()>` indicating success or failure of the initialization
    pub fn init(&mut self) -> KasuriResult<()> {
        self.app_cache = Some(self.load_applications()?);
        Ok(())
    }

    /// Handles application search requests by querying the application cache
    /// with the provided search term.
    ///
    /// The function performs fuzzy matching on application names and returns
    /// the top matches limited to the maximum display count. It uses the
    /// `sort_with_filter` method from `FuzzySorter` which filters results
    /// based on a minimum match score threshold.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string provided by the user
    ///
    /// # Returns
    ///
    /// A vector of simplified application objects ready to be displayed in the UI
    pub fn handle_search_application(&self, query: &str) -> Vec<AppForView> {
        let applications = self.app_cache.clone().unwrap_or_default();
        let sorted_apps = self.fuzzy_sorter.sort_with_filter(query, applications);
        let limit = std::cmp::min(sorted_apps.len(), SEARCH_RESULT_LIMIT);

        sorted_apps[..limit]
            .iter()
            .map(|app| AppForView {
                name: app.name.clone(),
                app_id: app.app_id.clone(),
            })
            .collect()
    }

    /// Load applications from the specified paths in settings.
    ///
    /// This method fetches applications from the file system and Windows Store
    /// based on configured paths, then updates the application repository.
    ///
    /// # Returns
    ///
    /// A `KasuriResult<Vec<Application>>` containing the loaded applications or an error
    fn load_applications(&self) -> KasuriResult<Vec<Application>> {
        if !self.is_search_application_needed() {
            log::debug!("Application search is not needed.");
            return self.application_repository.get_applications();
        }
        // Load applications from the specified paths
        let applications: Vec<Application> = self
            .settings
            .get_application_search_path_list()
            .iter()
            .flat_map(|path| {
                log::debug!("Loading applications from: {}", path);
                if path == SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP {
                    Application::from_app_store()
                } else {
                    Application::from_path(path)
                }
            })
            .collect();
        self.kasuri_repository.set_last_application_search_time()?;
        self.application_repository
            .renew_applications(applications.clone())?;
        Ok(applications)
    }

    /// Check if the application search is needed based on the last search time and interval.
    ///
    /// Determines whether the application should perform a new search for applications
    /// by comparing the time elapsed since the last search with the configured interval.
    /// This helps optimize performance by avoiding unnecessary searches.
    ///
    /// # Returns
    ///
    /// Returns `true` if a new application search should be performed, `false` otherwise
    fn is_search_application_needed(&self) -> bool {
        let last_application_search_time = self
            .kasuri_repository
            .get_last_application_search_time()
            .unwrap_or(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get current time")
            .as_secs();
        let elapsed_time = now - last_application_search_time;
        elapsed_time
            > self
                .settings
                .get_application_search_interval_on_startup_minute()
                * 60
    }
}
