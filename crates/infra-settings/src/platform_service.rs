use shared_kernel::{AppError, AppResult};
use std::{fs, path::PathBuf};

#[derive(Debug, Default, Clone, Copy)]
pub struct PlatformService;

impl PlatformService {
    pub fn app_data_dir(&self, app_name: &str) -> AppResult<PathBuf> {
        let base = dirs::data_local_dir().ok_or_else(|| {
            AppError::Infrastructure("failed to resolve local app data directory".into())
        })?;
        Ok(base.join(app_name))
    }

    pub fn default_runtime_candidates(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        #[cfg(target_os = "macos")]
        {
            candidates.extend([
                PathBuf::from("/opt/homebrew/bin/pi"),
                PathBuf::from("/opt/homebrew/bin/pi-mono"),
                PathBuf::from("/usr/local/bin/pi"),
                PathBuf::from("/usr/local/bin/pi-mono"),
                PathBuf::from("/Applications/Pi.app/Contents/MacOS/pi"),
                PathBuf::from("/Applications/pi-mono.app/Contents/MacOS/pi-mono"),
            ]);
        }

        #[cfg(target_os = "windows")]
        {
            candidates.extend([
                PathBuf::from(r"C:\Program Files\pi\pi.exe"),
                PathBuf::from(r"C:\Program Files\pi-mono\pi-mono.exe"),
                PathBuf::from(r"C:\Program Files (x86)\pi\pi.exe"),
                PathBuf::from(r"C:\Program Files (x86)\pi-mono\pi-mono.exe"),
            ]);
        }

        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            candidates.extend([
                PathBuf::from("/usr/local/bin/pi"),
                PathBuf::from("/usr/local/bin/pi-mono"),
                PathBuf::from("/usr/bin/pi"),
                PathBuf::from("/usr/bin/pi-mono"),
                PathBuf::from("/opt/pi/bin/pi"),
                PathBuf::from("/opt/pi-mono/bin/pi-mono"),
            ]);
        }

        if let Some(home) = dirs::home_dir() {
            candidates.extend([
                home.join(".local/bin/pi"),
                home.join(".local/bin/pi-mono"),
                home.join(".bun/bin/pi"),
                home.join(".bun/bin/pi-mono"),
            ]);

            let fnm_installations = home.join(".local/share/fnm/node-versions");
            if let Ok(entries) = fs::read_dir(fnm_installations) {
                for entry in entries.flatten() {
                    let base = entry.path().join("installation/bin");
                    candidates.push(base.join("pi"));
                    candidates.push(base.join("pi-mono"));
                }
            }
        }

        candidates
    }

    pub fn default_config_dir_candidates(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                candidates.push(home.join(".pi/agent"));
                candidates.push(home.join("Library/Application Support/pi/agent"));
                candidates.push(home.join("Library/Application Support/pi"));
                candidates.push(home.join("Library/Application Support/pi-mono"));
                candidates.push(home.join(".config/pi"));
                candidates.push(home.join(".config/pi-mono"));
            }
            return candidates;
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(config_dir) = dirs::config_dir() {
                candidates.push(config_dir.join("pi/agent"));
                candidates.push(config_dir.join("pi"));
                candidates.push(config_dir.join("pi-mono"));
            }
            if let Some(home) = dirs::home_dir() {
                candidates.push(home.join(".pi/agent"));
                candidates.push(home.join(".pi"));
            }
            return candidates;
        }

        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            if let Some(config_dir) = dirs::config_dir() {
                candidates.push(config_dir.join("pi/agent"));
                candidates.push(config_dir.join("pi"));
                candidates.push(config_dir.join("pi-mono"));
            }
            if let Some(home) = dirs::home_dir() {
                candidates.push(home.join(".pi/agent"));
                candidates.push(home.join(".config/pi"));
                candidates.push(home.join(".config/pi-mono"));
                candidates.push(home.join(".pi"));
            }
            candidates
        }
    }
}
