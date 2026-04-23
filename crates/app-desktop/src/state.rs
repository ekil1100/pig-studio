use domain::RuntimeHealth;
use ui_components::{
    ApprovalCardView, ProjectTreeItemView, SessionBannerView, StatusBadgeMeta, TimelineEntryView,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoticeTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoticeState {
    pub tone: NoticeTone,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveSessionState {
    pub project_id: String,
    pub project_name: String,
    pub session_id: String,
    pub session_name: String,
    pub status_badge: StatusBadgeMeta,
    pub banner: SessionBannerView,
    pub timeline: Vec<TimelineEntryView>,
    pub approvals: Vec<ApprovalCardView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceState {
    pub projects: Vec<ProjectTreeItemView>,
    pub active_project_id: Option<String>,
    pub active_session_id: Option<String>,
    pub active_session: Option<ActiveSessionState>,
    pub settings_open: bool,
    pub runtime_path: String,
    pub runtime_source_label: String,
    pub config_dir: String,
    pub config_dir_source_label: String,
    pub has_runtime_override: bool,
    pub has_config_dir_override: bool,
    pub runtime_health: RuntimeHealth,
    pub notice: Option<NoticeState>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            active_project_id: None,
            active_session_id: None,
            active_session: None,
            settings_open: false,
            runtime_path: String::new(),
            runtime_source_label: "未检测到".into(),
            config_dir: String::new(),
            config_dir_source_label: "未检测到".into(),
            has_runtime_override: false,
            has_config_dir_override: false,
            runtime_health: RuntimeHealth {
                available: false,
                version: None,
                reason: Some("runtime not checked".into()),
                checked_at: None,
            },
            notice: None,
        }
    }
}
