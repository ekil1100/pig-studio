use chrono::Utc;
use domain::{Approval, ApprovalRequest, Project, Run, Session, SessionStatus};
use infra_sqlite::Database;
use std::path::PathBuf;

#[test]
fn appends_events_without_overwriting_previous_rows() {
    let database = Database::in_memory().expect("database");
    let projects = database.project_repository();
    let sessions = database.session_repository();
    let runs = database.run_repository();
    let events = database.event_repository();
    let now = Utc::now();

    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-project"), now);
    projects.create(&project).expect("project created");

    let session = Session::new(
        project.id.clone(),
        "Session A",
        PathBuf::from("/tmp/pig-project"),
        now,
    );
    sessions.create(&session).expect("session created");

    let run = Run::new(session.id.clone(), "build it", now);
    runs.create(&run).expect("run created");

    let first = events
        .append(
            &session.id,
            Some(&run.id),
            "text_delta",
            "{\"text\":\"hello\"}",
            now,
        )
        .expect("first event");
    let second = events
        .append(
            &session.id,
            Some(&run.id),
            "text_delta",
            "{\"text\":\"world\"}",
            now,
        )
        .expect("second event");

    let stored = events.list_by_session(&session.id).expect("list events");

    assert_eq!(stored.len(), 2);
    assert_eq!(first.seq, 1);
    assert_eq!(second.seq, 2);
    assert_eq!(stored[0].seq, 1);
    assert_eq!(stored[1].seq, 2);
}

#[test]
fn restores_pending_approval_for_waiting_session() {
    let database = Database::in_memory().expect("database");
    let projects = database.project_repository();
    let sessions = database.session_repository();
    let runs = database.run_repository();
    let approvals = database.approval_repository();
    let now = Utc::now();

    let project = Project::new("Pig Studio", PathBuf::from("/tmp/pig-project"), now);
    projects.create(&project).expect("project created");

    let mut session = Session::new(
        project.id.clone(),
        "Needs approval",
        PathBuf::from("/tmp/pig-project"),
        now,
    );
    session.status = SessionStatus::WaitingApproval;
    sessions.create(&session).expect("session created");

    let mut run = Run::new(session.id.clone(), "delete file?", now);
    run.status = domain::RunStatus::WaitingApproval;
    runs.create(&run).expect("run created");

    let approval = Approval::new(
        run.id.clone(),
        ApprovalRequest {
            request_id: "req-1".into(),
            correlation_id: Some("corr-1".into()),
            request_type: "filesystem.delete".into(),
            request_payload_json: "{\"path\":\"/tmp/pig-project/README.md\"}".into(),
        },
        now,
    );
    approvals.create(&approval).expect("approval created");

    let restored = sessions
        .restore_view(&session.id)
        .expect("restore view")
        .expect("session exists");

    assert_eq!(restored.session.status, SessionStatus::WaitingApproval);
    assert_eq!(restored.pending_approvals.len(), 1);
    assert_eq!(restored.pending_approvals[0].request.request_id, "req-1");
}
