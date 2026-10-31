//! Unit tests of the launcher's decision:
//! the record on disk,
//! and when the warning is printed.
//! Starting a program and passing its streams and exit status through are covered by `tests/launcher.rs`,
//! which runs the built binary.

use std::fs;
use std::path::{Path, PathBuf};

use super::cargo_installs::older_lock_message;
use super::fakes::Rig;
use super::launcher::{LaunchRecord, ensure_launcher, parse_record, prepare_launch, record_path, render_record};

/// The id the tests use for the tool.
const TOOL: &str = "cargo:monochromatic-git-policy-cli";

/// A lock text pinning the tool at `version`.
fn lock_text(version: &str) -> String {
    return format!("{{\"schemaVersion\":1,\"tools\":{{\"{TOOL}\":{{\"version\":\"{version}\",\"route\":\"cargo-binstall\"}}}}}}");
}

/// A crate directory with versions `installed` and `current` pointing at `current`,
/// and the record a launcher in it would carry.
fn record_with_current(directory: &Path, installed: &[&str], current: &str) -> LaunchRecord {
    let crate_dir: PathBuf = directory.join("cargo").join("monochromatic-git-policy-cli");
    for version in installed {
        fs::create_dir_all(crate_dir.join(version).join("bin")).expect("make");
    }
    std::os::unix::fs::symlink(current, crate_dir.join("current")).expect("link current");
    return LaunchRecord {
        tool_id: TOOL.to_string(),
        lock_file: directory.join("toolchain.lock.json"),
        real: crate_dir.join("current").join("bin").join("cli-git-native"),
    };
}

#[test]
fn a_record_round_trips_and_a_bad_one_is_refused() {
    let record = LaunchRecord {
        tool_id: TOOL.to_string(),
        lock_file: PathBuf::from("/work/toolchain.lock.json"),
        real: PathBuf::from("/root/cargo/x/current/bin/x"),
    };
    assert_eq!(parse_record(&render_record(&record)), Some(record));
    assert_eq!(parse_record("not json"), None);
    assert_eq!(parse_record("{\"toolId\":\"a\"}"), None);
    assert_eq!(record_path(Path::new("/a/b/git")), PathBuf::from("/a/b/git.launch"));
}

#[test]
fn the_launcher_warns_when_the_version_that_runs_is_newer_than_the_lock() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    let launch = prepare_launch(&record, Some(&lock_text("0.4.0")));
    assert_eq!(launch.warning, Some(older_lock_message(TOOL, "0.4.0", "0.4.4")));
    assert_eq!(launch.program, record.real);
}

#[test]
fn the_launcher_is_silent_when_the_lock_and_the_version_that_runs_match() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    assert_eq!(prepare_launch(&record, Some(&lock_text("0.4.4"))).warning, None);
}

#[test]
fn the_launcher_never_blocks_a_run_for_an_unreadable_lock_a_missing_tool_or_a_missing_link() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    assert_eq!(prepare_launch(&record, None).warning, None, "no lock file");
    assert_eq!(prepare_launch(&record, Some("{")).warning, None, "a broken lock");
    assert_eq!(prepare_launch(&record, Some("{\"tools\":{}}")).warning, None, "the tool is not in the lock");
    let unlinked = LaunchRecord { real: rig._directory.path().join("elsewhere").join("bin").join("x"), ..record };
    assert_eq!(prepare_launch(&unlinked, Some(&lock_text("0.1.0"))).warning, None, "no current link");
}

#[test]
fn a_launcher_is_made_with_its_record_beside_it() {
    let rig = Rig::new();
    let real = rig.scope.toolchain_root.join("cargo").join("x").join("current").join("bin").join("x");
    let launcher = ensure_launcher(&rig.scope, TOOL, "git", &real).expect("made");
    assert_eq!(launcher, rig.scope.launcher_dir.join("git"));
    assert!(launcher.is_file());
    let record = parse_record(&fs::read_to_string(record_path(&launcher)).expect("a record")).expect("parses");
    assert_eq!(record.real, real);
    assert_eq!(record.lock_file, rig.scope.lock_file);
    // A second call replaces both without failing.
    ensure_launcher(&rig.scope, TOOL, "git", &real).expect("made again");
}
