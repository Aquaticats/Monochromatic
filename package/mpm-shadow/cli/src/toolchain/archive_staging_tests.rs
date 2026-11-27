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
    COMPLETE_MARKER,
    Owner,
    Plant,
    REPLACED_PURPOSE,
    Recovered,
    STAGING_PURPOSE,
    clear_the_plant,
    has_complete_marker,
    hidden_sibling,
    plant_these_steps,
    publish_staged,
    recover_leftovers,
    staging_dir_of,
    start_time_of,
    write_complete_marker,
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
    BIN,
    ID,
    context_of,
    finish_archive,
    lock_at,
    serve_releases,
    unmarked_archive,
};
/// Imports the install itself.
use super::sync::sync_tools;

/// The variable the installer this control kills reads the tool directory of its fixture from.
const TOOL_DIRECTORY_VARIABLE: &str = "MPM_SHADOW_TEST_FIXTURE_TOOL_DIRECTORY";

/// The variable that same installer reads the version it publishes from.
const VERSION_VARIABLE: &str = "MPM_SHADOW_TEST_FIXTURE_VERSION";

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
    write_complete_marker(&staging, ID, &version).expect("write the completion marker");
    plant_these_steps(Plant {
        refuse_the_exchange: true,
        kill_between_the_two_renames: true,
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
    /// as a hidden sibling's name records it.
    fn owner(&self) -> Owner {
        let pid: u32 = self.child.id();
        return Owner {
            pid,
            started: start_time_of(pid).expect("the system reports the program's start"),
        };
    }
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
    let started: u64 = start_time_of(1).expect("the system reports the first process's start");
    return Owner {
        pid: 1,
        started: started + 1,
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
    write_complete_marker(&staging, ID, "0.2.1").expect("write the completion marker");

    // The instant the two-rename path has is armed, and the exchange never reaches it: this call returns.
    plant_these_steps(Plant {
        refuse_the_exchange: false,
        kill_between_the_two_renames: true,
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
    write_complete_marker(&staging, ID, "0.2.1").expect("write the completion marker");

    plant_these_steps(Plant {
        refuse_the_exchange: true,
        kill_between_the_two_renames: false,
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
    write_complete_marker(&version_dir, ID, "0.2.1").expect("write the completion marker");
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

    let live: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        Owner {
            pid: 1,
            started: start_time_of(1).expect("the system reports the first process's start"),
        },
    );
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
    write_complete_marker(&finished_elsewhere, ID, "0.2.2").expect("write the completion marker");

    let recovered: Recovered = recover_leftovers(&version_dir, BIN).expect("recovered");
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
    write_complete_marker(&dead, "node", &rig.tool("node").version).expect("write the marker");

    let recovered: Recovered = recover_leftovers(&version_dir, "").expect("recovered");
    assert!(
        recovered.published,
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
