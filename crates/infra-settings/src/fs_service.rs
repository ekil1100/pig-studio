use shared_kernel::{AppError, AppResult};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Copy)]
pub struct FsService;

impl FsService {
    pub fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn ensure_project_directory(&self, path: &Path) -> AppResult<()> {
        if path.is_dir() {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "project directory does not exist: {}",
                path.display()
            )))
        }
    }

    pub fn is_git_repository(&self, path: &Path) -> bool {
        path.join(".git").exists()
    }

    pub fn validate_worktree_directory(&self, path: &Path) -> AppResult<()> {
        if path.exists() && !path.is_dir() {
            return Err(AppError::Validation(format!(
                "worktree path is not a directory: {}",
                path.display()
            )));
        }

        let parent = path.parent().map(PathBuf::from).ok_or_else(|| {
            AppError::Validation(format!("worktree path has no parent: {}", path.display()))
        })?;

        if !parent.exists() {
            return Err(AppError::Validation(format!(
                "worktree parent does not exist: {}",
                parent.display()
            )));
        }

        Ok(())
    }

    pub fn ensure_accessible(&self, path: &Path) -> AppResult<()> {
        std::fs::read_dir(path)
            .map(|_| ())
            .map_err(|error| AppError::Infrastructure(error.to_string()))
    }
}
