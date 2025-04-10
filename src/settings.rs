use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
};

const DEFAULT_SETTINGS_MARKER_DATA_DIR: &str = "<DATA_DIR>";
const SETTINGS_FILE_NAME: &str = "settings.toml";
pub const SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP: &str = "WindowsStoreApp";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    application_search_path_list: Vec<String>,
}

impl Settings {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if !Settings::is_existing_settings_file() {
            let settings = Settings::default();
            settings.save()?;
        }
        Settings::load_from_file()
    }

    pub fn get_application_search_path_list(&self) -> Vec<String> {
        self.application_search_path_list.clone()
    }

    fn is_existing_settings_file() -> bool {
        std::path::Path::new(SETTINGS_FILE_NAME).exists()
    }

    fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(SETTINGS_FILE_NAME)?;
        let mut buf = String::new();
        let size = file.read_to_string(&mut buf)?;
        if size == 0 {
            return Err("Settings file is empty".into());
        }
        let settings: Settings = toml::from_str(&buf)?;
        Ok(settings)
    }

    fn save(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(SETTINGS_FILE_NAME)?;
        let settings_str = toml::to_string_pretty(&self)?;
        file.write_all(settings_str.as_bytes())?;
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        let default_settings_str = include_str!("default_settings.toml");
        let default_settings_str = default_settings_str.replace(DEFAULT_SETTINGS_MARKER_DATA_DIR, data_dir().unwrap().to_str().unwrap());
        toml::from_str(default_settings_str.as_str()).unwrap()
    }
}
