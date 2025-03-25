use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use crate::application::Application;

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
            }).collect();
        scored_applications.sort_by(|a, b| b.1.cmp(&a.1));
        scored_applications.into_iter().map(|(app, _)| app).collect::<Vec<_>>()
    }
}