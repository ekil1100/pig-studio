use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{ProjectId, SessionId};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Idle,
    Running,
    WaitingApproval,
    Blocked,
    Completed,
    Failed,
    Interrupted,
}

impl SessionStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Interrupted)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceMode {
    Direct,
    Worktree,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub title: String,
    pub status: SessionStatus,
    pub pimono_session_id: Option<String>,
    pub workspace_cwd: PathBuf,
    pub workspace_mode: WorkspaceMode,
    pub worktree_path: Option<PathBuf>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(
        project_id: ProjectId,
        title: impl Into<String>,
        workspace_cwd: PathBuf,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: SessionId::new(),
            project_id,
            title: title.into(),
            status: SessionStatus::Idle,
            pimono_session_id: None,
            workspace_cwd,
            workspace_mode: WorkspaceMode::Direct,
            worktree_path: None,
            last_run_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SessionStatus;

    #[test]
    fn waiting_approval_is_not_terminal() {
        assert!(!SessionStatus::WaitingApproval.is_terminal());
    }

    #[test]
    fn blocked_is_not_terminal() {
        assert!(!SessionStatus::Blocked.is_terminal());
    }

    #[test]
    fn interrupted_is_terminal() {
        assert!(SessionStatus::Interrupted.is_terminal());
    }

    #[test]
    fn failed_is_terminal() {
        assert!(SessionStatus::Failed.is_terminal());
    }
}
