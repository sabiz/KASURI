use crate::KasuriResult;
use sqlite::{ConnectionThreadSafe, State::Row};
use std::time::{SystemTime, UNIX_EPOCH};

/// Key for storing the last application search timestamp in the database
const STATE_KEY_LAST_APPLICATION_SEARCH_TIME: &str = "last_application_search_time";

/// Repository for Kasuri application state
///
/// This struct provides methods to interact with the application state stored in SQLite database.
/// It handles operations like retrieving and updating application state data.
pub struct KasuriRepository {
    /// Thread-safe SQLite database connection
    connection: sqlite::ConnectionThreadSafe,
}

impl KasuriRepository {
    /// Creates a new KasuriRepository instance with a database connection
    ///
    /// # Arguments
    ///
    /// * `connection` - A thread-safe SQLite connection
    /// * `db_version` - The current database version for migration checks
    ///
    /// # Returns
    ///
    /// A Result containing the repository instance if successful
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection cannot be established or migration fails
    pub fn with_connection(
        connection: ConnectionThreadSafe,
        db_version: u32,
    ) -> KasuriResult<Self> {
        log::debug!(
            "Initializing KasuriRepository with database version {}",
            db_version
        );
        let connection = connection;
        let repository = Self { connection };
        repository.migrate(db_version)?;
        log::debug!("KasuriRepository initialization completed successfully");
        Ok(repository)
    }

    /// Retrieves the last application search time from the database
    ///
    /// # Returns
    ///
    /// A Result containing the timestamp as u64 if successful
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails or the stored value cannot be parsed
    pub fn get_last_application_search_time(&self) -> KasuriResult<u64> {
        log::debug!("Retrieving last application search time");
        let result = self.get_state(STATE_KEY_LAST_APPLICATION_SEARCH_TIME)?;
        let time = result
            .unwrap_or("0".to_string())
            .parse::<u64>()
            .map_err(|e| {
                log::error!("Failed to parse last application search time: {}", e);
                e
            })?;
        log::debug!("Retrieved last application search time: {}", time);
        Ok(time)
    }

    /// Sets the last application search time in the database to the current time
    ///
    /// # Returns
    ///
    /// A Result containing unit type if successful
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub fn set_last_application_search_time(&self) -> KasuriResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get current time")
            .as_secs();
        log::debug!("Setting last application search time to {}", now);
        self.save_state(STATE_KEY_LAST_APPLICATION_SEARCH_TIME, &now.to_string())
    }

    /// Performs database migrations based on the provided version
    ///
    /// # Arguments
    ///
    /// * `db_version` - The current database version
    ///
    /// # Returns
    ///
    /// A Result containing unit type if successful
    ///
    /// # Errors
    ///
    /// Returns an error if any migration step fails
    fn migrate(&self, db_version: u32) -> KasuriResult<()> {
        log::info!("Starting database migration from version {}", db_version);
        if db_version < 1 {
            log::info!("Creating app_state table for version 1");
            // Application state table
            self.connection.execute(
                "CREATE TABLE IF NOT EXISTS app_state (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at INTEGER DEFAULT (unixepoch())
                )",
            )?;
            log::debug!("app_state table created successfully");
        }

        log::info!("Database migration completed successfully");
        Ok(())
    }

    /// Saves the given key-value pair in the app_state table
    ///
    /// # Arguments
    ///
    /// * `key` - The key to store the value under
    /// * `value` - The value to store
    ///
    /// # Returns
    ///
    /// A Result containing unit type if successful
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    fn save_state(&self, key: &str, value: &str) -> KasuriResult<()> {
        log::debug!("Saving state with key '{}' and value '{}'", key, value);
        let mut statement = self
            .connection
            .prepare("INSERT OR REPLACE INTO app_state (key, value) VALUES (?, ?)")?;

        statement.bind((1, key))?;
        statement.bind((2, value))?;

        while let Ok(Row) = statement.next() {}

        log::debug!("State saved successfully for key '{}'", key);
        Ok(())
    }

    /// Retrieves the value associated with the given key from the app_state table
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value for
    ///
    /// # Returns
    ///
    /// A Result containing Option<String> if successful
    /// - None if the key does not exist
    /// - Some(value) if the key exists
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    fn get_state(&self, key: &str) -> KasuriResult<Option<String>> {
        log::debug!("Retrieving state for key '{}'", key);
        let mut statement = self
            .connection
            .prepare("SELECT value FROM app_state WHERE key = ?")?;
        statement.bind((1, key))?;

        if let Ok(Row) = statement.next() {
            let value = statement.read::<String, _>(0)?;
            log::debug!("Retrieved state for key '{}': '{}'", key, value);
            Ok(Some(value))
        } else {
            log::debug!("No state found for key '{}'", key);
            Ok(None)
        }
    }
}
