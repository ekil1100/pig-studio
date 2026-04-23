use crate::FsService;
use app_core::ports::{ResolvedWorkspace, WorkspaceServicePort};
use domain::WorkspaceMode;
use shared_kernel::{AppError, AppResult};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Copy)]
pub struct WorktreeService {
    fs: FsService,
}

impl Default for WorktreeService {
    fn default() -> Self {
        Self { fs: FsService }
    }
}

impl WorktreeService {
    pub fn new(fs: FsService) -> Self {
        Self { fs }
    }

    fn detached_worktree_dir(&self, root_path: &Path) -> PathBuf {
        let project_name = root_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("project");
        root_path
            .parent()
            .unwrap_or(root_path)
            .join(".pig-studio-worktrees")
            .join(format!("{project_name}-session"))
    }
}

impl WorkspaceServicePort for WorktreeService {
    fn ensure_project_directory(&self, root_path: &Path) -> AppResult<()> {
        self.fs.ensure_project_directory(root_path)
    }

    fn is_git_repository(&self, root_path: &Path) -> AppResult<bool> {
        Ok(self.fs.is_git_repository(root_path))
    }

    fn resolve_workspace(
        &self,
        root_path: &Path,
        prefer_worktree: bool,
    ) -> AppResult<ResolvedWorkspace> {
        self.ensure_project_directory(root_path)?;

        if !prefer_worktree || !self.fs.is_git_repository(root_path) {
            return Ok(ResolvedWorkspace {
                cwd: root_path.to_path_buf(),
                mode: WorkspaceMode::Direct,
                worktree_path: None,
            });
        }

        let worktree_dir = self.detached_worktree_dir(root_path);
        if worktree_dir.join(".git").exists() {
            return Ok(ResolvedWorkspace {
                cwd: worktree_dir.clone(),
                mode: WorkspaceMode::Worktree,
                worktree_path: Some(worktree_dir),
            });
        }

        if let Some(parent) = worktree_dir.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        }

        let output = Command::new("git")
            .arg("-C")
            .arg(root_path)
            .arg("worktree")
            .arg("add")
            .arg("--detach")
            .arg(&worktree_dir)
            .output()
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        if output.status.success() {
            Ok(ResolvedWorkspace {
                cwd: worktree_dir.clone(),
                mode: WorkspaceMode::Worktree,
                worktree_path: Some(worktree_dir),
            })
        } else {
            Err(AppError::Infrastructure(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ))
        }
    }
}
