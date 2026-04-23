use super::{parse_ts, ts};
use app_core::EventRepositoryPort;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use shared_kernel::{AppError, AppResult, RunId, SessionId};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredEvent {
    pub id: i64,
    pub session_id: SessionId,
    pub run_id: Option<RunId>,
    pub seq: i64,
    pub event_type: String,
    pub payload_json: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventRepository {
    connection: Rc<Connection>,
}

impl EventRepository {
    pub fn new(connection: Rc<Connection>) -> Self {
        Self { connection }
    }

    pub fn append(
        &self,
        session_id: &SessionId,
        run_id: Option<&RunId>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> AppResult<StoredEvent> {
        let next_seq: i64 = self
            .connection
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM events WHERE session_id = ?1",
                params![session_id.as_str()],
                |row| row.get(0),
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let event_type = event_type.into();
        let payload_json = payload_json.into();

        self.connection
            .execute(
                "INSERT INTO events (session_id, run_id, seq, event_type, payload_json, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    session_id.as_str(),
                    run_id.map(|value| value.as_str().to_string()),
                    next_seq,
                    &event_type,
                    &payload_json,
                    ts(created_at),
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        Ok(StoredEvent {
            id: self.connection.last_insert_rowid(),
            session_id: session_id.clone(),
            run_id: run_id.cloned(),
            seq: next_seq,
            event_type,
            payload_json,
            created_at,
        })
    }

    pub fn list_by_session(&self, session_id: &SessionId) -> AppResult<Vec<StoredEvent>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, session_id, run_id, seq, event_type, payload_json, created_at
                 FROM events WHERE session_id = ?1 ORDER BY seq ASC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map(params![session_id.as_str()], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        rows.map(|row| {
            let (id, session_id, run_id, seq, event_type, payload_json, created_at) =
                row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
            Ok(StoredEvent {
                id,
                session_id: SessionId::from_string(session_id),
                run_id: run_id.map(RunId::from_string),
                seq,
                event_type,
                payload_json,
                created_at: parse_ts(created_at)?,
            })
        })
        .collect::<AppResult<Vec<StoredEvent>>>()
    }
}

impl EventRepositoryPort for EventRepository {
    fn append_event(
        &self,
        session_id: &SessionId,
        run_id: Option<&RunId>,
        event_type: &str,
        payload_json: &str,
        created_at: DateTime<Utc>,
    ) -> AppResult<()> {
        EventRepository::append(
            self,
            session_id,
            run_id,
            event_type,
            payload_json,
            created_at,
        )
        .map(|_| ())
    }
}
