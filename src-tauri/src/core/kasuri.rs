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
        self.app_cache = Some(self.load_applications_from_search_path_if_needed(app_handle)?);
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
        } else {
            log::error!("Application with ID {} not found in cache", app_id);
        }
        Ok(())
    }

    pub fn load_applications_to_cache(
        &mut self,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<()> {
        self.app_cache = Some(self.load_applications_from_search_path(app_handle)?);
        Ok(())
    }

    fn load_applications_from_search_path_if_needed(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<Vec<Application>> {
        let mut applications: Vec<Application>;
        if !self.is_search_application_needed() {
            log::debug!("Application search is not needed.");
            applications = self.application_repository.get_applications()?;
            self.setup_applications_icon_path(&mut applications, app_handle)?;
        } else {
            applications = self.load_applications_from_search_path(app_handle)?;
        }

        Ok(applications)
    }

    fn load_applications_from_search_path(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<Vec<Application>> {
        let cache_path = self.get_app_cache_path(app_handle)?;
        // Load applications from the specified paths
        let mut applications: Vec<Application> = self
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
        let new_applications = self
            .application_repository
            .renew_applications(applications.clone())?;
        Application::create_app_icon(new_applications, &cache_path)?;
        self.setup_applications_icon_path(&mut applications, app_handle)?;
        Ok(applications)
    }

    fn setup_applications_icon_path(
        &self,
        applications: &mut Vec<Application>,
        app_handle: &tauri::AppHandle,
    ) -> KasuriResult<()> {
        let cache_path = PathBuf::from_str(self.get_app_cache_path(app_handle)?.as_str())?;
        applications.iter_mut().for_each(|app| {
            app.icon_path = Some(
                cache_path
                    .clone()
                    .join(app.get_icon_name())
                    .to_string_lossy()
                    .to_string(),
            );
        });
        Ok(())
    }

    fn get_app_cache_path(&self, app_handle: &tauri::AppHandle) -> KasuriResult<String> {
        let cache_path = app_handle
            .path()
            .app_cache_dir()?
            .into_os_string()
            .into_string()
            .unwrap();
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
        elapsed_time
            > self
                .settings
                .get_application_search_interval_on_startup_minute()
                * 60
    }
}
