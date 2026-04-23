use app_core::{
    event_bus::{InMemoryEventBus, RuntimeEvent},
    ports::{
        ApprovalRepositoryPort, EventRepositoryPort, InspectRunStatusRequest, PiMonoAdapterPort,
        ProjectRepositoryPort, ResolvedWorkspace, RespondApprovalRequest, ResumeSessionRequest,
        RunInspection, RunRepositoryPort, RuntimeEventSink, RuntimeSettings, SessionRepositoryPort,
        SettingsStorePort, StartSessionRunRequest, WorkspaceServicePort, default_workspace,
    },
    use_cases,
};
use chrono::Utc;
use domain::{
    Approval, ApprovalDecision, ApprovalRequest, ApprovalStatus, Project, Run, RunStatus, Session,
    SessionStatus, WorkspaceMode,
};
use shared_kernel::{AppError, AppResult, ApprovalId, ProjectId, RunId, SessionId};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Clone)]
struct FakeContext {
    projects: Rc<RefCell<Vec<Project>>>,
    sessions: Rc<RefCell<Vec<Session>>>,
    runs: Rc<RefCell<Vec<Run>>>,
    approvals: Rc<RefCell<Vec<Approval>>>,
    events: Rc<RefCell<Vec<(String, String)>>>,
    call_log: Rc<RefCell<Vec<String>>>,
    runtime_settings: Rc<RefCell<RuntimeSettings>>,
    is_git: bool,
    inspection: Rc<RefCell<Result<RunInspection, AppError>>>,
    emitted_events: Rc<RefCell<Vec<RuntimeEvent>>>,
}

impl Default for FakeContext {
    fn default() -> Self {
        Self {
            projects: Rc::new(RefCell::new(Vec::new())),
            sessions: Rc::new(RefCell::new(Vec::new())),
            runs: Rc::new(RefCell::new(Vec::new())),
            approvals: Rc::new(RefCell::new(Vec::new())),
            events: Rc::new(RefCell::new(Vec::new())),
            call_log: Rc::new(RefCell::new(Vec::new())),
            runtime_settings: Rc::new(RefCell::new(RuntimeSettings::default())),
            is_git: false,
            inspection: Rc::new(RefCell::new(Ok(RunInspection {
                running: false,
                terminal_event: None,
            }))),
            emitted_events: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl FakeContext {
    fn with_runtime_path(path: PathBuf) -> Self {
        Self {
            runtime_settings: Rc::new(RefCell::new(RuntimeSettings {
                runtime_path: Some(path),
                config_dir: None,
                environment: BTreeMap::new(),
                last_checked_at: None,
            })),
            inspection: Rc::new(RefCell::new(Ok(RunInspection {
                running: false,
                terminal_event: None,
            }))),
            ..Self::default()
        }
    }
}

impl ProjectRepositoryPort for FakeContext {
    fn create(&self, project: &Project) -> AppResult<()> {
        self.projects.borrow_mut().push(project.clone());
        Ok(())
    }

    fn find_by_root_path(&self, root_path: &Path) -> AppResult<Option<Project>> {
        Ok(self
            .projects
            .borrow()
            .iter()
            .find(|project| project.root_path == root_path)
            .cloned())
    }
}

impl SessionRepositoryPort for FakeContext {
    fn create(&self, session: &Session) -> AppResult<()> {
        self.sessions.borrow_mut().push(session.clone());
        Ok(())
    }

    fn update_status(
        &self,
        session_id: &SessionId,
        status: SessionStatus,
        _updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        self.call_log
            .borrow_mut()
            .push(format!("session-status:{status:?}"));
        if let Some(session) = self
            .sessions
            .borrow_mut()
            .iter_mut()
            .find(|session| &session.id == session_id)
        {
            session.status = status;
        }
        Ok(())
    }

    fn bind_runtime_session(
        &self,
        session_id: &SessionId,
        pimono_session_id: &str,
        _updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        self.call_log.borrow_mut().push("session-bind".into());
        if let Some(session) = self
            .sessions
            .borrow_mut()
            .iter_mut()
            .find(|session| &session.id == session_id)
        {
            session.pimono_session_id = Some(pimono_session_id.to_owned());
        }
        Ok(())
    }

    fn find(&self, session_id: &SessionId) -> AppResult<Option<Session>> {
        Ok(self
            .sessions
            .borrow()
            .iter()
            .find(|session| &session.id == session_id)
            .cloned())
    }

    fn list_active(&self) -> AppResult<Vec<Session>> {
        Ok(self
            .sessions
            .borrow()
            .iter()
            .filter(|session| {
                matches!(
                    session.status,
                    SessionStatus::Running | SessionStatus::WaitingApproval
                )
            })
            .cloned()
            .collect())
    }
}

impl RunRepositoryPort for FakeContext {
    fn create(&self, run: &Run) -> AppResult<()> {
        self.call_log.borrow_mut().push("run-create".into());
        self.runs.borrow_mut().push(run.clone());
        Ok(())
    }

    fn bind_runtime_run(&self, run_id: &RunId, pimono_run_id: &str) -> AppResult<()> {
        self.call_log.borrow_mut().push("run-bind".into());
        if let Some(run) = self
            .runs
            .borrow_mut()
            .iter_mut()
            .find(|run| &run.id == run_id)
        {
            run.pimono_run_id = Some(pimono_run_id.to_owned());
        }
        Ok(())
    }

    fn update_terminal_state(
        &self,
        run_id: &RunId,
        status: RunStatus,
        _ended_at: chrono::DateTime<Utc>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> AppResult<()> {
        self.call_log
            .borrow_mut()
            .push(format!("run-terminal:{status:?}"));
        if let Some(run) = self
            .runs
            .borrow_mut()
            .iter_mut()
            .find(|run| &run.id == run_id)
        {
            run.status = status;
            run.error_code = error_code;
            run.error_message = error_message;
        }
        Ok(())
    }

    fn list_active(&self) -> AppResult<Vec<Run>> {
        Ok(self
            .runs
            .borrow()
            .iter()
            .filter(|run| {
                matches!(
                    run.status,
                    RunStatus::Running | RunStatus::WaitingApproval | RunStatus::Queued
                )
            })
            .cloned()
            .collect())
    }
}

impl ApprovalRepositoryPort for FakeContext {
    fn create(&self, approval: &Approval) -> AppResult<()> {
        self.approvals.borrow_mut().push(approval.clone());
        Ok(())
    }

    fn record_decision(
        &self,
        approval_id: &ApprovalId,
        status: ApprovalStatus,
        decision: ApprovalDecision,
        _decided_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        self.call_log.borrow_mut().push("approval-record".into());
        if let Some(approval) = self
            .approvals
            .borrow_mut()
            .iter_mut()
            .find(|approval| &approval.id == approval_id)
        {
            approval.status = status;
            approval.decision = Some(decision);
        }
        Ok(())
    }
}

impl EventRepositoryPort for FakeContext {
    fn append_event(
        &self,
        _session_id: &SessionId,
        _run_id: Option<&RunId>,
        event_type: &str,
        payload_json: &str,
        _created_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        self.call_log
            .borrow_mut()
            .push(format!("event:{event_type}"));
        self.events
            .borrow_mut()
            .push((event_type.into(), payload_json.into()));
        Ok(())
    }
}

impl PiMonoAdapterPort for FakeContext {
    fn start_session_run(
        &self,
        _request: &StartSessionRunRequest,
        sink: &dyn RuntimeEventSink,
    ) -> AppResult<()> {
        self.call_log.borrow_mut().push("adapter-start".into());
        for event in self.emitted_events.borrow().clone() {
            sink.push(event)?;
        }
        Ok(())
    }

    fn resume_session(
        &self,
        _request: &ResumeSessionRequest,
        sink: &dyn RuntimeEventSink,
    ) -> AppResult<()> {
        for event in self.emitted_events.borrow().clone() {
            sink.push(event)?;
        }
        Ok(())
    }

    fn respond_approval(&self, _request: &RespondApprovalRequest) -> AppResult<()> {
        self.call_log.borrow_mut().push("adapter-respond".into());
        Ok(())
    }

    fn inspect_run_status(&self, _request: &InspectRunStatusRequest) -> AppResult<RunInspection> {
        self.inspection.borrow().clone()
    }
}

impl SettingsStorePort for FakeContext {
    fn load(&self) -> AppResult<RuntimeSettings> {
        Ok(self.runtime_settings.borrow().clone())
    }

    fn save(&self, settings: &RuntimeSettings) -> AppResult<()> {
        *self.runtime_settings.borrow_mut() = settings.clone();
        Ok(())
    }
}

impl WorkspaceServicePort for FakeContext {
    fn ensure_project_directory(&self, _root_path: &Path) -> AppResult<()> {
        Ok(())
    }

    fn is_git_repository(&self, _root_path: &Path) -> AppResult<bool> {
        Ok(self.is_git)
    }

    fn resolve_workspace(
        &self,
        root_path: &Path,
        _prefer_worktree: bool,
    ) -> AppResult<ResolvedWorkspace> {
        Ok(default_workspace(root_path))
    }
}

#[test]
fn create_project_rejects_duplicate_path() {
    let now = Utc::now();
    let context = FakeContext::default();
    let bus = InMemoryEventBus::default();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);
    ProjectRepositoryPort::create(&context, &project).expect("seed project");

    let result = use_cases::create_project::execute(
        &context,
        &context,
        &bus,
        use_cases::create_project::CreateProjectInput {
            name: "Pig Studio".into(),
            root_path: PathBuf::from("/tmp/pig-studio"),
        },
        now,
    );

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[test]
fn create_session_defaults_to_direct_for_non_git_projects() {
    let now = Utc::now();
    let context = FakeContext::default();
    let bus = InMemoryEventBus::default();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);

    let session = use_cases::create_session::execute(
        &context,
        &context,
        &bus,
        use_cases::create_session::CreateSessionInput {
            project,
            title: "New Session".into(),
            prefer_worktree: true,
        },
        now,
    )
    .expect("session created");

    assert_eq!(session.workspace_mode, WorkspaceMode::Direct);
}

#[test]
fn send_prompt_creates_run_before_consuming_runtime_events() {
    let now = Utc::now();
    let context = FakeContext::default();
    let bus = InMemoryEventBus::default();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);
    let session = Session::new(
        project.id.clone(),
        "Session",
        PathBuf::from("/tmp/pig-studio"),
        now,
    );
    context.sessions.borrow_mut().push(session.clone());
    context
        .emitted_events
        .borrow_mut()
        .push(RuntimeEvent::RunCompleted);

    use_cases::send_prompt::execute(
        &context,
        &context,
        &context,
        &context,
        &context,
        &bus,
        use_cases::send_prompt::SendPromptInput {
            session: session.clone(),
            request: StartSessionRunRequest {
                runtime_path: PathBuf::from("/usr/bin/pi-mono"),
                workspace_cwd: session.workspace_cwd.clone(),
                pimono_session_id: None,
                prompt: "hello".into(),
                env: BTreeMap::new(),
            },
        },
        now,
    )
    .expect("prompt sent");

    let log = context.call_log.borrow();
    let run_create_index = log
        .iter()
        .position(|entry| entry == "run-create")
        .expect("run created");
    let adapter_index = log
        .iter()
        .position(|entry| entry == "adapter-start")
        .expect("adapter started");
    assert!(run_create_index < adapter_index);
}

#[test]
fn send_prompt_binds_runtime_session_and_run_identifiers() {
    let now = Utc::now();
    let context = FakeContext::default();
    let bus = InMemoryEventBus::default();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);
    let session = Session::new(
        project.id.clone(),
        "Session",
        PathBuf::from("/tmp/pig-studio"),
        now,
    );
    context.sessions.borrow_mut().push(session.clone());
    context.emitted_events.borrow_mut().extend([
        RuntimeEvent::SessionBound {
            pimono_session_id: "remote-session-1".into(),
        },
        RuntimeEvent::RunStarted {
            pimono_run_id: "remote-run-1".into(),
            pimono_session_id: Some("remote-session-1".into()),
        },
    ]);

    let run = use_cases::send_prompt::execute(
        &context,
        &context,
        &context,
        &context,
        &context,
        &bus,
        use_cases::send_prompt::SendPromptInput {
            session: session.clone(),
            request: StartSessionRunRequest {
                runtime_path: PathBuf::from("/usr/bin/pi-mono"),
                workspace_cwd: session.workspace_cwd.clone(),
                pimono_session_id: None,
                prompt: "hello".into(),
                env: BTreeMap::new(),
            },
        },
        now,
    )
    .expect("prompt sent");

    let stored_session = context.sessions.borrow()[0].clone();
    let stored_run = context
        .runs
        .borrow()
        .iter()
        .find(|candidate| candidate.id == run.id)
        .cloned()
        .expect("stored run");
    assert_eq!(
        stored_session.pimono_session_id.as_deref(),
        Some("remote-session-1")
    );
    assert_eq!(stored_run.pimono_run_id.as_deref(), Some("remote-run-1"));
}

#[test]
fn send_prompt_persists_approval_requests_for_recovery() {
    let now = Utc::now();
    let context = FakeContext::default();
    let bus = InMemoryEventBus::default();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);
    let session = Session::new(
        project.id.clone(),
        "Session",
        PathBuf::from("/tmp/pig-studio"),
        now,
    );
    context.sessions.borrow_mut().push(session.clone());
    context
        .emitted_events
        .borrow_mut()
        .push(RuntimeEvent::ApprovalRequested {
            request_id: "req-1".into(),
            request_type: "filesystem.delete".into(),
            payload_json: "{\"path\":\"README.md\"}".into(),
        });

    use_cases::send_prompt::execute(
        &context,
        &context,
        &context,
        &context,
        &context,
        &bus,
        use_cases::send_prompt::SendPromptInput {
            session: session.clone(),
            request: StartSessionRunRequest {
                runtime_path: PathBuf::from("/usr/bin/pi-mono"),
                workspace_cwd: session.workspace_cwd.clone(),
                pimono_session_id: None,
                prompt: "hello".into(),
                env: BTreeMap::new(),
            },
        },
        now,
    )
    .expect("prompt sent");

    let approvals = context.approvals.borrow();
    assert_eq!(approvals.len(), 1);
    assert_eq!(approvals[0].request.request_id, "req-1");
    assert_eq!(approvals[0].request.request_type, "filesystem.delete");
}

#[test]
fn respond_approval_persists_decision_before_calling_adapter() {
    let now = Utc::now();
    let context = FakeContext::default();
    let bus = InMemoryEventBus::default();
    let approval = Approval::new(
        RunId::new(),
        ApprovalRequest {
            request_id: "req-1".into(),
            correlation_id: None,
            request_type: "filesystem.delete".into(),
            request_payload_json: "{}".into(),
        },
        now,
    );
    context.approvals.borrow_mut().push(approval.clone());

    use_cases::respond_approval::execute(
        &context,
        &context,
        &context,
        &bus,
        use_cases::respond_approval::RespondApprovalInput {
            approval_id: approval.id.clone(),
            session_id: SessionId::new(),
            run_id: approval.run_id.clone(),
            request: RespondApprovalRequest {
                runtime_path: PathBuf::from("/usr/bin/pi-mono"),
                workspace_cwd: PathBuf::from("/tmp/pig-studio"),
                request_id: "req-1".into(),
                approve: true,
                env: BTreeMap::new(),
            },
            decision: ApprovalDecision::Approve,
        },
        now,
    )
    .expect("approval responded");

    let log = context.call_log.borrow();
    let persist_index = log
        .iter()
        .position(|entry| entry == "approval-record")
        .expect("approval recorded");
    let event_index = log
        .iter()
        .position(|entry| entry == "event:approval_decision")
        .expect("decision event appended");
    let adapter_index = log
        .iter()
        .position(|entry| entry == "adapter-respond")
        .expect("adapter called");
    assert!(persist_index < event_index);
    assert!(event_index < adapter_index);
}

#[test]
fn reconcile_active_runs_marks_unrecoverable_runs_interrupted() {
    let now = Utc::now();
    let context = FakeContext::with_runtime_path(PathBuf::from("/usr/bin/pi-mono"));
    let bus = InMemoryEventBus::default();
    let project_id = ProjectId::new();
    let session = Session::new(project_id, "Session", PathBuf::from("/tmp/pig-studio"), now);
    let mut run = Run::new(session.id.clone(), "hello", now);
    run.status = RunStatus::Running;
    run.pimono_run_id = Some("run-1".into());
    context.sessions.borrow_mut().push(session.clone());
    context.runs.borrow_mut().push(run.clone());
    *context.inspection.borrow_mut() = Err(AppError::External("not found".into()));

    use_cases::reconcile_active_runs::execute(
        &context, &context, &context, &context, &context, &bus, now,
    )
    .expect("reconcile succeeds");

    let updated_run = context.runs.borrow()[0].clone();
    let updated_session = context.sessions.borrow()[0].clone();
    assert_eq!(updated_run.status, RunStatus::Interrupted);
    assert_eq!(updated_session.status, SessionStatus::Interrupted);
}
