use app_core::{SettingsStorePort, ports::RuntimeSettings};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{AppError, AppResult};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub runtime_path: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub environment: BTreeMap<String, String>,
    pub last_checked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> AppResult<AppSettings> {
        if !self.path.exists() {
            return Ok(AppSettings::default());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        serde_json::from_str(&content).map_err(|error| AppError::Infrastructure(error.to_string()))
    }

    pub fn save(&self, settings: &AppSettings) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        }

        let content = serde_json::to_string_pretty(settings)
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        fs::write(&self.path, content)
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }
}

impl From<AppSettings> for RuntimeSettings {
    fn from(value: AppSettings) -> Self {
        Self {
            runtime_path: value.runtime_path,
            config_dir: value.config_dir,
            environment: value.environment,
            last_checked_at: value.last_checked_at,
        }
    }
}

impl From<RuntimeSettings> for AppSettings {
    fn from(value: RuntimeSettings) -> Self {
        Self {
            runtime_path: value.runtime_path,
            config_dir: value.config_dir,
            environment: value.environment,
            last_checked_at: value.last_checked_at,
        }
    }
}

impl SettingsStorePort for SettingsStore {
    fn load(&self) -> AppResult<RuntimeSettings> {
        SettingsStore::load(self).map(Into::into)
    }

    fn save(&self, settings: &RuntimeSettings) -> AppResult<()> {
        SettingsStore::save(self, &settings.clone().into())
    }
}
