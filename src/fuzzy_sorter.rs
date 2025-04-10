use crate::application::Application;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct FuzzySorter {
    matcher: SkimMatcherV2,
}

impl FuzzySorter {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn sort(&self, query: &str, applications: Vec<Application>) -> Vec<Application> {
        let mut scored_applications: Vec<(Application, i64)> = applications
            .into_iter()
            .map(|app| {
                let score = self.matcher.fuzzy_match(&app.name, query).unwrap_or(0);
                (app, score)
            })
            .collect();
        scored_applications.sort_by(|a, b| b.1.cmp(&a.1));
        scored_applications
            .into_iter()
            .map(|(app, _)| app)
            .collect::<Vec<_>>()
    }
}

#[test]
fn test_fuzzy_sort() {
    let applications = vec![
        Application::new("Firefox".to_string(), "".to_string()),
        Application::new("Chrome".to_string(), "".to_string()),
        Application::new("Visual Studio Code".to_string(), "".to_string()),
        Application::new("File Explorer".to_string(), "".to_string()),
        Application::new("Notepad".to_string(), "".to_string()),
    ];
    let sorter = FuzzySorter::new();
    let query = "e";

    let results = sorter.sort(query, applications);
    println!("Results: {:?}", results);
    assert_eq!(results.len(), 5);
    assert_eq!(results[0].name, "File Explorer");
    assert_eq!(results[1].name, "Firefox");
    assert_eq!(results[2].name, "Chrome");
    assert_eq!(results[3].name, "Visual Studio Code");
    assert_eq!(results[4].name, "Notepad");
}

#[test]
fn test_fuzzy_sort_empty_query() {
    let applications = vec![
        Application::new("Firefox".to_string(), "".to_string()),
        Application::new("Chrome".to_string(), "".to_string()),
        Application::new("Visual Studio Code".to_string(), "".to_string()),
        Application::new("File Explorer".to_string(), "".to_string()),
        Application::new("Notepad".to_string(), "".to_string()),
    ];
    let sorter = FuzzySorter::new();
    let query = "";

    let results = sorter.sort(query, applications);

    assert_eq!(results.len(), 5);
}

#[test]
fn test_fuzzy_sort_no_match() {
    let applications = vec![
        Application::new("Firefox".to_string(), "".to_string()),
        Application::new("Chrome".to_string(), "".to_string()),
        Application::new("Visual Studio Code".to_string(), "".to_string()),
        Application::new("File Explorer".to_string(), "".to_string()),
        Application::new("Notepad".to_string(), "".to_string()),
    ];
    let sorter = FuzzySorter::new();
    let query = "z";

    let results = sorter.sort(query, applications);

    assert_eq!(results.len(), 5);
}
