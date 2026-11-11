//! The toolchain repair:
//! which launcher records are stale,
//! and the order and gating of the two mpm-shadow runs.

use super::*;
use crate::test_support::executable;
use crate::package_repair_request::RepairEnvironment;
use crate::package_repair_report::{Outcome, Tool};
use crate::test_support::{fixture, remove};
use std::time::Duration;

/// The launcher directory of a worktree,
/// relative to its root.
const LAUNCHERS: &str = ".cache/toolchain-launchers";

/// Write a launcher record that names `lock`.
fn record(destination: &Path, name: &str, lock: &Path) {
    let directory: PathBuf = destination.join(LAUNCHERS);
    std::fs::create_dir_all(&directory).expect("launchers");
    let text: String = format!("{{\"toolId\":\"{name}\",\"lockFile\":\"{}\",\"real\":\"/tools/real\",\"scope\":\"repo\"}}", lock.display());
    std::fs::write(directory.join(format!("{name}.launch")), text).expect("record");
}

/// A destination,
/// a fixture directory for tools and the environment that searches it.
fn scene(name: &str) -> (PathBuf, PathBuf, PathBuf, RepairEnvironment) {
    let root: PathBuf = fixture(name);
    let destination: PathBuf = root.join("destination");
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir_all(&destination).expect("destination");
    std::fs::create_dir_all(&bin).expect("bin");
    let environment: RepairEnvironment = RepairEnvironment { search_path: Some(bin.clone().into_os_string()), quiet_window: Duration::from_secs(60), ceiling: Duration::from_secs(600), workers: 1 };
    return (root, destination, bin, environment);
}

/// Only records that name another lock are stale;
/// a record of this worktree and a file that is not a record are not.
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

/// A fixture `mpm-shadow` that records every start,
/// prints the summary on standard output,
/// rewrites the stale record,
/// and exits
///  with `install_status`.
fn fake_mpm(root: &Path, bin: &Path, destination: &Path, install_status: i32) {
    let script: String = format!(
        // The install's own children read the search path the repair resolved it on, so the fixture rewrites its launcher
        //  record with the shell's own `printf` and reaches no other program.
        "#!/bin/sh\necho \"$* in $(pwd -P) root=$MONOCHROMATIC_WORKTREE_ROOT\" >> '{record}'\necho '{SUMMARY}'\nif [ {install_status} -eq 0 ]; then printf '{{\"lockFile\":\"%s\"}}\\n' '{own}' > '{launch}'; fi\nexit {install_status}\n",
        record = root.join("record").display(),
        launch = destination.join(LAUNCHERS).join("foreign.launch").display(),
        own = destination.join(LOCK_FILE).display(),
    );
    executable(bin.join("mpm-shadow").as_path(), script.as_bytes());
}

/// The install runs once,
/// frozen,
/// in the worktree with the worktree as the toolchain root,
/// whether or not the installed tools
///  already match the lock (the owner accepted an online install during an add),
/// and the summary is reported.
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

/// A failing install never fails the add:
/// it is a failure finding with the reason and the person's own command.
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

/// An install that leaves a stale record is a failure,
/// not a success.
#[test]
fn a_record_that_stays_stale_is_a_failure() {
    let (root, destination, bin, environment) = scene("toolchain-stays");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    executable(bin.join("mpm-shadow").as_path(), b"#!/bin/sh\nexit 0\n");
    let finding: Finding = repair_toolchain(destination.as_path(), &environment).expect("a finding");
    assert!(matches!(finding.outcome, Outcome::Failed { .. }), "{finding:?}");
    remove(&root);
}

/// No stale record,
/// no start;
/// no `mpm-shadow` anywhere,
/// a failure that names the command.
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

/// How long an answer that must not wait may take in a control.
const PATIENCE: Duration = Duration::from_secs(10);

/// A named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// What:
///  A launcher record that is a named pipe with no writer is left alone,
///  so the repair answers and the waits beside it keep their deadlines.
/// Why:
///   The toolchain repair shares one task with the pnpm and Cargo repairs (`package_repair.rs`),
/// and a record that is opened by its extension alone blocks the whole task for ever.
/// A copy can leave any entry in the launcher directory,
/// and a pipe with no writer is the one entry that never answers.
#[test]
fn a_record_that_is_a_named_pipe_is_left_alone() {
    let (root, destination, _bin, environment) = scene("toolchain-pipe");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    make_pipe(destination.join(LAUNCHERS).join("stalled.launch").as_path());
    // Two workers, so a read that blocks on the pipe leaves a worker to time the answer.
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("a runtime");
    let repair = runtime.spawn(async move { return repair_toolchain_awaiting(destination.as_path(), &environment).await; });
    let answered = runtime.block_on(async { return tokio::time::timeout(PATIENCE, repair).await; });
    // The runtime is never dropped: a read that did block on the pipe holds a worker for ever, and dropping the runtime
    // would wait for that worker, so a control that caught the defect would hang instead of reporting it.
    std::mem::forget(runtime);
    remove(&root);
    let Ok(joined) = answered else {
        panic!("a launcher record that is a named pipe with no writer stopped the repair and every wait that shares its task, and nothing ended the wait")
    };
    let Some(Finding { outcome: Outcome::Failed { run_instead, .. }, .. }) = joined.expect("the repair task") else { panic!("the stale record beside the pipe is still repaired") };
    assert!(run_instead.contains(MANUAL_COMMAND), "{run_instead}");
}

/// What:
///  The launcher records are read as a job of the process's blocking pool,
///  never on the task that polls the repairs.
/// Why:
///   A synchronous read on the shared task stops the pnpm repair's quiet window and ceiling for as long as it takes,
/// and a control holds the process's only place,
/// so it proves the read reached the bound by asking for it.
#[test]
fn the_launcher_records_are_read_as_a_job_of_the_pool() {
    let (root, destination, _bin, environment) = scene("toolchain-asks-for-a-place");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    let (_given_back, finished) = crate::parallel_test_support::hold_the_only_place();
    let repair = crate::lazy_runtime::spawn_on_runtime(async move { return repair_toolchain_awaiting(destination.as_path(), &environment).await; });
    assert!(repair.is_some(), "this process's runtime has a worker of its own");
    let asked: bool = finished.recv_timeout(crate::parallel_test_support::PATIENCE).expect("the control that holds the only place answered");
    remove(&root);
    assert!(
        asked,
        "the launcher records were read without asking for a place of the process's bound on blocking jobs, so the read ran on the task that polls the repairs"
    );
}
