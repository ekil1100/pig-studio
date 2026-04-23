use app_desktop::bootstrap::{StartupSnapshot, recover_workspace};
use chrono::Utc;
use domain::{Approval, ApprovalRequest, Project, Session, SessionStatus};
use shared_kernel::RunId;
use std::{collections::BTreeSet, path::PathBuf};

#[test]
fn restores_projects_and_recent_session_entry_on_startup() {
    let now = Utc::now();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);
    let session = Session::new(
        project.id.clone(),
        "Recent Session",
        PathBuf::from("/tmp/pig-studio"),
        now,
    );

    let state = recover_workspace(StartupSnapshot {
        projects: vec![project],
        sessions: vec![session.clone()],
        active_session_id: Some(session.id.clone()),
        ..StartupSnapshot::default()
    });

    assert_eq!(state.projects.len(), 1);
    assert_eq!(state.projects[0].sessions.len(), 1);
    assert_eq!(
        state.active_session.expect("active session").session_name,
        "Recent Session"
    );
}

#[test]
fn rebuilds_pending_approval_view_for_waiting_session() {
    let now = Utc::now();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);
    let mut session = Session::new(
        project.id.clone(),
        "Needs Approval",
        PathBuf::from("/tmp/pig-studio"),
        now,
    );
    session.status = SessionStatus::WaitingApproval;
    let approval = Approval::new(
        RunId::new(),
        ApprovalRequest {
            request_id: "req-1".into(),
            correlation_id: None,
            request_type: "filesystem.delete".into(),
            request_payload_json: "{\"path\":\"README.md\"}".into(),
        },
        now,
    );

    let state = recover_workspace(StartupSnapshot {
        projects: vec![project],
        sessions: vec![session.clone()],
        pending_approvals: vec![approval],
        active_session_id: Some(session.id.clone()),
        ..StartupSnapshot::default()
    });

    assert_eq!(
        state
            .active_session
            .expect("active session")
            .approvals
            .len(),
        1
    );
}

#[test]
fn marks_unrecoverable_running_session_as_interrupted() {
    let now = Utc::now();
    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-studio"), now);
    let mut session = Session::new(
        project.id.clone(),
        "Running Session",
        PathBuf::from("/tmp/pig-studio"),
        now,
    );
    session.status = SessionStatus::Running;
    let session_id = session.id.clone();

    let state = recover_workspace(StartupSnapshot {
        projects: vec![project],
        sessions: vec![session],
        active_session_id: Some(session_id.clone()),
        unrecoverable_running_sessions: BTreeSet::from([session_id]),
        ..StartupSnapshot::default()
    });

    assert_eq!(
        state
            .active_session
            .expect("active session")
            .status_badge
            .label,
        "已中断"
    );
}
