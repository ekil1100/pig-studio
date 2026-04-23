use crate::ports::{ResolvedWorkspace, WorkspaceServicePort, default_workspace};
use shared_kernel::AppResult;
use std::path::PathBuf;

pub struct ManageWorktreeInput {
    pub project_root: PathBuf,
    pub prefer_worktree: bool,
}

pub fn execute<W>(workspace: &W, input: ManageWorktreeInput) -> AppResult<ResolvedWorkspace>
where
    W: WorkspaceServicePort,
{
    if !input.prefer_worktree {
        return Ok(default_workspace(&input.project_root));
    }

    if !workspace.is_git_repository(&input.project_root)? {
        return Ok(default_workspace(&input.project_root));
    }

    match workspace.resolve_workspace(&input.project_root, true) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(default_workspace(&input.project_root)),
    }
}
