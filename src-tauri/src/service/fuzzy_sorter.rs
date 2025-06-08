use crate::model::application::Application;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::cmp::Ordering;

/// Minimum score required for a fuzzy match to be considered relevant.
/// Applications with scores below this threshold will be filtered out.
const MINIMUM_MATCH_SCORE: i64 = 19;

/// Service for fuzzy searching and sorting applications based on name relevance.
///
/// This struct encapsulates the functionality needed to perform fuzzy matching
/// on a collection of applications and sort them by relevance to a search query.
pub struct FuzzySorter {
    /// The fuzzy matcher implementation used for scoring matches
    matcher: SkimMatcherV2,
}

impl FuzzySorter {
    /// Creates a new FuzzySorter instance with default configuration.
    ///
    /// Initializes a new FuzzySorter with the default SkimMatcherV2 matcher.
    ///
    /// # Returns
    ///
    /// A new FuzzySorter instance ready for use in application filtering and sorting.
    pub fn new() -> Self {
        log::debug!("Initializing new FuzzySorter with default matcher");
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Sorts applications based on fuzzy matching against the provided query
    /// and filters out results below a minimum score threshold.
    ///
    /// This method performs the following operations:
    /// 1. Calculates a fuzzy match score for each application name against the query
    /// 2. Sorts applications by descending score (best matches first)
    /// 3. Filters out applications with scores below MINIMUM_MATCH_SCORE
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string to match against application names
    /// * `applications` - A vector of Application objects to sort and filter
    ///
    /// # Returns
    ///
    /// A sorted and filtered vector of Application objects, with best matches first
    pub fn sort_with_filter(
        &self,
        query: &str,
        applications: Vec<Application>,
    ) -> Vec<Application> {
        log::debug!(
            "Performing fuzzy search with query: '{}' on {} applications",
            query,
            applications.len()
        );

        // Calculate fuzzy match scores for each application
        log::debug!("Calculating fuzzy match scores for all applications");
        let mut applications_with_scores: Vec<_> = applications
            .into_iter()
            .map(|app| {
                let score = self.matcher.fuzzy_match(&app.name, query).unwrap_or(0);
                (app, score)
            })
            .collect();

        // Sort applications by score in descending order
        log::debug!("Sorting applications by fuzzy match score");
        applications_with_scores.sort_by(|a, b| match b.1.cmp(&a.1) {
            Ordering::Equal => {
                b.0.usage_recency_score
                    .partial_cmp(&a.0.usage_recency_score)
                    .unwrap_or(Ordering::Equal)
            }
            order => order,
        });

        // Filter and return applications above minimum score threshold
        let initial_count = applications_with_scores.len();
        let filtered_results = applications_with_scores
            .into_iter()
            .filter(|(_, score)| *score > MINIMUM_MATCH_SCORE)
            .map(|(app, score)| {
                log::debug!(
                    "Fuzzy match score for '{}': {} (above threshold {})",
                    app.name,
                    score,
                    MINIMUM_MATCH_SCORE
                );
                app
            })
            // .map(|(app, _)| app)
            .collect::<Vec<Application>>();

        let filtered_count = filtered_results.len();
        log::debug!(
            "Fuzzy search complete: {} of {} applications matched above threshold score",
            filtered_count,
            initial_count
        );

        filtered_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::application::Application;

    #[test]
    fn test_fuzzy_sort() {
        let mut app1 = Application::new("Firefox".to_string(), "".to_string(), "".to_string());
        app1.usage_recency_score = 10.0;
        let mut app2 = Application::new("Chrome".to_string(), "".to_string(), "".to_string());
        app2.usage_recency_score = 30.0;
        let mut app3 = Application::new(
            "Visual Studio Code".to_string(),
            "".to_string(),
            "".to_string(),
        );
        app3.usage_recency_score = 20.0;
        let mut app4 =
            Application::new("File Explorer".to_string(), "".to_string(), "".to_string());
        app4.usage_recency_score = 40.0;
        let mut app5 = Application::new("Notepad".to_string(), "".to_string(), "".to_string());
        app5.usage_recency_score = 50.0;
        let applications = vec![app1, app2, app3, app4, app5];
        let sorter = FuzzySorter::new();
        let query = "e";

        let results = sorter.sort_with_filter(query, applications);
        assert!(results.len() <= 5);
        assert!(
            results
                .iter()
                .all(|app| app.name.contains('e') || app.name.contains('E'))
        );
        for i in 1..results.len() {
            let prev = &results[i - 1];
            let curr = &results[i];
            let prev_score = sorter.matcher.fuzzy_match(&prev.name, query).unwrap_or(0);
            let curr_score = sorter.matcher.fuzzy_match(&curr.name, query).unwrap_or(0);
            if prev_score == curr_score {
                assert!(prev.usage_recency_score >= curr.usage_recency_score);
            } else {
                assert!(prev_score >= curr_score);
            }
        }
    }

    #[test]
    fn test_fuzzy_sort_empty_query() {
        let applications = vec![
            Application::new("Firefox".to_string(), "".to_string(), "".to_string()),
            Application::new("Chrome".to_string(), "".to_string(), "".to_string()),
            Application::new(
                "Visual Studio Code".to_string(),
                "".to_string(),
                "".to_string(),
            ),
            Application::new("File Explorer".to_string(), "".to_string(), "".to_string()),
            Application::new("Notepad".to_string(), "".to_string(), "".to_string()),
        ];
        let sorter = FuzzySorter::new();
        let query = "";

        let results = sorter.sort_with_filter(query, applications);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fuzzy_sort_no_match() {
        let applications = vec![
            Application::new("Firefox".to_string(), "".to_string(), "".to_string()),
            Application::new("Chrome".to_string(), "".to_string(), "".to_string()),
            Application::new(
                "Visual Studio Code".to_string(),
                "".to_string(),
                "".to_string(),
            ),
            Application::new("File Explorer".to_string(), "".to_string(), "".to_string()),
            Application::new("Notepad".to_string(), "".to_string(), "".to_string()),
        ];
        let sorter = FuzzySorter::new();
        let query = "z";

        let results = sorter.sort_with_filter(query, applications);
        assert_eq!(results.len(), 0);
    }
}
