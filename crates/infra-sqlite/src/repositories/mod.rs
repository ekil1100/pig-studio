pub mod approval_repository;
pub mod event_repository;
pub mod project_repository;
pub mod run_repository;
pub mod session_repository;

pub use approval_repository::ApprovalRepository;
pub use event_repository::{EventRepository, StoredEvent};
pub use project_repository::ProjectRepository;
pub use run_repository::RunRepository;
pub use session_repository::{RestoredSessionView, SessionRepository};

use chrono::{DateTime, Utc};
use domain::{ApprovalDecision, ApprovalStatus, RunStatus, SessionStatus, WorkspaceMode};
use shared_kernel::AppError;

pub(crate) fn ts(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(crate) fn parse_ts(value: String) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(&value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|error| AppError::Infrastructure(error.to_string()))
}

pub(crate) fn session_status_to_str(value: SessionStatus) -> &'static str {
    match value {
        SessionStatus::Idle => "idle",
        SessionStatus::Running => "running",
        SessionStatus::WaitingApproval => "waiting_approval",
        SessionStatus::Blocked => "blocked",
        SessionStatus::Completed => "completed",
        SessionStatus::Failed => "failed",
        SessionStatus::Interrupted => "interrupted",
    }
}

pub(crate) fn session_status_from_str(value: &str) -> Result<SessionStatus, AppError> {
    match value {
        "idle" => Ok(SessionStatus::Idle),
        "running" => Ok(SessionStatus::Running),
        "waiting_approval" => Ok(SessionStatus::WaitingApproval),
        "blocked" => Ok(SessionStatus::Blocked),
        "completed" => Ok(SessionStatus::Completed),
        "failed" => Ok(SessionStatus::Failed),
        "interrupted" => Ok(SessionStatus::Interrupted),
        other => Err(AppError::Infrastructure(format!(
            "unknown session status: {other}"
        ))),
    }
}

pub(crate) fn workspace_mode_to_str(value: WorkspaceMode) -> &'static str {
    match value {
        WorkspaceMode::Direct => "direct",
        WorkspaceMode::Worktree => "worktree",
    }
}

pub(crate) fn workspace_mode_from_str(value: &str) -> Result<WorkspaceMode, AppError> {
    match value {
        "direct" => Ok(WorkspaceMode::Direct),
        "worktree" => Ok(WorkspaceMode::Worktree),
        other => Err(AppError::Infrastructure(format!(
            "unknown workspace mode: {other}"
        ))),
    }
}

pub(crate) fn run_status_to_str(value: RunStatus) -> &'static str {
    match value {
        RunStatus::Queued => "queued",
        RunStatus::Running => "running",
        RunStatus::WaitingApproval => "waiting_approval",
        RunStatus::Completed => "completed",
        RunStatus::Failed => "failed",
        RunStatus::Interrupted => "interrupted",
    }
}

pub(crate) fn run_status_from_str(value: &str) -> Result<RunStatus, AppError> {
    match value {
        "queued" => Ok(RunStatus::Queued),
        "running" => Ok(RunStatus::Running),
        "waiting_approval" => Ok(RunStatus::WaitingApproval),
        "completed" => Ok(RunStatus::Completed),
        "failed" => Ok(RunStatus::Failed),
        "interrupted" => Ok(RunStatus::Interrupted),
        other => Err(AppError::Infrastructure(format!(
            "unknown run status: {other}"
        ))),
    }
}

pub(crate) fn approval_status_to_str(value: ApprovalStatus) -> &'static str {
    match value {
        ApprovalStatus::Pending => "pending",
        ApprovalStatus::Approved => "approved",
        ApprovalStatus::Rejected => "rejected",
        ApprovalStatus::Expired => "expired",
        ApprovalStatus::Interrupted => "interrupted",
    }
}

pub(crate) fn approval_status_from_str(value: &str) -> Result<ApprovalStatus, AppError> {
    match value {
        "pending" => Ok(ApprovalStatus::Pending),
        "approved" => Ok(ApprovalStatus::Approved),
        "rejected" => Ok(ApprovalStatus::Rejected),
        "expired" => Ok(ApprovalStatus::Expired),
        "interrupted" => Ok(ApprovalStatus::Interrupted),
        other => Err(AppError::Infrastructure(format!(
            "unknown approval status: {other}"
        ))),
    }
}

pub(crate) fn approval_decision_to_str(value: ApprovalDecision) -> &'static str {
    match value {
        ApprovalDecision::Approve => "approve",
        ApprovalDecision::Reject => "reject",
    }
}

pub(crate) fn approval_decision_from_str(value: &str) -> Result<ApprovalDecision, AppError> {
    match value {
        "approve" => Ok(ApprovalDecision::Approve),
        "reject" => Ok(ApprovalDecision::Reject),
        other => Err(AppError::Infrastructure(format!(
            "unknown approval decision: {other}"
        ))),
    }
}
