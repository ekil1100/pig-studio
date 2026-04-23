use chrono::{DateTime, Utc};
use domain::{ApprovalStatus, SessionStatus};
use serde::{Deserialize, Serialize};
use shared_kernel::{ApprovalId, ProjectId, RunId, SessionId};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeEvent {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplicationEvent {
    ProjectCreated {
        project_id: ProjectId,
    },
    SessionCreated {
        session_id: SessionId,
    },
    SessionStatusChanged {
        session_id: SessionId,
        status: SessionStatus,
    },
    RuntimeEventAppended {
        session_id: SessionId,
        run_id: Option<RunId>,
        event: RuntimeEvent,
    },
    ApprovalUpdated {
        approval_id: ApprovalId,
        status: ApprovalStatus,
    },
    RuntimeSettingsUpdated {
        checked_at: DateTime<Utc>,
    },
}

pub trait EventBusPort {
    fn publish(&self, event: ApplicationEvent);
}

#[derive(Clone, Default)]
pub struct InMemoryEventBus {
    events: Rc<RefCell<Vec<ApplicationEvent>>>,
}

impl InMemoryEventBus {
    pub fn events(&self) -> Vec<ApplicationEvent> {
        self.events.borrow().clone()
    }
}

impl EventBusPort for InMemoryEventBus {
    fn publish(&self, event: ApplicationEvent) {
        self.events.borrow_mut().push(event);
    }
}
