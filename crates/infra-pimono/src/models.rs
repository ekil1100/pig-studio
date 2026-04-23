use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PiMonoEvent {
    SessionBound {
        pimono_session_id: String,
    },
    RunStarted {
        pimono_run_id: String,
        pimono_session_id: Option<String>,
    },
    TextDelta {
        text: String,
    },
    ApprovalRequested {
        request_id: String,
        request_type: String,
        payload_json: String,
    },
    RunFailed {
        code: Option<String>,
        message: String,
    },
    RunCompleted,
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
    pub terminal_event: Option<PiMonoEvent>,
}
