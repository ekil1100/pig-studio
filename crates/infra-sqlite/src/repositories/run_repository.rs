use super::{parse_ts, run_status_from_str, run_status_to_str, ts};
use app_core::RunRepositoryPort;
use chrono::{DateTime, Utc};
use domain::{Run, RunStatus};
use rusqlite::{Connection, params};
use shared_kernel::{AppError, AppResult, RunId, SessionId};
use std::rc::Rc;

#[derive(Clone)]
pub struct RunRepository {
    connection: Rc<Connection>,
}

impl RunRepository {
    pub fn new(connection: Rc<Connection>) -> Self {
        Self { connection }
    }

    pub fn create(&self, run: &Run) -> AppResult<()> {
        self.connection
            .execute(
                "INSERT INTO runs (id, session_id, pimono_run_id, trigger_input, status, started_at, ended_at, error_code, error_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    run.id.as_str(),
                    run.session_id.as_str(),
                    run.pimono_run_id,
                    run.trigger_input,
                    run_status_to_str(run.status),
                    ts(run.started_at),
                    run.ended_at.map(ts),
                    run.error_code,
                    run.error_message,
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn bind_runtime_run(&self, run_id: &RunId, pimono_run_id: &str) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE runs SET pimono_run_id = ?2 WHERE id = ?1",
                params![run_id.as_str(), pimono_run_id],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn update_terminal_state(
        &self,
        run_id: &RunId,
        status: RunStatus,
        ended_at: DateTime<Utc>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE runs SET status = ?2, ended_at = ?3, error_code = ?4, error_message = ?5 WHERE id = ?1",
                params![
                    run_id.as_str(),
                    run_status_to_str(status),
                    ts(ended_at),
                    error_code,
                    error_message,
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn list_by_session(&self, session_id: &SessionId) -> AppResult<Vec<Run>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, session_id, pimono_run_id, trigger_input, status, started_at, ended_at, error_code, error_message
                 FROM runs WHERE session_id = ?1 ORDER BY started_at ASC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map(params![session_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        rows.map(|row| {
            let (
                id,
                session_id,
                pimono_run_id,
                trigger_input,
                status,
                started_at,
                ended_at,
                error_code,
                error_message,
            ) = row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
            Ok(Run {
                id: RunId::from_string(id),
                session_id: SessionId::from_string(session_id),
                pimono_run_id,
                trigger_input,
                status: run_status_from_str(&status)?,
                started_at: parse_ts(started_at)?,
                ended_at: ended_at.map(parse_ts).transpose()?,
                error_code,
                error_message,
            })
        })
        .collect::<AppResult<Vec<Run>>>()
    }

    pub fn list_active(&self) -> AppResult<Vec<Run>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, session_id, pimono_run_id, trigger_input, status, started_at, ended_at, error_code, error_message
                 FROM runs
                 WHERE status IN ('queued', 'running', 'waiting_approval')
                 ORDER BY started_at ASC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        rows.map(|row| {
            let (
                id,
                session_id,
                pimono_run_id,
                trigger_input,
                status,
                started_at,
                ended_at,
                error_code,
                error_message,
            ) = row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
            Ok(Run {
                id: RunId::from_string(id),
                session_id: SessionId::from_string(session_id),
                pimono_run_id,
                trigger_input,
                status: run_status_from_str(&status)?,
                started_at: parse_ts(started_at)?,
                ended_at: ended_at.map(parse_ts).transpose()?,
                error_code,
                error_message,
            })
        })
        .collect::<AppResult<Vec<Run>>>()
    }
}

impl RunRepositoryPort for RunRepository {
    fn create(&self, run: &Run) -> AppResult<()> {
        RunRepository::create(self, run)
    }

    fn bind_runtime_run(&self, run_id: &RunId, pimono_run_id: &str) -> AppResult<()> {
        RunRepository::bind_runtime_run(self, run_id, pimono_run_id)
    }

    fn update_terminal_state(
        &self,
        run_id: &RunId,
        status: RunStatus,
        ended_at: DateTime<Utc>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> AppResult<()> {
        RunRepository::update_terminal_state(
            self,
            run_id,
            status,
            ended_at,
            error_code,
            error_message,
        )
    }

    fn list_active(&self) -> AppResult<Vec<Run>> {
        RunRepository::list_active(self)
    }
}
