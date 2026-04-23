use crate::{
    event_bus::{ApplicationEvent, EventBusPort},
    ports::{
        ApprovalRepositoryPort, EventRepositoryPort, PiMonoAdapterPort, ResumeSessionRequest,
        RunRepositoryPort, SessionRepositoryPort,
    },
    use_cases::send_prompt::PersistingRuntimeSink,
};
use chrono::{DateTime, Utc};
use domain::{Run, Session, SessionStatus};
use shared_kernel::AppResult;

pub struct ResumeSessionInput {
    pub session: Session,
    pub active_run: Run,
    pub request: ResumeSessionRequest,
}

pub fn execute<R, S, P, E, A, B>(
    runs: &R,
    sessions: &S,
    approvals: &P,
    events: &E,
    adapter: &A,
    bus: &B,
    input: ResumeSessionInput,
    now: DateTime<Utc>,
) -> AppResult<()>
where
    R: RunRepositoryPort,
    S: SessionRepositoryPort,
    P: ApprovalRepositoryPort,
    E: EventRepositoryPort,
    A: PiMonoAdapterPort,
    B: EventBusPort,
{
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
        session_id: input.session.id,
        run_id: input.active_run.id,
        now,
    };
    adapter.resume_session(&input.request, &sink)
}
