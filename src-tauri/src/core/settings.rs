use crate::KasuriResult;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

/// Placeholder for data directory
const DEFAULT_SETTINGS_MARKER_DATA_DIR: &str = "<DATA_DIR>";
/// Settings file name
const SETTINGS_FILE_NAME: &str = "settings.toml";
/// Constant value indicating Windows Store App
pub const SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP: &str = "WindowsStoreApp";

/// Structure that holds application settings.
///
/// This structure contains all configurable parameters for the Kasuri application.
/// Settings are loaded from a TOML file and can be saved back to the file when modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// List of paths to search for applications
    application_search_path_list: Vec<String>,

    /// Interval in minutes between application searches at startup
    application_search_interval_on_startup_minute: u64,

    /// Logging level (error, warn, info, debug)
    log_level: String,

    /// Width of the main application window
    width: u32,

    /// Flag indicating whether the application should start automatically at system startup
    auto_startup: bool,

    /// Global shortcut key combination to show/hide the application
    shortcut_key: String,

    /// List of application name aliases
    application_name_aliases: Vec<ApplicationNameAlias>,
}

/// Internal structure for partial settings deserializatión.
///
/// This structure is used when loading settings from a file that may not contain
/// all settings fields. Any missing fields will be filled with default values.
#[derive(Debug, Clone, Deserialize)]
struct PartialSettings {
    /// Optional list of application search paths
    application_search_path_list: Option<Vec<String>>,

    /// Optional interval between application searches
    application_search_interval_on_startup_minute: Option<u64>,

    /// Optional logging level
    log_level: Option<String>,

    /// Optional window width
    width: Option<u32>,

    /// Optional auto startup flag
    auto_startup: Option<bool>,

    /// Optional global shortcut key
    shortcut_key: Option<String>,

    /// Optional list of application name aliases
    application_name_aliases: Option<Vec<ApplicationNameAlias>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationNameAlias {
    /// The path to the application executable
    pub path: String,

    /// The alias for the application
    pub alias: String,
}

impl Settings {
    /// Load settings from the settings file.
    ///
    /// If the settings file does not exist, this method creates default settings
    /// and saves them to a new settings file before loading them.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the loaded `Settings` object.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or writing the settings file fails.
    pub fn load() -> KasuriResult<Self> {
        log::debug!("Loading settings from file: {}", SETTINGS_FILE_NAME);

        if !Self::is_existing_settings_file() {
            log::info!("Settings file not found, creating default settings");
            let settings = Self::default();
            settings.save()?;
        }

        Self::load_from_file()
    }

    /// Returns a clone of the application search path list.
    ///
    /// This method provides access to the list of directories to be searched
    /// for applications. The list may include special paths like "WindowsStoreApp"
    /// for Windows Store applications.
    ///
    /// # Returns
    ///
    /// A vector of directory paths as strings.
    pub fn get_application_search_path_list(&self) -> &Vec<String> {
        log::debug!(
            "Retrieving application search paths: {:?}",
            self.application_search_path_list
        );
        &self.application_search_path_list
    }

    /// Returns the application search interval on startup in minutes.
    ///
    /// This interval determines how frequently the application should
    /// perform a full scan for applications during startup.
    ///
    /// # Returns
    ///
    /// The interval in minutes as a u64 value.
    pub fn get_application_search_interval_on_startup_minute(&self) -> u64 {
        log::debug!(
            "Retrieving application search interval: {} minutes",
            self.application_search_interval_on_startup_minute
        );
        self.application_search_interval_on_startup_minute
    }

    /// Returns the configured log level.
    ///
    /// Possible values include "error", "warn", "info", and "debug",
    /// which determine the verbosity of application logging.
    ///
    /// # Returns
    ///
    /// The log level as a string.
    pub fn get_log_level(&self) -> &String {
        log::debug!("Retrieving log level: {}", self.log_level);
        &self.log_level
    }

    /// Returns the width of the main application window.
    ///
    /// # Returns
    ///
    /// The window width in logical pixels.
    pub fn get_width(&self) -> u32 {
        log::debug!("Retrieving window width: {}", self.width);
        self.width
    }

    /// Returns the auto startup setting.
    ///
    /// When true, the application will be configured to start
    /// automatically when the user logs into the system.
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether auto startup is enabled.
    pub fn get_auto_startup(&self) -> bool {
        log::debug!("Retrieving auto startup setting: {}", self.auto_startup);
        self.auto_startup
    }

    /// Returns the global shortcut key.
    ///
    /// This shortcut key is used to show or hide the application
    /// from anywhere in the system.
    ///
    /// # Returns
    ///
    /// The shortcut key combination as a string.
    pub fn get_shortcut_key(&self) -> &String {
        log::debug!("Retrieving shortcut key: {}", self.shortcut_key);
        &self.shortcut_key
    }

    /// Returns the list of application name aliases.
    ///
    /// This method provides access to the list of aliases for application names,
    /// which can be used to refer to applications by alternative names.
    /// # Returns
    ///
    /// A vector of `ApplicationNameAlias` objects.
    pub fn get_application_name_aliases(&self) -> &Vec<ApplicationNameAlias> {
        log::debug!(
            "Retrieving application name aliases: {:?}",
            self.application_name_aliases
        );
        &self.application_name_aliases
    }

    /// Checks if the settings file exists in the expected location.
    ///
    /// This is a helper method used to determine whether default settings
    /// need to be created during application initialization.
    ///
    /// # Returns
    ///
    /// `true` if the settings file exists, `false` otherwise.
    fn is_existing_settings_file() -> bool {
        let path = Self::get_settings_file_path();
        let exists = path.exists();
        log::debug!(
            "Checking if settings file exists at {}: {}",
            path.to_string_lossy(),
            if exists { "yes" } else { "no" }
        );
        exists
    }

    /// Loads settings from the settings file.
    ///
    /// This method reads the settings file, parses its contents as TOML,
    /// and constructs a Settings object. It handles partial settings by
    /// filling in missing values with defaults.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Settings` object.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file contents cannot be read
    /// - The file is empty
    /// - The TOML parsing fails
    fn load_from_file() -> KasuriResult<Self> {
        let path = Self::get_settings_file_path();
        log::debug!("Opening settings file: {:?}", path);
        let mut file = File::open(path)?;

        let mut buf = String::new();
        let size = file.read_to_string(&mut buf)?;
        log::debug!("Read {} bytes from settings file", size);

        if size == 0 {
            log::warn!("Settings file is empty");
            return Err("Settings file is empty".into());
        }

        log::debug!("Parsing settings from TOML");
        let partial_settings: PartialSettings = toml::from_str(&buf)?;

        log::debug!("Creating default settings to fill in any missing values");
        let default_settings = Self::default();

        log::debug!("Merging parsed settings with default values");
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
            width: partial_settings
                .width
                .unwrap_or_else(|| default_settings.width),
            auto_startup: partial_settings
                .auto_startup
                .unwrap_or_else(|| default_settings.auto_startup),
            shortcut_key: partial_settings
                .shortcut_key
                .unwrap_or_else(|| default_settings.shortcut_key),
            application_name_aliases: partial_settings
                .application_name_aliases
                .unwrap_or_else(|| default_settings.application_name_aliases),
        };

        log::debug!("Settings loaded successfully: {:?}", settings);
        Ok(settings)
    }

    /// Saves current settings to the settings file.
    ///
    /// This method serializes the Settings object to TOML format
    /// and writes it to the settings file. It uses pretty formatting
    /// to make the file more human-readable.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the save operation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be created
    /// - The settings cannot be serialized to TOML
    /// - The data cannot be written to the file
    fn save(self) -> KasuriResult<()> {
        let path = Self::get_settings_file_path();
        log::debug!("Creating settings file: {:?}", path);
        let mut file = File::create(path)?;

        log::debug!("Serializing settings to TOML");
        let settings_str = toml::to_string_pretty(&self)?;

        log::debug!("Writing {} bytes to settings file", settings_str.len());
        file.write_all(settings_str.as_bytes())?;

        log::info!("Settings saved successfully");
        Ok(())
    }

    /// Returns the path to the settings file.
    ///
    /// This method constructs the path to the settings file based on the current executable's directory.
    /// It assumes the settings file is located in the same directory as the executable.
    /// # Returns
    ///
    /// A `PathBuf` representing the path to the settings file.
    fn get_settings_file_path() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(SETTINGS_FILE_NAME)
    }
}

impl Default for Settings {
    /// Creates default settings from embedded template.
    ///
    /// This method loads the default settings from the embedded `default_settings.toml` file,
    /// replaces placeholders with system-specific paths, and deserializes the result
    /// into a Settings object.
    ///
    /// # Returns
    ///
    /// A new Settings object with default values.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// - The system data directory cannot be determined
    /// - The default settings template cannot be parsed as valid TOML
    fn default() -> Self {
        log::debug!("Creating default settings from template");
        let default_settings_str = include_str!("default_settings.toml");

        let data_dir_path = match data_dir() {
            Some(path) => {
                let path_str = path.to_str().unwrap_or("<unknown>");
                log::debug!("Using system data directory: {}", path_str);
                path_str.to_string()
            }
            None => {
                log::warn!("Could not determine system data directory, using empty string");
                String::new()
            }
        };

        let default_settings_str =
            default_settings_str.replace(DEFAULT_SETTINGS_MARKER_DATA_DIR, &data_dir_path);

        log::debug!("Parsing default settings from template");
        let settings = toml::from_str(default_settings_str.as_str()).unwrap();

        log::debug!("Default settings created successfully");
        settings
    }
}
