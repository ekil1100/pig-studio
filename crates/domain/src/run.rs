use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{RunId, SessionId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Queued,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Interrupted,
}

impl RunStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Interrupted)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    pub id: RunId,
    pub session_id: SessionId,
    pub pimono_run_id: Option<String>,
    pub trigger_input: String,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

impl Run {
    pub fn new(
        session_id: SessionId,
        trigger_input: impl Into<String>,
        started_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: RunId::new(),
            session_id,
            pimono_run_id: None,
            trigger_input: trigger_input.into(),
            status: RunStatus::Queued,
            started_at,
            ended_at: None,
            error_code: None,
            error_message: None,
        }
    }
}
