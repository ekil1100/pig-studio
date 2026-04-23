use super::{parse_ts, ts};
use app_core::ProjectRepositoryPort;
use chrono::{DateTime, Utc};
use domain::Project;
use rusqlite::{Connection, params};
use shared_kernel::{AppError, AppResult, ProjectId};
use std::{path::Path, rc::Rc};

#[derive(Clone)]
pub struct ProjectRepository {
    connection: Rc<Connection>,
}

impl ProjectRepository {
    pub fn new(connection: Rc<Connection>) -> Self {
        Self { connection }
    }

    pub fn create(&self, project: &Project) -> AppResult<()> {
        self.connection
            .execute(
                "INSERT INTO projects (id, name, root_path, pinned, last_opened_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    project.id.as_str(),
                    project.name,
                    project.root_path.to_string_lossy(),
                    i64::from(project.pinned),
                    project.last_opened_at.map(ts),
                    ts(project.created_at),
                    ts(project.updated_at),
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn find_by_root_path(&self, root_path: &Path) -> AppResult<Option<Project>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, name, root_path, pinned, last_opened_at, created_at, updated_at
                 FROM projects WHERE root_path = ?1",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let mut rows = statement
            .query(params![root_path.to_string_lossy().to_string()])
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        match rows
            .next()
            .map_err(|error| AppError::Infrastructure(error.to_string()))?
        {
            Some(row) => Ok(Some(Project {
                id: ProjectId::from_string(
                    row.get::<_, String>(0)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                ),
                name: row
                    .get(1)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                root_path: row
                    .get::<_, String>(2)
                    .map(std::path::PathBuf::from)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                pinned: row
                    .get::<_, i64>(3)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?
                    == 1,
                last_opened_at: row
                    .get::<_, Option<String>>(4)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?
                    .map(parse_ts)
                    .transpose()?,
                created_at: parse_ts(
                    row.get(5)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                )?,
                updated_at: parse_ts(
                    row.get(6)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                )?,
            })),
            None => Ok(None),
        }
    }

    pub fn list(&self) -> AppResult<Vec<Project>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, name, root_path, pinned, last_opened_at, created_at, updated_at
                 FROM projects ORDER BY pinned DESC, last_opened_at DESC, created_at DESC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        rows.map(|row| {
            let (id, name, root_path, pinned, last_opened_at, created_at, updated_at) =
                row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
            Ok(Project {
                id: ProjectId::from_string(id),
                name,
                root_path: root_path.into(),
                pinned: pinned == 1,
                last_opened_at: last_opened_at.map(parse_ts).transpose()?,
                created_at: parse_ts(created_at)?,
                updated_at: parse_ts(updated_at)?,
            })
        })
        .collect::<AppResult<Vec<Project>>>()
    }

    pub fn mark_opened(&self, project_id: &ProjectId, opened_at: DateTime<Utc>) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE projects SET last_opened_at = ?2, updated_at = ?2 WHERE id = ?1",
                params![project_id.as_str(), ts(opened_at)],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }
}

impl ProjectRepositoryPort for ProjectRepository {
    fn create(&self, project: &Project) -> AppResult<()> {
        ProjectRepository::create(self, project)
    }

    fn find_by_root_path(&self, root_path: &Path) -> AppResult<Option<Project>> {
        ProjectRepository::find_by_root_path(self, root_path)
    }
}
