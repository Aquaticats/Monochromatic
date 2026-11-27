//! What:
//! The controls of the copies a recovery may not publish:
//! a finished copy of a version older than one installed in the other toolchain root,
//! and a copy whose marker matches the lock's row on some of its three fields but not all.
//! Why:
//! Each rule decides whether a hidden copy becomes the version,
//! and each was held only for the case next to it:
//! the older-version rule was shown with the newer version in the scope's own root,
//! and the marker rule with the tool and the version changed together,
//! so an edit that asked one root or compared one field fewer kept every test green.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-copies-a-recovery-may-not-publish-tests.ts
//! test('a recovery never publishes an older version beside a newer one in the other root');
//! test('a marker that matches on some of the three fields vouches for nothing');
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports the owned path type this file's fixtures are built from (not `&Path`).
use std::path::PathBuf;

/// Imports the marker's own name, the hidden names of a version directory, and the step that reads them.
use super::super::archive_staging::{
    AnOlderVersion,
    COMPLETE_MARKER,
    FinishedCopy,
    STAGING_PURPOSE,
    hidden_sibling,
    recover_leftovers,
};
/// Imports the tool directory of a tool id.
use super::super::layout::tool_directory;
/// Imports the question that tells a finished install from an unfinished one.
use super::super::published_archive::is_finished;
/// Imports the install itself.
use super::super::sync::sync_tools;
/// Imports the question whether a finished copy of the version is still on disk,
/// the pair of a process that no longer runs, and the record a lock holds for one copy.
use super::{
    a_finished_copy_is_on_disk,
    an_installer_that_no_longer_runs,
    the_locks_record,
};
/// Imports the rig, the names a tool directory holds, and the writer of an executable stand-in.
use super::{
    Rig,
    names_of,
    write_executable,
};
/// Imports the fixture's tool id, its program, the lock of one version, its archive digest,
/// and the finished copy of one version written by hand.
use super::{
    BIN,
    ID,
    finish_archive,
    lock_at,
    release_digest,
};
/// Imports the state a kill between the two renames leaves, and the context one tree's sync reads.
use super::{
    context_of,
    what_a_kill_between_the_two_renames_leaves,
};

/// What:
/// A recovery of an older version whose newer finished copy is in the other toolchain root.
/// Why:
/// This repository's own tools are one set across both roots,
/// so the rule that an older version is never kept holds wherever the newer copy sits,
/// and a recovery that asked only its own root would publish the older version beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a recovery never publishes an older version beside a newer one in the other root', () => {
///   finishArchive(rig.scope.otherToolchainRoot, '0.2.2');
///   whatAKillBetweenTheTwoRenamesLeaves(versionDir);
///   const report = syncTools(contextOf(rig, rig.scope, lockAt(rig, '0.2.1')), undefined);
///   expect(versionDir.exists()).toBe(false);
/// });
/// ```
#[test]
fn a_recovery_never_publishes_an_older_version_beside_a_newer_one_in_the_other_root() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // What: the newer finished version, which sits in the other toolchain root, as an install with `--global` leaves it.
    // Why: an older version is not kept beside a newer one wherever that newer one is, and a control that had it in the
    //  scope's own root only could not tell which root the rule asks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // finishArchive(rig.scope.otherToolchainRoot, '0.2.2');
    // ```
    finish_archive(&rig.scope.other_toolchain_root, "0.2.2");
    what_a_kill_between_the_two_renames_leaves(&version_dir);

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(
        !version_dir.exists(),
        "an older repository tool is never kept, whichever root holds the newer one: {:?}",
        names_of(&tool_dir)
    );
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(report.installed.is_empty(), "{report:?}");
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

/// What:
/// The markers that record the lock's archive digest and the version this run installs,
/// but not the tool,
/// and the tool and the archive digest but a version no release has.
/// Why:
/// A publish of a hidden copy compares all three fields with the lock's row,
/// and a control that changed two of them at once could not tell which comparison is missing
/// when one is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a marker that matches on some of the three fields vouches for nothing', () => {
///   for (const [case, markerText] of cases) {
///     expect(recoverLeftovers(versionDir, BIN, 'may-stay', theLocksRecord('0.2.1')).finishedCopy)
///       .toBe('nothing-to-publish');
///   }
/// });
/// ```
#[test]
fn a_marker_that_matches_on_some_of_the_three_fields_vouches_for_nothing() {
    // What: the version no release has, which stands for no program's output and marks only that the field differs.
    // Why: the version a marker names is compared as text, so any other text is the same test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const aVersionNoReleaseHas = '9.9.9';
    // ```
    let cases: [(&str, String); 2] = [
        (
            "another tool, with this version and this archive",
            format!("some-other-tool 0.2.1 sha256:{}\n", release_digest("0.2.1")),
        ),
        (
            "this tool, with a version no release has, and this archive",
            format!("{ID} 9.9.9 sha256:{}\n", release_digest("0.2.1")),
        ),
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
            "{case}: a marker the lock's row does not match is never published: {recovered:?}"
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
