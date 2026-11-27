//! What:
//! The two publishes of one install that no control held before the `0.2.7-alpha.15` pass:
//! the one a second install races between the two renames of a file system without the exchange,
//! and the one an install's own failure leaves behind.
//! Why:
//! Both cost a person a wrong status or the only finished copy of a version,
//! and neither could be seen without a control that stops the publish at the instant and asks again:
//! the exchange refusal, the refused rename and the second install are plants of the package's own
//! (`a_control_plants_these_steps`),
//! so the red of each depends on nothing but the code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-publishes-that-fail-or-race-tests.ts
//! test('a second install that publishes between the two renames leaves nothing to report');
//! test('an install whose own publish fails keeps the finished copy for the next run');
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports the owned path type this file's fixtures are built from (not `&Path`).
use std::path::PathBuf;

/// Imports the answer of a publish and the plants that stop one at a step.
use super::super::archive_staging::{
    Plant,
    Published,
    RenameRefused,
    clear_the_plant,
    plant_these_steps,
    publish_staged,
    staging_dir_of,
    write_complete_marker,
};
/// Imports the tool directory of a tool id.
use super::super::layout::tool_directory;
/// Imports the question that tells a finished install from an unfinished one.
use super::super::published_archive::is_finished;
/// Imports the install itself.
use super::super::sync::sync_tools;
/// Imports the context one tree's sync reads.
use super::context_of;
/// Imports the error one rename of a publish is answered with, and the question whether a finished copy of the
/// version is still on disk.
use super::{
    AN_INPUT_OUTPUT_ERROR,
    a_finished_copy_at_the_version_as_another_install,
    a_finished_copy_is_on_disk,
};
/// Imports the fixture's tool id, its program, the lock of one version, its archive digest,
/// the archive that was there without its marker, and the releases the fake network serves.
use super::{
    BIN,
    ID,
    lock_at,
    release_digest,
    serve_releases,
    unmarked_archive,
};
/// Imports the rig, the names a tool directory holds, and the writer of an executable stand-in.
use super::{
    Rig,
    names_of,
    write_executable,
};

/// What:
/// A second install that publishes the version in the instant between this publish's two renames,
/// on a file system that cannot exchange the two names.
/// Why:
/// The version is finished by that install,
/// so this publish has no copy left to put in place and none to put back:
/// its own copy is redundant and the copy it moved aside is unfinished,
/// and a failed status here says an install broke while the version sits there whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a second install that publishes between the two renames leaves nothing to report', () => {
///   unmarkedArchive(root, '0.2.1');
///   writeFinishedCopy(stagingDirOf(versionDir));
///   plantTheseSteps({ refuseTheExchange: true, anotherInstallActs: aFinishedCopyAtTheVersionAsAnotherInstall });
///   expect(publishStaged(staging, versionDir, BIN)).toBe('already-there');
/// });
/// ```
#[test]
fn a_second_install_that_publishes_between_the_two_renames_leaves_nothing_to_report() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The copy that was there, unfinished, and the finished copy this publish holds.
    unmarked_archive(&root, "0.2.1");
    let staging: PathBuf = staging_dir_of(&version_dir);
    fs::create_dir_all(staging.join("release-folder")).expect("make the staging directory");
    write_executable(&staging.join("release-folder").join(BIN));
    write_complete_marker(&staging, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    // What: the file system cannot exchange the two names, and another install publishes the finished version in the
    //  instant between this publish's two renames.
    // Why: that instant is where a second install acts, and the control picks it exactly instead of timing it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plantTheseSteps({ refuseTheExchange: true, anotherInstallActs: aFinishedCopyAtTheVersionAsAnotherInstall });
    // ```
    plant_these_steps(Plant {
        refuse_the_exchange: true,
        another_install_acts: Some(a_finished_copy_at_the_version_as_another_install),
        ..Plant::default()
    });
    let published = publish_staged(&staging, &version_dir, BIN);
    clear_the_plant();
    // What: the answer of that publish.
    // Why: the version's name holds the finished copy, so this call dropped the copy it held and nothing failed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcome = published.expect('the version is finished by the second install');
    // ```
    let outcome: Published = published.expect(
        "the version is finished by the second install, so this publish has nothing left to do",
    );
    assert_eq!(outcome, Published::AlreadyThere, "{outcome:?}");
    assert!(
        is_finished(&version_dir, BIN),
        "the version's name holds the copy the second install published"
    );
    assert_eq!(
        names_of(&tool_dir),
        vec!["0.2.1".to_string()],
        "nothing of the kill or of this publish stays beside the finished version"
    );
}

/// What:
/// An install whose own publish fails,
/// with a release served and unpacked,
/// so the failure happens after the download and the marker.
/// Why:
/// The finished copy is the only way an offline machine gets that version back,
/// so a failed publish must leave it under the name the run holds it at,
/// which the next run publishes without downloading.
/// The install path is driven here, never `publish_staged` alone:
/// the step this control holds is the one `install_archive` gives the publish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('an install whose own publish fails keeps the finished copy for the next run', () => {
///   serveReleases(rig, ['0.2.1']);
///   plantTheseSteps({ refuseTheRename: ['the-finished-copy-goes-in', EIO] });
///   const report = syncTools(contextOf(rig, rig.scope, lockAt(rig, '0.2.1')), undefined);
///   expect(report.failed.length).toBe(1);
///   expect(aFinishedCopyIsOnDisk(versionDir, BIN)).toBeDefined();
/// });
/// ```
#[test]
fn an_install_whose_own_publish_fails_keeps_the_finished_copy_for_the_next_run() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The release the fake network serves, which the fake `tar` unpacks: this run downloads and marks a copy before it
    // publishes it.
    serve_releases(&rig, &["0.2.1"]);

    // What: the one rename that would give the version's name the finished copy,
    //  answered with an error instead of being made.
    // Why: a publish that cannot finish must leave the copy it holds for the next run, and no disposable fixture can
    //  make that rename fail on demand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plantTheseSteps({ refuseTheRename: ['the-finished-copy-goes-in', EIO] });
    // ```
    plant_these_steps(Plant {
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    clear_the_plant();
    assert_eq!(report.failed.len(), 1, "{report:?}");
    assert!(
        report.failed[0]
            .1
            .contains("cannot move the finished install to"),
        "the failure names the rename that failed: {report:?}"
    );
    assert!(
        !is_finished(&version_dir, BIN),
        "the version's name holds nothing: the publish that would have given it the copy failed"
    );
    assert!(
        a_finished_copy_is_on_disk(&version_dir, BIN).is_some(),
        "the finished copy is still under the name this run held it at: {:?}",
        names_of(&tool_dir)
    );

    // A run after that publishes the copy this one left, so the version costs no second download.
    let next = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(next.failed.is_empty(), "{next:?}");
    assert!(
        is_finished(&version_dir, BIN),
        "the next run publishes the copy the failed one left: {next:?}"
    );
    assert_eq!(
        names_of(&tool_dir),
        vec!["0.2.1".to_string(), "current".to_string()],
        "nothing of the failed publish stays"
    );
}
