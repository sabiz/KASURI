use crate::core::kasuri::KasuriResult;
use sqlite::{ConnectionThreadSafe, State::Row};
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_KEY_LAST_APPLICATION_SEARCH_TIME: &str = "last_application_search_time";

/// Repository for Kasuri application state
pub struct KasuriRepository {
    connection: sqlite::ConnectionThreadSafe,
}

impl KasuriRepository {
    /// Creates a new KasuriRepository instance with a database connection
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection cannot be established
    pub fn with_connection(
        connection: ConnectionThreadSafe,
        db_version: u32,
    ) -> KasuriResult<Self> {
        let connection = connection;
        let repository = Self { connection };
        repository.migrate(db_version)?;
        Ok(repository)
    }

    /// Saves the last application search time in the database
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub fn get_last_application_search_time(&self) -> KasuriResult<u64> {
        let result = self.get_state(STATE_KEY_LAST_APPLICATION_SEARCH_TIME)?;
        result
            .unwrap_or("0".to_string())
            .parse::<u64>()
            .map_err(|e| e.into())
    }

    /// Sets the last application search time in the database
    pub fn set_last_application_search_time(&self) -> KasuriResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get current time")
            .as_secs();
        self.save_state(STATE_KEY_LAST_APPLICATION_SEARCH_TIME, &now.to_string())
    }

    fn migrate(&self, db_version: u32) -> KasuriResult<()> {
        if db_version < 1 {
            log::debug!("Create app_state table");
            // Application state table
            self.connection.execute(
                "CREATE TABLE IF NOT EXISTS app_state (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at INTEGER DEFAULT (unixepoch())
                )",
            )?;
        }

        Ok(())
    }

    /// Saves the given key-value pair in the app_state table
    /// # Errors
    /// Returns an error if the database operation fails  
    /// Returns Ok(()) if the operation is successful  
    fn save_state(&self, key: &str, value: &str) -> KasuriResult<()> {
        let mut statement = self
            .connection
            .prepare("INSERT OR REPLACE INTO app_state (key, value) VALUES (?, ?)")?;

        statement.bind((1, key))?;
        statement.bind((2, value))?;

        while let Ok(Row) = statement.next() {}

        Ok(())
    }

    /// Retrieves the value associated with the given key from the app_state table
    /// # Errors
    /// Returns an error if the database operation fails  
    /// Returns Ok(None) if the key does not exist in the table  
    /// Returns Ok(Some(value)) if the key exists and the value is retrieved successfully  
    fn get_state(&self, key: &str) -> KasuriResult<Option<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT value FROM app_state WHERE key = ?")?;
        statement.bind((1, key))?;

        if let Ok(Row) = statement.next() {
            let value = statement.read::<String, _>(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
