//! What:
//!  A named pipe at a record's name in the merge record folder:
//!  every read of that folder answers at once,
//!  with the value an unreadable record earns,
//!  instead of waiting for a writer that may never come.
//! Why:
//!  Opening a pipe for reading waits for a writer that may never come,
//!  and the first read of this folder runs on the start of every wrapped command
//!  (`invocation/startup_gate.rs`),
//!  so one pipe at `.git/monochromatic-git-policy-cli-merges/x.json` held every command
//!  in that repository with nothing said.
//!  Every file this program reads from its own record folders is opened as a plain file
//!  (`SPEC.md`,
//!  "Every file this wrapper reads from its own record folders is opened as a plain file"),
//!  and this folder's records are one of them.
//!  Each control plants the pipe where one of the folder's readers looks,
//!  and fails at the bound `finished_within` keeps rather than holding the run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! mkfifoSync(join(gitDir, 'monochromatic-git-policy-cli-merges', 'x.json'), 0o600);
//! expect(await finishedWithin(() => mergeIntentsToRecover(gitDir))).toBeDefined();
//! ```

use super::super::*;
use super::fixtures::{
    World,
    place,
    world,
};
use crate::merge_isolation_carry_landed::{
    landed_record_path,
    read_landed_record,
};
use crate::merge_isolation_carry_parts::Capture;
use crate::merge_isolation_carry_ref::another_record_still_needs_the_capture;
use crate::merge_isolation_journal::{
    intent_directory,
    intent_path,
};
use crate::merge_isolation_kept_copies_decisions_sweep::merges_whose_owner_runs;
use crate::merge_isolation_recovery_records::{
    merge_intents_to_recover,
    recover_interrupted_merges,
};
use crate::private_storage::create_private_directory;
use crate::recovery_error::RecoveryError;
use crate::test_support::{
    finished_within,
    remove,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The world of one control with its record folder made and a named pipe standing where a record's
///  name is.
/// Why:
///  Every reader below looks the record up by its transaction ID,
///  so the pipe answers the same question a record does while no writer ever opens it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipe = pipeWhereARecordGoes('merge-recovery-pipe-…', '1111…-11');
/// ```
fn pipe_where_a_record_goes(
    name: &str,
    transaction_id: &str,
) -> (World, PathBuf) {
    let world: World = world(name);
    let folder: PathBuf = intent_directory(world.git_dir.as_path());
    create_private_directory(folder.as_path()).expect("the record folder");
    let pipe: PathBuf = intent_path(world.git_dir.as_path(), transaction_id);
    crate::subvolume_sync_attrs::make_fifo(pipe.as_path(), 0o600)
        .expect("a named pipe where a record goes");
    return (world, pipe);
}

/// What:
///  The words every control below reads in the answer and in the pipe's name.
/// Why:
///  One refusal says what failed,
///  the name of the file and what stands there,
///  and what a person does about it,
///  so a control that matches all of it proves the command told the person what to look at and how to end the stop.
///  The last words are the package's rule for a diagnostic
///  (`AGENTS.md`,
///  rule `DGT`):
///  a line that names the file and no way out leaves the person with a repository no command of it can enter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(message).toContain('not a plain file'); expect(message).toContain(pipe); expect(message).toContain('remove it'); expect(message).toContain('run the command again');
/// ```
fn the_record_it_could_not_read(
    message: &str,
    pipe: &Path,
) {
    assert!(
        message.contains("not a plain file"),
        "the answer says what stands at the name: {message}"
    );
    assert!(
        message.contains(pipe.to_string_lossy().as_ref()),
        "the answer names the record: {message}"
    );
    assert!(
        message.contains("Nothing was changed"),
        "the answer says that it changed nothing: {message}"
    );
    assert!(
        message.contains("remove it,") && message.contains("run the command again"),
        "the answer says which file a person removes and that the command then runs: {message}"
    );
}

/// A record that is a named pipe stops the scan of the folder at once,
/// naming itself,
/// rather than waiting for a writer that may never come (issue 6415's shape,
/// in the folder the start of every command reads).
#[test]
fn a_record_that_is_a_named_pipe_stops_the_scan_at_once() {
    let (world, pipe): (World, PathBuf) = pipe_where_a_record_goes(
        "merge-recovery-pipe-scan",
        "11111111-0000-4000-8000-000000000011",
    );
    let inside: PathBuf = world.git_dir.clone();
    let answered = finished_within(move || return merge_intents_to_recover(inside.as_path()));
    let Some(answer) = answered else {
        panic!("the scan of the merge records waited on a named pipe instead of answering")
    };
    let error: RecoveryError = answer.expect_err("a record this command cannot read stops it");
    the_record_it_could_not_read(error.0.as_str(), &pipe);
    assert!(
        pipe.exists(),
        "the pipe is left where it is, as any unreadable record is"
    );
    remove(world.parent.as_path());
}

/// A record that is a named pipe stops the recovery that reads every record before settling any
/// at once,
/// naming itself,
/// so the command the start of every wrapped command runs answers too.
#[test]
fn a_record_that_is_a_named_pipe_stops_the_recovery_at_once() {
    let (world, pipe): (World, PathBuf) = pipe_where_a_record_goes(
        "merge-recovery-pipe-recovery",
        "22222222-0000-4000-8000-000000000012",
    );
    let recovery_place: RecoveryPlace = place(&world);
    let inside: PathBuf = world.git_dir.clone();
    let answered = finished_within(move || {
        let mut out: Vec<u8> = Vec::new();
        return recover_interrupted_merges(&recovery_place, inside.as_path(), &mut out)
            .map(|outcomes| return outcomes.len());
    });
    let Some(answer) = answered else {
        panic!("the recovery of the merge records waited on a named pipe instead of answering")
    };
    let error: RecoveryError = answer.expect_err("a record this command cannot read stops it");
    the_record_it_could_not_read(error.0.as_str(), &pipe);
    remove(world.parent.as_path());
}

/// A record that is a named pipe names no other record's capture,
/// so the capture's ref is dropped as it would be for a record that is not there.
#[test]
fn a_record_that_is_a_named_pipe_is_no_other_records_capture() {
    let (world, _pipe): (World, PathBuf) = pipe_where_a_record_goes(
        "merge-recovery-pipe-capture",
        "33333333-0000-4000-8000-000000000013",
    );
    let capture: Capture = Capture {
        commit: "c".repeat(40),
        reference: Some(String::from("refs/cli-git/kept/13")),
        paths: Vec::new(),
    };
    let inside: PathBuf = world.git_dir.clone();
    let answered = finished_within(move || {
        return another_record_still_needs_the_capture(inside.as_path(), &capture, None);
    });
    let Some(answer) = answered else {
        panic!("the search for a record that needs a capture waited on a named pipe")
    };
    assert!(
        !answer,
        "a pipe is no record, so it is no claim on the capture"
    );
    remove(world.parent.as_path());
}

/// A record that is a named pipe names no merge whose owner runs,
/// so the command beside a kept copy decides the copy as it would for a record that is not there.
#[test]
fn a_record_that_is_a_named_pipe_names_no_running_merge() {
    let (world, _pipe): (World, PathBuf) = pipe_where_a_record_goes(
        "merge-recovery-pipe-running",
        "44444444-0000-4000-8000-000000000014",
    );
    let inside: PathBuf = world.git_dir.clone();
    let answered = finished_within(move || return merges_whose_owner_runs(inside.as_path()));
    let Some(answer) = answered else {
        panic!("the reading of the merges that run waited on a named pipe")
    };
    assert!(
        answer.is_empty(),
        "a pipe is no record, so it names no merge: {answer:?}"
    );
    remove(world.parent.as_path());
}

/// A landed reading that is a named pipe is no reading,
/// so the recovery judges the paths against the bytes alone rather than waiting on the pipe.
#[test]
fn a_landed_reading_that_is_a_named_pipe_is_no_reading() {
    let (world, _pipe): (World, PathBuf) = pipe_where_a_record_goes(
        "merge-recovery-pipe-landed",
        "55555555-0000-4000-8000-000000000015",
    );
    let landed: PathBuf = landed_record_path(
        world.git_dir.as_path(),
        "55555555-0000-4000-8000-000000000015",
    );
    crate::subvolume_sync_attrs::make_fifo(landed.as_path(), 0o600)
        .expect("a named pipe where a landed reading goes");
    let inside: PathBuf = world.git_dir.clone();
    let answered = finished_within(move || {
        return read_landed_record(inside.as_path(), "55555555-0000-4000-8000-000000000015");
    });
    let Some(answer) = answered else {
        panic!("the reading of a merge's landed files waited on a named pipe")
    };
    assert!(answer.is_empty(), "a pipe is no reading: {answer:?}");
    remove(world.parent.as_path());
}
