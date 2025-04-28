use crate::application::Application;
use crate::application_repository::ApplicationRepository;
use crate::fuzzy_sorter::FuzzySorter;
use crate::settings::{SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP, Settings};

pub type KasuriResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Kasuri {
    settings: Settings,
    application_repository: ApplicationRepository,
}

impl Kasuri {
    pub fn new() -> KasuriResult<Self> {
        let settings = Settings::load();
        if settings.is_err() {
            return Err(format!("Failed to load settings: {}", settings.unwrap_err()).into());
        }
        let settings = settings?;
        let application_repository = ApplicationRepository::new()?;
        Ok(Self {
            settings,
            application_repository,
        })
    }

    pub fn run(&self) -> KasuriResult<()> {
        println!("{:#?}", self.settings);

        let applications = self.load_applications()?;
        self.application_repository
            .renew_applications(applications)?;

        // let sorter = FuzzySorter::new();
        // let query = "e";

        // let results = sorter.sort(query, applications);

        // for app in results {
        //     println!("- {}:{}", app.name, app.path);
        // }
        Ok(())
    }

    fn load_applications(&self) -> KasuriResult<Vec<Application>> {
        // Load applications from the specified paths
        let applications: Vec<Application> = self
            .settings
            .get_application_search_path_list()
            .iter()
            .flat_map(|path| {
                println!("Loading applications from: {}", path);
                if path == SETTINGS_VALUE_APPLICATION_SEARCH_PATH_LIST_WINDOWS_STORE_APP {
                    Application::from_app_store()
                } else {
                    Application::from_path(path)
                }
            })
            .collect();
        Ok(applications)
    }
}
