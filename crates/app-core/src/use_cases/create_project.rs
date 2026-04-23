use crate::{
    event_bus::{ApplicationEvent, EventBusPort},
    ports::{ProjectRepositoryPort, WorkspaceServicePort},
};
use chrono::{DateTime, Utc};
use domain::Project;
use shared_kernel::{AppError, AppResult};
use std::path::PathBuf;

pub struct CreateProjectInput {
    pub name: String,
    pub root_path: PathBuf,
}

pub fn execute<P, W, B>(
    projects: &P,
    workspace: &W,
    bus: &B,
    input: CreateProjectInput,
    now: DateTime<Utc>,
) -> AppResult<Project>
where
    P: ProjectRepositoryPort,
    W: WorkspaceServicePort,
    B: EventBusPort,
{
    workspace.ensure_project_directory(&input.root_path)?;

    if projects.find_by_root_path(&input.root_path)?.is_some() {
        return Err(AppError::Conflict(format!(
            "project already exists for path: {}",
            input.root_path.display()
        )));
    }

    let project = Project::new(input.name, input.root_path, now);
    projects.create(&project)?;
    bus.publish(ApplicationEvent::ProjectCreated {
        project_id: project.id.clone(),
    });
    Ok(project)
}
