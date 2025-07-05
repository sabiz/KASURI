/// Module that handles application management and operations.
/// This module provides functionality to work with Windows applications including
/// standard executable files, shortcuts, and Windows Store apps.
use md5::{Digest, Md5};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{ffi::OsStr, path::PathBuf, str::FromStr};

use crate::{
    core::kasuri::KasuriResult,
    repositories::application_repository::ApplicationRepositoryRecord,
    service::powershell::{PowerShell, PowerShellResult},
};
use walkdir::WalkDir;

const GET_STORE_APP_SCRIPT: &str = include_str!("../scripts/get_store_app.ps1");
const SAVE_APP_ICON_SCRIPT: &str = include_str!("../scripts/save_app_icon.ps1");

/// Represents an application that can be managed and launched by the KASURI application.
///
/// This structure holds essential information about an application, including its name,
/// identifier, path, and optional icon path. It supports various types of applications
/// including standard executables (.exe), shortcuts (.lnk), and Windows Store apps.
#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
    pub alias: Option<String>,
    pub app_id: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub usage_recency_score: f64,
}

/// Structure representing a Windows Store application.
///
/// This structure is used to deserialize data received from PowerShell scripts
/// that query the Windows Store applications.
#[derive(serde::Deserialize, Debug)]
struct WindowsStoreApp {
    pub name: String,
    pub app_id: String,
    pub package_fullname: String,
}

impl Application {
    /// Creates a new Application instance with the provided name, application ID, and path.
    ///
    /// # Arguments
    ///
    /// * `name` - The display name of the application
    /// * `app_id` - The unique identifier for the application
    /// * `path` - The file system path or identifier for the application
    ///
    /// # Returns
    ///
    /// A new `Application` instance with the specified properties and `None` for icon_path
    pub fn new(name: String, app_id: String, path: String) -> Self {
        Self {
            name,
            alias: None, // Alias is optional and can be set later
            app_id,
            path,
            icon_path: None,
            usage_recency_score: 0.0, // Default score
        }
    }

    /// Creates a list of Application instances by scanning a directory for executable files and shortcuts.
    ///
    /// This method recursively traverses the given directory path to find .exe and .lnk files,
    /// and creates an Application instance for each valid file found.
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to scan for applications
    ///
    /// # Returns
    ///
    /// A vector of Application instances representing the discovered applications
    pub fn from_path(path: &str) -> Vec<Self> {
        log::info!("Scanning directory for applications: {}", path);
        let applications: Vec<Self> = WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let ext = match path.extension() {
                    Some(ext) => ext.to_ascii_lowercase(),
                    None => {
                        log::debug!("Skipping file with no extension: {:?}", path);
                        return None;
                    }
                };

                if ext != "exe" && ext != "lnk" {
                    log::debug!("Skipping non-executable file: {:?}", path);
                    return None;
                }

                let name = match path.file_stem() {
                    Some(stem) => stem.to_string_lossy().to_string(),
                    None => {
                        log::warn!("Could not extract file stem from path: {:?}", path);
                        return None;
                    }
                };
                let path_str = path.to_string_lossy().to_string();
                log::debug!("Found application: {} at {}", name, path_str);

                Some(Self::new(name, path_str.clone(), path_str))
            })
            .collect();

        log::info!(
            "Found {} applications in directory: {}",
            applications.len(),
            path
        );
        applications
    }

    /// Retrieves a list of Windows Store applications installed on the system.
    ///
    /// This method uses PowerShell to execute a script that queries the Windows Store
    /// for installed applications and converts them to Application instances.
    ///
    /// # Returns
    ///
    /// A vector of Application instances representing the discovered Windows Store applications
    pub fn from_app_store() -> Vec<Self> {
        log::info!("Retrieving applications from Windows Store");
        let powershell = PowerShell::new();
        powershell
            .run(GET_STORE_APP_SCRIPT)
            .and_then(|result| {
                log::debug!("Windows Store apps query result: {}", result.stdout);
                if !result._stderr.is_empty() {
                    log::warn!("Windows Store apps query stderr: {}", result._stderr);
                }
                PowerShellResult::to_struct::<Vec<WindowsStoreApp>>(result)
            })
            .map(|apps| {
                log::info!("Found {} Windows Store applications", apps.len());
                apps.iter().map(Self::from_windows_store_app).collect()
            })
            .unwrap_or_else(|e| {
                log::error!("Failed to get applications from Windows Store: {}", e);
                Vec::new()
            })
    }

    /// Generates icon files for a list of applications.
    ///
    /// This method uses PowerShell to extract and save icons from the application executables
    /// to the specified base path. Each icon is named based on the application's ID.
    ///
    /// # Arguments
    ///
    /// * `applications` - A vector of Application instances to generate icons for
    /// * `store_base_path` - Base directory to store the generated icon files
    ///
    /// # Returns
    ///
    /// A Result indicating success or containing an error if the operation failed
    pub fn create_app_icon(applications: Vec<Self>, store_base_path: &String) -> KasuriResult<()> {
        log::info!(
            "Creating application icons for {} applications",
            applications.len()
        );
        log::debug!("Icon storage path: {}", store_base_path);

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

                    log::debug!("Processing icon for app: {}", app.name);
                    if app.path.contains("\\") {
                        log::debug!("Standard app path: {}", app.path);
                        e_path.push(app.path.clone());
                    } else {
                        // For windows store apps
                        let package_id =
                            app.path.clone().split("_").collect::<Vec<_>>()[0].to_string();
                        log::debug!("Windows Store app package ID: {}", package_id);
                        e_path.push(package_id);
                    }
                    log::debug!("Icon will be saved to: {}", icon_path);
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

        log::debug!("Preparing PowerShell command to extract icons");
        let command = SAVE_APP_ICON_SCRIPT
            .replace("{EXE_PATH_ARR}", &app_paths)
            .replace("{OUTPUT_PATH_ARR}", &icon_paths);

        let result = powershell.run(&command);
        match result {
            Ok(output) => {
                log::debug!("Icon extraction completed successfully");
                log::debug!("Icon extraction stdout: {}", output.stdout);
                if !output._stderr.is_empty() {
                    log::warn!("Icon extraction stderr: {}", output._stderr);
                }
            }
            Err(e) => {
                log::error!("Failed to create app icons: {}", e);
                return Err(format!("Icon extraction failed: {}", e).into());
            }
        }

        log::info!("Successfully created icons for all applications");
        Ok(())
    }

    /// Generates a unique icon filename for the application based on its ID.
    ///
    /// This method creates a deterministic filename based on an MD5 hash of the application ID,
    /// ensuring consistent and unique filenames for each application.
    ///
    /// # Returns
    ///
    /// A string representing the icon filename with .png extension
    pub fn get_icon_name(&self) -> String {
        log::debug!("Generating icon name for application: {}", self.name);
        let mut hasher = Md5::new();
        hasher.update(self.app_id.as_bytes());
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        let icon_name = format!("{}.png", hash[..16].to_string());
        log::debug!("Generated icon name: {}", icon_name);
        icon_name
    }

    /// Launches the application based on its path type.
    ///
    /// This method determines the appropriate launch method based on the application path:
    /// - Executable files (.exe): Launches using the system's default handler
    /// - Shortcuts (.lnk): Launches using the system's default handler
    /// - Windows Store apps: Launches using PowerShell commands
    ///
    /// # Returns
    ///
    /// A Result indicating success or containing an error if the launch failed
    pub fn launch(&self) -> KasuriResult<()> {
        log::info!("Launching application: {}", self.name);
        log::debug!("Application path: {}", self.path);

        match self.path.as_str() {
            path if path.ends_with(".exe") => {
                log::debug!("Launching as executable (.exe) file");
                self.launch_exe()?
            }
            path if path.ends_with(".lnk") => {
                log::debug!("Launching as shortcut (.lnk) file");
                self.launch_lnk()?
            }
            path if !path.contains("\\") => {
                log::debug!("Launching as Windows Store app");
                self.launch_store_app()?
            }
            _ => {
                log::error!("Invalid application path format: {}", self.path);
                return Err("Invalid application path".into());
            }
        }

        log::info!("Successfully launched application: {}", self.name);
        Ok(())
    }

    /// Launches an executable (.exe) file application.
    ///
    /// Uses the `open` crate to launch the application in a detached process.
    ///
    /// # Returns
    ///
    /// A Result indicating success or containing an error if the launch failed
    fn launch_exe(&self) -> KasuriResult<()> {
        log::debug!("Launching executable: {}", self.path);
        open::that_detached(OsStr::new(self.path.as_str())).map_err(|e| {
            log::error!("Failed to launch executable '{}': {}", self.path, e);
            e
        })?;
        log::debug!("Successfully initiated executable launch process");
        Ok(())
    }

    /// Launches a shortcut (.lnk) file application.
    ///
    /// Uses the `open` crate to launch the shortcut in a detached process.
    ///
    /// # Returns
    ///
    /// A Result indicating success or containing an error if the launch failed
    fn launch_lnk(&self) -> KasuriResult<()> {
        log::debug!("Launching shortcut: {}", self.path);
        open::that_detached(OsStr::new(self.path.as_str())).map_err(|e| {
            log::error!("Failed to launch shortcut '{}': {}", self.path, e);
            e
        })?;
        log::debug!("Successfully initiated shortcut launch process");
        Ok(())
    }

    /// Launches a Windows Store application.
    ///
    /// Uses PowerShell to execute a command that launches the Windows Store app
    /// using the shell:AppsFolder protocol.
    ///
    /// # Returns
    ///
    /// A Result indicating success or containing an error if the launch failed
    fn launch_store_app(&self) -> KasuriResult<()> {
        log::debug!("Launching Windows Store app with ID: {}", self.app_id);
        let powershell = PowerShell::new();
        let command = format!("Start-Process \"shell:AppsFolder\\{}\"", self.app_id);
        log::debug!("PowerShell command: {}", command);

        powershell
            .run(&command)
            .map_err(|e| {
                log::error!(
                    "Failed to launch Windows Store app '{}': {}",
                    self.app_id,
                    e
                );
                e
            })
            .map(|result| {
                log::debug!("Windows Store app launch command executed");
                if !result.stdout.is_empty() {
                    log::debug!("Launch stdout: {}", result.stdout);
                }
                if !result._stderr.is_empty() {
                    log::warn!("Launch stderr: {}", result._stderr);
                }
            })?;

        log::debug!("Successfully initiated Windows Store app launch process");
        Ok(())
    }

    /// Converts a WindowsStoreApp instance to an Application instance.
    ///
    /// # Arguments
    ///
    /// * `store_app` - A reference to a WindowsStoreApp instance to convert
    ///
    /// # Returns
    ///
    /// A new Application instance initialized with the Windows Store app information
    fn from_windows_store_app(store_app: &WindowsStoreApp) -> Self {
        log::debug!(
            "Converting Windows Store app '{}' to Application",
            store_app.name
        );
        Self::new(
            store_app.name.clone(),
            store_app.app_id.clone(),
            store_app.package_fullname.clone(),
        )
    }
}

impl From<ApplicationRepositoryRecord> for Application {
    /// Converts an ApplicationRepositoryRecord to an Application instance.
    ///
    /// # Arguments
    ///
    /// * `record` - An ApplicationRepositoryRecord instance to convert
    ///
    /// # Returns
    ///
    /// A new Application instance initialized with the record's properties
    fn from(record: ApplicationRepositoryRecord) -> Self {
        // Convert ApplicationRepositoryRecord to Application and calculate time-decay score
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Calculate days since last used
        let days_since_last_used = if record.last_used > 0 && now > record.last_used {
            (now - record.last_used) / 86400
        } else {
            0
        };
        // Calculate usage recency score: usage_count / (days_since_last_used + 1)
        let usage_recency_score = record.usage_count as f64 / (days_since_last_used as f64 + 1.0);
        Self {
            name: record.name,
            alias: None,
            app_id: record.app_id,
            path: record.path,
            icon_path: None,
            usage_recency_score,
        }
    }
}
