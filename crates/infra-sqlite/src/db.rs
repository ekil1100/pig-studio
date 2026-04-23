use crate::{
    migrations::INITIAL_MIGRATION,
    repositories::{
        ApprovalRepository, EventRepository, ProjectRepository, RunRepository, SessionRepository,
    },
};
use rusqlite::Connection;
use shared_kernel::{AppError, AppResult};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

#[derive(Clone)]
pub struct Database {
    connection: Rc<Connection>,
    path: Option<PathBuf>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();
        let connection =
            Connection::open(&path).map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Self::from_connection(connection, Some(path))
    }

    pub fn in_memory() -> AppResult<Self> {
        let connection = Connection::open_in_memory()
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Self::from_connection(connection, None)
    }

    fn from_connection(connection: Connection, path: Option<PathBuf>) -> AppResult<Self> {
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        if path.is_some() {
            connection
                .execute_batch("PRAGMA journal_mode = WAL;")
                .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        }
        connection
            .execute_batch(INITIAL_MIGRATION)
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        Ok(Self {
            connection: Rc::new(connection),
            path,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn connection(&self) -> Rc<Connection> {
        Rc::clone(&self.connection)
    }

    pub fn project_repository(&self) -> ProjectRepository {
        ProjectRepository::new(self.connection())
    }

    pub fn session_repository(&self) -> SessionRepository {
        SessionRepository::new(self.connection())
    }

    pub fn run_repository(&self) -> RunRepository {
        RunRepository::new(self.connection())
    }

    pub fn approval_repository(&self) -> ApprovalRepository {
        ApprovalRepository::new(self.connection())
    }

    pub fn event_repository(&self) -> EventRepository {
        EventRepository::new(self.connection())
    }
}
