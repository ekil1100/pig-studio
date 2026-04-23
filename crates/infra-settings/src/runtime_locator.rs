use crate::{FsService, PlatformService, settings_store::AppSettings};
use chrono::Utc;
use domain::RuntimeHealth;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeSource {
    Settings,
    Env,
    Path,
    PlatformDefault,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeAttempt {
    pub source: RuntimeSource,
    pub path: Option<PathBuf>,
    pub success: bool,
    pub version: Option<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeLookupResult {
    pub resolved_path: Option<PathBuf>,
    pub source: Option<RuntimeSource>,
    pub health: RuntimeHealth,
    pub attempts: Vec<RuntimeAttempt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigDirectorySource {
    Settings,
    Env,
    PlatformDefault,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDirectoryAttempt {
    pub source: ConfigDirectorySource,
    pub path: Option<PathBuf>,
    pub success: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDirectoryLookupResult {
    pub resolved_path: Option<PathBuf>,
    pub source: Option<ConfigDirectorySource>,
    pub reason: Option<String>,
    pub attempts: Vec<ConfigDirectoryAttempt>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeLocator {
    fs: FsService,
    platform: PlatformService,
}

impl RuntimeLocator {
    pub fn new(fs: FsService, platform: PlatformService) -> Self {
        Self { fs, platform }
    }

    pub fn locate(
        &self,
        settings: &AppSettings,
        env: &BTreeMap<String, String>,
    ) -> RuntimeLookupResult {
        let mut attempts = Vec::new();

        if let Some(path) = settings.runtime_path.clone() {
            if let Some(result) = self.try_candidate(RuntimeSource::Settings, path, &mut attempts) {
                return result;
            }
        } else {
            attempts.push(RuntimeAttempt {
                source: RuntimeSource::Settings,
                path: None,
                success: false,
                version: None,
                reason: Some("runtime_path not configured".into()),
            });
        }

        let env_runtime_path = env
            .get("PI_PATH")
            .or_else(|| env.get("PI_MONO_PATH"))
            .map(PathBuf::from);
        if let Some(path) = env_runtime_path {
            if let Some(result) = self.try_candidate(RuntimeSource::Env, path, &mut attempts) {
                return result;
            }
        } else {
            attempts.push(RuntimeAttempt {
                source: RuntimeSource::Env,
                path: None,
                success: false,
                version: None,
                reason: Some("PI_PATH / PI_MONO_PATH not set".into()),
            });
        }

        let mut found_in_path = false;
        if let Some(path_var) = env.get("PATH") {
            for directory in std::env::split_paths(path_var) {
                for executable_name in executable_names() {
                    let candidate = directory.join(executable_name);
                    if candidate.exists() {
                        found_in_path = true;
                        if let Some(result) =
                            self.try_candidate(RuntimeSource::Path, candidate, &mut attempts)
                        {
                            return result;
                        }
                    }
                }
            }
        }
        if !found_in_path {
            attempts.push(RuntimeAttempt {
                source: RuntimeSource::Path,
                path: None,
                success: false,
                version: None,
                reason: Some("pi runtime not found in PATH".into()),
            });
        }

        let defaults = self.platform.default_runtime_candidates();
        let mut found_default = false;
        for candidate in defaults {
            if candidate.exists() {
                found_default = true;
                if let Some(result) =
                    self.try_candidate(RuntimeSource::PlatformDefault, candidate, &mut attempts)
                {
                    return result;
                }
            }
        }
        if !found_default {
            attempts.push(RuntimeAttempt {
                source: RuntimeSource::PlatformDefault,
                path: None,
                success: false,
                version: None,
                reason: Some("no platform default runtime candidate found".into()),
            });
        }

        RuntimeLookupResult {
            resolved_path: None,
            source: None,
            health: RuntimeHealth::blocked("missing runtime", Utc::now()),
            attempts,
        }
    }

    pub fn locate_config_directory(
        &self,
        settings: &AppSettings,
        env: &BTreeMap<String, String>,
    ) -> ConfigDirectoryLookupResult {
        let mut attempts = Vec::new();

        if let Some(path) = settings.config_dir.clone() {
            if let Some(result) =
                self.try_config_candidate(ConfigDirectorySource::Settings, path, &mut attempts)
            {
                return result;
            }
        } else {
            attempts.push(ConfigDirectoryAttempt {
                source: ConfigDirectorySource::Settings,
                path: None,
                success: false,
                reason: Some("config_dir not configured".into()),
            });
        }

        let env_config_path = env
            .get("PI_CODING_AGENT_DIR")
            .or_else(|| env.get("PI_CONFIG_DIR"))
            .or_else(|| env.get("PI_MONO_CONFIG_DIR"))
            .map(PathBuf::from);
        if let Some(path) = env_config_path {
            if let Some(result) =
                self.try_config_candidate(ConfigDirectorySource::Env, path, &mut attempts)
            {
                return result;
            }
        } else {
            attempts.push(ConfigDirectoryAttempt {
                source: ConfigDirectorySource::Env,
                path: None,
                success: false,
                reason: Some(
                    "PI_CODING_AGENT_DIR / PI_CONFIG_DIR / PI_MONO_CONFIG_DIR not set".into(),
                ),
            });
        }

        let mut found_default = false;
        for candidate in self.platform.default_config_dir_candidates() {
            if candidate.exists() {
                found_default = true;
                if let Some(result) = self.try_config_candidate(
                    ConfigDirectorySource::PlatformDefault,
                    candidate,
                    &mut attempts,
                ) {
                    return result;
                }
            }
        }

        if !found_default {
            attempts.push(ConfigDirectoryAttempt {
                source: ConfigDirectorySource::PlatformDefault,
                path: None,
                success: false,
                reason: Some("no platform default config directory candidate found".into()),
            });
        }

        ConfigDirectoryLookupResult {
            resolved_path: None,
            source: None,
            reason: Some("missing config directory".into()),
            attempts,
        }
    }

    fn try_candidate(
        &self,
        source: RuntimeSource,
        path: PathBuf,
        attempts: &mut Vec<RuntimeAttempt>,
    ) -> Option<RuntimeLookupResult> {
        match self.validate_candidate(&path) {
            Ok(version) => {
                attempts.push(RuntimeAttempt {
                    source: source.clone(),
                    path: Some(path.clone()),
                    success: true,
                    version: Some(version.clone()),
                    reason: None,
                });
                Some(RuntimeLookupResult {
                    resolved_path: Some(path),
                    source: Some(source),
                    health: RuntimeHealth::available(version, Utc::now()),
                    attempts: attempts.clone(),
                })
            }
            Err(reason) => {
                attempts.push(RuntimeAttempt {
                    source,
                    path: Some(path),
                    success: false,
                    version: None,
                    reason: Some(reason),
                });
                None
            }
        }
    }

    fn try_config_candidate(
        &self,
        source: ConfigDirectorySource,
        path: PathBuf,
        attempts: &mut Vec<ConfigDirectoryAttempt>,
    ) -> Option<ConfigDirectoryLookupResult> {
        match self.validate_config_directory(&path) {
            Ok(()) => {
                attempts.push(ConfigDirectoryAttempt {
                    source: source.clone(),
                    path: Some(path.clone()),
                    success: true,
                    reason: None,
                });
                Some(ConfigDirectoryLookupResult {
                    resolved_path: Some(path),
                    source: Some(source),
                    reason: None,
                    attempts: attempts.clone(),
                })
            }
            Err(reason) => {
                attempts.push(ConfigDirectoryAttempt {
                    source,
                    path: Some(path),
                    success: false,
                    reason: Some(reason),
                });
                None
            }
        }
    }

    fn validate_candidate(&self, path: &Path) -> Result<String, String> {
        if !self.fs.path_exists(path) {
            return Err(format!("missing executable: {}", path.display()));
        }

        if !path.is_file() {
            return Err(format!("runtime path is not a file: {}", path.display()));
        }

        let output = Command::new(path)
            .arg("--version")
            .output()
            .map_err(|error| format!("failed to execute runtime: {error}"))?;

        if !output.status.success() {
            return Err(format!(
                "runtime version probe failed with status {:?}",
                output.status.code()
            ));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if version.is_empty() {
            return Err("runtime did not return a version string".into());
        }

        Ok(version)
    }

    fn validate_config_directory(&self, path: &Path) -> Result<(), String> {
        if !self.fs.path_exists(path) {
            return Err(format!("missing config directory: {}", path.display()));
        }

        if !path.is_dir() {
            return Err(format!(
                "config path is not a directory: {}",
                path.display()
            ));
        }

        Ok(())
    }
}

fn executable_names() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["pi-mono.exe", "pi.exe"]
    }

    #[cfg(not(target_os = "windows"))]
    {
        &["pi-mono", "pi"]
    }
}
