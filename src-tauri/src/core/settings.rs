use crate::core::kasuri::KasuriResult;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
};

/// Placeholder for data directory
const DEFAULT_SETTINGS_MARKER_DATA_DIR: &str = "<DATA_DIR>";
/// Settings file name
const SETTINGS_FILE_NAME: &str = "settings.toml";
/// Constant value indicating Windows Store App
pub const SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP: &str = "WindowsStoreApp";

/// Structure that holds application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    application_search_path_list: Vec<String>,
    application_search_interval_on_startup_minute: u64,
    log_level: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PartialSettings {
    application_search_path_list: Option<Vec<String>>,
    application_search_interval_on_startup_minute: Option<u64>,
    log_level: Option<String>,
}

impl Settings {
    /// Load settings. If the settings file does not exist, create default settings and save them
    ///
    /// # Errors
    ///
    /// Returns an error if reading or writing the settings file fails
    pub fn load() -> KasuriResult<Self> {
        if !Self::is_existing_settings_file() {
            let settings = Self::default();
            settings.save()?;
        }
        Self::load_from_file()
    }

    /// Returns a clone of the application search path list
    pub fn get_application_search_path_list(&self) -> Vec<String> {
        self.application_search_path_list.clone()
    }

    /// Returns the application search interval on startup in minutes
    pub fn get_application_search_interval_on_startup_minute(&self) -> u64 {
        self.application_search_interval_on_startup_minute
    }

    /// Returns the log level
    pub fn get_log_level(&self) -> String {
        self.log_level.clone()
    }

    /// Check if the settings file exists
    fn is_existing_settings_file() -> bool {
        std::path::Path::new(SETTINGS_FILE_NAME).exists()
    }

    /// Load settings from the settings file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened, read, parsed,
    /// or if the settings file is empty
    fn load_from_file() -> KasuriResult<Self> {
        let mut file = File::open(SETTINGS_FILE_NAME)?;
        let mut buf = String::new();
        let size = file.read_to_string(&mut buf)?;
        if size == 0 {
            return Err("Settings file is empty".into());
        }

        let partial_settings: PartialSettings = toml::from_str(&buf)?;

        let default_settings = Self::default();

        let settings = Settings {
            application_search_path_list: partial_settings
                .application_search_path_list
                .unwrap_or_else(|| default_settings.application_search_path_list),
            application_search_interval_on_startup_minute: partial_settings
                .application_search_interval_on_startup_minute
                .unwrap_or_else(|| default_settings.application_search_interval_on_startup_minute),
            log_level: partial_settings
                .log_level
                .unwrap_or_else(|| default_settings.log_level),
        };

        Ok(settings)
    }

    /// Save current settings to the settings file
    ///
    /// # Errors
    ///
    /// Returns an error if file creation or writing fails,
    /// or if settings serialization fails
    fn save(self) -> KasuriResult<()> {
        let mut file = File::create(SETTINGS_FILE_NAME)?;
        let settings_str = toml::to_string_pretty(&self)?;
        file.write_all(settings_str.as_bytes())?;
        Ok(())
    }
}

impl Default for Settings {
    /// Create default settings
    ///
    /// Load from default_settings.toml file,
    /// and replace placeholders with actual system paths
    fn default() -> Self {
        let default_settings_str = include_str!("default_settings.toml");
        let default_settings_str = default_settings_str.replace(
            DEFAULT_SETTINGS_MARKER_DATA_DIR,
            data_dir().unwrap().to_str().unwrap(),
        );
        toml::from_str(default_settings_str.as_str()).unwrap()
    }
}
