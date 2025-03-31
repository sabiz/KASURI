use config::Config;
use serde::{Deserialize, Serialize};
use dirs::data_dir;

const SETTINGS_FILE_NAME: &str = "settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    application_search_path_list: Vec<String>,
}

impl Settings {
    pub fn load() -> Self {
        if !Settings::is_existing_settings_file() {
            Settings::default();
        }
        let config = Settings::get_config();
        config.try_deserialize::<Settings>().unwrap()
    }

    fn get_config() -> Config {
        Config::builder()
            .add_source(config::File::with_name(SETTINGS_FILE_NAME))
            .build()
            .unwrap()
    }

    fn is_existing_settings_file() -> bool {
        std::path::Path::new(SETTINGS_FILE_NAME).exists()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            application_search_path_list: vec![
                
            ],
        }
    }
}