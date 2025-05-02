use crate::core::kasuri::KasuriResult;
use crate::repositories::application_repository::ApplicationRepository;
use crate::repositories::kasuri_repository::KasuriRepository;
use sqlite::Connection;
use sqlite::State::Row;

const DB_NAME: &str = "kasuri.db";
const DB_VERSION: u32 = 1;

pub struct RepositoryInitializer {}

pub struct Repositories {
    pub kasuri_repository: KasuriRepository,
    pub application_repository: ApplicationRepository,
}

impl RepositoryInitializer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_repositories(&self) -> KasuriResult<Repositories> {
        let connection = sqlite::Connection::open_thread_safe(DB_NAME)?;
        let db_version = Self::get_db_version(&connection)?;
        let kasuri_repository = KasuriRepository::with_connection(connection, db_version)?;
        let connection = sqlite::Connection::open_thread_safe(DB_NAME)?;
        let application_repository =
            ApplicationRepository::with_connection(connection, db_version)?;
        let connection = sqlite::Connection::open_thread_safe(DB_NAME)?;
        Self::update_db_version(&connection)?;
        let repositories = Repositories {
            kasuri_repository,
            application_repository,
        };
        Ok(repositories)
    }

    fn get_db_version(connection: &Connection) -> KasuriResult<u32> {
        let mut statement = connection.prepare("PRAGMA user_version")?;
        let mut version = 0;
        if let Ok(Row) = statement.next() {
            version = statement.read::<i64, _>(0)? as u32;
        }
        Ok(version)
    }

    fn update_db_version(connection: &Connection) -> KasuriResult<()> {
        connection.execute(format!("PRAGMA user_version = {}", DB_VERSION))?;
        Ok(())
    }
}
