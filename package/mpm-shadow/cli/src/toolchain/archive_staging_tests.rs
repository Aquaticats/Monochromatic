//! Unit tests of the publish step of an archive install:
//! the one exchange that replaces a copy,
//! the two renames a file system without it needs,
//! and what a kill at each of those steps leaves behind,
//! which a control shows by ending a real installer between the two renames
//! (never by timing)
//! and by planting the refusal of the exchange where the machine's file systems can all exchange.
//! Each runs against the fake runner and the fake network inside disposable homes.

/// Imports `fs` from `std`.
use std::fs;
/// Imports the trait that sets and reads a directory's permission bits.
use std::os::unix::fs::PermissionsExt;
/// Imports the trait that reads the signal a killed process ended with.
use std::os::unix::process::ExitStatusExt;
/// Imports what these tests use from `std::path`.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the types that start and wait for the installer a control kills,
/// and the program a test keeps running while it judges which installer still runs.
use std::process::{
    Child,
    Command,
    ExitStatus,
    Stdio,
};

/// Imports the names directly inside a directory.
use super::archive_install_tests::names_of;
/// Imports what the publish step answers with.
use super::archive_staging::{
    AnOlderVersion,
    COMPLETE_MARKER,
    CopyRecord,
    FinishedCopy,
    Owner,
    Plant,
    RECOVERED_PURPOSE,
    REPLACED_PURPOSE,
    Recovered,
    RenameRefused,
    STAGING_PURPOSE,
    clear_the_plant,
    has_complete_marker,
    hidden_sibling,
    hidden_siblings,
    moment_in,
    plant_these_steps,
    publish_staged,
    recover_leftovers,
    staging_dir_of,
    start_time_of_a_process,
    write_complete_marker,
};
/// Imports the reader facts a control forces,
/// and the question whose answer it checks.
use super::archive_staging::{
    ForcedStart,
    Liveness,
    ReaderView,
    read_as,
};
/// Imports the rig and the writer of an executable stand-in.
use super::fakes::{
    Rig,
    write_executable,
};
/// Imports the tool directory of a tool id.
use super::layout::tool_directory;
/// Imports the question that tells a finished install from an unfinished one.
use super::published_archive::is_finished;
/// Imports what these tests share with the tests of the newest-only layout.
use super::published_archive_tests::{
    A_TEST_SHA256,
    BIN,
    ID,
    context_of,
    finish_archive,
    lock_at,
    release_digest,
    serve_releases,
    unmarked_archive,
};
/// Imports the install itself.
use super::sync::sync_tools;

/// The control of a publish a second install races between its two renames,
/// and of the copy an install's own failed publish keeps.
#[path = "archive_staging_tests/the_publishes_that_fail_or_race_tests.rs"]
mod the_publishes_that_fail_or_race_tests;

/// The controls of the copies a recovery may not publish:
/// an older version beside a newer one in either toolchain root,
/// and a marker that matches the lock's row on some fields but not all.
#[path = "archive_staging_tests/the_copies_a_recovery_may_not_publish_tests.rs"]
mod the_copies_a_recovery_may_not_publish_tests;

/// The variable the installer this control kills reads the tool directory of its fixture from.
const TOOL_DIRECTORY_VARIABLE: &str = "MPM_SHADOW_TEST_FIXTURE_TOOL_DIRECTORY";

/// The variable that same installer reads the version it publishes from.
const VERSION_VARIABLE: &str = "MPM_SHADOW_TEST_FIXTURE_VERSION";

/// The variable the fixture whose exchange a policy refuses reads that refusal's operating system number from.
const REFUSAL_VARIABLE: &str = "MPM_SHADOW_TEST_FIXTURE_EXCHANGE_REFUSAL";

/// The test a control starts to publish a copy whose exchange the file system refuses,
/// named the way the test binary's own filter names it.
const INSTALLER_WHOSE_EXCHANGE_IS_REFUSED: &str =
    "toolchain::archive_staging_tests::the_installer_whose_exchange_a_policy_refuses";

/// What:
/// The test a control starts as the installer it kills,
/// named the way the test binary's own filter names it.
/// Why:
/// The installer has to be a process of its own,
/// and the test binary is the only program that carries the fake network and the fake runner an archive install needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHILD_TEST = 'toolchain::archive_staging_tests::the_installer_a_control_kills_between_the_two_renames';
/// ```
const INSTALLER_A_CONTROL_KILLS: &str =
    "toolchain::archive_staging_tests::the_installer_a_control_kills_between_the_two_renames";

/// The number `signal(7)` gives `SIGKILL`,
/// which no handler catches and after which no cleanup runs.
const SIGKILL_NUMBER: i32 = 9;

/// What:
/// Starts this test binary again to run the installer a control kills,
/// and answers how that installer ended.
/// Why:
/// A kill inside this process would end the whole test binary,
/// so the installer is a process of its own,
/// and the test binary is the only program that can install an archive without a network.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheInstallerAControlKills(toolDirectory: string, version: string): ExitStatus
/// ```
fn run_the_installer_a_control_kills(version_dir: &Path) -> ExitStatus {
    let tool_directory: &Path = version_dir.parent().expect("a tool directory");
    let output = Command::new(std::env::current_exe().expect("the test binary's own path"))
        .arg("--exact")
        .arg(INSTALLER_A_CONTROL_KILLS)
        .arg("--ignored")
        .arg("--test-threads=1")
        .env(TOOL_DIRECTORY_VARIABLE, tool_directory)
        .env(VERSION_VARIABLE, super::util::file_name_text(version_dir))
        .output()
        .expect("start the installer this control kills");
    return output.status;
}

#[test]
fn a_kill_between_the_two_renames_leaves_the_version_missing_and_a_later_run_finishes_the_publish()
{
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // What a republish starts from: an earlier release unpacked 0.2.1 in place, without the completion marker.
    unmarked_archive(&root, "0.2.1");

    let status: ExitStatus = run_the_installer_a_control_kills(&version_dir);
    assert_eq!(
        status.signal(),
        Some(SIGKILL_NUMBER),
        "the installer has to end at the instant between the two renames, not after it: {status:?}"
    );

    // The window, listed: the version's name holds nothing, and two hidden directories hold the copies.
    let after_the_kill: Vec<String> = names_of(&tool_dir);
    assert!(
        !version_dir.exists(),
        "the version's name is absent after a kill between the two renames: {after_the_kill:?}"
    );
    assert_eq!(after_the_kill.len(), 2, "{after_the_kill:?}");
    let finished: Vec<&String> = after_the_kill
        .iter()
        .filter(|name| return has_complete_marker(&tool_dir.join(name)))
        .collect();
    assert_eq!(
        finished.len(),
        1,
        "one of the two copies is the finished install: {after_the_kill:?}"
    );

    // A later run, with nothing to download: the finished copy becomes the version's name again.
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(
        rig.http.requests().is_empty(),
        "the later run finishes the publish from what the killed one left ({after_the_kill:?}), so it downloads nothing: {:?}",
        rig.http.requests()
    );
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(
        is_finished(&version_dir, BIN),
        "the version is installed again: {report:?}"
    );
    // The version starts: the program the version holds runs, which is what a start through `current` does.
    let start = Command::new(version_dir.join("release-folder").join(BIN))
        .output()
        .expect("start the program the recovered version holds");
    assert!(
        start.status.success(),
        "the version a kill left and a later run published starts: {start:?}"
    );
    assert_eq!(
        report.recovered.len(),
        1,
        "the report says the publish was finished: {report:?}"
    );
    assert_eq!(
        names_of(&tool_dir),
        vec!["0.2.1".to_string(), "current".to_string()],
        "nothing of the killed installer stays behind"
    );

    // A run after that finds the installed version and changes nothing.
    let second = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert_eq!(second.unchanged, vec![ID.to_string()], "{second:?}");
    assert!(second.recovered.is_empty(), "{second:?}");
}

/// What:
/// The installer a control kills:
/// it unpacks nothing,
/// writes the finished copy an install would publish,
/// and publishes it,
/// which ends this process at the instant between the two renames.
/// Why:
/// The whole publish runs for real,
/// so what the kill leaves is what a killed install leaves,
/// and this test refuses rather than passes when the process was not ended at that instant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the installer a control kills', () => { const staging = stagingDirOf(finalDir); writeFinishedCopy(staging); plant({ refuseTheExchange: true, killBetweenTheTwoRenames: true }); publishStaged(staging, finalDir, BIN); fail(); });
/// ```
#[test]
#[ignore = "a fixture a control starts and kills: it ends this process with SIGKILL between the two renames"]
fn the_installer_a_control_kills_between_the_two_renames() {
    let tool_directory: String = std::env::var(TOOL_DIRECTORY_VARIABLE)
        .expect("a control names the fixture's tool directory");
    let version: String =
        std::env::var(VERSION_VARIABLE).expect("a control names the version to publish");
    let version_dir: PathBuf = PathBuf::from(tool_directory).join(&version);
    let staging: PathBuf = staging_dir_of(&version_dir);
    // The copy an install publishes: the program inside the folder a release archive unpacks into, and the marker last.
    fs::create_dir_all(&staging).expect("make the staging directory");
    write_executable(&staging.join("release-folder").join(BIN));
    write_complete_marker(&staging, ID, &version, &release_digest(&version))
        .expect("write the completion marker");
    plant_these_steps(Plant {
        refuse_the_exchange: true,
        kill_between_the_two_renames: true,
        ..Plant::default()
    });
    publish_staged(&staging, &version_dir, BIN)
        .expect("the publish of a file system that cannot exchange the two names ends in the kill");
    clear_the_plant();
    panic!(
        "the publish returned, so no kill between the two renames happened: the process was not ended at that instant"
    );
}

/// What:
/// A program a test keeps running while it judges which installer still runs,
/// ended and reaped when the test's body leaves.
/// Why:
/// A stand-in child that outlives its control is left behind by a failure,
/// and one left behind holds the process count of the machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RunningProgram { constructor() { this.child = spawn('sleep', ['120']); } owner(): Owner; [Symbol.dispose]() { this.child.kill(); } }
/// ```
struct RunningProgram {
    /// The program itself.
    child: Child,
}

/// The methods of `RunningProgram`.
impl RunningProgram {
    /// Starts a program that stays until the guard drops.
    fn start() -> RunningProgram {
        let child: Child = Command::new("sleep")
            .arg("120")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start a program that stays running");
        return RunningProgram {
            child,
        };
    }

    /// The owner of that program,
    /// as a hidden sibling's name records it,
    /// in this reader's boot and namespace.
    fn owner(&self) -> Owner {
        let here: Owner = Owner::here();
        let pid: u32 = self.child.id();
        return Owner {
            pid,
            started: start_time_of_a_process(pid).expect("the system reports the program's start"),
            boot: here.boot,
            namespace: here.namespace,
        };
    }
}

/// What:
/// The owner of one process this reader can see,
/// carrying this reader's boot and namespace.
/// Why:
/// A control that names a live process needs a name this build can judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerOf(pid: number): Owner;
/// ```
fn owner_of(pid: u32) -> Owner {
    let here: Owner = Owner::here();
    return Owner {
        pid,
        started: start_time_of_a_process(pid).expect("the system reports the process's start"),
        boot: here.boot,
        namespace: here.namespace,
    };
}

/// What:
/// What a lock's row for one copy records.
/// Why:
/// A control that publishes a hidden copy must give the repair the same facts the marker records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aCopyRecord(tool: string, version: string, archiveSha256: string): CopyRecord;
/// ```
fn a_copy_record(
    tool: &str,
    version: &str,
    archive_sha256: &str,
) -> CopyRecord {
    return CopyRecord {
        tool: tool.to_string(),
        version: version.to_string(),
        archive_sha256: archive_sha256.to_string(),
    };
}

/// What:
/// The record the lock of a control's rig holds for one version of the tool under test,
/// whose archive digest its marker must carry for a recovery to publish the copy.
/// Why:
/// Every control that publishes a hidden copy goes through the vouching check,
/// and the lock's row is the only record that can satisfy it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theLocksRecord(version: string): CopyRecord;
/// ```
fn the_locks_record(version: &str) -> CopyRecord {
    return a_copy_record(ID, version, &release_digest(version));
}

/// Ends and reaps the program when the test's body leaves,
/// however it left.
impl Drop for RunningProgram {
    fn drop(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }
}

/// What:
/// An installer's pair for a process the system no longer runs.
/// Why:
/// A control needs a hidden sibling that a later install may remove,
/// and the pair is dead when the start time belongs to no process with that number:
/// the first process is as long-lived as the system,
/// and no process ever carries the moment after its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anInstallerThatNoLongerRuns(): Owner { return { pid: 1, started: startTimeOf(1)! + 1 }; }
/// ```
fn an_installer_that_no_longer_runs() -> Owner {
    let here: Owner = Owner::here();
    let started: u64 =
        start_time_of_a_process(1).expect("the system reports the first process's start");
    return Owner {
        pid: 1,
        started: started + 1,
        boot: here.boot,
        namespace: here.namespace,
    };
}

#[test]
fn a_copy_that_is_there_is_replaced_in_one_exchange_that_no_kill_can_interrupt() {
    let rig = Rig::new();
    let tool_dir: PathBuf = tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The copy an earlier release left: the program, and no completion marker.
    write_executable(&version_dir.join("release-folder").join(BIN));
    let staging: PathBuf = staging_dir_of(&version_dir);
    fs::create_dir_all(staging.join("release-folder")).expect("make the staging directory");
    write_executable(&staging.join("release-folder").join(BIN));
    write_complete_marker(&staging, ID, "0.2.1", A_TEST_SHA256)
        .expect("write the completion marker");

    // The instant the two-rename path has is armed, and the exchange never reaches it: this call returns.
    plant_these_steps(Plant {
        refuse_the_exchange: false,
        kill_between_the_two_renames: true,
        ..Plant::default()
    });
    let published = publish_staged(&staging, &version_dir, BIN);
    clear_the_plant();
    assert!(
        published.is_ok(),
        "the exchange replaced the copy, so no instant between two renames existed: {published:?}"
    );
    assert!(is_finished(&version_dir, BIN));
    assert_eq!(
        names_of(&tool_dir),
        vec!["0.2.1".to_string()],
        "the copy that was there is gone and nothing is left under a hidden name"
    );
}

#[test]
fn where_the_file_system_cannot_exchange_the_two_names_the_copy_is_still_replaced_in_full() {
    let rig = Rig::new();
    let tool_dir: PathBuf = tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    write_executable(&version_dir.join("release-folder").join(BIN));
    let staging: PathBuf = staging_dir_of(&version_dir);
    fs::create_dir_all(staging.join("release-folder")).expect("make the staging directory");
    write_executable(&staging.join("release-folder").join(BIN));
    write_complete_marker(&staging, ID, "0.2.1", A_TEST_SHA256)
        .expect("write the completion marker");

    plant_these_steps(Plant {
        refuse_the_exchange: true,
        kill_between_the_two_renames: false,
        ..Plant::default()
    });
    let published = publish_staged(&staging, &version_dir, BIN);
    clear_the_plant();
    assert!(published.is_ok(), "{published:?}");
    assert!(is_finished(&version_dir, BIN));
    assert_eq!(
        names_of(&tool_dir),
        vec!["0.2.1".to_string()],
        "the copy that was moved aside is removed once the finished one is in place"
    );
}

#[test]
fn a_finished_copy_another_run_installed_is_kept_and_the_staged_one_is_discarded() {
    let rig = Rig::new();
    let tool_dir: PathBuf = tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    write_executable(&version_dir.join("release-folder").join(BIN));
    write_complete_marker(&version_dir, ID, "0.2.1", A_TEST_SHA256)
        .expect("write the completion marker");
    let staging: PathBuf = staging_dir_of(&version_dir);
    fs::create_dir_all(&staging).expect("make the staging directory");
    fs::write(staging.join("the other run's copy"), "x").expect("write");

    publish_staged(&staging, &version_dir, BIN).expect("the finished copy is kept");
    assert!(
        !version_dir.join("the other run's copy").exists(),
        "the copy that was there is the one that stays"
    );
    assert!(!staging.exists(), "the staged copy is discarded");
    assert_eq!(names_of(&tool_dir), vec!["0.2.1".to_string()]);
}

#[test]
fn a_staged_copy_that_is_gone_leaves_the_copy_that_was_there_untouched() {
    let rig = Rig::new();
    let tool_dir: PathBuf = tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    write_executable(&version_dir.join("release-folder").join(BIN));
    let missing: PathBuf = staging_dir_of(&version_dir);

    let published = publish_staged(&missing, &version_dir, BIN).expect_err("nothing to publish");
    assert!(
        published
            .message
            .contains("cannot put the finished install in place of the copy at"),
        "the failure names the copy the install could not replace: {published}"
    );
    assert!(
        version_dir.join("release-folder").join(BIN).exists(),
        "the copy that was there is untouched"
    );
    assert_eq!(names_of(&tool_dir), vec!["0.2.1".to_string()]);
}

#[test]
fn a_hidden_directory_of_an_installer_that_still_runs_is_left_where_it_is() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let running = RunningProgram::start();
    let live: PathBuf = hidden_sibling(&version_dir, STAGING_PURPOSE, running.owner());
    fs::create_dir_all(&live).expect("make the running install's staging directory");
    fs::write(live.join("what it is unpacking"), "x").expect("write");

    serve_releases(&rig, &["0.2.1"]);
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(
        live.is_dir(),
        "the staging directory of an installer that still runs is untouched"
    );
    assert!(is_finished(&version_dir, BIN), "{report:?}");
    assert!(
        !live.join(COMPLETE_MARKER).exists(),
        "nothing was published from it"
    );
}

#[test]
fn a_number_another_program_now_holds_is_never_read_as_the_installer() {
    let rig = Rig::new();
    let tool_dir: PathBuf = tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The finished version, so the recovery of it has nothing to publish.
    finish_archive(&rig.scope.toolchain_root, "0.2.1");

    let live: PathBuf = hidden_sibling(&version_dir, STAGING_PURPOSE, owner_of(1));
    let gone: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    // A name an earlier release wrote, which carries no start time and so cannot be judged.
    let older: PathBuf = tool_dir.join(format!(".0.2.1.{STAGING_PURPOSE}.42"));
    // A kill in another version's install, which no later install of that version may come to clear.
    let another_version: PathBuf = hidden_sibling(
        &tool_dir.join("0.2.0"),
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    for path in [&live, &gone, &older, &another_version] {
        fs::create_dir_all(path).expect("make a hidden directory");
    }

    // A finished copy of another version, which may be the only copy of it and so is no leftover to remove.
    let finished_elsewhere: PathBuf = hidden_sibling(
        &tool_dir.join("0.2.2"),
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(finished_elsewhere.join("release-folder"))
        .expect("make the hidden directory");
    write_executable(&finished_elsewhere.join("release-folder").join(BIN));
    write_complete_marker(&finished_elsewhere, ID, "0.2.2", A_TEST_SHA256)
        .expect("write the completion marker");

    let recovered: Recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&a_copy_record(ID, "0.2.1", A_TEST_SHA256)),
    )
    .expect("recovered");
    assert!(
        live.is_dir(),
        "an installer that still runs keeps its directory"
    );
    assert!(
        older.exists(),
        "a name written without a start time cannot be judged, so it stays as it is: {:?}",
        recovered.removed
    );
    let removed: Vec<&PathBuf> = recovered.removed.iter().collect();
    assert!(
        removed.contains(&&gone),
        "the number of a process that ended is read as gone: {removed:?}"
    );
    assert!(
        removed.contains(&&another_version),
        "so is the remains of a kill in another version's install: {removed:?}"
    );
    assert!(
        finished_elsewhere.is_dir(),
        "a finished copy of another version is left for that version's own install: {removed:?}"
    );
}

#[test]
fn the_copy_a_kill_moved_aside_comes_back_when_the_finished_one_is_gone() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // What a kill in the window leaves, without the finished copy, which a person deleted.
    unmarked_archive(&root, "0.2.1");
    let aside: PathBuf = hidden_sibling(
        &version_dir,
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::rename(&version_dir, &aside).expect("move the copy aside the way a killed install did");

    // The release is not served, so the install that follows the recovery cannot download anything: the copy comes
    // back before it, which is what this control shows.
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(
        version_dir.is_dir(),
        "the copy that was there is back at the version's name: {report:?}"
    );
    assert!(!aside.exists(), "and the aside name is gone");
    assert_eq!(
        report.recovered.len(),
        1,
        "the report says the copy was put back: {report:?}"
    );
    assert_eq!(
        report.failed.len(),
        1,
        "the archive was not served, so the install after the recovery failed: {report:?}"
    );
}

#[test]
fn a_third_party_archive_tool_also_finishes_the_publish_a_kill_left() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    // A tool of another repository: it finishes on the marker alone, and no program of it is looked for.
    let tool_dir: PathBuf = tool_directory(&root, "node").expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(rig.tool("node").version);
    let dead: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(&dead).expect("make the finished copy a killed install left");
    write_complete_marker(&dead, "node", &rig.tool("node").version, A_TEST_SHA256)
        .expect("write the marker");

    let version: String = rig.tool("node").version.clone();
    let recovered: Recovered = recover_leftovers(
        &version_dir,
        "",
        AnOlderVersion::MayStay,
        Some(&a_copy_record("node", &version, A_TEST_SHA256)),
    )
    .expect("recovered");
    assert!(
        matches!(recovered.finished_copy, FinishedCopy::PublishedHere),
        "the finished copy is published: {recovered:?}"
    );
    assert!(
        has_complete_marker(&version_dir),
        "the version's name holds the finished copy"
    );
    assert_eq!(
        names_of(&tool_dir),
        vec![rig.tool("node").version],
        "and nothing else"
    );
}

#[test]
fn a_record_the_kernel_did_not_finish_is_never_read_as_a_dead_installer() {
    // A real record: this process's own, as the kernel writes it (its program name may hold anything).
    let record: Vec<u8> = fs::read("/proc/self/stat").expect("read this process's own record");
    let printed: String = String::from_utf8_lossy(&record).to_string();
    assert_eq!(
        moment_in(&record),
        Some(start_time_of_a_process(std::process::id()).expect("this process's own start")),
        "the moment is the one the record names: {printed}"
    );

    // A program name holding a bracket and a space: the fields are counted from the last closing bracket.
    let mut named: Vec<u8> = record.clone();
    let closing = named
        .iter()
        .rposition(|byte| return *byte == b')')
        .expect("a name in brackets");
    named.splice(2..closing, b"a ) b".iter().copied());
    assert_eq!(
        moment_in(&named),
        moment_in(&record),
        "{}",
        String::from_utf8_lossy(&named)
    );

    // A record that ends before its twenty-second field says nothing, and nothing may be concluded from it.
    let field = record[closing + 1..]
        .split(|byte| return byte.is_ascii_whitespace())
        .filter(|part| return !part.is_empty())
        .nth(19)
        .expect("a start field");
    let offset: usize = field.as_ptr() as usize - record.as_ptr() as usize;
    for cut in [0usize, 2, offset] {
        assert_eq!(
            moment_in(&record[..cut]),
            None,
            "a record cut at {cut} bytes of {} says nothing",
            record.len()
        );
    }

    // A start field that is not a number is no record either.
    let mut corrupt: Vec<u8> = record.clone();
    corrupt.splice(offset..offset + 1, b"x".iter().copied());
    assert_eq!(
        moment_in(&corrupt),
        None,
        "a start field that is not a number says nothing"
    );
}

#[test]
fn a_copy_a_killed_recovery_had_claimed_is_published_by_the_next_install() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // A kill during a recovery, after the finished copy was claimed and before it was published.
    let claimed: PathBuf = hidden_sibling(
        &version_dir,
        RECOVERED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(claimed.join("release-folder")).expect("make the claimed copy");
    write_executable(&claimed.join("release-folder").join(BIN));
    write_complete_marker(&claimed, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(
        rig.http.requests().is_empty(),
        "nothing was downloaded: {:?}",
        rig.http.requests()
    );
    assert!(
        is_finished(&version_dir, BIN),
        "the claimed copy became the version: {report:?}"
    );
    assert!(!claimed.exists(), "and the claimed name is gone");
}

//region The controls for the orders that must never cost a finished copy

/// The name of the file that marks the copy a version's name held before a killed install moved it aside.
const WHAT_THE_VERSION_HELD: &str = "what-the-version-held";

/// The name of the file that marks a copy an installer was unpacking when it was killed.
const WHAT_WAS_BEING_UNPACKED: &str = "what-was-being-unpacked";

/// The operating system's number for an input/output error:
/// an answer no file system gives for a flag it does not support,
/// so the list of answers that mean "this file system cannot exchange two names" does not take it.
const AN_INPUT_OUTPUT_ERROR: i32 = rustix::io::Errno::IO.raw_os_error();

/// What:
/// The state a kill between the two renames leaves:
/// the version's name holding nothing,
/// the finished copy under a dead installer's staging name,
/// and the copy that was there under that installer's aside name.
/// Why:
/// Every order of steps from the claim to the publish starts here,
/// and a control builds the state by hand so nothing it shows depends on timing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatAKillBetweenTheTwoRenamesLeaves(versionDir: string): [string, string] {
///   const dead = anInstallerThatNoLongerRuns();
///   const finished = hiddenSibling(versionDir, 'staging', dead);
///   writeFinishedCopy(finished);
///   const aside = hiddenSibling(versionDir, 'replaced', dead);
///   writeUnfinishedCopy(aside, WHAT_THE_VERSION_HELD);
///   return [finished, aside];
/// }
/// ```
fn what_a_kill_between_the_two_renames_leaves(version_dir: &Path) -> (PathBuf, PathBuf) {
    let dead: Owner = an_installer_that_no_longer_runs();
    let finished: PathBuf = hidden_sibling(version_dir, STAGING_PURPOSE, dead);
    fs::create_dir_all(finished.join("release-folder")).expect("make the finished copy");
    write_executable(&finished.join("release-folder").join(BIN));
    write_complete_marker(
        &finished,
        ID,
        &super::util::file_name_text(version_dir),
        &release_digest(&super::util::file_name_text(version_dir)),
    )
    .expect("write the completion marker");
    let aside: PathBuf = the_copy_a_kill_moved_aside(version_dir);
    return (finished, aside);
}

/// What:
/// The copy that was there,
/// under a dead installer's aside name:
/// what a kill leaves when the finished copy is gone or when the copy was moved aside and nothing replaced it.
/// Why:
/// Several orders start from that one copy,
/// and a control needs it without a finished copy beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCopyAKillMovedAside(versionDir: string): string {
///   const aside = hiddenSibling(versionDir, 'replaced', anInstallerThatNoLongerRuns());
///   writeExecutable(join(aside, 'release-folder', BIN));
///   writeFileSync(join(aside, WHAT_THE_VERSION_HELD), 'the copy that was there');
///   return aside;
/// }
/// ```
fn the_copy_a_kill_moved_aside(version_dir: &Path) -> PathBuf {
    let aside: PathBuf = hidden_sibling(
        version_dir,
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(aside.join("release-folder")).expect("make the copy that was there");
    write_executable(&aside.join("release-folder").join(BIN));
    fs::write(aside.join(WHAT_THE_VERSION_HELD), "the copy that was there").expect("write");
    return aside;
}

/// What:
/// The finished copy of `version_dir` a later install could publish,
/// when there is one:
/// at the version's name,
/// or under a hidden name of that version.
/// Why:
/// The one thing no order of steps may cost is the last finished copy,
/// because offline it is the only way the version comes back;
/// every control asserts this and names what it found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aFinishedCopyIsOnDisk(versionDir: string, firstBin: string): string | undefined {
///   if (isFinished(versionDir, firstBin)) return versionDir;
///   for (const name of namesOf(dirname(versionDir))) {
///     if (!name.startsWith(`.${basename(versionDir)}.`)) continue;
///     const candidate = join(dirname(versionDir), name);
///     if (isFinished(candidate, firstBin)) return candidate;
///   }
///   return undefined;
/// }
/// ```
fn a_finished_copy_is_on_disk(
    version_dir: &Path,
    first_bin: &str,
) -> Option<PathBuf> {
    if is_finished(version_dir, first_bin) {
        return Some(version_dir.to_path_buf());
    }
    let directory: &Path = version_dir.parent()?;
    let prefix: String = format!(".{}.", super::util::file_name_text(version_dir));
    for name in names_of(directory) {
        if !name.starts_with(&prefix) {
            continue;
        }
        let candidate: PathBuf = directory.join(&name);
        if is_finished(&candidate, first_bin) {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
/// The copy that was there,
/// put back at the version's name:
/// what a second install does when it may claim no finished copy and finds the version's name absent.
/// Why:
/// The order that lost the finished copy (the review's M2) needs that one rename to happen in the instant between this
/// publish's check and its rename,
/// and a control runs it exactly there instead of relying on timing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putTheCopyThatWasThereBack(versionDir: string): void {
///   for (const name of namesOf(dirname(versionDir))) {
///     if (!name.startsWith(`.${basename(versionDir)}.replaced.`)) continue;
///     if (rename(join(dirname(versionDir), name), versionDir) === undefined) return;
///   }
/// }
/// ```
fn put_the_copy_that_was_there_back(version_dir: &Path) {
    let Some(directory) = version_dir.parent() else {
        return;
    };
    let prefix: String = format!(
        ".{}.{REPLACED_PURPOSE}.",
        super::util::file_name_text(version_dir)
    );
    for name in names_of(directory) {
        if !name.starts_with(&prefix) {
            continue;
        }
        let aside: PathBuf = directory.join(&name);
        if fs::rename(&aside, version_dir).is_ok() {
            return;
        }
    }
}

/// The action a control plants as "another install put the copy that was there back":
/// the one rename a second recovery makes at that step.
fn put_the_copy_that_was_there_back_as_another_install(
    _staging: &Path,
    final_dir: &Path,
) {
    put_the_copy_that_was_there_back(final_dir);
}

/// What:
/// A finished copy written at the version's name,
/// which is what another install's publish leaves there.
/// Why:
/// The instant a control stops this publish at is also the instant another install's publish can finish,
/// and this is what the report then has to say about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aFinishedCopyAtTheVersionAsAnotherInstall(_staging: string, finalDir: string): void { writeFinishedCopy(finalDir); }
/// ```
fn a_finished_copy_at_the_version_as_another_install(
    _staging: &Path,
    final_dir: &Path,
) {
    fs::create_dir_all(final_dir.join("release-folder")).expect("make the version");
    write_executable(&final_dir.join("release-folder").join(BIN));
    write_complete_marker(
        final_dir,
        ID,
        &super::util::file_name_text(final_dir),
        A_TEST_SHA256,
    )
    .expect("write the completion marker");
}

/// What:
/// This process's hidden sibling under the name it would carry once this process ended:
/// the purpose it was made for,
/// with the four facts of an installer that no longer runs.
/// Why:
/// A later install claims only a name whose installer the system reports as ended,
/// and a control cannot end its own test process,
/// so it renames the leftover to the name a run after this process's end reads.
/// The purpose is named by the caller:
/// a hidden name carries the boot and the namespace after the process id and its start time,
/// so the purpose is not a fixed distance from the end of the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function asAfterThisProcessEnded(path: string, versionDir: string, purpose: string): string {
///   const ended = hiddenSibling(versionDir, purpose, anInstallerThatNoLongerRuns());
///   renameSync(path, ended);
///   return ended;
/// }
/// ```
fn as_after_this_process_ended(
    path: &Path,
    version_dir: &Path,
    purpose: &str,
) -> PathBuf {
    let ended: PathBuf = hidden_sibling(version_dir, purpose, an_installer_that_no_longer_runs());
    fs::rename(path, &ended)
        .expect("rename the leftover to the name a later run reads as a dead installer's");
    return ended;
}

/// Sets a directory's permission bits.
fn set_mode(
    path: &Path,
    mode: u32,
) {
    let mut permissions = fs::metadata(path).expect("the directory").permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions).expect("set the mode");
}

/// What:
/// A directory made read-only for the length of a control,
/// and writable again when the control leaves.
/// Why:
/// A rename inside a directory that refuses writes fails with "permission denied",
/// which is the one claim failure a disposable fixture can make on its own,
/// and a directory left read-only would keep its scratch folder from being removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReadOnly {
///   constructor(path: string) { this.mode = statSync(path).mode; chmodSync(path, this.mode & ~0o222); }
///   [Symbol.dispose]() { chmodSync(this.path, this.mode); }
/// }
/// ```
struct ReadOnly {
    /// The directory itself.
    path: PathBuf,
    /// The mode it had before,
    /// which is what it gets back when the guard drops.
    mode: u32,
}

/// The methods of `ReadOnly`.
impl ReadOnly {
    /// Makes a directory refuse writes to what it holds.
    fn new(path: &Path) -> ReadOnly {
        let mode: u32 = fs::metadata(path)
            .expect("the directory")
            .permissions()
            .mode();
        set_mode(path, mode & !0o222);
        return ReadOnly {
            path: path.to_path_buf(),
            mode,
        };
    }
}

/// Gives the directory its own mode back when the control leaves,
/// however it left.
impl Drop for ReadOnly {
    fn drop(&mut self) {
        set_mode(&self.path, self.mode);
    }
}

#[test]
fn a_publish_that_fails_with_the_versions_name_absent_keeps_the_finished_copy_for_the_next_install()
{
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);
    let claim: PathBuf = hidden_sibling(&version_dir, RECOVERED_PURPOSE, Owner::here());

    plant_these_steps(Plant {
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    );
    clear_the_plant();
    let error =
        recovered.expect_err("the rename that gives an absent name the finished copy failed");
    assert!(
        error
            .message
            .contains("cannot move the finished install to"),
        "the failure names the rename that failed: {error}"
    );
    assert!(
        error
            .message
            .contains(&format!("still at {}", claim.display())),
        "the failure names the copy that is on disk: {error}"
    );
    let notice: String = format!(
        "no order costs the finished copy: {:?}",
        names_of(&tool_dir)
    );
    let Some(kept) = a_finished_copy_is_on_disk(&version_dir, BIN) else {
        panic!("{notice}");
    };

    // The run that failed ends, so its claim carries a dead installer's pair, which is what the next install reads.
    as_after_this_process_ended(&kept, &version_dir, RECOVERED_PURPOSE);
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(
        rig.http.requests().is_empty(),
        "the next install publishes the copy the failed one left, so it downloads nothing: {:?}",
        rig.http.requests()
    );
    assert!(
        is_finished(&version_dir, BIN),
        "the version is installed again: {report:?}"
    );
    assert_eq!(
        names_of(&tool_dir),
        vec!["0.2.1".to_string(), "current".to_string()],
        "nothing of the kill or of the failed publish stays"
    );
}

#[test]
fn a_publish_whose_exchange_fails_for_another_reason_keeps_the_finished_copy_and_names_both_paths()
{
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The copy that was there, unfinished, at the version's name, and a dead install's finished copy beside it.
    unmarked_archive(&root, "0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);
    let claim: PathBuf = hidden_sibling(&version_dir, RECOVERED_PURPOSE, Owner::here());

    plant_these_steps(Plant {
        answer_the_exchange_with: Some(AN_INPUT_OUTPUT_ERROR),
        ..Plant::default()
    });
    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    );
    clear_the_plant();
    let error =
        recovered.expect_err("the exchange failed with an answer that is not \"cannot exchange\"");
    assert!(
        error.message.contains("that copy is as it was"),
        "the failure says the copy at the version's name is as it was: {error}"
    );
    assert!(
        error
            .message
            .contains(&format!("still at {}", claim.display())),
        "the failure says where the finished copy is: {error}"
    );
    assert!(
        a_finished_copy_is_on_disk(&version_dir, BIN).is_some(),
        "no order costs the finished copy: {:?}",
        names_of(&tool_dir)
    );
    assert!(
        version_dir.join("release-folder").join(BIN).is_file()
            && !has_complete_marker(&version_dir),
        "the copy at the version's name is untouched"
    );
}

#[test]
fn a_publish_whose_first_aside_rename_fails_keeps_the_finished_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    unmarked_archive(&root, "0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);
    let claim: PathBuf = hidden_sibling(&version_dir, RECOVERED_PURPOSE, Owner::here());

    plant_these_steps(Plant {
        refuse_the_exchange: true,
        refuse_the_rename: Some((RenameRefused::MovesTheCopyAside, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    );
    clear_the_plant();
    let error = recovered.expect_err("the rename that moves the copy there aside failed");
    assert!(
        error.message.contains("cannot move the copy at"),
        "the failure names the rename that failed: {error}"
    );
    assert!(
        error
            .message
            .contains(&format!("still at {}", claim.display())),
        "the failure says where the finished copy is: {error}"
    );
    assert!(
        a_finished_copy_is_on_disk(&version_dir, BIN).is_some(),
        "no order costs the finished copy: {:?}",
        names_of(&tool_dir)
    );
    assert!(
        version_dir.join("release-folder").join(BIN).is_file()
            && !has_complete_marker(&version_dir),
        "the copy at the version's name never moved"
    );
}

#[test]
fn a_publish_whose_second_rename_fails_puts_the_copy_back_and_keeps_the_finished_one() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    unmarked_archive(&root, "0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);
    let claim: PathBuf = hidden_sibling(&version_dir, RECOVERED_PURPOSE, Owner::here());

    plant_these_steps(Plant {
        refuse_the_exchange: true,
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    );
    clear_the_plant();
    let error =
        recovered.expect_err("the rename that gives the version's name the finished copy failed");
    assert!(
        error.message.contains("the copy that was there is back at"),
        "the failure says the copy that was there came back: {error}"
    );
    assert!(
        error
            .message
            .contains(&format!("still at {}", claim.display())),
        "the failure says where the finished copy is: {error}"
    );
    assert!(
        version_dir.join("release-folder").join(BIN).is_file()
            && !has_complete_marker(&version_dir),
        "the copy that was there is back at the version's name"
    );
    assert!(
        a_finished_copy_is_on_disk(&version_dir, BIN).is_some(),
        "no order costs the finished copy: {:?}",
        names_of(&tool_dir)
    );
}

#[test]
fn a_publish_whose_put_back_fails_keeps_the_finished_copy_and_names_the_aside_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    unmarked_archive(&root, "0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);
    let claim: PathBuf = hidden_sibling(&version_dir, RECOVERED_PURPOSE, Owner::here());

    // A second install puts the copy that was there back in the instant between the two renames, so both the rename
    // into the version's name and the rename that would put that copy back again fail.
    plant_these_steps(Plant {
        refuse_the_exchange: true,
        another_install_acts: Some(put_the_copy_that_was_there_back_as_another_install),
        ..Plant::default()
    });
    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    );
    clear_the_plant();
    let error = recovered.expect_err("the rename into the version's name failed");
    assert!(
        error.message.contains("could not be put back from"),
        "the failure names the put-back that failed too: {error}"
    );
    assert!(
        error
            .message
            .contains(&format!("still at {}", claim.display())),
        "the failure says where the finished copy is: {error}"
    );
    assert!(
        a_finished_copy_is_on_disk(&version_dir, BIN).is_some(),
        "no order costs the finished copy: {:?}",
        names_of(&tool_dir)
    );
    assert!(
        version_dir.join(WHAT_THE_VERSION_HELD).is_file(),
        "the copy the second install put there is at the version's name"
    );
}

#[test]
fn a_publish_that_finds_a_copy_put_back_meanwhile_replaces_it_and_keeps_the_finished_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The state a kill between the two renames leaves, and the file system can exchange, so one rename is enough.
    what_a_kill_between_the_two_renames_leaves(&version_dir);

    // The order that lost the finished copy: a second install may claim nothing, sees the version's name absent, and
    // puts the copy that was there back, all in the instant before this publish's one rename.
    plant_these_steps(Plant {
        another_install_acts: Some(put_the_copy_that_was_there_back_as_another_install),
        ..Plant::default()
    });
    let attempt = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    );
    clear_the_plant();
    let outcome = attempt.expect("the publish replaces the copy that appeared instead of failing");
    assert_eq!(
        outcome.finished_copy,
        FinishedCopy::PublishedHere,
        "{outcome:?}"
    );
    assert!(
        is_finished(&version_dir, BIN),
        "the finished copy became the version's name: {outcome:?}"
    );
    assert_eq!(
        names_of(&tool_dir),
        vec!["0.2.1".to_string()],
        "the copy the second install put back was replaced, and nothing of the kill stays"
    );
}

#[test]
fn the_copy_a_kill_moved_aside_is_not_put_back_while_a_live_installer_holds_a_claim() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let (finished, aside) = what_a_kill_between_the_two_renames_leaves(&version_dir);
    // A live install holds the finished copy: the name of a process the system reports as running.
    let running = RunningProgram::start();
    let live_claim: PathBuf = hidden_sibling(&version_dir, RECOVERED_PURPOSE, running.owner());
    fs::rename(&finished, &live_claim).expect("the live install claims the finished copy");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(
        !version_dir.exists(),
        "nothing is put at the version's name while a live install holds a claim: {recovered:?}"
    );
    assert!(live_claim.is_dir(), "the live install's claim is untouched");
    assert!(
        aside.is_dir(),
        "the copy that was there waits for the live install: {recovered:?}"
    );
    assert!(recovered.left_for_a_live_install, "{recovered:?}");
    assert!(!recovered.restored, "{recovered:?}");
}

#[test]
fn the_copy_a_kill_moved_aside_is_not_put_back_while_a_live_installer_holds_a_staging_name() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let (finished, aside) = what_a_kill_between_the_two_renames_leaves(&version_dir);
    let running = RunningProgram::start();
    let live_staging: PathBuf = hidden_sibling(&version_dir, STAGING_PURPOSE, running.owner());
    fs::rename(&finished, &live_staging).expect("give the live install its staging name");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(
        !version_dir.exists(),
        "nothing is put at the version's name while a live install holds a staging name: {recovered:?}"
    );
    assert!(
        live_staging.is_dir(),
        "the live install's staging name is untouched"
    );
    assert!(
        aside.is_dir(),
        "the copy that was there waits: {recovered:?}"
    );
    assert!(recovered.left_for_a_live_install, "{recovered:?}");
}

#[test]
fn a_live_installer_of_another_version_does_not_hold_the_copy_back() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // Only the copy that was there: this version's finished copy is gone.
    let aside: PathBuf = the_copy_a_kill_moved_aside(&version_dir);
    let running = RunningProgram::start();
    let elsewhere: PathBuf =
        hidden_sibling(&tool_dir.join("0.2.2"), STAGING_PURPOSE, running.owner());
    fs::create_dir_all(&elsewhere).expect("make another version's staging name");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(
        recovered.restored,
        "another version's live install is nothing to do with this one: {recovered:?}"
    );
    assert!(version_dir.join(WHAT_THE_VERSION_HELD).is_file());
    assert!(!aside.exists(), "the aside name is gone");
    assert!(
        elsewhere.is_dir(),
        "the other version's staging name is untouched"
    );
}

#[test]
fn the_report_counts_one_hidden_directory_in_the_singular() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let unpacking: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(&unpacking).expect("make the unfinished staging directory");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert_eq!(
        report.recovered,
        vec![format!(
            "removed 1 hidden directory that a killed install of {ID} left behind"
        )],
        "{report:?}"
    );
}

#[test]
fn the_report_counts_several_hidden_directories_in_the_plural() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let first: Owner = an_installer_that_no_longer_runs();
    // The pair beside the one the helper gives: the first process's number with a moment no process carrying it ever
    // started at, so a second dead installer's directory can be made.
    let second: Owner = Owner {
        started: first.started + 1,
        ..first.clone()
    };
    for owner in [first, second] {
        let unpacking: PathBuf = hidden_sibling(&version_dir, STAGING_PURPOSE, owner);
        fs::create_dir_all(&unpacking).expect("make an unfinished staging directory");
    }

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert_eq!(
        report.recovered,
        vec![format!(
            "removed 2 hidden directories that killed installs of {ID} left behind"
        )],
        "{report:?}"
    );
}

#[test]
fn a_publish_another_install_made_is_reported_as_theirs_and_not_as_a_publish_this_run_made() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);

    plant_these_steps(Plant {
        another_install_acts: Some(a_finished_copy_at_the_version_as_another_install),
        ..Plant::default()
    });
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    clear_the_plant();
    assert_eq!(report.recovered.len(), 1, "{report:?}");
    assert!(
        report.recovered[0].contains("another install published that version meanwhile"),
        "the report says whose publish it was: {report:?}"
    );
    assert!(
        !report.recovered[0].contains("finished the publish"),
        "no publish happened here, so none is reported: {report:?}"
    );
    assert!(is_finished(&version_dir, BIN), "{report:?}");
}

#[test]
fn a_recovery_never_publishes_an_older_version_beside_a_newer_installed_one() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // A newer finished version is installed, and a killed install left a finished copy of the older locked one.
    finish_archive(&root, "0.2.2");
    what_a_kill_between_the_two_renames_leaves(&version_dir);

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(
        !version_dir.exists(),
        "an older repository tool is never kept: {:?}",
        names_of(&tool_dir)
    );
    assert_eq!(
        super::published_archive::newest_installed_archive(&root, ID, BIN).as_deref(),
        Some("0.2.2"),
        "the newer version is the only one installed"
    );
    assert!(report.installed.is_empty(), "{report:?}");
    assert_eq!(report.unchanged, vec![ID.to_string()], "{report:?}");
    assert_eq!(report.recovered.len(), 1, "{report:?}");
    assert!(
        report.recovered[0]
            .contains("0.2.2 is installed and an older repository tool is never kept"),
        "the report states the rule that kept the copy out: {report:?}"
    );
    assert!(
        a_finished_copy_is_on_disk(&version_dir, BIN).is_some(),
        "the finished copy stays hidden, since it may be the only one once 0.2.2 is gone: {:?}",
        names_of(&tool_dir)
    );
}

#[test]
fn a_finished_copy_of_an_older_version_that_no_lock_vouches_for_is_removed_beside_a_newer_one() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    finish_archive(&root, "0.2.2");
    // A dead install whose marker an earlier build wrote: it records no archive digest, so no install can publish it,
    // whichever version is locked.
    let dead: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(dead.join("release-folder")).expect("make the hidden copy");
    write_executable(&dead.join("release-folder").join(BIN));
    fs::write(dead.join(COMPLETE_MARKER), format!("{ID} 0.2.1\n")).expect("write the marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(!version_dir.exists(), "{report:?}");
    assert!(
        !dead.exists(),
        "a finished copy no install can publish is not kept: {:?}",
        names_of(&tool_dir)
    );
    assert!(
        !report
            .recovered
            .iter()
            .any(|sentence| return sentence.contains("left the finished copy")),
        "the report says nothing was left behind: {report:?}"
    );
}

#[test]
fn a_run_whose_recovery_failed_still_installs_the_version_it_could_download() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    serve_releases(&rig, &["0.2.1"]);
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);

    plant_these_steps(Plant {
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    clear_the_plant();
    assert!(
        !rig.http.requests().is_empty(),
        "the install after the failed recovery downloaded the release: {report:?}"
    );
    assert_eq!(report.installed, vec![ID.to_string()], "{report:?}");
    assert!(
        report.failed.is_empty(),
        "the version is in place, so the recovery's problem is a warning: {report:?}"
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| return warning.contains("cannot move the finished install")),
        "the recovery's problem is reported: {report:?}"
    );
    assert!(is_finished(&version_dir, BIN), "{report:?}");
}

#[test]
fn a_recovery_that_failed_and_an_install_that_failed_are_one_failure_naming_both() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    what_a_kill_between_the_two_renames_leaves(&version_dir);

    plant_these_steps(Plant {
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    // The release is not served, so the install after the failed recovery cannot download anything either.
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    clear_the_plant();
    assert_eq!(report.failed.len(), 1, "{report:?}");
    assert!(
        report.failed[0]
            .1
            .contains("cannot move the finished install")
            && report.failed[0]
                .1
                .contains("the install that followed it also failed"),
        "the failure names both causes: {report:?}"
    );
    assert!(
        a_finished_copy_is_on_disk(&version_dir, BIN).is_some(),
        "the finished copy is still there for the next install: {:?}",
        names_of(&tool_dir)
    );
}

#[test]
fn the_claim_never_takes_an_unfinished_copy_for_the_version() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The finished copy is gone: what is left is an unfinished unpacking and the copy the version had.
    let unpacking: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(unpacking.join("release-folder")).expect("make the unfinished copy");
    write_executable(&unpacking.join("release-folder").join(BIN));
    fs::write(unpacking.join(WHAT_WAS_BEING_UNPACKED), "half a release").expect("write");
    let aside: PathBuf = hidden_sibling(
        &version_dir,
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(aside.join("release-folder")).expect("make the copy that was there");
    write_executable(&aside.join("release-folder").join(BIN));
    fs::write(aside.join(WHAT_THE_VERSION_HELD), "the copy that was there").expect("write");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(recovered.restored, "{recovered:?}");
    assert!(
        version_dir.join(WHAT_THE_VERSION_HELD).is_file(),
        "the version gets the copy it had, never the half-unpacked one: {:?}",
        names_of(&tool_dir)
    );
    assert!(!has_complete_marker(&version_dir));
    assert!(!unpacking.exists(), "the unfinished copy is removed");
}

#[test]
fn another_versions_finished_copy_is_never_published_under_this_versions_name() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // This version's name holds nothing, and the only finished copy in the directory belongs to another version.
    let elsewhere: PathBuf = hidden_sibling(
        &tool_dir.join("0.2.2"),
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(elsewhere.join("release-folder")).expect("make the other version's copy");
    write_executable(&elsewhere.join("release-folder").join(BIN));
    write_complete_marker(&elsewhere, ID, "0.2.2", A_TEST_SHA256).expect("write the marker");
    let aside: PathBuf = hidden_sibling(
        &version_dir,
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(aside.join("release-folder")).expect("make the copy that was there");
    write_executable(&aside.join("release-folder").join(BIN));
    fs::write(aside.join(WHAT_THE_VERSION_HELD), "the copy that was there").expect("write");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(recovered.restored, "{recovered:?}");
    assert!(
        version_dir.join(WHAT_THE_VERSION_HELD).is_file(),
        "this version gets the copy it had: {:?}",
        names_of(&tool_dir)
    );
    assert!(
        !is_finished(&version_dir, BIN),
        "another version's finished copy is never this version's"
    );
    assert!(
        elsewhere.is_dir(),
        "the other version's finished copy is left for its own install: {recovered:?}"
    );

    // The same answer for a copy of another version whose marker happens to record this version's tool, version and
    // archive digest: the name it sits under is the fact that decides, never the marker alone.
    let second_rig = Rig::new();
    let second_root = second_rig.scope.toolchain_root.clone();
    let second_tool_dir: PathBuf = tool_directory(&second_root, ID).expect("a plain name");
    let second_version_dir: PathBuf = second_tool_dir.join("0.2.1");
    let vouching_elsewhere: PathBuf = hidden_sibling(
        &second_tool_dir.join("0.2.2"),
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(vouching_elsewhere.join("release-folder"))
        .expect("make the other version's copy");
    write_executable(&vouching_elsewhere.join("release-folder").join(BIN));
    write_complete_marker(&vouching_elsewhere, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the marker");

    let second = recover_leftovers(
        &second_version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(
        second.finished_copy == FinishedCopy::NothingToPublish,
        "a vouched marker under another version's name vouches for nothing here: {second:?}"
    );
    assert!(!second_version_dir.exists());
    assert!(
        vouching_elsewhere.is_dir(),
        "the copy stays under its own version's name: {second:?}"
    );
}

#[test]
fn a_claim_that_fails_for_another_reason_is_reported() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let (finished, _aside) = what_a_kill_between_the_two_renames_leaves(&version_dir);
    // A directory that refuses writes inside it refuses the claim's rename, and the answer is not "no such file".
    let read_only = ReadOnly::new(&tool_dir);

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    );
    drop(read_only);
    let error = recovered.expect_err("the claim's rename was refused");
    assert!(
        error
            .message
            .contains("cannot claim the finished install at"),
        "the failure names the claim: {error}"
    );
    assert!(
        finished.is_dir(),
        "the finished copy is still there: {:?}",
        names_of(&tool_dir)
    );
}

//endregion
#[test]
fn a_permission_denial_on_the_exchange_is_named_on_standard_error_and_the_other_fallbacks_are_silent()
 {
    // What: `Errno::ACCESS.raw_os_error()` is the operating system's own number for "permission denied", and
    //  `Errno::INVAL` the one for a flag the file system does not support (TS: `error.code`).
    // Why: the first is the answer a policy that refuses only the exchange gives, which must be named, and the second is
    //  the answer a file system that cannot exchange gives, which stays silent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const denied = runTheInstallerWhoseExchangeIsRefused(toolDir, version, 'EACCES');
    // const unsupported = runTheInstallerWhoseExchangeIsRefused(toolDir, version, 'EINVAL');
    // ```
    let denied: std::process::Output =
        run_the_installer_whose_exchange_a_policy_refuses(rustix::io::Errno::ACCESS.raw_os_error());
    assert!(denied.status.success(), "{denied:?}");
    let named: String = String::from_utf8_lossy(&denied.stderr).to_string();
    assert!(
        named.starts_with("mpm-shadow: warning:"),
        "the denial is one line on standard error: {named}"
    );
    assert!(
        named.contains("Permission denied"),
        "the line names the error: {named}"
    );
    assert!(
        named.contains("in one exchange was refused"),
        "the line says which call was refused: {named}"
    );

    let silent: std::process::Output =
        run_the_installer_whose_exchange_a_policy_refuses(rustix::io::Errno::INVAL.raw_os_error());
    assert!(silent.status.success(), "{silent:?}");
    assert!(
        silent.stderr.is_empty(),
        "an answer that means the file system cannot exchange the two names falls back in silence: {}",
        String::from_utf8_lossy(&silent.stderr)
    );
}

/// What:
/// Starts this test binary again to publish a finished copy over an unfinished one whose exchange the file system refuses,
/// and answers what that process wrote on standard error.
/// Why:
/// The one line a permission denial prints on standard error can only be read from the process that writes it,
/// and the fixture is a process of its own because a plant lives in a thread of the publish that reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheInstallerWhoseExchangeIsRefused(refusal: number): ChildProcessOutput {
///   return spawnSync(process.execPath, [testBinary, '--exact', INSTALLER_WHOSE_EXCHANGE_IS_REFUSED, '--ignored', '--test-threads=1'], {
///     env: { ...process.env, MPM_SHADOW_TEST_FIXTURE_TOOL_DIRECTORY: toolDir, MPM_SHADOW_TEST_FIXTURE_VERSION: version, MPM_SHADOW_TEST_FIXTURE_EXCHANGE_REFUSAL: String(refusal) },
///   });
/// }
/// ```
fn run_the_installer_whose_exchange_a_policy_refuses(refusal: i32) -> std::process::Output {
    let rig = Rig::new();
    let tool_directory: PathBuf =
        tool_directory(&rig.scope.toolchain_root, ID).expect("a plain name");
    let output = Command::new(std::env::current_exe().expect("the test binary's own path"))
        .arg("--exact")
        .arg(INSTALLER_WHOSE_EXCHANGE_IS_REFUSED)
        .arg("--ignored")
        .arg("--test-threads=1")
        // What: `--nocapture` keeps the test harness from taking the child's standard error for its own report, so the
        //  line the publish writes reaches this process.
        // Why: a captured line is printed only when the child fails, and this child is expected to succeed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // childArgs.push('--nocapture');
        // ```
        .arg("--nocapture")
        .env(TOOL_DIRECTORY_VARIABLE, &tool_directory)
        .env(VERSION_VARIABLE, "0.2.1")
        .env(REFUSAL_VARIABLE, refusal.to_string())
        .output()
        .expect("start the installer whose exchange is refused");
    return output;
}

/// What:
/// The installer a control starts whose exchange a policy refuses:
/// it writes the copy an install publishes over an unfinished one,
/// and publishes it with the refusal planted.
/// Why:
/// The line an exchange's permission denial prints is read from this process's standard error,
/// and this test refuses rather than passes when the publish did not fall back to the two renames.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the installer whose exchange a policy refuses', () => {
///   publishStaged(staging, versionDir, BIN).expect('the two renames publish the copy');
/// });
/// ```
#[test]
#[ignore = "a fixture a control starts: it publishes a copy whose exchange a policy refuses"]
fn the_installer_whose_exchange_a_policy_refuses() {
    let tool_directory: String = std::env::var(TOOL_DIRECTORY_VARIABLE)
        .expect("a control names the fixture's tool directory");
    let version: String = std::env::var(VERSION_VARIABLE).expect("a control names the version");
    let refusal: String =
        std::env::var(REFUSAL_VARIABLE).expect("a control names the answer the exchange gives");
    let version_dir: PathBuf = PathBuf::from(tool_directory).join(&version);
    // The unfinished copy at the version's name, and the finished copy an install would publish.
    write_executable(&version_dir.join("release-folder").join(BIN));
    let staging: PathBuf = staging_dir_of(&version_dir);
    fs::create_dir_all(staging.join("release-folder")).expect("make the staging directory");
    write_executable(&staging.join("release-folder").join(BIN));
    write_complete_marker(&staging, ID, &version, A_TEST_SHA256)
        .expect("write the completion marker");

    plant_these_steps(Plant {
        answer_the_exchange_with: Some(refusal.parse().expect("an operating system error number")),
        ..Plant::default()
    });
    publish_staged(&staging, &version_dir, BIN).expect("the two renames publish the copy");
    clear_the_plant();
}

#[test]
fn a_boot_another_namespace_and_an_unreadable_record_are_never_read_as_an_installer_that_ended() {
    let here: Owner = Owner::here();
    let live: Owner = Owner {
        pid: here.pid,
        started: here.started,
        boot: here.boot.clone(),
        namespace: here.namespace,
    };
    assert_eq!(live.liveness(), Liveness::Running, "this process is here");

    let another_boot: &str = "11111111-1111-1111-1111-111111111111";
    {
        let _view = read_as(ReaderView {
            boot: Some(Some(another_boot.to_string())),
            namespace: Some(here.namespace),
            start: Some(ForcedStart::Gone),
        });
        assert_eq!(
            live.liveness(),
            Liveness::Gone,
            "a name from another boot is an installer that reboot certainly ended"
        );
    }
    {
        let elsewhere: Option<u64> = here.namespace.map(|space| return space + 1);
        let _view = read_as(ReaderView {
            boot: Some(here.boot.clone()),
            namespace: Some(elsewhere),
            start: Some(ForcedStart::Gone),
        });
        assert_eq!(
            live.liveness(),
            Liveness::Unknown,
            "a name from another namespace is an installer this reader cannot see"
        );
    }
    {
        let _view = read_as(ReaderView {
            boot: Some(here.boot.clone()),
            namespace: Some(here.namespace),
            start: Some(ForcedStart::Unreadable),
        });
        assert_eq!(
            live.liveness(),
            Liveness::Unknown,
            "an unreadable record is not a dead process"
        );
    }
    {
        let _view = read_as(ReaderView {
            boot: Some(here.boot.clone()),
            namespace: Some(here.namespace),
            start: Some(ForcedStart::At {
                moment: here.started,
            }),
        });
        assert_eq!(
            live.liveness(),
            Liveness::Running,
            "the recorded moment is this process's own start"
        );
    }
    {
        let _view = read_as(ReaderView {
            boot: Some(here.boot.clone()),
            namespace: Some(here.namespace),
            start: Some(ForcedStart::Gone),
        });
        assert_eq!(
            live.liveness(),
            Liveness::Gone,
            "a free number in this boot and namespace is ended"
        );
    }
    {
        let no_moment: Owner = Owner {
            started: 0,
            ..live.clone()
        };
        let _view = read_as(ReaderView {
            boot: Some(here.boot.clone()),
            namespace: Some(here.namespace),
            start: Some(ForcedStart::Gone),
        });
        assert_eq!(
            no_moment.liveness(),
            Liveness::Unknown,
            "a name with no start time is never judged"
        );
    }
}

#[test]
fn names_written_by_earlier_builds_are_read_and_never_judged() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    fs::create_dir_all(&tool_dir).expect("make the tool directory");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The shape alpha.13 and earlier wrote: the purpose and the process id alone.
    let pid_only: PathBuf = tool_dir.join(format!(".0.2.1.{STAGING_PURPOSE}.42"));
    // The shape alpha.14 wrote: the purpose, the process id and its start time.
    let start_only: PathBuf = tool_dir.join(format!(".0.2.1.{STAGING_PURPOSE}.42.100"));
    // The shape this build writes: all four facts.
    let newest: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    for path in [&pid_only, &start_only, &newest] {
        fs::create_dir_all(path).expect("make a hidden directory");
    }

    let found = hidden_siblings(&tool_dir).expect("list the hidden names");
    let pid_leftover = found
        .iter()
        .find(|leftover| return leftover.path == pid_only)
        .expect("the oldest shape is still read");
    assert_eq!(pid_leftover.version, "0.2.1");
    assert_eq!(
        pid_leftover.liveness(),
        Liveness::Unknown,
        "a name with no start time cannot be judged"
    );
    let start_leftover = found
        .iter()
        .find(|leftover| return leftover.path == start_only)
        .expect("the middle shape is still read");
    assert_eq!(start_leftover.version, "0.2.1");
    assert_eq!(
        start_leftover.liveness(),
        Liveness::Unknown,
        "a start time without a boot and a namespace cannot be judged either"
    );
    let newest_leftover = found
        .iter()
        .find(|leftover| return leftover.path == newest)
        .expect("the newest shape is read");
    assert_eq!(newest_leftover.version, "0.2.1");
    assert_eq!(
        newest_leftover.liveness(),
        Liveness::Gone,
        "the dead installer of the newest shape is judged"
    );
}

#[test]
fn a_finished_copy_the_lock_does_not_vouch_for_is_never_published() {
    let cases: [(&str, String); 4] = [
        (
            "another tool and version",
            format!("some-other-tool 9.9.9 sha256:{A_TEST_SHA256}\n"),
        ),
        (
            "another archive",
            format!("{ID} 0.2.1 sha256:{A_TEST_SHA256}\n"),
        ),
        ("an empty marker", String::new()),
        ("a marker an earlier build wrote", format!("{ID} 0.2.1\n")),
    ];
    for (case, marker_text) in cases {
        let rig = Rig::new();
        let root = rig.scope.toolchain_root.clone();
        let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
        let version_dir: PathBuf = tool_dir.join("0.2.1");
        let dead: PathBuf = hidden_sibling(
            &version_dir,
            STAGING_PURPOSE,
            an_installer_that_no_longer_runs(),
        );
        fs::create_dir_all(dead.join("release-folder")).expect("make the hidden copy");
        write_executable(&dead.join("release-folder").join(BIN));
        fs::write(dead.join(COMPLETE_MARKER), marker_text).expect("write the marker");

        let recovered = recover_leftovers(
            &version_dir,
            BIN,
            AnOlderVersion::MayStay,
            Some(&the_locks_record("0.2.1")),
        )
        .expect("the leftovers are read");
        assert!(
            recovered.finished_copy == FinishedCopy::NothingToPublish,
            "{case}: a copy the lock does not vouch for is never published: {recovered:?}"
        );
        assert!(
            !is_finished(&version_dir, BIN),
            "{case}: no version was given that copy"
        );
        assert!(
            !dead.exists(),
            "{case}: a dead unvouched copy of this version is removed"
        );
    }
}

#[test]
fn a_marker_whose_program_is_missing_is_never_published() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let dead: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(&dead).expect("make the hidden copy without its program");
    write_complete_marker(&dead, ID, "0.2.1", &release_digest("0.2.1")).expect("write the marker");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("the leftovers are read");
    assert!(
        recovered.finished_copy == FinishedCopy::NothingToPublish,
        "a marker alone does not show a finished install: {recovered:?}"
    );
    assert!(!version_dir.exists());
    assert!(!dead.exists());
}

#[test]
fn a_vouched_finished_copy_of_this_version_is_published_without_a_download() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let dead: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(dead.join("release-folder")).expect("make the hidden copy");
    write_executable(&dead.join("release-folder").join(BIN));
    write_complete_marker(&dead, ID, "0.2.1", &release_digest("0.2.1")).expect("write the marker");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("the leftovers are read");
    assert!(
        recovered.finished_copy == FinishedCopy::PublishedHere,
        "{recovered:?}"
    );
    assert!(is_finished(&version_dir, BIN));
    assert!(!dead.exists());
}

#[test]
fn the_copy_a_kill_moved_aside_waits_while_an_installer_this_reader_cannot_judge_may_hold_the_version()
 {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // Only the copy that was there, under a dead installer's aside name.
    let aside: PathBuf = the_copy_a_kill_moved_aside(&version_dir);
    // A name of this boot whose installer this reader cannot see: a live install in another process namespace.
    let here: Owner = Owner::here();
    let elsewhere_space: u64 = here
        .namespace
        .map(|space| return space + 1)
        .expect("the kernel reports this process namespace");
    let unseeable: PathBuf = hidden_sibling(
        &version_dir,
        RECOVERED_PURPOSE,
        Owner {
            pid: here.pid,
            started: here.started,
            boot: here.boot.clone(),
            namespace: Some(elsewhere_space),
        },
    );
    fs::create_dir_all(&unseeable).expect("make the name this reader cannot judge");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(
        !version_dir.exists(),
        "a name that may belong to a live install in another namespace holds the put-back: {recovered:?}"
    );
    assert!(
        aside.is_dir(),
        "the copy that was there waits where it is: {recovered:?}"
    );
    assert!(recovered.left_for_a_live_install, "{recovered:?}");
    assert!(!recovered.restored, "{recovered:?}");
    assert!(unseeable.is_dir(), "nothing of an unjudged name is touched");
}

#[test]
fn a_name_of_an_earlier_build_does_not_hold_the_copy_back() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let aside: PathBuf = the_copy_a_kill_moved_aside(&version_dir);
    // The shape alpha.14 wrote, which carries no boot and no namespace: no install ever removes it, so waiting for it
    // would keep the copy aside for ever.
    let earlier: PathBuf = tool_dir.join(format!(".0.2.1.{STAGING_PURPOSE}.42.100"));
    fs::create_dir_all(&earlier).expect("make the earlier build's name");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("recovered");
    assert!(
        recovered.restored,
        "the copy that was there comes back rather than waiting for ever: {recovered:?}"
    );
    assert!(version_dir.join(WHAT_THE_VERSION_HELD).is_file());
    assert!(!aside.exists(), "the aside name is gone");
    assert!(
        earlier.is_dir(),
        "the name an earlier build wrote is left where it is: {recovered:?}"
    );
}
