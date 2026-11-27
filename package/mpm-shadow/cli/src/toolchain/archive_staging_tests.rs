//! Unit tests of the publish step of an archive install:
//! what a kill between its two renames leaves behind,
//! which a control shows by ending a real installer at that instant,
//! and never by timing.
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
/// Imports the types that start and wait for the installer a control kills.
use std::process::{
    Command,
    ExitStatus,
};

/// Imports the names directly inside a directory.
use super::archive_install_tests::names_of;
/// Imports what the publish step answers with.
use super::archive_staging::{
    clear_the_plant,
    has_complete_marker,
    plant_ending_the_process,
    publish_staged,
    staging_dir_of,
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
    lock_at,
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
        names_of(&tool_dir),
        vec!["0.2.1".to_string(), "current".to_string()],
        "nothing of the killed installer stays behind"
    );

    // A run after that finds the installed version and changes nothing.
    let second = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert_eq!(second.unchanged, vec![ID.to_string()], "{second:?}");
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
    plant_ending_the_process();
    publish_staged(&staging, &version_dir, BIN)
        .expect("the publish ends in the kill at the instant between its two renames");
    clear_the_plant();
    panic!(
        "the publish returned, so no kill between the two renames happened: the process was not ended at that instant"
    );
}
