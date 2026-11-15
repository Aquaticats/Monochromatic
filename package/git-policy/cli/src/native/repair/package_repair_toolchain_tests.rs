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
/// rewrites the record of the launcher named `rewrites` (which is the one an install of a real lock answers for),
/// and exits
///  with `install_status`.
fn fake_mpm(root: &Path, bin: &Path, destination: &Path, rewrites: &str, install_status: i32) {
    let script: String = format!(
        // The install's own children read the search path the repair resolved it on, so the fixture rewrites its launcher
        //  record with the shell's own `printf` and reaches no other program.
        "#!/bin/sh\necho \"$* in $(pwd -P) root=$MONOCHROMATIC_WORKTREE_ROOT\" >> '{record}'\necho '{SUMMARY}'\nif [ {install_status} -eq 0 ]; then printf '{{\"lockFile\":\"%s\"}}\\n' '{own}' > '{launch}'; fi\nexit {install_status}\n",
        record = root.join("record").display(),
        launch = destination.join(LAUNCHERS).join(format!("{rewrites}.launch")).display(),
        own = destination.join(LOCK_FILE).display(),
    );
    executable(bin.join("mpm-shadow").as_path(), script.as_bytes());
}

/// Write the worktree's own lock,
/// declaring one tool with these bins,
/// which are the launcher names an install of that lock rewrites.
fn lock(destination: &Path, bins: &[&str]) {
    let declared: Vec<String> = bins.iter().map(|bin| format!("\"{bin}\"")).collect();
    let text: String = format!("{{\"schemaVersion\":1,\"tools\":{{\"cargo:own\":{{\"bins\":[{}]}}}}}}", declared.join(","));
    std::fs::write(destination.join(LOCK_FILE), text).expect("lock");
}

/// Write the launcher a record sits beside,
/// which is a file an install writes and nothing but a copy leaves behind.
fn launcher(destination: &Path, name: &str) {
    std::fs::create_dir_all(destination.join(LAUNCHERS)).expect("launchers");
    std::fs::write(destination.join(LAUNCHERS).join(name), b"#!/bin/sh\nexit 0\n").expect("launcher");
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
    fake_mpm(&root, &bin, &destination, "foreign", 0);
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
    fake_mpm(&root, &bin, &destination, "foreign", 1);
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

/// What:
///  A record for a name this worktree's lock declares nowhere is a copy of an earlier release's layout:
///  no install rewrites it,
///  so the relink removes it and reports the install's own success.
/// Why:
///  An add copies `.cache/toolchain-launchers` from the checkout the worktree was copied from,
///  and `mpm-shadow install` writes a launcher only for the bins the worktree's own lock declares.
///  A record left by a renamed bin was therefore counted as still stale after every install,
///  so every new worktree was reported as not relinked although the install had done everything its lock asked,
///  and the service retried an install that could never change that record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lock(destination, ['own']); record('own', source); record('renamed', source);
/// assert(outcomeOf(repairToolchain(destination)) === 'repaired'); assert(!exists('renamed.launch'));
/// ```
#[test]
fn a_record_the_lock_declares_nowhere_is_removed_instead_of_failing_the_relink() {
    let (root, destination, bin, environment) = scene("toolchain-copy-of-an-older-layout");
    lock(&destination, &["own"]);
    record(&destination, "own", root.join("source/toolchain.lock.json").as_path());
    record(&destination, "renamed", root.join("source/toolchain.lock.json").as_path());
    launcher(&destination, "renamed");
    // The install rewrites the record of the name the lock declares, which is all a real install does.
    fake_mpm(&root, &bin, &destination, "own", 0);
    let finding: Finding = repair_toolchain(destination.as_path(), &environment).expect("a finding");
    assert!(matches!(finding.outcome, Outcome::Repaired { .. }), "an install that did everything its lock asks is not a failed relink: {finding:?}");
    assert!(!destination.join(LAUNCHERS).join("renamed.launch").exists(), "the record no install rewrites is left in the worktree");
    assert!(!destination.join(LAUNCHERS).join("renamed").exists(), "the launcher beside that record is left in the worktree");
    assert!(destination.join(LAUNCHERS).join("own.launch").exists(), "the record the install rewrote is still there");
    remove(&root);
}

/// What:
///  A worktree whose only stale records are such copies starts no install at all,
///  and the copies still go.
/// Why:
///   Nothing an install does would change them,
///   so starting one costs a minute of the add or the service and answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lock(destination, ['own']); record('renamed', source);
/// assert(repairToolchain(destination) === undefined); assert(!exists('renamed.launch'));
/// ```
#[test]
fn a_worktree_whose_only_stale_records_are_copies_starts_no_install() {
    let (root, destination, bin, environment) = scene("toolchain-only-copies");
    lock(&destination, &["own"]);
    record(&destination, "renamed", root.join("source/toolchain.lock.json").as_path());
    fake_mpm(&root, &bin, &destination, "own", 0);
    assert!(repair_toolchain(destination.as_path(), &environment).is_none(), "an install was reported for records no install answers for");
    assert!(!root.join("record").exists(), "an install was started for records no install answers for");
    assert!(!destination.join(LAUNCHERS).join("renamed.launch").exists(), "the copy is left in the worktree");
    remove(&root);
}

/// What:
///  A record whose name the worktree's lock does declare is still a failed relink when the install leaves it stale,
///  and the worktree keeps it.
/// Why:
///   This is the state the repair exists to catch:
///   the layout this release writes was not rewritten,
///   and the worktree would run the copied checkout's tool versions.
///   The rule above may never turn that into a success.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lock(destination, ['own', 'declared']); record('own', source); record('declared', source);
/// assert(outcomeOf(repairToolchain(destination)).reason.includes('1 launcher records still name another lock file'));
/// ```
#[test]
fn a_record_the_lock_declares_is_still_a_failed_relink_when_the_install_leaves_it_stale() {
    let (root, destination, bin, environment) = scene("toolchain-declared-stays");
    lock(&destination, &["own", "declared"]);
    record(&destination, "own", root.join("source/toolchain.lock.json").as_path());
    record(&destination, "declared", root.join("source/toolchain.lock.json").as_path());
    fake_mpm(&root, &bin, &destination, "own", 0);
    let finding: Finding = repair_toolchain(destination.as_path(), &environment).expect("a finding");
    let Outcome::Failed { reason, run_instead } = finding.outcome else { panic!("a record the lock declares and the install leaves stale is a failure: {finding:?}") };
    assert!(reason.contains("1 launcher records still name another lock file"), "{reason}");
    assert!(run_instead.contains(MANUAL_COMMAND), "{run_instead}");
    assert!(destination.join(LAUNCHERS).join("declared.launch").exists(), "a record the check counts is never removed");
    remove(&root);
}

/// What:
///  The service's own face removes the copies too,
///  so a worktree relinked by the commit service holds nothing another checkout wrote.
/// Why:
///  The add's face and the service's face settle the worktree from the same decision,
///  and the service is the one that runs in production when a service answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await repairToolchainAwaiting(destination); assert(!exists('renamed.launch'));
/// ```
#[test]
fn the_service_face_removes_the_copies_as_well() {
    let (root, destination, bin, environment) = scene("toolchain-service-copies");
    lock(&destination, &["own"]);
    record(&destination, "own", root.join("source/toolchain.lock.json").as_path());
    record(&destination, "renamed", root.join("source/toolchain.lock.json").as_path());
    launcher(&destination, "renamed");
    fake_mpm(&root, &bin, &destination, "own", 0);
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime");
    let finding: Option<Finding> = runtime.block_on(async { return repair_toolchain_awaiting(destination.as_path(), &environment).await });
    let Some(answered) = finding else { panic!("the record the lock declares is still relinked") };
    assert!(matches!(answered.outcome, Outcome::Repaired { .. }), "{answered:?}");
    assert!(!destination.join(LAUNCHERS).join("renamed.launch").exists(), "the service face leaves the copy in the worktree");
    remove(&root);
}

/// What:
///  A `toolchain.lock.json` that is a named pipe with no writer is refused at once,
///  so the repair answers instead of holding the add's thread, or a job of the service's pool, for ever.
/// Why:
///   The repair reads the lock to learn which launcher names an install answers for,
///   and both faces of it are on a waiting path: the add's own thread, and a job the service awaits with no ceiling.
///  The record beside the lock is already read only when it is a plain file for the same reason
///  (`a_record_that_is_a_named_pipe_is_left_alone`);
///  the lock is a tracked file, so a pipe at its name takes unusual local state,
///  and nothing the wrapper reads may wait without end for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// record('foreign', source); mkfifo(join(destination, 'toolchain.lock.json'));
/// const answered = await timeout(PATIENCE, repairToolchainAwaiting(...)); assert(answered.outcome === 'failed');
/// ```
#[test]
fn a_lock_that_is_a_named_pipe_is_refused_without_waiting() {
    let (root, destination, _bin, environment) = scene("toolchain-lock-pipe");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    make_pipe(destination.join(LOCK_FILE).as_path());
    // Two workers, so a read that blocks on the pipe leaves a worker to time the answer.
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("a runtime");
    let repair = runtime.spawn(async move { return repair_toolchain_awaiting(destination.as_path(), &environment).await; });
    let answered = runtime.block_on(async { return tokio::time::timeout(PATIENCE, repair).await; });
    // The runtime is never dropped: a read that did block on the pipe holds a worker for ever, and dropping the runtime
    // would wait for that worker, so a control that caught the defect would hang instead of reporting it.
    std::mem::forget(runtime);
    remove(&root);
    let Ok(joined) = answered else {
        panic!("a toolchain.lock.json that is a named pipe with no writer stopped the repair and every wait that shares its task, and nothing ended the wait")
    };
    let Some(Finding { outcome: Outcome::Failed { run_instead, .. }, .. }) = joined.expect("the repair task") else {
        panic!("a lock that cannot be read leaves every record counted, which is a failure that names the command")
    };
    assert!(run_instead.contains(MANUAL_COMMAND), "{run_instead}");
}

/// What:
///  The add's own face refuses a lock that is a named pipe at once too,
///  so a `git worktree add` is not held by a file it cannot read.
/// Why:
///   The add asks the same question on its own thread (`prepare_install_on_a_caller_thread`),
///   and a blocking read there holds the command the person ran rather than a worker they do not see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(lock); const answer = await within(PATIENCE, () => repairToolchain(destination, environment)); assert(answer);
/// ```
#[test]
fn the_adds_face_refuses_a_lock_that_is_a_named_pipe_without_waiting() {
    let (root, destination, _bin, environment) = scene("toolchain-lock-pipe-add");
    record(&destination, "foreign", root.join("source/toolchain.lock.json").as_path());
    make_pipe(destination.join(LOCK_FILE).as_path());
    let (answered, answer) = std::sync::mpsc::channel::<Option<Finding>>();
    let thread = std::thread::spawn(move || {
        let finding: Option<Finding> = repair_toolchain(destination.as_path(), &environment);
        let _sent = answered.send(finding);
    });
    let within_the_patience: Result<Option<Finding>, _> = answer.recv_timeout(PATIENCE);
    // A thread that did block on the pipe is never joined, so a control that caught the defect reports it rather than
    // waiting on that thread for ever.
    if within_the_patience.is_ok() {
        thread.join().expect("the repair thread answers");
    } else {
        std::mem::forget(thread);
    }
    remove(&root);
    let Ok(answered_with) = within_the_patience else { panic!("the add's own repair waited for a toolchain.lock.json that is a named pipe, and nothing ended it") };
    let Some(Finding { outcome: Outcome::Failed { run_instead, .. }, .. }) = answered_with else { panic!("a lock that cannot be read leaves every record counted, which is a failure that names the command") };
    assert!(run_instead.contains(MANUAL_COMMAND), "{run_instead}");
}

/// What:
///  A record and launcher a concurrent `toolchain:update` and install write for a name the lock now declares
///  are left exactly as that command wrote them,
///  while a copy that is still a copy when the removal reaches it still goes.
/// Why:
///   The repair classifies records from one read of the lock and the launcher directory,
///   and the removal runs after that:
///   a person's update and install landing in between writes a correct record and launcher for a name the lock
///   declares,
///   and a removal that deletes the paths it classified without looking again discards what that command just made.
///   A launcher beside a record that is already gone is not this pass's to touch either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lock(destination, ['own']); record('own', source); record('renamed', source); launcher('renamed');
/// whenTheRemovalBegins(() => { lock(destination, ['own', 'renamed']); writeRecord('renamed', ownLock); launcher('renamed'); });
/// repairToolchain(destination, environment);
/// assert(readFile('renamed.launch').includes(ownLock) && exists('renamed') && exists('still-a-copy.launch') === false);
/// ```
#[test]
fn a_record_the_installer_wrote_in_the_removals_window_is_left_as_it_wrote_it() {
    let (root, destination, _bin, environment) = scene("toolchain-removal-window");
    lock(&destination, &["own"]);
    record(&destination, "own", root.join("source/toolchain.lock.json").as_path());
    record(&destination, "renamed", root.join("source/toolchain.lock.json").as_path());
    launcher(&destination, "renamed");
    record(&destination, "gone", root.join("source/toolchain.lock.json").as_path());
    let own_lock: PathBuf = destination.join(LOCK_FILE);
    let launchers: PathBuf = destination.join(LAUNCHERS);
    let installer_lock: PathBuf = own_lock.clone();
    let installer_launchers: PathBuf = launchers.clone();
    // The concurrent update and install: the lock now declares the name, and the record and launcher for it name this
    //  worktree. `ensure_launcher` writes the record first and then the launcher, so the window holds both.
    crate::package_repair_toolchain::WHEN_THE_REMOVAL_BEGINS.with(|cell: &std::cell::RefCell<Option<Box<dyn FnOnce()>>>| {
        *cell.borrow_mut() = Some(Box::new(move || {
            let tools: String = format!(
                "{{\"schemaVersion\":1,\"tools\":{{\"cargo:own\":{{\"bins\":[\"own\"]}},\"cargo:renamed\":{{\"bins\":[\"renamed\"]}}}}}}",
            );
            std::fs::write(installer_lock.as_path(), tools).expect("the installer's lock");
            let fresh: String = format!("{{\"toolId\":\"cargo:renamed\",\"lockFile\":\"{}\",\"real\":\"/tools/renamed\",\"scope\":\"repo\"}}\n", installer_lock.display());
            std::fs::write(installer_launchers.join("renamed.launch"), fresh).expect("the installer's record");
            std::fs::write(installer_launchers.join("renamed"), b"#!/bin/sh\nexit 0\n").expect("the installer's launcher");
        }));
    });
    let _finding: Option<Finding> = repair_toolchain(destination.as_path(), &environment);
    // The installer's record names this worktree's lock, so nothing in the pass may remove it or the launcher it wrote.
    let fresh_record: String = std::fs::read_to_string(launchers.join("renamed.launch")).expect("the installer's record is gone");
    assert!(fresh_record.contains(own_lock.to_str().expect("utf8")), "the installer's record was replaced: {fresh_record}");
    assert!(launchers.join("renamed").is_file(), "the installer's launcher is gone");
    // The copy that was still a copy when the removal reached it is gone, record and launcher.
    assert!(!launchers.join("gone.launch").exists(), "a copy no install rewrites is left behind");
    remove(&root);
}

/// A process number no process holds now,
/// which is what the name a record is set aside under carries once the pass that set it aside is killed.
fn dead_process_number() -> u32 {
    loop {
        let child: std::process::Child = std::process::Command::new("true").spawn().expect("a short-lived process");
        let pid: u32 = child.id();
        let _waited = child.wait_with_output();
        if !std::path::Path::new(format!("/proc/{pid}").as_str()).exists() {
            return pid;
        }
    }
}

/// What:
///  A record an install wrote while the removal was looking is put back by the pass after a killed one,
///  so the launcher beside it keeps its record and the tool keeps working.
/// Why:
///   The removal takes a record aside, reads it, and puts it back or removes it.
///   A pass killed between taking and putting back leaves the record under a name of its own,
///   and a concurrent `toolchain:update` plus install can have written that record after the classification:
///   the next pass must read what is there and put it back, never discard it,
///   because a launcher with no record beside it does not run its tool and nothing else recreates the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// record('zzz', source); whenTheRemovalBegins(() => writeTheInstallersOwn('zzz')); killAfterTheRecordIsAside();
/// repairToolchain(destination); renameTheAsidesToADeadPid(); repairToolchain(destination);
/// assert(readFile('zzz.launch').includes(ownLock) && exists('zzz'));
/// ```
#[test]
fn a_record_a_killed_pass_took_aside_is_put_back_by_the_next_pass() {
    let (root, destination, _bin, environment) = scene("toolchain-killed-aside");
    lock(&destination, &["own"]);
    record(&destination, "own", destination.join(LOCK_FILE).as_path());
    record(&destination, "zzz", root.join("source/toolchain.lock.json").as_path());
    launcher(&destination, "zzz");
    let own_lock: PathBuf = destination.join(LOCK_FILE);
    let launchers: PathBuf = destination.join(LAUNCHERS);
    // The install that lands after the classification: the lock declares the name, and a record and launcher for it
    //  name this worktree, as `mpm-shadow` writes them (the record first).
    let installer_lock: PathBuf = own_lock.clone();
    let installer_launchers: PathBuf = launchers.clone();
    crate::package_repair_toolchain::WHEN_THE_REMOVAL_BEGINS.with(|cell: &std::cell::RefCell<Option<Box<dyn FnOnce()>>>| {
        *cell.borrow_mut() = Some(Box::new(move || {
            let tools: String = String::from("{\"schemaVersion\":1,\"tools\":{\"cargo:own\":{\"bins\":[\"own\"]},\"cargo:zzz\":{\"bins\":[\"zzz\"]}}}");
            std::fs::write(installer_lock.as_path(), tools).expect("the installer's lock");
            let fresh: String = format!("{{\"toolId\":\"cargo:zzz\",\"lockFile\":\"{}\",\"real\":\"/tools/zzz\",\"scope\":\"repo\"}}\n", installer_lock.display());
            std::fs::write(installer_launchers.join("zzz.launch"), fresh).expect("the installer's record");
            std::fs::write(installer_launchers.join("zzz"), b"#!/bin/sh\n# the installer's launcher\nexit 0\n").expect("the installer's launcher");
        }));
    });
    crate::package_repair_toolchain::KILLED_AFTER_THE_RECORD_IS_ASIDE.with(|killed: &std::cell::Cell<bool>| killed.set(true));
    let _killed_pass: Option<Finding> = repair_toolchain(destination.as_path(), &environment);
    crate::package_repair_toolchain::KILLED_AFTER_THE_RECORD_IS_ASIDE.with(|killed: &std::cell::Cell<bool>| killed.set(false));
    // The pass that took a record aside is gone: what it left carries the name of a dead process, which is how the next
    //  pass tells a finished pass from one still running.
    let dead: u32 = dead_process_number();
    let mut left_aside: usize = 0;
    for entry in std::fs::read_dir(launchers.as_path()).expect("the launcher directory").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some((_, owner)) = name.rsplit_once(".removing-") else { continue };
        let Some((_written_pid, token)) = owner.split_once('-') else { continue };
        let killed_name: String = format!("{}.removing-{dead}-{token}", name.rsplit_once(".removing-").expect("split").0);
        std::fs::rename(entry.path(), launchers.join(killed_name)).expect("the killed pass's name");
        left_aside += 1;
    }
    assert!(left_aside <= 1, "one pass leaves one record aside at most: {left_aside}");
    let _next_pass: Option<Finding> = repair_toolchain(destination.as_path(), &environment);
    let record_after: String = std::fs::read_to_string(launchers.join("zzz.launch")).unwrap_or_default();
    assert!(record_after.contains(own_lock.to_str().expect("utf8")), "the installer's record is not where it belongs: {record_after:?}");
    let launcher_after: String = std::fs::read_to_string(launchers.join("zzz")).unwrap_or_default();
    assert!(launcher_after.contains("the installer's launcher"), "the installer's launcher is not the one beside it: {launcher_after:?}");
    let leftovers: Vec<String> = std::fs::read_dir(launchers.as_path()).expect("the launcher directory").flatten().map(|entry| return entry.file_name().to_string_lossy().into_owned()).filter(|name| return name.contains(".removing-")).collect();
    assert!(leftovers.is_empty(), "a record is still aside after the pass that follows: {leftovers:?}");
    remove(&root);
}

/// What:
///  A record that names this worktree's own lock through a link, or through a path with `..`,
///  is never taken for a copy of another checkout and never removed.
/// Why:
///   One file has many spellings:
///   a link to the worktree, a path through a directory and back, and a bind mount of the same folder all name the same
///   `toolchain.lock.json`, and only the file's identity tells that from a real copy of another checkout's lock.
///   A pass that compares spellings deletes a record an install wrote for a name the lock declares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlink(destination, alias); record('via-link', alias + '/toolchain.lock.json'); record('via-dots', destination + '/sub/../toolchain.lock.json');
/// repairToolchain(destination); assert(exists('via-link.launch') && exists('via-dots.launch'));
/// ```
#[test]
fn a_record_that_names_this_worktrees_lock_by_another_spelling_is_never_removed() {
    let (root, destination, _bin, environment) = scene("toolchain-spellings");
    lock(&destination, &["own"]);
    record(&destination, "own", destination.join(LOCK_FILE).as_path());
    // The worktree through a link, and through a path that names a directory and comes back out of it.
    let alias: PathBuf = root.join("alias");
    std::os::unix::fs::symlink(&destination, alias.as_path()).expect("a link to the worktree");
    std::fs::create_dir_all(destination.join("sub")).expect("a directory to walk into");
    record(&destination, "via-link", alias.join(LOCK_FILE).as_path());
    launcher(&destination, "via-link");
    record(&destination, "via-dots", destination.join("sub/../toolchain.lock.json").as_path());
    launcher(&destination, "via-dots");
    let _finding: Option<Finding> = repair_toolchain(destination.as_path(), &environment);
    let launchers: PathBuf = destination.join(LAUNCHERS);
    for name in ["via-link", "via-dots"] {
        assert!(launchers.join(format!("{name}.launch")).is_file(), "a record naming this worktree's lock through {name} was removed");
        assert!(launchers.join(name).is_file(), "the launcher beside {name}'s record was removed");
    }
    remove(&root);
}

/// What:
///  A record a killed pass left aside whose own name holds another record is kept and named for the person,
///  with the command that puts it back,
///  and is never discarded.
/// Why:
///   The pass after a kill cannot tell a copy of an earlier layout from the record an install had just written:
///   both are records, and both were left aside by the killed pass.
///   Putting it back would replace the record that stands at the name by then,
///   so it is kept where it is, and one line in the worktree's own Git directory says which file it is,
///   where it lies, and how to put it back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(asideOfADeadPid('zzz.launch'), freshRecord); write('zzz.launch', anotherRecord);
/// repairToolchain(destination); assert(exists(aside)); assert(takeNotices(gitDir)[0].includes('mv --'));
/// ```
#[test]
fn a_record_a_killed_pass_left_where_its_name_is_taken_is_kept_and_named_for_the_person() {
    let (root, destination, _bin, environment) = scene("toolchain-kept-aside");
    std::fs::create_dir_all(destination.join(".git")).expect("a Git directory to leave the words in");
    lock(&destination, &["own"]);
    record(&destination, "own", destination.join(LOCK_FILE).as_path());
    let launchers: PathBuf = destination.join(LAUNCHERS);
    // What a killed pass left: the record an install had written, under the name that pass gave it.
    let dead: u32 = dead_process_number();
    let aside: PathBuf = launchers.join(format!("zzz.launch.removing-{dead}-0"));
    let own_lock: PathBuf = destination.join(LOCK_FILE);
    let kept: String = format!("{{\"toolId\":\"cargo:zzz\",\"lockFile\":\"{}\",\"real\":\"/tools/zzz\",\"scope\":\"repo\"}}\n", own_lock.display());
    std::fs::write(aside.as_path(), kept.as_bytes()).expect("the killed pass's record");
    // Another record stands at the name by the time the next pass reads it.
    record(&destination, "zzz", root.join("source/toolchain.lock.json").as_path());
    let _finding: Option<Finding> = repair_toolchain(destination.as_path(), &environment);
    assert!(aside.is_file(), "the record a killed pass left aside was discarded");
    assert_eq!(std::fs::read_to_string(aside.as_path()).expect("the kept record"), kept, "the kept record is not the one that was left");
    let told: Vec<String> = crate::package_repair_relink_notice::take_notices(destination.join(".git").as_path());
    assert_eq!(told.len(), 1, "the person is not told about the record that was kept: {told:?}");
    let words: &String = &told[0];
    assert!(words.contains(aside.to_str().expect("utf8")), "the line does not name the file that was kept: {words}");
    assert!(words.contains("mv -- "), "the line does not name the command that puts it back: {words}");
    assert!(words.contains(destination.join(LAUNCHERS).join("zzz.launch").to_str().expect("utf8")), "the line does not name where it belongs: {words}");
    remove(&root);
}
