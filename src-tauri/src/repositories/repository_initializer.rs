//! Repository initializer module.
//!
//! This module provides functionality for initializing database repositories
//! and managing database versions for the KASURI application.

use crate::core::kasuri::KasuriResult;
use crate::repositories::application_repository::ApplicationRepository;
use crate::repositories::kasuri_repository::KasuriRepository;
use sqlite::Connection;
use sqlite::State::Row;

/// Name of the SQLite database file
const DB_NAME: &str = "kasuri.db";
/// Current database schema version
const DB_VERSION: u32 = 1;

/// Repository initializer responsible for setting up and managing database connections
///
/// This struct handles the initialization of repositories and database version management.
pub struct RepositoryInitializer {}

/// Container for all application repositories
///
/// This struct holds instances of all available repositories in the application.
pub struct Repositories {
    /// Repository for KASURI core functionality
    pub kasuri_repository: KasuriRepository,
    /// Repository for application-related operations
    pub application_repository: ApplicationRepository,
}

impl RepositoryInitializer {
    /// Creates a new instance of the RepositoryInitializer
    ///
    /// # Returns
    ///
    /// A new RepositoryInitializer instance
    pub fn new() -> Self {
        log::debug!("Creating new RepositoryInitializer instance");
        Self {}
    }

    /// Initializes and returns all application repositories
    ///
    /// This method:
    /// 1. Checks the current database version
    /// 2. Initializes all repositories with database connections
    /// 3. Updates the database version if needed
    ///
    /// # Returns
    ///
    /// * `KasuriResult<Repositories>` - A Result containing the initialized repositories or an error
    pub fn get_repositories(&self) -> KasuriResult<Repositories> {
        log::info!(
            "Initializing application repositories with database: {}",
            DB_NAME
        );

        // Open connection for version check
        log::debug!("Opening database connection for version check");
        let connection = sqlite::Connection::open_thread_safe(DB_NAME)?;

        // Get current database version
        let db_version = self.get_db_version(&connection)?;
        log::info!(
            "Database version check completed: current={}, required={}",
            db_version,
            DB_VERSION
        );

        // Initialize KasuriRepository
        log::debug!("Initializing KasuriRepository");
        let kasuri_repository = KasuriRepository::with_connection(connection, db_version)?;

        // Initialize ApplicationRepository with a new connection
        log::debug!("Opening database connection for ApplicationRepository");
        let connection = sqlite::Connection::open_thread_safe(DB_NAME)?;
        log::debug!("Initializing ApplicationRepository");
        let application_repository =
            ApplicationRepository::with_connection(connection, db_version)?;

        // Update database version if needed
        log::debug!("Opening database connection for version update check");
        let connection = sqlite::Connection::open_thread_safe(DB_NAME)?;
        if db_version < DB_VERSION {
            self.update_db_version(&connection)?;
        }

        // Create repositories container
        let repositories = Repositories {
            kasuri_repository,
            application_repository,
        };

        log::info!("All repositories successfully initialized");
        Ok(repositories)
    }

    /// Retrieves the current version of the database
    ///
    /// This method queries the SQLite user_version pragma to determine
    /// the current version of the database schema.
    ///
    /// # Arguments
    ///
    /// * `connection` - A reference to an active SQLite connection
    ///
    /// # Returns
    ///
    /// * `KasuriResult<u32>` - The current database version or an error
    fn get_db_version(&self, connection: &Connection) -> KasuriResult<u32> {
        log::debug!("Querying database version using PRAGMA user_version");
        let mut statement = connection.prepare("PRAGMA user_version")?;
        let mut version = 0;
        if let Ok(Row) = statement.next() {
            version = statement.read::<i64, _>(0)? as u32;
            log::debug!("Successfully read database version: {}", version);
        } else {
            log::warn!(
                "Failed to read database version, using default value: {}",
                version
            );
        }

        log::info!(
            "Database version check result: current={}, required={}",
            version,
            DB_VERSION
        );

        Ok(version)
    }

    /// Updates the database version to the current application version
    ///
    /// This method updates the SQLite user_version pragma to match
    /// the current application's required database version.
    ///
    /// # Arguments
    ///
    /// * `connection` - A reference to an active SQLite connection
    ///
    /// # Returns
    ///
    /// * `KasuriResult<()>` - Success or an error
    fn update_db_version(&self, connection: &Connection) -> KasuriResult<()> {
        log::info!(
            "Updating database version from previous version to {}",
            DB_VERSION
        );

        let sql = format!("PRAGMA user_version = {}", DB_VERSION);
        log::debug!("Executing SQL: {}", sql);

        match connection.execute(&sql) {
            Ok(_) => {
                log::info!("Database version successfully updated to {}", DB_VERSION);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to update database version: {}", e);
                Err(e.into())
            }
        }
    }
}
