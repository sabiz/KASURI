use tauri::Manager;

use crate::core::kasuri_app::AppForView;
use crate::core::settings::{
    SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP, Settings,
};
use crate::model::application::Application;
use crate::repositories::application_repository::ApplicationRepository;
use crate::repositories::kasuri_repository::KasuriRepository;
use crate::repositories::repository_initializer::RepositoryInitializer;
use crate::service::fuzzy_sorter::FuzzySorter;
use std::path::PathBuf;
use std::str::FromStr;
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
pub struct Kasuri {
    /// Application settings loaded from configuration file.
    pub settings: Settings,
    /// Repository for application data access.
    application_repository: ApplicationRepository,
    /// Repository for Kasuri's internal data.
    kasuri_repository: KasuriRepository,
    /// Service for fuzzy searching and sorting applications.
    fuzzy_sorter: FuzzySorter,
    /// In-memory cache of available applications.
    app_cache: Option<Vec<Application>>,
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
    pub fn init(&mut self, app_handle: &tauri::AppHandle) -> KasuriResult<()> {
        self.set_app_cache(self.load_applications_from_search_path_if_needed(app_handle)?)?;
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
                icon_path: app.icon_path.clone().unwrap_or_default(),
            })
            .collect()
    }

    /// Launches the specified application using its app ID.
    ///
    /// This method retrieves the application from the cache and invokes its launch method.
    /// If the application is not found in the cache, an error is logged.
    /// # Arguments
    ///
    /// * `app_id` - The unique identifier of the application to launch
    ///
    /// # Returns
    ///
    /// A `KasuriResult<()>` indicating success or failure of the launch operation
    ///
    /// # Errors
    ///
    /// Returns an error if the application cache is not initialized or if the application is not found
    /// in the cache.
    pub fn handle_launch_application(&self, app_id: &str) -> KasuriResult<()> {
        let Some(app_cache) = &self.app_cache else {
            return Err("Application cache is not initialized".into());
        };
        let app = app_cache.iter().find(|app| app.app_id == app_id);
        if let Some(app) = app {
            log::debug!("Launching application: {}", app.name);
            app.launch()?;
            let _ = self.application_repository.update_usage(&app).map_err(|e| {
                log::error!("Failed to update application usage: {}", e);
            });
        } else {
            log::error!("Application with ID {} not found in cache", app_id);
        }
        Ok(())
    }

    /// Forces a reload of applications into the cache from search paths.
    ///
    /// This method is typically used when the user explicitly requests a refresh
    /// of application data, or when application data may have changed externally.
    ///
    /// # Arguments
    ///
    /// * `app_handle` - The Tauri application handle, used to access app resources
    ///
    /// # Returns
    ///
    /// A `KasuriResult<()>` indicating success or failure of the cache reload operation
    pub fn load_applications_to_cache(
        &mut self,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<()> {
        log::debug!("Forcing reload of applications into cache");
        self.load_applications_from_search_path(app_handle)?;
        let mut applications = self.load_application_from_repository()?;
        self.setup_applications_icon_path(&mut applications, app_handle)?;
        self.set_app_cache(applications)?;
        Ok(())
    }

    /// Loads applications from search paths only if needed based on time interval.
    ///
    /// This method checks if a new application search is needed based on the time
    /// since the last search. If not needed, it loads applications from the repository.
    /// If needed, it performs a full search of the file system for applications.
    ///
    /// # Arguments
    ///
    /// * `app_handle` - The Tauri application handle, used to access app resources
    ///
    /// # Returns
    ///
    /// A `KasuriResult<Vec<Application>>` containing the loaded applications or an error
    fn load_applications_from_search_path_if_needed(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<Vec<Application>> {
        let mut applications: Vec<Application>;

        if self.is_search_application_needed() {
            log::debug!("Application search needed, scanning search paths");
            self.load_applications_from_search_path(app_handle)?;
        }

        log::debug!("Application search not needed, loading from repository");
        applications = self.load_application_from_repository()?;
        self.setup_applications_icon_path(&mut applications, app_handle)?;

        Ok(applications)
    }

    /// Loads applications from the repository.
    ///
    /// This method retrieves all applications stored in the application's repository.
    /// # Returns
    ///
    /// A `KasuriResult<Vec<Application>>` containing the loaded applications or an error
    fn load_application_from_repository(&self) -> KasuriResult<Vec<Application>> {
        log::debug!("Loading applications from repository");
        let applications = self.application_repository.get_applications()?;
        log::debug!("Loaded {} applications from repository", applications.len());
        Ok(applications)
    }

    /// Loads applications from all configured search paths.
    ///
    /// This method scans all directories specified in the application settings,
    /// collects application data, updates the repository, and sets up icon paths.
    ///
    /// # Arguments
    ///
    /// * `app_handle` - The Tauri application handle, used to access app resources
    ///
    /// # Returns
    ///
    /// A `KasuriResult<Vec<Application>>` containing the loaded applications or an error
    fn load_applications_from_search_path(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<()> {
        log::debug!("Beginning application scan from configured search paths");
        let cache_path = self.get_app_cache_path(app_handle)?;
        // Load applications from the specified paths
        let search_path_applications: Vec<Application> = self
            .settings
            .get_application_search_path_list()
            .iter()
            .flat_map(|path| {
                log::debug!("Loading applications from path: {}", path);
                if path == SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP {
                    log::debug!("Scanning Windows Store applications");
                    Application::from_app_store()
                } else {
                    log::debug!("Scanning filesystem path: {}", path);
                    Application::from_path(path)
                }
            })
            .collect();
        log::debug!("Updating last application search time");
        self.kasuri_repository.set_last_application_search_time()?;

        log::debug!(
            "Found {} applications, updating repository",
            search_path_applications.len()
        );
        let new_applications = self
            .application_repository
            .renew_applications(search_path_applications.clone())?;

        log::debug!(
            "Creating application icons for {} new applications",
            new_applications.len()
        );
        Application::create_app_icon(new_applications, &cache_path)?;
        Ok(())
    }

    /// Sets up icon paths for applications based on the application cache directory.
    ///
    /// This method updates each application's icon_path field to point to the
    /// correct location in the application cache directory.
    ///
    /// # Arguments
    ///
    /// * `applications` - A mutable reference to the vector of applications to update
    /// * `app_handle` - The Tauri application handle, used to access app resources
    ///
    /// # Returns
    ///
    /// A `KasuriResult<()>` indicating success or failure of the operation
    fn setup_applications_icon_path(
        &self,
        applications: &mut Vec<Application>,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<()> {
        let cache_path = PathBuf::from_str(self.get_app_cache_path(app_handle)?.as_str())?;
        log::debug!(
            "Setting up icon paths using cache directory: {}",
            cache_path.display()
        );

        applications.iter_mut().for_each(|app| {
            let icon_name = app.get_icon_name();
            let icon_path = cache_path
                .clone()
                .join(&icon_name)
                .to_string_lossy()
                .to_string();

            log::debug!("Setting icon path for '{}': {}", app.name, icon_path);
            app.icon_path = Some(icon_path);
        });
        Ok(())
    }

    /// Retrieves the application cache directory path as a string.
    ///
    /// This method gets the platform-specific application cache directory
    /// where Kasuri stores cached data like application icons.
    ///
    /// # Arguments
    ///
    /// * `app_handle` - The Tauri application handle, used to access app resources
    ///
    /// # Returns
    ///
    /// A `KasuriResult<String>` containing the cache directory path or an error
    fn get_app_cache_path(&self, app_handle: &tauri::AppHandle) -> KasuriResult<String> {
        let cache_path = app_handle
            .path()
            .app_cache_dir()?
            .into_os_string()
            .into_string()
            .unwrap();

        log::debug!("Application cache path: {}", cache_path);
        Ok(cache_path)
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
        let interval_seconds = self
            .settings
            .get_application_search_interval_on_startup_minute()
            * 60;

        log::debug!(
            "Time since last application search: {}s (interval: {}s)",
            elapsed_time,
            interval_seconds
        );

        elapsed_time > interval_seconds
    }

    /// Sets the application cache with a list of applications.
    /// This method updates the in-memory cache of applications
    /// and assigns aliases to applications based on the settings.
    /// # Arguments
    ///
    /// * `applications` - A vector of `Application` objects to cache
    ///
    /// # Returns
    ///
    /// A `KasuriResult<()>` indicating success or failure of the operation
    fn set_app_cache(&mut self, applications: Vec<Application>) -> KasuriResult<()> {
        log::debug!(
            "Setting application cache with {} applications",
            applications.len()
        );
        let alias_map = self
            .settings
            .get_application_name_aliases()
            .iter()
            .map(|v| (v.path.clone(), v.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        self.app_cache = Some(
            applications
                .into_iter()
                .map(|mut app| {
                    if let Some(alias) = alias_map.get(&app.path).map(|v| v.alias.clone()) {
                        log::debug!("Setting alias '{}' for application '{}'", alias, app.name);
                        app.alias = Some(alias);
                    }
                    app
                })
                .collect::<Vec<_>>(),
        );
        Ok(())
    }
}
