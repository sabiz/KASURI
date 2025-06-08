use crate::core::kasuri::KasuriResult;
use crate::model::application::Application;
use sqlite::ConnectionThreadSafe;
use sqlite::State::Row;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ApplicationRepositoryRecord {
    /// Unique identifier for the application
    pub app_id: String,
    /// Name of the application
    pub name: String,
    /// Path to the application executable
    pub path: String,
    /// Number of times the application has been used
    pub usage_count: i64,
    /// Timestamp of the last time the application was used
    pub last_used: i64,
}

/// Repository for Application data and statistics
///
/// This repository manages the storage and retrieval of application data in the SQLite database.
/// It provides methods for retrieving, adding, updating, and deleting application records.
pub struct ApplicationRepository {
    /// SQLite database connection used for all database operations
    connection: ConnectionThreadSafe,
}

impl ApplicationRepository {
    /// Creates a new ApplicationRepository instance with a database connection
    ///
    /// This method initializes the repository and performs any necessary database migrations
    /// based on the provided database version.
    ///
    /// # Arguments
    ///
    /// * `connection` - An established SQLite database connection
    /// * `db_version` - The current database schema version
    ///
    /// # Returns
    ///
    /// A new instance of ApplicationRepository wrapped in KasuriResult
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection cannot be established or if database migration fails
    pub fn with_connection(
        connection: ConnectionThreadSafe,
        db_version: u32,
    ) -> KasuriResult<Self> {
        let repository = Self { connection };
        repository.migrate(db_version)?;
        Ok(repository)
    }

    /// Renews the applications list in the database
    ///
    /// This method compares the provided applications list with the existing records in the database.
    /// It will delete applications that are no longer present and insert new applications that
    /// weren't previously in the database. Applications that exist in both lists remain unchanged.
    ///
    /// # Arguments
    ///
    /// * `applications` - A vector of Application objects to synchronize with the database
    ///
    /// # Returns
    ///
    /// A vector of newly added Application objects wrapped in KasuriResult
    ///
    /// # Errors
    ///
    /// Returns an error if any database operation fails (prepare, bind, insert, delete)
    pub fn renew_applications(
        &self,
        applications: Vec<Application>,
    ) -> KasuriResult<Vec<Application>> {
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
            .map(|(_, app)| (**app).clone())
            .collect::<Vec<Application>>();

        if delete_applications.len() > 0 {
            log::info!(
                "Deleting {} applications from database: {:?}",
                delete_applications.len(),
                delete_applications
            );
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
            log::info!(
                "Inserting {} new applications into database: {:?}",
                new_applications.len(),
                new_applications
            );
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

        Ok(new_applications)
    }

    /// Retrieves all applications stored in the database
    ///
    /// This method fetches all application records from the database and returns them as a vector.
    ///
    /// # Returns
    ///
    /// A vector of Application objects wrapped in KasuriResult
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails or if any row cannot be read
    pub fn get_applications(&self) -> KasuriResult<Vec<Application>> {
        let mut applications = vec![];
        log::debug!("Retrieving all applications from database");
        let mut statement = self
            .connection
            .prepare("SELECT app_id, name, path, usage_count, last_used FROM applications")?;
        while let Ok(Row) = statement.next() {
            let app_id = statement.read::<String, _>(0)?;
            let name = statement.read::<String, _>(1)?;
            let path = statement.read::<String, _>(2)?;
            let usage_count = statement.read::<i64, _>(3)?;
            let last_used = statement.read::<i64, _>(4)?;
            log::debug!(
                "Retrieved application: app_id={}, name={}, path={}, usage_count={}, last_used={}",
                app_id,
                name,
                path,
                usage_count,
                last_used
            );

            applications.push(
                (ApplicationRepositoryRecord {
                    app_id,
                    name,
                    path,
                    usage_count,
                    last_used,
                })
                .into(),
            );
        }
        log::debug!(
            "Retrieved {} applications from database",
            applications.len()
        );
        Ok(applications)
    }

    pub fn update_usage(&self, application: &Application) -> KasuriResult<()> {
        log::debug!(
            "Updating usage for application: app_id={},",
            application.app_id
        );
        if application.app_id.is_empty() {
            log::warn!("Cannot update usage for application with empty app_id");
            return Ok(());
        }
        let mut statement = self.connection.prepare(
            "UPDATE applications SET usage_count = usage_count + 1, last_used = (unixepoch()) WHERE app_id = ?",
        )?;
        statement.bind((1, application.app_id.as_str()))?;
        while let Ok(Row) = statement.next() {
            log::debug!(
                "Updated successfully usage for application: app_id={}",
                application.app_id
            );
        }
        Ok(())
    }

    /// Performs database migrations to ensure the schema is up to date
    ///
    /// This method checks the current database version and applies any necessary
    /// schema changes to bring the database structure up to the latest version.
    ///
    /// # Arguments
    ///
    /// * `db_version` - The current database schema version
    ///
    /// # Returns
    ///
    /// Unit type wrapped in KasuriResult indicating success or failure
    ///
    /// # Errors
    ///
    /// Returns an error if any database migration operation fails
    fn migrate(&self, db_version: u32) -> KasuriResult<()> {
        if db_version < 1 {
            log::debug!(
                "Creating applications table in database as part of migration to version 1"
            );
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
        }

        log::debug!(
            "Database migration completed successfully to version {}",
            db_version
        );
        Ok(())
    }
}
