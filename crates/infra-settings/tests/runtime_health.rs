use infra_settings::{AppSettings, FsService, PlatformService, RuntimeLocator, RuntimeSource};
use std::{collections::BTreeMap, fs, path::Path};
use tempfile::tempdir;

#[test]
fn missing_runtime_path_returns_blocked_health() {
    let locator = RuntimeLocator::new(FsService, PlatformService);
    let settings = AppSettings {
        runtime_path: Some(Path::new("/tmp/does-not-exist-pi-mono").to_path_buf()),
        ..AppSettings::default()
    };

    let result = locator.locate(&settings, &BTreeMap::new());

    assert!(!result.health.available);
    assert!(result.health.reason.expect("reason").contains("missing"));
    assert_eq!(result.source, None);
}

#[test]
fn existing_executable_returns_available_health() {
    let temp = tempdir().expect("tempdir");
    let executable = temp.path().join(if cfg!(target_os = "windows") {
        "pi-mono.bat"
    } else {
        "pi-mono"
    });

    #[cfg(target_os = "windows")]
    fs::write(&executable, "@echo off\necho pi-mono 0.1.0\n").expect("write executable");

    #[cfg(not(target_os = "windows"))]
    {
        fs::write(&executable, "#!/bin/sh\necho pi-mono 0.1.0\n").expect("write executable");
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("chmod");
    }

    let locator = RuntimeLocator::new(FsService, PlatformService);
    let settings = AppSettings {
        runtime_path: Some(executable.clone()),
        ..AppSettings::default()
    };

    let result = locator.locate(&settings, &BTreeMap::new());

    assert!(result.health.available);
    assert_eq!(result.resolved_path.as_ref(), Some(&executable));
    assert!(
        result
            .health
            .version
            .expect("version")
            .contains("pi-mono 0.1.0")
    );
}

#[test]
fn finds_pi_binary_from_path_without_manual_configuration() {
    let temp = tempdir().expect("tempdir");
    let executable = temp.path().join(if cfg!(target_os = "windows") {
        "pi.exe"
    } else {
        "pi"
    });

    #[cfg(target_os = "windows")]
    fs::write(&executable, "@echo off\necho pi 0.2.0\n").expect("write executable");

    #[cfg(not(target_os = "windows"))]
    {
        fs::write(&executable, "#!/bin/sh\necho pi 0.2.0\n").expect("write executable");
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("chmod");
    }

    let locator = RuntimeLocator::new(FsService, PlatformService);
    let mut env = BTreeMap::new();
    env.insert("PATH".into(), temp.path().display().to_string());

    let result = locator.locate(&AppSettings::default(), &env);

    assert!(result.health.available);
    assert_eq!(result.source, Some(RuntimeSource::Path));
    assert_eq!(result.resolved_path.as_ref(), Some(&executable));
}

#[test]
fn finds_config_directory_from_env_without_manual_configuration() {
    let temp = tempdir().expect("tempdir");
    let config_dir = temp.path().join("pi-config");
    fs::create_dir_all(&config_dir).expect("config dir");

    let locator = RuntimeLocator::new(FsService, PlatformService);
    let mut env = BTreeMap::new();
    env.insert(
        "PI_CODING_AGENT_DIR".into(),
        config_dir.display().to_string(),
    );

    let result = locator.locate_config_directory(&AppSettings::default(), &env);

    assert_eq!(result.resolved_path.as_ref(), Some(&config_dir));
}
