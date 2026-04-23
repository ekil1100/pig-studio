use super::{
    approval_status_from_str, parse_ts, session_status_from_str, session_status_to_str, ts,
    workspace_mode_from_str, workspace_mode_to_str,
};
use app_core::SessionRepositoryPort;
use chrono::{DateTime, Utc};
use domain::{Approval, ApprovalRequest, Session, SessionStatus};
use rusqlite::{Connection, params};
use shared_kernel::{AppError, AppResult, ApprovalId, ProjectId, RunId, SessionId};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoredSessionView {
    pub session: Session,
    pub pending_approvals: Vec<Approval>,
}

#[derive(Clone)]
pub struct SessionRepository {
    connection: Rc<Connection>,
}

impl SessionRepository {
    pub fn new(connection: Rc<Connection>) -> Self {
        Self { connection }
    }

    pub fn create(&self, session: &Session) -> AppResult<()> {
        self.connection
            .execute(
                "INSERT INTO sessions (id, project_id, title, status, pimono_session_id, workspace_cwd, workspace_mode, worktree_path, last_run_at, created_at, updated_at, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, NULL)",
                params![
                    session.id.as_str(),
                    session.project_id.as_str(),
                    session.title,
                    session_status_to_str(session.status),
                    session.pimono_session_id,
                    session.workspace_cwd.to_string_lossy(),
                    workspace_mode_to_str(session.workspace_mode),
                    session.worktree_path.as_ref().map(|path| path.to_string_lossy().to_string()),
                    session.last_run_at.map(ts),
                    ts(session.created_at),
                    ts(session.updated_at),
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn update_status(
        &self,
        session_id: &SessionId,
        status: SessionStatus,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE sessions SET status = ?2, updated_at = ?3 WHERE id = ?1",
                params![
                    session_id.as_str(),
                    session_status_to_str(status),
                    ts(updated_at)
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn list_by_project(&self, project_id: &ProjectId) -> AppResult<Vec<Session>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, project_id, title, status, pimono_session_id, workspace_cwd, workspace_mode, worktree_path, last_run_at, created_at, updated_at
                 FROM sessions WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC, created_at DESC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map(params![project_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        rows.map(|row| {
            let (
                id,
                project_id,
                title,
                status,
                pimono_session_id,
                workspace_cwd,
                workspace_mode,
                worktree_path,
                last_run_at,
                created_at,
                updated_at,
            ) = row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
            Ok(Session {
                id: SessionId::from_string(id),
                project_id: ProjectId::from_string(project_id),
                title,
                status: session_status_from_str(&status)?,
                pimono_session_id,
                workspace_cwd: workspace_cwd.into(),
                workspace_mode: workspace_mode_from_str(&workspace_mode)?,
                worktree_path: worktree_path.map(Into::into),
                last_run_at: last_run_at.map(parse_ts).transpose()?,
                created_at: parse_ts(created_at)?,
                updated_at: parse_ts(updated_at)?,
            })
        })
        .collect::<AppResult<Vec<Session>>>()
    }

    pub fn rename(
        &self,
        session_id: &SessionId,
        title: &str,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE sessions SET title = ?2, updated_at = ?3 WHERE id = ?1",
                params![session_id.as_str(), title, ts(updated_at)],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn soft_delete(&self, session_id: &SessionId, deleted_at: DateTime<Utc>) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE sessions SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
                params![session_id.as_str(), ts(deleted_at)],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn bind_runtime_session(
        &self,
        session_id: &SessionId,
        pimono_session_id: &str,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE sessions SET pimono_session_id = ?2, updated_at = ?3 WHERE id = ?1",
                params![session_id.as_str(), pimono_session_id, ts(updated_at)],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn mark_run_started(
        &self,
        session_id: &SessionId,
        started_at: DateTime<Utc>,
    ) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE sessions SET last_run_at = ?2, updated_at = ?2 WHERE id = ?1",
                params![session_id.as_str(), ts(started_at)],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn restore_view(&self, session_id: &SessionId) -> AppResult<Option<RestoredSessionView>> {
        let session = self.find(session_id)?;
        let Some(session) = session else {
            return Ok(None);
        };

        let mut statement = self
            .connection
            .prepare(
                "SELECT a.id, a.run_id, a.request_id, a.correlation_id, a.request_type, a.request_payload_json, a.status, a.decision, a.created_at, a.decided_at
                 FROM approvals a
                 INNER JOIN runs r ON r.id = a.run_id
                 WHERE r.session_id = ?1 AND a.status = 'pending'
                 ORDER BY a.created_at ASC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map(params![session_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let pending_approvals = rows
            .map(|row| {
                let (
                    id,
                    run_id,
                    request_id,
                    correlation_id,
                    request_type,
                    request_payload_json,
                    status,
                    decision,
                    created_at,
                    decided_at,
                ) = row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
                Ok(Approval {
                    id: ApprovalId::from_string(id),
                    run_id: RunId::from_string(run_id),
                    request: ApprovalRequest {
                        request_id,
                        correlation_id,
                        request_type,
                        request_payload_json,
                    },
                    status: approval_status_from_str(&status)?,
                    decision: decision
                        .map(|_| unreachable!("pending approvals never have a decision")),
                    created_at: parse_ts(created_at)?,
                    decided_at: decided_at.map(parse_ts).transpose()?,
                })
            })
            .collect::<AppResult<Vec<Approval>>>()?;

        Ok(Some(RestoredSessionView {
            session,
            pending_approvals,
        }))
    }

    pub fn find(&self, session_id: &SessionId) -> AppResult<Option<Session>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, project_id, title, status, pimono_session_id, workspace_cwd, workspace_mode, worktree_path, last_run_at, created_at, updated_at
                 FROM sessions WHERE id = ?1 AND deleted_at IS NULL",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let mut rows = statement
            .query(params![session_id.as_str()])
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        match rows
            .next()
            .map_err(|error| AppError::Infrastructure(error.to_string()))?
        {
            Some(row) => Ok(Some(Session {
                id: SessionId::from_string(
                    row.get::<_, String>(0)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                ),
                project_id: ProjectId::from_string(
                    row.get::<_, String>(1)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                ),
                title: row
                    .get(2)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                status: session_status_from_str(
                    &row.get::<_, String>(3)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                )?,
                pimono_session_id: row
                    .get(4)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                workspace_cwd: row
                    .get::<_, String>(5)
                    .map(std::path::PathBuf::from)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                workspace_mode: workspace_mode_from_str(
                    &row.get::<_, String>(6)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                )?,
                worktree_path: row
                    .get::<_, Option<String>>(7)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?
                    .map(Into::into),
                last_run_at: row
                    .get::<_, Option<String>>(8)
                    .map_err(|error| AppError::Infrastructure(error.to_string()))?
                    .map(parse_ts)
                    .transpose()?,
                created_at: parse_ts(
                    row.get(9)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                )?,
                updated_at: parse_ts(
                    row.get(10)
                        .map_err(|error| AppError::Infrastructure(error.to_string()))?,
                )?,
            })),
            None => Ok(None),
        }
    }

    pub fn list_active(&self) -> AppResult<Vec<Session>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, project_id, title, status, pimono_session_id, workspace_cwd, workspace_mode, worktree_path, last_run_at, created_at, updated_at
                 FROM sessions
                 WHERE deleted_at IS NULL AND status IN ('running', 'waiting_approval')
                 ORDER BY updated_at DESC, created_at DESC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        rows.map(|row| {
            let (
                id,
                project_id,
                title,
                status,
                pimono_session_id,
                workspace_cwd,
                workspace_mode,
                worktree_path,
                last_run_at,
                created_at,
                updated_at,
            ) = row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
            Ok(Session {
                id: SessionId::from_string(id),
                project_id: ProjectId::from_string(project_id),
                title,
                status: session_status_from_str(&status)?,
                pimono_session_id,
                workspace_cwd: workspace_cwd.into(),
                workspace_mode: workspace_mode_from_str(&workspace_mode)?,
                worktree_path: worktree_path.map(Into::into),
                last_run_at: last_run_at.map(parse_ts).transpose()?,
                created_at: parse_ts(created_at)?,
                updated_at: parse_ts(updated_at)?,
            })
        })
        .collect::<AppResult<Vec<Session>>>()
    }
}

impl SessionRepositoryPort for SessionRepository {
    fn create(&self, session: &Session) -> AppResult<()> {
        SessionRepository::create(self, session)
    }

    fn update_status(
        &self,
        session_id: &SessionId,
        status: SessionStatus,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        SessionRepository::update_status(self, session_id, status, updated_at)
    }

    fn bind_runtime_session(
        &self,
        session_id: &SessionId,
        pimono_session_id: &str,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        SessionRepository::bind_runtime_session(self, session_id, pimono_session_id, updated_at)
    }

    fn find(&self, session_id: &SessionId) -> AppResult<Option<Session>> {
        SessionRepository::find(self, session_id)
    }

    fn list_active(&self) -> AppResult<Vec<Session>> {
        SessionRepository::list_active(self)
    }
}
