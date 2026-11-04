//! The toolchain repair: which launcher records are stale, and the order and gating of the two mpm-shadow runs.

use super::*;
use crate::test_support::executable;
use crate::package_repair::RepairEnvironment;
use crate::package_repair_report::{Outcome, Tool};
use crate::test_support::{fixture, remove};
use std::time::Duration;

/// The launcher directory of a worktree, relative to its root.
const LAUNCHERS: &str = ".cache/toolchain-launchers";

/// Write a launcher record that names `lock`.
fn record(destination: &Path, name: &str, lock: &Path) {
    let directory: PathBuf = destination.join(LAUNCHERS);
    std::fs::create_dir_all(&directory).expect("launchers");
    let text: String = format!("{{\"toolId\":\"{name}\",\"lockFile\":\"{}\",\"real\":\"/tools/real\",\"scope\":\"repo\"}}", lock.display());
    std::fs::write(directory.join(format!("{name}.launch")), text).expect("record");
}

/// A destination, a fixture directory for tools and the environment that searches it.
fn scene(name: &str) -> (PathBuf, PathBuf, PathBuf, RepairEnvironment) {
    let root: PathBuf = fixture(name);
    let destination: PathBuf = root.join("destination");
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir_all(&destination).expect("destination");
    std::fs::create_dir_all(&bin).expect("bin");
    let environment: RepairEnvironment = RepairEnvironment { search_path: Some(bin.clone().into_os_string()), quiet_window: Duration::from_secs(60), ceiling: Duration::from_secs(600), workers: 1 };
    return (root, destination, bin, environment);
}

/// Only records that name another lock are stale; a record of this worktree and a file that is not a record are not.
#[test]
fn finds_the_records_that_name_another_lock() {
    let (root, destination, _bin, _environment) = scene("toolchain-stale");
    record(&destination, "own", destination.join("toolchain.lock.json").as_path());
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    std::fs::write(destination.join(LAUNCHERS).join("junk.launch"), b"not json").expect("junk");
    std::fs::write(destination.join(LAUNCHERS).join("tool"), b"binary").expect("launcher");
    let stale: Vec<PathBuf> = stale_launch_records(destination.as_path());
    assert_eq!(stale, vec![destination.join(LAUNCHERS).join("foreign.launch")]);
    remove(&root);
}

/// A worktree with no launcher directory has nothing stale.
#[test]
fn a_worktree_without_launchers_has_nothing_stale() {
    let (root, destination, _bin, _environment) = scene("toolchain-none");
    assert!(stale_launch_records(destination.as_path()).is_empty());
    remove(&root);
}

/// The summary line a real `mpm-shadow install` prints on standard output.
const SUMMARY: &str = "toolchain install: 3 installed, 62 unchanged, 0 failed";

/// A fixture `mpm-shadow` that records every start, prints the summary on standard output, rewrites the stale record, and exits
///  with `install_status`.
fn fake_mpm(root: &Path, bin: &Path, destination: &Path, install_status: i32) {
    let script: String = format!(
        "#!/bin/sh\necho \"$* in $(pwd -P) root=$MONOCHROMATIC_WORKTREE_ROOT\" >> '{record}'\necho '{SUMMARY}'\nif [ {install_status} -eq 0 ]; then sed -i 's#/source/#/destination/#' '{launch}'; fi\nexit {install_status}\n",
        record = root.join("record").display(),
        launch = destination.join(LAUNCHERS).join("foreign.launch").display(),
    );
    executable(bin.join("mpm-shadow").as_path(), script.as_bytes());
}

/// The install runs once, frozen, in the worktree with the worktree as the toolchain root, whether or not the installed tools
///  already match the lock (the owner accepted an online install during an add), and the summary is reported.
#[test]
fn installs_frozen_once_in_the_worktree_and_reports_the_summary() {
    let (root, destination, bin, environment) = scene("toolchain-install");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    fake_mpm(&root, &bin, &destination, 0);
    let finding: Finding = repair_toolchain(destination.as_path(), &environment).expect("a finding");
    assert_eq!(finding.tool, Tool::Toolchain);
    let Outcome::Repaired { detail, .. } = &finding.outcome else { panic!("expected a success: {finding:?}") };
    assert!(detail.contains("3 installed"), "the summary of what was installed is kept: {detail}");
    let recorded: String = std::fs::read_to_string(root.join("record")).expect("record");
    let lines: Vec<&str> = recorded.lines().collect();
    assert_eq!(lines.len(), 1, "{recorded}");
    assert!(lines[0].starts_with("install") && lines[0].contains("--frozen-lockfile"), "{recorded}");
    assert!(lines[0].contains(destination.to_str().expect("utf8")), "{recorded}");
    remove(&root);
}

/// A failing install never fails the add: it is a failure finding with the reason and the person's own command.
#[test]
fn a_failing_install_names_its_reason_and_the_command() {
    let (root, destination, bin, environment) = scene("toolchain-install-fails");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    fake_mpm(&root, &bin, &destination, 1);
    let finding: Finding = repair_toolchain(destination.as_path(), &environment).expect("a finding");
    let Outcome::Failed { reason, run_instead } = finding.outcome else { panic!("expected a failure") };
    assert!(reason.contains("exit status"), "{reason}");
    assert!(run_instead.contains(MANUAL_COMMAND), "{run_instead}");
    remove(&root);
}

/// An install that leaves a stale record is a failure, not a success.
#[test]
fn a_record_that_stays_stale_is_a_failure() {
    let (root, destination, bin, environment) = scene("toolchain-stays");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    executable(bin.join("mpm-shadow").as_path(), b"#!/bin/sh\nexit 0\n");
    let finding: Finding = repair_toolchain(destination.as_path(), &environment).expect("a finding");
    assert!(matches!(finding.outcome, Outcome::Failed { .. }), "{finding:?}");
    remove(&root);
}

/// No stale record, no start; no `mpm-shadow` anywhere, a failure that names the command.
#[test]
fn applies_only_to_stale_records_and_reports_a_missing_tool() {
    let (root, destination, bin, environment) = scene("toolchain-applies");
    executable(bin.join("mpm-shadow").as_path(), format!("#!/bin/sh\necho started >> '{}'\n", root.join("record").display()).as_bytes());
    record(&destination, "own", destination.join("toolchain.lock.json").as_path());
    assert!(repair_toolchain(destination.as_path(), &environment).is_none());
    assert!(!root.join("record").exists());
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    std::fs::remove_file(bin.join("mpm-shadow")).expect("remove tool");
    let Some(Finding { outcome: Outcome::Failed { run_instead, .. }, .. }) = repair_toolchain(destination.as_path(), &environment) else { panic!("expected a failure") };
    assert!(run_instead.contains(MANUAL_COMMAND), "{run_instead}");
    remove(&root);
}
