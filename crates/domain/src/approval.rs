use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{ApprovalId, RunId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approve,
    Reject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub request_type: String,
    pub request_payload_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approval {
    pub id: ApprovalId,
    pub run_id: RunId,
    pub request: ApprovalRequest,
    pub status: ApprovalStatus,
    pub decision: Option<ApprovalDecision>,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

impl Approval {
    pub fn new(run_id: RunId, request: ApprovalRequest, created_at: DateTime<Utc>) -> Self {
        Self {
            id: ApprovalId::new(),
            run_id,
            request,
            status: ApprovalStatus::Pending,
            decision: None,
            created_at,
            decided_at: None,
        }
    }
}
