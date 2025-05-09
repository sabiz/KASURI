use tauri::Manager;
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

pub type KasuriResult<T> = Result<T, Box<dyn std::error::Error>>;

const SEARCH_RESULT_LIMIT: usize = 6;

struct Kasuri {
    settings: Settings,
    repository_initializer: RepositoryInitializer,
    application_repository: ApplicationRepository,
    kasuri_repository: KasuriRepository,
    fuzzy_sorter: FuzzySorter,
    app_cache: Option<Vec<Application>>,
}

#[derive(serde::Serialize)]
struct AppForView {
    name: String,
    app_id: String,
}

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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running kasuri");
    Ok(())
}

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

#[tauri::command]
fn search_application(query: &str, app_state: tauri::State<'_, Kasuri>) -> Vec<AppForView> {
    log::debug!("Searching for application: {}", query);
    let kasuri = app_state.inner();
    kasuri.handle_search_application(query)
}

impl Kasuri {
    pub fn new() -> KasuriResult<Self> {
        let settings = Settings::load();
        if settings.is_err() {
            return Err(format!("Failed to load settings: {}", settings.unwrap_err()).into());
        }
        let settings = settings?;
        Kasuri::with_settings(settings)
    }

    pub fn with_settings(settings: Settings) -> KasuriResult<Self> {
        let repository_initializer = RepositoryInitializer::new();
        let repositories = repository_initializer.get_repositories()?;
        let application_repository = repositories.application_repository;
        let kasuri_repository = repositories.kasuri_repository;
        Ok(Self {
            settings,
            repository_initializer,
            application_repository,
            kasuri_repository,
            fuzzy_sorter: FuzzySorter::new(),
            app_cache: None,
        })
    }

    pub fn init(&mut self) -> KasuriResult<()> {
        self.app_cache = Some(self.load_applications()?);
        Ok(())
    }

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

    /// Load applications from the specified paths  
    /// and renew the application repository  
    /// Returns a vector of loaded applications  
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

    /// Check if the application search is needed based on the last search time  
    /// and the configured interval  
    /// Returns true if the search is needed, false otherwise  
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
