use crate::model::application::Application;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

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
        applications_with_scores.sort_by(|a, b| b.1.cmp(&a.1));

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

#[test]
fn test_fuzzy_sort() {
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
    let query = "e";

    let results = sorter.sort_with_filter(query, applications);
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

    assert_eq!(results.len(), 5);
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

    assert_eq!(results.len(), 5);
}
