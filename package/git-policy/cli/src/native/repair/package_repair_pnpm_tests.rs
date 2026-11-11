//! The pnpm repair with a fixture `pnpm`:
//! when it applies,
//! how it is started,
//! and what each end is called.

use super::*;
use crate::test_support::executable;
use crate::package_repair_report::{Outcome, Tool};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;
use std::time::Duration;

/// A destination,
/// a fixture directory for tools,
/// a record file and the environment that searches only that directory.
fn scene(name: &str) -> (PathBuf, PathBuf, PathBuf, RepairEnvironment) {
    let root: PathBuf = fixture(name);
    let destination: PathBuf = root.join("destination");
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir_all(destination.join("node_modules")).expect("node_modules");
    std::fs::create_dir_all(&bin).expect("bin");
    std::fs::write(destination.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("lockfile");
    let environment: RepairEnvironment = RepairEnvironment { search_path: Some(bin.clone().into_os_string()), quiet_window: Duration::from_secs(60), ceiling: Duration::from_secs(600), workers: 1 };
    return (root, destination, bin, environment);
}

/// pnpm is started in the worktree with the offline and frozen-lockfile switches,
/// the words the owner asked for.
#[test]
fn starts_pnpm_offline_and_frozen_in_the_worktree() {
    let (root, destination, bin, environment) = scene("pnpm-args");
    let record: PathBuf = root.join("record");
    executable(bin.join("pnpm").as_path(), format!("#!/bin/sh\necho \"$* in $(pwd -P)\" > '{}'\n", record.display()).as_bytes());
    let finding: Finding = repair_pnpm(destination.as_path(), &environment).expect("a finding");
    assert_eq!(finding.tool, Tool::Pnpm);
    assert!(matches!(finding.outcome, Outcome::Repaired { .. }), "{finding:?}");
    let recorded: String = std::fs::read_to_string(&record).expect("record");
    assert!(recorded.contains("install") && recorded.contains("--offline") && recorded.contains("--frozen-lockfile"), "{recorded}");
    assert!(recorded.trim_end().ends_with(destination.to_str().expect("utf8")), "{recorded}");
    remove(&root);
}

/// Without a copied `node_modules` or without a lockfile there is nothing to relink and nothing is started.
#[test]
fn applies_only_with_a_lockfile_and_node_modules() {
    let (root, destination, bin, environment) = scene("pnpm-applies");
    let record: PathBuf = root.join("record");
    executable(bin.join("pnpm").as_path(), format!("#!/bin/sh\necho started > '{}'\n", record.display()).as_bytes());
    std::fs::remove_dir(destination.join("node_modules")).expect("remove node_modules");
    assert!(repair_pnpm(destination.as_path(), &environment).is_none());
    std::fs::create_dir(destination.join("node_modules")).expect("node_modules");
    std::fs::remove_file(destination.join("pnpm-lock.yaml")).expect("remove lockfile");
    assert!(repair_pnpm(destination.as_path(), &environment).is_none());
    assert!(!record.exists());
    remove(&root);
}

/// A failing pnpm is a failure whose command names the worktree,
/// so it runs from anywhere.
#[test]
fn a_failure_names_a_command_that_runs_from_anywhere() {
    let (root, destination, bin, environment) = scene("pnpm-fails");
    executable(bin.join("pnpm").as_path(), b"#!/bin/sh\necho 'ERR_PNPM_NO_OFFLINE_TARBALL' >&2\nexit 1\n");
    let finding: Finding = repair_pnpm(destination.as_path(), &environment).expect("a finding");
    let Outcome::Failed { reason, run_instead } = finding.outcome else { panic!("expected a failure") };
    assert!(reason.contains("ERR_PNPM_NO_OFFLINE_TARBALL"), "{reason}");
    assert!(run_instead.contains(destination.to_str().expect("utf8")) && run_instead.contains("pnpm"), "{run_instead}");
    remove(&root);
}

/// A pnpm that shows no sign of work for its window is stopped and reported,
/// not waited for.
#[test]
fn a_slow_pnpm_is_stopped_and_reported() {
    let (root, destination, bin, mut environment) = scene("pnpm-slow");
    environment.quiet_window = Duration::from_millis(300);
    // The install's own children read the search path the repair resolved it on, so the fixture names what it needs
    //  absolutely and reaches nothing else.
    executable(bin.join("pnpm").as_path(), b"#!/bin/sh\nexec /usr/bin/sleep 30\n");
    let finding: Finding = repair_pnpm(destination.as_path(), &environment).expect("a finding");
    let Outcome::Failed { reason, .. } = finding.outcome else { panic!("expected a failure") };
    assert!(reason.contains("no sign of work"), "{reason}");
    remove(&root);
}
