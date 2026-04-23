use app_core::{
    ports::{ResolvedWorkspace, WorkspaceServicePort, default_workspace},
    use_cases::manage_worktree,
};
use domain::WorkspaceMode;
use shared_kernel::{AppError, AppResult};
use std::path::{Path, PathBuf};

struct FakeWorkspaceService {
    git: bool,
    fail: bool,
}

impl WorkspaceServicePort for FakeWorkspaceService {
    fn ensure_project_directory(&self, _root_path: &Path) -> AppResult<()> {
        Ok(())
    }

    fn is_git_repository(&self, _root_path: &Path) -> AppResult<bool> {
        Ok(self.git)
    }

    fn resolve_workspace(
        &self,
        root_path: &Path,
        prefer_worktree: bool,
    ) -> AppResult<ResolvedWorkspace> {
        if self.fail {
            return Err(AppError::Infrastructure("worktree failed".into()));
        }

        if prefer_worktree {
            Ok(ResolvedWorkspace {
                cwd: root_path.join(".pig-studio-worktrees/session-1"),
                mode: WorkspaceMode::Worktree,
                worktree_path: Some(root_path.join(".pig-studio-worktrees/session-1")),
            })
        } else {
            Ok(default_workspace(root_path))
        }
    }
}

#[test]
fn manage_worktree_creates_isolated_workspace_for_git_projects() {
    let root = PathBuf::from("/tmp/pig-studio");
    let service = FakeWorkspaceService {
        git: true,
        fail: false,
    };

    let workspace = manage_worktree::execute(
        &service,
        manage_worktree::ManageWorktreeInput {
            project_root: root.clone(),
            prefer_worktree: true,
        },
    )
    .expect("workspace");

    assert_eq!(workspace.mode, WorkspaceMode::Worktree);
    assert_ne!(workspace.cwd, root);
}

#[test]
fn manage_worktree_keeps_non_git_projects_in_direct_mode() {
    let root = PathBuf::from("/tmp/pig-studio");
    let service = FakeWorkspaceService {
        git: false,
        fail: false,
    };

    let workspace = manage_worktree::execute(
        &service,
        manage_worktree::ManageWorktreeInput {
            project_root: root.clone(),
            prefer_worktree: true,
        },
    )
    .expect("workspace");

    assert_eq!(workspace.mode, WorkspaceMode::Direct);
    assert_eq!(workspace.cwd, root);
}

#[test]
fn manage_worktree_falls_back_to_direct_mode_when_creation_fails() {
    let root = PathBuf::from("/tmp/pig-studio");
    let service = FakeWorkspaceService {
        git: true,
        fail: true,
    };

    let workspace = manage_worktree::execute(
        &service,
        manage_worktree::ManageWorktreeInput {
            project_root: root.clone(),
            prefer_worktree: true,
        },
    )
    .expect("workspace");

    assert_eq!(workspace.mode, WorkspaceMode::Direct);
    assert_eq!(workspace.cwd, root);
}
