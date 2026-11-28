//! What:
//! The words a failed publish prints,
//! and what the install that follows does with the copy those words name.
//! Why:
//! `0.2.7-alpha.18` ended every such message at the path the finished copy is at,
//! and the sentence that says which later install acts on that copy was left only to the warning a failed
//! recovery becomes;
//! an ordinary install has no install after it in the same run,
//! so its failure is the only place a person learns where the version comes back from.
//! Each control here builds one arm an ordinary install reaches,
//! fails that one call,
//! reads the line a person sees,
//! and then runs the install that follows:
//! it publishes the copy the words named,
//! and it downloads nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-words-a-failed-publish-prints-tests.ts
//! test('a failed publish of an absent name says what a later install does with the copy');
//! test('a failed exchange says what a later install does with the copy it kept');
//! test('a failed aside rename says what a later install does with the copy it kept');
//! test('a failed rename into a name put back says what a later install does with the kept copy');
//! test('a put-back a second install forces says what a later install does with the kept copy');
//! test('a failed claim says what a later install does with the finished copy it named');
//! test('a failed put-back of the copy that was there claims nothing about a finished copy');
//! ```

/// Imports `fs` from `std`,
/// for the fixture's own modes and names.
use std::fs;
/// Imports the owned path type this file's fixtures are built from (not `&Path`).
use std::path::PathBuf;

/// Imports what a recovery answers,
/// the sentence every failed publish ends with,
/// and the plants that stop one at a step.
use super::super::archive_staging::{
    AnOlderVersion,
    Plant,
    RenameRefused,
    STAGING_PURPOSE,
    WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY,
    WhatTheFailureNamed,
    clear_the_plant,
    hidden_sibling,
    plant_these_steps,
    recover_leftovers,
    staging_dir_of,
};
/// Imports the tool directory of a tool id.
use super::super::layout::tool_directory;
/// Imports the question that tells a finished install from an unfinished one.
use super::super::published_archive::is_finished;
/// Imports the install itself and one sync's report.
use super::super::sync::{
    SyncReport,
    sync_tools,
};
/// Imports the text `install` prints for a report.
use super::super::sync_report_text::format_sync_report;
/// Imports the fixture:
/// the rig,
/// its one tool and program,
/// the lock of one version,
/// the archive that is there without its marker,
/// and the releases the fake network serves.
use super::{
    AN_INPUT_OUTPUT_ERROR,
    BIN,
    ID,
    ReadOnly,
    Rig,
    a_finished_copy_at_the_version_as_another_install,
    an_installer_that_no_longer_runs,
    as_after_this_process_ended,
    context_of,
    lock_at,
    names_of,
    release_digest,
    serve_releases,
    the_copy_a_kill_moved_aside,
    the_locks_record,
    unmarked_archive,
    write_complete_marker,
    write_executable,
};

/// The version every control here installs.
const THE_VERSION: &str = "0.2.1";

/// What:
/// What the next install printed,
/// after this run's failed publish left a finished copy for it.
/// Why:
/// A sentence about a later install is worth nothing until that install has run,
/// and this reads its lines rather than the code's intention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theNextInstallPublishesTheCopy(rig: Rig, version: string, downloadsBefore: number): void {
///   const next = syncTools(contextOf(rig, rig.scope, lockAt(rig, version)));
///   expect(next.failed).toEqual([]);
///   expect(formatSyncReport(next)).toContain(`toolchain install: finished the publish of ${ID} ${version} from the finished copy an earlier install left at`);
///   expect(rig.http.requests()).toHaveLength(downloadsBefore);
/// }
/// ```
fn the_next_install_publishes_the_copy(
    rig: &Rig,
    version: &str,
    downloads_before: usize,
) {
    let next: SyncReport = sync_tools(&context_of(rig, &rig.scope, &lock_at(rig, version)), None);
    assert!(next.failed.is_empty(), "{next:?}");
    let printed: String = format_sync_report(&next);
    assert!(
        printed.contains(&format!(
            "toolchain install: finished the publish of {ID} {version} from the finished copy an earlier install left at"
        )),
        "the next install publishes the copy the failure named: {printed}"
    );
    assert_eq!(
        rig.http.requests().len(),
        downloads_before,
        "the next install publishes that copy without downloading: {:?}",
        rig.http.requests()
    );
}

/// What:
/// The line an ordinary install prints when its own publish fails,
/// checked against the arm that failed,
/// and the install that follows.
/// Why:
/// Every arm ends with the same sentence,
/// which is the one a reader acts on,
/// and the copy it names has to be the one the next install publishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFailureSaysWhatALaterInstallDoes(rig: Rig, report: SyncReport, opening: string, copy: string, version: string): void {
///   expect(report.failed).toHaveLength(1);
///   const message = formatSyncReport(report);
///   expect(message).toContain('toolchain install: ' + ID + ' failed: ' + opening);
///   expect(message).toContain(`still at ${copy}. ${WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY}`);
///   expect(isFinished(copy, BIN)).toBe(true);
///   const downloads = rig.http.requests().length;
///   asAfterThisProcessEnded(copy, versionDir, 'staging');
///   theNextInstallPublishesTheCopy(rig, version, downloads);
/// }
/// ```
fn the_failure_says_what_a_later_install_does(
    rig: &Rig,
    report: &SyncReport,
    opening: &str,
    copy: &str,
    version_dir: &std::path::Path,
) {
    assert_eq!(report.failed.len(), 1, "{report:?}");
    let printed: String = format_sync_report(report);
    assert!(
        printed.contains(&format!("toolchain install: {ID} failed: {opening}")),
        "the failure names the call that failed: {printed}"
    );
    assert!(
        printed.contains(&format!(
            "still at {copy}. {WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY}\n"
        )),
        "the failure names the copy and says what a later install does with it: {printed}"
    );
    // What: the three facts L-4 is about, read out of the printed line rather than from the constant.
    // Why: a sentence that stops saying which later install acts, and which one does not, is the defect this control
    //  exists for; reading the constant back would hold nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(printed).toContain('publishes that copy without downloading when the version\'s name holds no finished copy by then');
    // expect(printed).toContain('an install in another process namespace does not judge that name and leaves the copy where it is');
    // expect(printed).toContain('prune --toolchain prints that name and what deleting it costs');
    // ```
    assert!(
        printed.contains(
            "publishes that copy without downloading when the version's name holds no finished copy by then"
        ),
        "the sentence says which later install publishes the copy: {printed}"
    );
    assert!(
        printed.contains(
            "an install in another process namespace does not judge that name and leaves the copy where it is"
        ),
        "the sentence says which later install does not act on it: {printed}"
    );
    assert!(
        printed.contains("prune --toolchain prints that name and what deleting it costs"),
        "the sentence says what a person does then: {printed}"
    );
    assert!(
        is_finished(std::path::Path::new(copy), BIN),
        "the finished copy the message names is on disk: {:?}",
        names_of(version_dir.parent().expect("the tool's directory"))
    );
    let downloads_before: usize = rig.http.requests().len();
    as_after_this_process_ended(std::path::Path::new(copy), version_dir, STAGING_PURPOSE);
    the_next_install_publishes_the_copy(rig, THE_VERSION, downloads_before);
}

#[test]
fn an_ordinary_installs_failed_publish_says_what_a_later_install_does_with_the_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(THE_VERSION);
    serve_releases(&rig, &[THE_VERSION]);
    plant_these_steps(Plant {
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let report: SyncReport = sync_tools(
        &context_of(&rig, &rig.scope, &lock_at(&rig, THE_VERSION)),
        None,
    );
    clear_the_plant();
    let copy: String = staging_dir_of(&version_dir).display().to_string();
    the_failure_says_what_a_later_install_does(
        &rig,
        &report,
        &format!(
            "cannot move the finished install to {}: Input/output error (os error 5)",
            version_dir.display()
        ),
        &copy,
        &version_dir,
    );
}

#[test]
fn a_failed_exchange_says_what_a_later_install_does_with_the_copy_it_kept() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(THE_VERSION);
    serve_releases(&rig, &[THE_VERSION]);
    // A copy an earlier release unpacked in place: the program, no completion marker.
    unmarked_archive(&root, THE_VERSION);
    plant_these_steps(Plant {
        answer_the_exchange_with: Some(AN_INPUT_OUTPUT_ERROR),
        ..Plant::default()
    });
    let report: SyncReport = sync_tools(
        &context_of(&rig, &rig.scope, &lock_at(&rig, THE_VERSION)),
        None,
    );
    clear_the_plant();
    let copy: String = staging_dir_of(&version_dir).display().to_string();
    the_failure_says_what_a_later_install_does(
        &rig,
        &report,
        &format!(
            "cannot put the finished install in place of the copy at {}: Input/output error (os error 5); that copy is as it was",
            version_dir.display()
        ),
        &copy,
        &version_dir,
    );
}

#[test]
fn a_failed_aside_rename_says_what_a_later_install_does_with_the_copy_it_kept() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(THE_VERSION);
    serve_releases(&rig, &[THE_VERSION]);
    unmarked_archive(&root, THE_VERSION);
    plant_these_steps(Plant {
        refuse_the_exchange: true,
        refuse_the_rename: Some((RenameRefused::MovesTheCopyAside, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let report: SyncReport = sync_tools(
        &context_of(&rig, &rig.scope, &lock_at(&rig, THE_VERSION)),
        None,
    );
    clear_the_plant();
    let copy: String = staging_dir_of(&version_dir).display().to_string();
    the_failure_says_what_a_later_install_does(
        &rig,
        &report,
        &format!(
            "cannot move the copy at {} aside to install over it: Input/output error (os error 5); that copy is as it was",
            version_dir.display()
        ),
        &copy,
        &version_dir,
    );
}

#[test]
fn a_failed_rename_into_a_name_put_back_says_what_a_later_install_does_with_the_kept_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(THE_VERSION);
    serve_releases(&rig, &[THE_VERSION]);
    unmarked_archive(&root, THE_VERSION);
    plant_these_steps(Plant {
        refuse_the_exchange: true,
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let report: SyncReport = sync_tools(
        &context_of(&rig, &rig.scope, &lock_at(&rig, THE_VERSION)),
        None,
    );
    clear_the_plant();
    let copy: String = staging_dir_of(&version_dir).display().to_string();
    the_failure_says_what_a_later_install_does(
        &rig,
        &report,
        &format!(
            "cannot move the finished install to {}: Input/output error (os error 5); the copy that was there is back at {}",
            version_dir.display(),
            version_dir.display()
        ),
        &copy,
        &version_dir,
    );
}

#[test]
fn a_put_back_a_second_install_forces_says_what_a_later_install_does_with_the_kept_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(THE_VERSION);
    serve_releases(&rig, &[THE_VERSION]);
    unmarked_archive(&root, THE_VERSION);
    // A second install publishes the version in the instant before this one moves the copy aside,
    // so the copy this publish moves aside is a finished one and its put-back is refused.
    plant_these_steps(Plant {
        refuse_the_exchange: true,
        another_install_publishes_before_the_aside_rename: Some(
            a_finished_copy_at_the_version_as_another_install,
        ),
        refuse_the_rename: Some((RenameRefused::PutsTheCopyBack, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let report: SyncReport = sync_tools(
        &context_of(&rig, &rig.scope, &lock_at(&rig, THE_VERSION)),
        None,
    );
    clear_the_plant();
    let copy: String = staging_dir_of(&version_dir).display().to_string();
    the_failure_says_what_a_later_install_does(
        &rig,
        &report,
        &format!(
            "cannot put the finished install that was at {} back from ",
            version_dir.display()
        ),
        &copy,
        &version_dir,
    );
}

#[test]
fn a_failed_claim_says_what_a_later_install_does_with_the_finished_copy_it_named() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(THE_VERSION);
    let finished: PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(finished.join("release-folder")).expect("make the finished copy");
    write_executable(&finished.join("release-folder").join(BIN));
    write_complete_marker(&finished, ID, THE_VERSION, &release_digest(THE_VERSION))
        .expect("write the completion marker");
    // What: a tool directory that refuses writes, which refuses the claim's rename and nothing else this control reads.
    // Why: the claim's failure names a finished copy a dead install left, and no other rename of that step can be made
    //  to fail on a disposable fixture.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const readOnly = ReadOnly.new(toolDir);
    // ```
    let read_only = ReadOnly::new(&tool_dir);
    let failed = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record(THE_VERSION)),
    )
    .expect_err("the claim's rename was refused");
    drop(read_only);
    assert!(
        failed
            .error
            .message
            .contains("cannot claim the finished install at"),
        "the failure names the claim: {failed:?}"
    );
    assert_eq!(
        failed.named,
        WhatTheFailureNamed::AFinishedCopy,
        "the failure named a finished copy, so the warning says what a later install does with it: {failed:?}"
    );
    assert_eq!(
        failed.what_a_later_install_does(),
        format!("so the run does not fail for it. {WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY}"),
        "the sentence about that copy is the one every failed publish ends with: {failed:?}"
    );
    assert!(
        finished.is_dir(),
        "the finished copy is still under the hidden name the message names: {:?}",
        names_of(&tool_dir)
    );
}

#[test]
fn a_failed_put_back_of_the_copy_that_was_there_claims_nothing_about_a_finished_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join(THE_VERSION);
    // The version's name empty and the copy that was there under its aside name: a kill between the two renames whose
    // finished copy is gone.
    let aside: PathBuf = the_copy_a_kill_moved_aside(&version_dir);
    let read_only = ReadOnly::new(&tool_dir);
    let failed = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record(THE_VERSION)),
    )
    .expect_err("the put-back's rename was refused");
    drop(read_only);
    assert!(
        failed.error.message.contains("cannot put back the copy at"),
        "the failure names the put-back: {failed:?}"
    );
    assert_eq!(
        failed.named,
        WhatTheFailureNamed::TheCopyThatWasMovedAside,
        "the copy it named is the one that was there, not a finished one: {failed:?}"
    );
    assert_eq!(
        failed.what_a_later_install_does(),
        "so the run does not fail for it",
        "no sentence about a finished copy belongs to that copy: {failed:?}"
    );
    assert!(
        aside.is_dir(),
        "the copy that was there is still under its aside name: {:?}",
        names_of(&tool_dir)
    );
}
