use crate::powershell::PowerShell;
use walkdir::WalkDir;

const GET_STORE_APP_SCRIPT: &str = include_str!("./scripts/get_store_app.ps1");

#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
    pub app_id: String,
    pub path: String,
}

#[derive(serde::Deserialize, Debug)]
struct WindowsStoreApp {
    pub name: String,
    pub app_id: String,
    pub package_fullname: String,
}

impl Application {
    pub fn new(name: String, app_id: String, path: String) -> Self {
        Self { name, app_id, path }
    }

    pub fn from_path(path: &str) -> Vec<Self> {
        let mut applications = Vec::new();
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            if let Some(ext) = entry.path().extension() {
                if !(ext.to_ascii_lowercase() == "exe") && !(ext.to_ascii_lowercase() == "lnk") {
                    continue;
                }
                if let Some(name) = entry.path().file_name() {
                    let name = name.to_string_lossy().to_string();
                    let path = entry.path().to_string_lossy().to_string();
                    applications.push(Application::new(name, path.clone(), path));
                }
            }
        }
        applications
    }

    pub fn from_app_store() -> Vec<Self> {
        let powershell = PowerShell::new();
        powershell
            .run(GET_STORE_APP_SCRIPT)
            .map_err(|e| {
                eprintln!("Failed to run PowerShell script: {}", e);
            })
            .unwrap()
            .to_struct::<Vec<WindowsStoreApp>>()
            .map_err(|e| {
                eprintln!("Failed to parse PowerShell output: {}", e);
            })
            .unwrap()
            .iter()
            .map(Application::from_windows_store_app)
            .collect::<Vec<Self>>()
    }

    fn from_windows_store_app(store_app: &WindowsStoreApp) -> Self {
        let name = store_app.name.clone();
        let app_id = store_app.app_id.clone();
        let path = store_app.package_fullname.clone();
        Application::new(name, app_id, path)
    }
}
