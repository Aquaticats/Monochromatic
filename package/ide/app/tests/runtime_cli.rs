//! Build-only asset preparation must not write ordinary user configuration.

/// Capture the real helper's output without invoking a shell or a GUI.
use std::process::Command;

/// Help succeeds even when no build-specific environment is supplied.
#[test]
fn runtime_help_does_not_prepare_assets() {
    let fixture = tempfile::tempdir().expect("isolated helper directory");
    let output = Command::new(env!("CARGO_BIN_EXE_ide-runtime"))
        .current_dir(fixture.path())
        .arg("--help")
        .output()
        .expect("run runtime help");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: ide-runtime"));
    assert!(output.stderr.is_empty());
    assert_eq!(
        std::fs::read_dir(fixture.path())
            .expect("inspect helper fixture")
            .count(),
        0
    );
}

/// Invalid operations stop before resolving or modifying configuration.
#[test]
fn runtime_rejects_unknown_operations() {
    let output = Command::new(env!("CARGO_BIN_EXE_ide-runtime"))
        .args(["unknown", "/unused/runtime"])
        .output()
        .expect("run invalid operation");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Expected fetch or build"));
}

/// Even a valid operation fails closed outside its target-owned configuration boundary.
#[test]
fn runtime_rejects_non_build_configuration_before_writing() {
    let fixture = tempfile::tempdir().expect("isolated helper directory");
    let outside = fixture.path().join("not-build-configuration");
    let output = Command::new(env!("CARGO_BIN_EXE_ide-runtime"))
        .current_dir(fixture.path())
        .env("XDG_CONFIG_HOME", &outside)
        .env_remove("CARGO_MANIFEST_DIR")
        .args(["fetch", "/unused/runtime"])
        .output()
        .expect("run guarded operation");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("writes only under"));
    assert!(!outside.exists());
}
