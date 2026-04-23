use crate::{
    event_bus::{ApplicationEvent, EventBusPort, RuntimeEvent},
    ports::{
        ApprovalRepositoryPort, EventRepositoryPort, PiMonoAdapterPort, RunRepositoryPort,
        RuntimeEventSink, SessionRepositoryPort, StartSessionRunRequest,
    },
};
use chrono::{DateTime, Utc};
use domain::{Approval, ApprovalRequest, Run, RunStatus, Session, SessionStatus};
use serde_json::json;
use shared_kernel::{AppResult, RunId, SessionId};

pub struct SendPromptInput {
    pub session: Session,
    pub request: StartSessionRunRequest,
}

pub fn execute<R, S, P, E, A, B>(
    runs: &R,
    sessions: &S,
    approvals: &P,
    events: &E,
    adapter: &A,
    bus: &B,
    input: SendPromptInput,
    now: DateTime<Utc>,
) -> AppResult<Run>
where
    R: RunRepositoryPort,
    S: SessionRepositoryPort,
    P: ApprovalRepositoryPort,
    E: EventRepositoryPort,
    A: PiMonoAdapterPort,
    B: EventBusPort,
{
    let mut run = Run::new(input.session.id.clone(), input.request.prompt.clone(), now);
    run.status = RunStatus::Running;
    runs.create(&run)?;
    sessions.update_status(&input.session.id, SessionStatus::Running, now)?;
    bus.publish(ApplicationEvent::SessionStatusChanged {
        session_id: input.session.id.clone(),
        status: SessionStatus::Running,
    });

    let sink = PersistingRuntimeSink {
        runs,
        sessions,
        approvals,
        events,
        bus,
        session_id: input.session.id.clone(),
        run_id: run.id.clone(),
        now,
    };
    adapter.start_session_run(&input.request, &sink)?;
    Ok(run)
}

pub(crate) struct PersistingRuntimeSink<'a, R, S, P, E, B> {
    pub(crate) runs: &'a R,
    pub(crate) sessions: &'a S,
    pub(crate) approvals: &'a P,
    pub(crate) events: &'a E,
    pub(crate) bus: &'a B,
    pub(crate) session_id: SessionId,
    pub(crate) run_id: RunId,
    pub(crate) now: DateTime<Utc>,
}

impl<R, S, P, E, B> RuntimeEventSink for PersistingRuntimeSink<'_, R, S, P, E, B>
where
    R: RunRepositoryPort,
    S: SessionRepositoryPort,
    P: ApprovalRepositoryPort,
    E: EventRepositoryPort,
    B: EventBusPort,
{
    fn push(&self, event: RuntimeEvent) -> AppResult<()> {
        let (event_type, payload_json) = runtime_event_payload(&event);
        self.events.append_event(
            &self.session_id,
            Some(&self.run_id),
            event_type,
            &payload_json,
            self.now,
        )?;
        self.bus.publish(ApplicationEvent::RuntimeEventAppended {
            session_id: self.session_id.clone(),
            run_id: Some(self.run_id.clone()),
            event: event.clone(),
        });

        match event {
            RuntimeEvent::SessionBound { pimono_session_id } => {
                self.sessions.bind_runtime_session(
                    &self.session_id,
                    &pimono_session_id,
                    self.now,
                )?;
            }
            RuntimeEvent::RunStarted {
                pimono_run_id,
                pimono_session_id,
            } => {
                self.runs.bind_runtime_run(&self.run_id, &pimono_run_id)?;
                if let Some(pimono_session_id) = pimono_session_id {
                    self.sessions.bind_runtime_session(
                        &self.session_id,
                        &pimono_session_id,
                        self.now,
                    )?;
                }
            }
            RuntimeEvent::ApprovalRequested {
                request_id,
                request_type,
                payload_json,
            } => {
                let approval = Approval::new(
                    self.run_id.clone(),
                    ApprovalRequest {
                        request_id,
                        correlation_id: None,
                        request_type,
                        request_payload_json: payload_json,
                    },
                    self.now,
                );
                self.approvals.create(&approval)?;
                self.sessions.update_status(
                    &self.session_id,
                    SessionStatus::WaitingApproval,
                    self.now,
                )?;
                self.bus.publish(ApplicationEvent::SessionStatusChanged {
                    session_id: self.session_id.clone(),
                    status: SessionStatus::WaitingApproval,
                });
            }
            RuntimeEvent::RunFailed { code, message } => {
                self.runs.update_terminal_state(
                    &self.run_id,
                    RunStatus::Failed,
                    self.now,
                    code,
                    Some(message),
                )?;
                self.sessions
                    .update_status(&self.session_id, SessionStatus::Failed, self.now)?;
                self.bus.publish(ApplicationEvent::SessionStatusChanged {
                    session_id: self.session_id.clone(),
                    status: SessionStatus::Failed,
                });
            }
            RuntimeEvent::RunCompleted => {
                self.runs.update_terminal_state(
                    &self.run_id,
                    RunStatus::Completed,
                    self.now,
                    None,
                    None,
                )?;
                self.sessions.update_status(
                    &self.session_id,
                    SessionStatus::Completed,
                    self.now,
                )?;
                self.bus.publish(ApplicationEvent::SessionStatusChanged {
                    session_id: self.session_id.clone(),
                    status: SessionStatus::Completed,
                });
            }
            RuntimeEvent::TextDelta { .. } => {}
        }

        Ok(())
    }
}

fn runtime_event_payload(event: &RuntimeEvent) -> (&'static str, String) {
    match event {
        RuntimeEvent::SessionBound { pimono_session_id } => (
            "session_bound",
            json!({ "pimono_session_id": pimono_session_id }).to_string(),
        ),
        RuntimeEvent::RunStarted {
            pimono_run_id,
            pimono_session_id,
        } => (
            "run_started",
            json!({
                "pimono_run_id": pimono_run_id,
                "pimono_session_id": pimono_session_id,
            })
            .to_string(),
        ),
        RuntimeEvent::TextDelta { text } => ("text_delta", json!({ "text": text }).to_string()),
        RuntimeEvent::ApprovalRequested {
            request_id,
            request_type,
            payload_json,
        } => (
            "approval_requested",
            json!({
                "request_id": request_id,
                "request_type": request_type,
                "payload_json": payload_json,
            })
            .to_string(),
        ),
        RuntimeEvent::RunFailed { code, message } => (
            "run_failed",
            json!({ "code": code, "message": message }).to_string(),
        ),
        RuntimeEvent::RunCompleted => ("run_completed", json!({}).to_string()),
    }
}
