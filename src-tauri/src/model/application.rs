use md5::{Digest, Md5};
use std::{path::PathBuf, str::FromStr};

use crate::{
    core::kasuri::KasuriResult,
    service::powershell::{PowerShell, PowerShellResult},
};
use walkdir::WalkDir;

const GET_STORE_APP_SCRIPT: &str = include_str!("../scripts/get_store_app.ps1");
const SAVE_APP_ICON_SCRIPT: &str = include_str!("../scripts/save_app_icon.ps1");

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
                log::error!("Failed to get applications from Windows Store: {}", e);
                Vec::new()
            })
    }

    pub fn create_app_icon(applications: Vec<Self>, store_base_path: String) -> KasuriResult<()> {
        let powershell = PowerShell::new();
        let (app_paths, icon_paths) =
            applications
                .iter()
                .fold((vec![], vec![]), |(mut e_path, mut i_path), app| {
                    let icon_path = PathBuf::from_str(&store_base_path)
                        .unwrap()
                        .join(app.get_icon_name())
                        .into_os_string()
                        .into_string()
                        .unwrap();
                    if app.path.contains("\\") {
                        e_path.push(app.path.clone());
                    } else {
                        e_path.push(app.path.clone().split("_").collect::<Vec<_>>()[0].to_string()); // for windows store apps
                    }
                    i_path.push(icon_path.clone());
                    (e_path, i_path)
                });
        let app_paths = app_paths
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");
        let icon_paths = icon_paths
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");
        let command = SAVE_APP_ICON_SCRIPT
            .replace("{EXE_PATH_ARR}", &app_paths)
            .replace("{OUTPUT_PATH_ARR}", &icon_paths);
        let _ = powershell
            .run(&command)
            .map_err(|e| {
                log::error!("Failed to create app icon: {}", e);
                e
            })
            .map(|result| {
                log::debug!("create app icon stdout: {}", result.stdout);
                log::warn!("create app icon : {}", result._stderr);
            });

        Ok(())
    }

    fn get_icon_name(&self) -> String {
        let mut hasher = Md5::new();
        hasher.update(self.app_id.as_bytes());
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        format!("{}.png", hash[..16].to_string())
    }

    fn from_windows_store_app(store_app: &WindowsStoreApp) -> Self {
        Self::new(
            store_app.name.clone(),
            store_app.app_id.clone(),
            store_app.package_fullname.clone(),
        )
    }
}
