use crate::application::Application;
use crate::fuzzy_sorter::FuzzySorter;
use crate::settings::{SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP, Settings};

pub struct Kasuri {
    settings: Settings,
}

impl Kasuri {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let settings = Settings::load();
        if settings.is_err() {
            return Err(format!("Failed to load settings: {}", settings.unwrap_err()).into());
        }
        let settings = settings.unwrap();
        Ok(Self { settings })
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{:#?}", self.settings);
        // Load applications from the specified paths
        let mut applications = Vec::new();
        for path in &self.settings.get_application_search_path_list() {
            if SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP == path {
                // TODO Handle Windows Store App
                continue;
            }
            println!("Loading applications from: {}", path);
            let apps = Application::from_path(path);
            applications.extend(apps);
        }

        let sorter = FuzzySorter::new();
        let query = "e";

        let results = sorter.sort(query, applications);

        for app in results {
            println!("- {}", app.name);
        }
        Ok(())
    }
}
