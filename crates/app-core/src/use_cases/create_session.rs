use crate::{
    event_bus::{ApplicationEvent, EventBusPort},
    ports::{SessionRepositoryPort, WorkspaceServicePort, default_workspace},
};
use chrono::{DateTime, Utc};
use domain::{Project, Session};
use shared_kernel::AppResult;

pub struct CreateSessionInput {
    pub project: Project,
    pub title: String,
    pub prefer_worktree: bool,
}

pub fn execute<S, W, B>(
    sessions: &S,
    workspace: &W,
    bus: &B,
    input: CreateSessionInput,
    now: DateTime<Utc>,
) -> AppResult<Session>
where
    S: SessionRepositoryPort,
    W: WorkspaceServicePort,
    B: EventBusPort,
{
    let is_git = workspace.is_git_repository(&input.project.root_path)?;
    let resolved_workspace = if is_git {
        workspace.resolve_workspace(&input.project.root_path, input.prefer_worktree)?
    } else {
        default_workspace(&input.project.root_path)
    };

    let mut session = Session::new(input.project.id, input.title, resolved_workspace.cwd, now);
    session.workspace_mode = resolved_workspace.mode;
    session.worktree_path = resolved_workspace.worktree_path;

    sessions.create(&session)?;
    bus.publish(ApplicationEvent::SessionCreated {
        session_id: session.id.clone(),
    });
    Ok(session)
}
