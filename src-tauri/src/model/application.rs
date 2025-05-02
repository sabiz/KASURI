use crate::service::powershell::{PowerShell, PowerShellResult};
use walkdir::WalkDir;

const GET_STORE_APP_SCRIPT: &str = include_str!("../scripts/get_store_app.ps1");

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
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let ext = path.extension()?.to_ascii_lowercase();

                if ext != "exe" && ext != "lnk" {
                    return None;
                }

                let name = path.file_name()?.to_string_lossy().to_string();
                let path_str = path.to_string_lossy().to_string();

                Some(Self::new(name, path_str.clone(), path_str))
            })
            .collect()
    }

    pub fn from_app_store() -> Vec<Self> {
        let powershell = PowerShell::new();
        powershell
            .run(GET_STORE_APP_SCRIPT)
            .and_then(PowerShellResult::to_struct::<Vec<WindowsStoreApp>>)
            .map(|apps| apps.iter().map(Self::from_windows_store_app).collect())
            .unwrap_or_else(|e| {
                eprintln!("Failed to get applications from Windows Store: {}", e);
                Vec::new()
            })
    }

    fn from_windows_store_app(store_app: &WindowsStoreApp) -> Self {
        Self::new(
            store_app.name.clone(),
            store_app.app_id.clone(),
            store_app.package_fullname.clone(),
        )
    }
}
