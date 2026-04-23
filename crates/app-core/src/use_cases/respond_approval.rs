use crate::{
    event_bus::{ApplicationEvent, EventBusPort},
    ports::{
        ApprovalRepositoryPort, EventRepositoryPort, PiMonoAdapterPort, RespondApprovalRequest,
    },
};
use chrono::{DateTime, Utc};
use domain::{ApprovalDecision, ApprovalStatus};
use serde_json::json;
use shared_kernel::{AppResult, ApprovalId, RunId, SessionId};

pub struct RespondApprovalInput {
    pub approval_id: ApprovalId,
    pub session_id: SessionId,
    pub run_id: RunId,
    pub request: RespondApprovalRequest,
    pub decision: ApprovalDecision,
}

pub fn execute<A, P, E, B>(
    approvals: &A,
    adapter: &P,
    events: &E,
    bus: &B,
    input: RespondApprovalInput,
    now: DateTime<Utc>,
) -> AppResult<()>
where
    A: ApprovalRepositoryPort,
    P: PiMonoAdapterPort,
    E: EventRepositoryPort,
    B: EventBusPort,
{
    let status = match input.decision {
        ApprovalDecision::Approve => ApprovalStatus::Approved,
        ApprovalDecision::Reject => ApprovalStatus::Rejected,
    };

    approvals.record_decision(&input.approval_id, status, input.decision, now)?;
    events.append_event(
        &input.session_id,
        Some(&input.run_id),
        "approval_decision",
        &json!({
            "approval_id": input.approval_id.as_str(),
            "request_id": input.request.request_id,
            "decision": match input.decision {
                ApprovalDecision::Approve => "approve",
                ApprovalDecision::Reject => "reject",
            },
        })
        .to_string(),
        now,
    )?;
    bus.publish(ApplicationEvent::ApprovalUpdated {
        approval_id: input.approval_id,
        status,
    });
    adapter.respond_approval(&input.request)
}
