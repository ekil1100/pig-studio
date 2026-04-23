use crate::event_bus::RuntimeEvent;
use chrono::{DateTime, Utc};
use domain::{ApprovalDecision, ApprovalStatus, Project, Run, RunStatus, Session, WorkspaceMode};
use serde::{Deserialize, Serialize};
use shared_kernel::{AppResult, ApprovalId, RunId, SessionId};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSettings {
    pub runtime_path: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub environment: BTreeMap<String, String>,
    pub last_checked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartSessionRunRequest {
    pub runtime_path: PathBuf,
    pub workspace_cwd: PathBuf,
    pub pimono_session_id: Option<String>,
    pub prompt: String,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumeSessionRequest {
    pub runtime_path: PathBuf,
    pub workspace_cwd: PathBuf,
    pub pimono_session_id: String,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RespondApprovalRequest {
    pub runtime_path: PathBuf,
    pub workspace_cwd: PathBuf,
    pub request_id: String,
    pub approve: bool,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectRunStatusRequest {
    pub runtime_path: PathBuf,
    pub workspace_cwd: PathBuf,
    pub pimono_run_id: String,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunInspection {
    pub running: bool,
    pub terminal_event: Option<RuntimeEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedWorkspace {
    pub cwd: PathBuf,
    pub mode: WorkspaceMode,
    pub worktree_path: Option<PathBuf>,
}

pub trait ProjectRepositoryPort {
    fn create(&self, project: &Project) -> AppResult<()>;
    fn find_by_root_path(&self, root_path: &Path) -> AppResult<Option<Project>>;
}

pub trait SessionRepositoryPort {
    fn create(&self, session: &Session) -> AppResult<()>;
    fn update_status(
        &self,
        session_id: &SessionId,
        status: domain::SessionStatus,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()>;
    fn bind_runtime_session(
        &self,
        session_id: &SessionId,
        pimono_session_id: &str,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()>;
    fn find(&self, session_id: &SessionId) -> AppResult<Option<Session>>;
    fn list_active(&self) -> AppResult<Vec<Session>>;
}

pub trait RunRepositoryPort {
    fn create(&self, run: &Run) -> AppResult<()>;
    fn bind_runtime_run(&self, run_id: &RunId, pimono_run_id: &str) -> AppResult<()>;
    fn update_terminal_state(
        &self,
        run_id: &RunId,
        status: RunStatus,
        ended_at: DateTime<Utc>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> AppResult<()>;
    fn list_active(&self) -> AppResult<Vec<Run>>;
}

pub trait ApprovalRepositoryPort {
    fn create(&self, approval: &domain::Approval) -> AppResult<()>;
    fn record_decision(
        &self,
        approval_id: &ApprovalId,
        status: ApprovalStatus,
        decision: ApprovalDecision,
        decided_at: DateTime<Utc>,
    ) -> AppResult<()>;
}

pub trait EventRepositoryPort {
    fn append_event(
        &self,
        session_id: &SessionId,
        run_id: Option<&RunId>,
        event_type: &str,
        payload_json: &str,
        created_at: DateTime<Utc>,
    ) -> AppResult<()>;
}

pub trait RuntimeEventSink {
    fn push(&self, event: RuntimeEvent) -> AppResult<()>;
}

pub trait PiMonoAdapterPort {
    fn start_session_run(
        &self,
        request: &StartSessionRunRequest,
        sink: &dyn RuntimeEventSink,
    ) -> AppResult<()>;
    fn resume_session(
        &self,
        request: &ResumeSessionRequest,
        sink: &dyn RuntimeEventSink,
    ) -> AppResult<()>;
    fn respond_approval(&self, request: &RespondApprovalRequest) -> AppResult<()>;
    fn inspect_run_status(&self, request: &InspectRunStatusRequest) -> AppResult<RunInspection>;
}

pub trait SettingsStorePort {
    fn load(&self) -> AppResult<RuntimeSettings>;
    fn save(&self, settings: &RuntimeSettings) -> AppResult<()>;
}

pub trait WorkspaceServicePort {
    fn ensure_project_directory(&self, root_path: &Path) -> AppResult<()>;
    fn is_git_repository(&self, root_path: &Path) -> AppResult<bool>;
    fn resolve_workspace(
        &self,
        root_path: &Path,
        prefer_worktree: bool,
    ) -> AppResult<ResolvedWorkspace>;
}

pub fn default_workspace(root_path: &Path) -> ResolvedWorkspace {
    ResolvedWorkspace {
        cwd: root_path.to_path_buf(),
        mode: WorkspaceMode::Direct,
        worktree_path: None,
    }
}
