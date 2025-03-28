mod application;
mod fuzzy_sorter;

use crate::application::Application;
use crate::fuzzy_sorter::FuzzySorter;

fn main() {
    let applications = vec![
        Application::new("Firefox".to_string()),
        Application::new("Chrome".to_string()),
        Application::new("Visual Studio Code".to_string()),
        Application::new("File Explorer".to_string()),
        Application::new("Notepad".to_string()),
    ];
    let sorter = FuzzySorter::new();
    let query = "e";

    let results = sorter.sort(query, applications);

    for app in results {
        println!("- {}", app.name);
    }
}
