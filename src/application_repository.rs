use crate::application::Application;
use crate::kasuri::KasuriResult;
use sqlite::State::Row;
use std::collections::HashMap;
use std::hash::Hash;

const DB_VERSION: u32 = 1;
const DB_NAME: &str = "kasuri.db";

/// Repository for Application data and statistics
pub struct ApplicationRepository {
    connection: sqlite::ConnectionThreadSafe,
}

impl ApplicationRepository {
    /// Creates a new ApplicationRepository instance
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection cannot be established
    pub fn new() -> KasuriResult<Self> {
        // let repository = Self { con };
        // repository.migrate()?;
        let connection = sqlite::Connection::open_thread_safe(DB_NAME)?;
        let repository = Self { connection };
        repository.migrate()?;
        Ok(repository)
    }

    /// Renews the applications list in the database
    ///
    /// This method will clear all existing applications and insert the new ones
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub fn renew_applications(&self, applications: Vec<Application>) -> KasuriResult<()> {
        let mut hash_map = applications
            .iter()
            .map(|v| (v.app_id.clone(), v))
            .collect::<HashMap<_, _>>();
        let mut delete_applications: Vec<String> = vec![];

        let mut statement = self.connection.prepare("SELECT app_id FROM applications")?;
        while let Ok(Row) = statement.next() {
            let app_id = statement.read::<String, _>(0)?;
            if hash_map.contains_key(&app_id) {
                hash_map.remove(&app_id);
            } else {
                delete_applications.push(app_id.clone());
            }
        }

        let new_applications = hash_map
            .iter()
            .map(|(_, app)| app)
            .collect::<Vec<&&Application>>();

        if delete_applications.len() > 0 {
            println!("Deleting applications: {:?}", delete_applications);
            let param_count_question = (0..delete_applications.len())
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",");
            let mut statement = self.connection.prepare(format!(
                "DELETE FROM applications WHERE app_id in ({});",
                param_count_question
            ))?;
            delete_applications
                .iter()
                .enumerate()
                .for_each(|(i, app_id)| {
                    let _ = statement.bind((i, app_id.as_str()));
                });
            while let Ok(Row) = statement.next() {}
        }

        if new_applications.len() > 0 {
            println!("Inserting new applications: {:?}", new_applications);
            let values_placeholders = (0..new_applications.len())
                .map(|_| "(?, ?, ?)")
                .collect::<Vec<_>>()
                .join(", ");

            let mut statement = self.connection.prepare(format!(
                "INSERT INTO applications (app_id, name, path) VALUES {};",
                values_placeholders
            ))?;

            new_applications.iter().enumerate().for_each(|(i, app)| {
                let _ = statement.bind((i * 3 + 1, app.app_id.as_str()));
                let _ = statement.bind((i * 3 + 2, app.name.as_str()));
                let _ = statement.bind((i * 3 + 3, app.path.as_str()));
            });

            while let Ok(Row) = statement.next() {}
        }

        Ok(())
    }

    fn migrate(&self) -> KasuriResult<()> {
        let version = self.get_db_version()?;
        if version == DB_VERSION {
            return Ok(());
        }

        if version == 0 {
            self.connection.execute(
                "CREATE TABLE IF NOT EXISTS applications (
                    app_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    usage_count INTEGER DEFAULT 0,
                    last_used INTEGER,
                    added_date INTEGER DEFAULT (unixepoch())
                )",
            )?;
            self.connection.execute("PRAGMA user_version = 1")?;
        }

        Ok(())
    }

    fn get_db_version(&self) -> KasuriResult<u32> {
        let mut statement = self.connection.prepare("PRAGMA user_version")?;
        let mut version = 0;

        if let Ok(Row) = statement.next() {
            version = statement.read::<i64, _>(0)? as u32;
        }

        println!("Database version: {}", version);
        Ok(version)
    }
}
