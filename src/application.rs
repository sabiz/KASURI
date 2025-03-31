use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
    pub path: String,
}

impl Application {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
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
                    applications.push(Application::new(
                        name,
                        entry.path().to_string_lossy().to_string(),
                    ));
                }
            }
        }
        applications
    }
}
