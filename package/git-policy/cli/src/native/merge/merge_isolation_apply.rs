//! What:
//!  The writing half of an isolated merge:
//! the clear of the person's paths,
//! Git's own two-way update that puts the merge in the checkout,
//! the carry of their work after it,
//! and the report the command ends with.
//! Why:
//!  `merge_isolation.rs` decides and loops over rounds,
//! and one round is `merge_isolation_round.rs`;
//! the steps that move the checkout are a file of their own that they import and that imports nothing from them.
//!  The record written before any byte changes,
//! the look at `index.lock`,
//! the merge commit and the lines that name the files the word pass merged are files of their own as well,
//! so each file here stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await apply(input, session, plan); // merge --ff-only, then the carry
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// Where Git's words about the wrapper's own fast-forward go now that the run carries no `--quiet`.
use crate::inner_git_words::record_streams;
/// The refusal findings.
use crate::merge_isolation_apply_findings::failure_text;
/// The words a merge that could not be finished now prints,
/// which its own file holds so this one stays inside the code-line limit.
use crate::merge_isolation_apply_findings::{
    WhyTheMergeIsSaved,
    a_refused_cleanup,
};
/// The carry of a merge that landed,
/// its lines,
/// and what a branch that moved under it does about them.
use crate::merge_isolation_apply_moved::{
    CarryEnding,
    carry_and_settle,
};
/// What a merge wrote before any byte of the checkout changed,
/// and the marker phase a crash control arms.
use crate::merge_isolation_apply_record::{
    TheRecordAndTheLockLook,
    reach_phase,
    write_the_record_and_look_at_the_index,
};
/// The save of a merge that met the index lock for a later command,
/// kept in a file of its own so this one stays inside the code-line limit.
use crate::merge_isolation_apply_save::save_for_a_later_finish;
use crate::merge_isolation_apply_stops::{
    a_clear_that_met_newer_bytes,
    the_end_of_a_failed_step,
};
use crate::merge_isolation_carry_apply::{
    LockWait,
    changes_between,
    clear,
};
/// What Git's own update left at each path,
/// and the read of it.
use crate::merge_isolation_carry_landed::{
    LandedFile,
    landed_of,
    write_landed_record,
};
/// The person's own work in the paths the merge changes:
/// the clear before Git's update,
/// the carry after it,
/// and the ref that keeps their bytes.
use crate::merge_isolation_carry_ref::drop_capture_ref;
use crate::merge_isolation_carry_write_failures::CarryFailure;
/// The raw differences between the two commits of the merge.
use crate::merge_isolation_changes::Change;
/// The questions an isolated merge asks Git.
use crate::merge_isolation_git_reads::Session;
/// The intent record,
/// and its removing once the merge has ended.
use crate::merge_isolation_journal::{
    MergeIntent,
    remove_merge_intent,
};
/// Recovery of records.
use crate::merge_isolation_recovery::RecoveryPlace;
use crate::merge_isolation_run_types::{
    Bail,
    IsolatedMerge,
    IsolatedMergeInput,
    Plan,
    TAG,
};
/// The command-line reading.
use crate::merge_options::MergeShape;
/// Real Git.
use crate::transaction_git::{
    GitContext,
    GitOutput,
};
/// The marker phases a crash control arms.
use crate::transaction_phase::Phase;
/// Owned operating-system text.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// The merge tree of an isolated merge and the pass over its conflicts,
/// which moved to a file of their own and keep their names here.
pub use crate::merge_isolation_merge_tree::{
    merge_tree,
    merge_tree_awaiting,
};

/// What:
///  Record the intent,
/// let Git move the checkout to the merge commit,
/// and report.
/// Why:
///   Git's fast-forward is the two-way update that keeps every other staged entry and unstaged change,
/// refuses an
///  untracked file in the way,
/// and moves the branch only after the checkout is updated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function apply({ session, shape, locations, commits }): Promise<IsolatedMerge>;
/// ```
pub fn apply(
    input: &IsolatedMergeInput<'_>,
    session: &Session,
    plan: &Plan<'_>,
) -> Result<IsolatedMerge, Bail> {
    let (git_dir, root): (&Path, &Path) = (plan.git_dir, plan.root);
    let (old_oid, new_oid): (&str, &str) = (plan.old_oid, plan.new_oid);
    let shape: &MergeShape = plan.shape;
    let reference: String = shape.reference.to_string_lossy().into_owned();
    // The record names the person's own bytes and the branch this merge is made on, and it is written before any byte of the
    //  checkout changes; the look at the lock then decides whether that record is what a later command finishes or what this
    //  command does now.
    let recorded: TheRecordAndTheLockLook = write_the_record_and_look_at_the_index(session, plan)?;
    let (transaction_id, intent, record): (String, MergeIntent, PathBuf) =
        (recorded.transaction_id, recorded.intent, recorded.record);
    if let Some(reason) = recorded.not_now {
        // The owner's answer for a live foreign `index.lock` (`SPEC.md`, "Merges and commits never fail"): the merge saves its
        //  result on a side ref, exits 0, and the next command finishes the checkout when the lock is free.
        // Nothing of the clear has run at this instant, so the files hold the person's own bytes where they were.
        return save_for_a_later_finish(
            session,
            plan,
            &intent,
            record.as_path(),
            reference.as_str(),
            reason,
            false,
        );
    }
    let place: RecoveryPlace = RecoveryPlace::settling(&session.context, git_dir, root);
    let changes: Vec<Change> = if plan.carried.is_some() {
        match changes_between(&session.context, root, old_oid, new_oid) {
            Ok(found) => found,
            Err(reason) => {
                return the_end_of_a_failed_step(
                    session,
                    plan,
                    &intent,
                    record.as_path(),
                    reference.as_str(),
                    reason.as_str(),
                    false,
                    false,
                    2,
                    b"",
                );
            }
        }
    } else {
        Vec::new()
    };
    // Git's two-way update refuses a path that holds anything but the branch's content, so the person's paths are put back to
    //  the branch before it runs. Their bytes are the objects the record names, so nothing of theirs is lost by it; a path written
    //  after the capture stops the clear with nothing written, and the round is made again on the newer bytes.
    if let Some(capture) = plan.carried
        && let Err(failure) = clear(
            &session.context,
            root,
            git_dir,
            changes.as_slice(),
            capture,
            transaction_id.as_str(),
            LockWait::BeforeTheApply,
            &mut std::io::stderr(),
        )
    {
        // A path holding bytes newer than the capture is the person's, so the clear wrote nothing to it, the capture goes back
        //  where it can, and the merge is made again on what the file holds now instead of being refused for a save.
        if matches!(failure, CarryFailure::Foreign(_) | CarryFailure::Changed(_)) {
            return Err(a_clear_that_met_newer_bytes(
                &place,
                record.as_path(),
                &intent,
                reference.as_str(),
                &failure,
            ));
        }
        // A lock another program took after the look means nothing of the clear was written, so the merge is saved for a later
        //  finish rather than made to wait, refused, or tried again in a round that would meet the same lock.
        if matches!(failure, CarryFailure::IndexHeld(_)) {
            return save_for_a_later_finish(
                session,
                plan,
                &intent,
                record.as_path(),
                reference.as_str(),
                WhyTheMergeIsSaved::AnotherProgramTookIt,
                false,
            );
        }
        let detail: String = failure_text(&failure);
        return the_end_of_a_failed_step(
            session,
            plan,
            &intent,
            record.as_path(),
            reference.as_str(),
            detail.as_str(),
            false,
            false,
            2,
            b"",
        );
    }
    if plan.carried.is_some() {
        reach_phase(Phase::MergeCleared)?;
    }
    let mut overlay: Vec<(OsString, OsString)> = input.overlay.to_vec();
    overlay.push((
        OsString::from("GIT_REFLOG_ACTION"),
        OsString::from(format!("merge {reference}")),
    ));
    let mover: Session = Session {
        context: GitContext {
            overlay,
            ..session.context.clone()
        },
    };
    // A start failure is handled like a refusal: the record settles whatever Git began.
    let moved: GitOutput = match mover.run(&["merge", "--ff-only", "--no-edit", new_oid]) {
        Ok(output) => output,
        Err(reason) => GitOutput {
            code: None,
            stdout: Vec::new(),
            stderr: reason.into_bytes(),
        },
    };
    if !moved.succeeded() {
        let detail: String = moved.error_text();
        let lock_taken: bool = detail.contains("index.lock");
        return the_end_of_a_failed_step(
            session,
            plan,
            &intent,
            record.as_path(),
            reference.as_str(),
            detail.as_str(),
            lock_taken,
            // This step runs after the clear, so with a capture the paths the merge changes were put back to the branch.
            plan.carried.is_some(),
            moved.code.unwrap_or(2),
            moved.stderr.as_slice(),
        );
    }
    // The file the file system hands out at every path Git just wrote, read before the marker a control pauses at: a save that
    //  arrives afterwards is a different entry, so the carry after the merge never replaces it with the person's older layer.
    let mut written_paths: Vec<Vec<u8>> = Vec::with_capacity(changes.len());
    for change in &changes {
        written_paths.push(change.path.clone());
    }
    let landed_paths: Vec<LandedFile> = landed_of(root, written_paths.as_slice());
    let landed: &[LandedFile] = landed_paths.as_slice();
    // What Git wrote is recorded beside the merge's record, so a command that is killed from here to the end of the carry
    //  reads the same entries this command read, and a save that arrives after the kill is never read as the merge's own file.
    write_landed_record(git_dir, transaction_id.as_str(), landed);
    reach_phase(Phase::MergeApplied)?;
    // The merge's own paths hold the merge now, and the person's two layers are merged with it: an edit away from the merge's
    //  lines comes back where it was, a staged edit stays staged, and an edit of the merge's own lines keeps both sides between
    //  markers. A carry that cannot write the index says where the bytes are and exits 0, and the next command finishes it.
    let mut report_stderr: String = String::new();
    let mut carry_finished: bool = true;
    if plan.carried.is_some() {
        let ending: CarryEnding = carry_and_settle(
            session,
            plan,
            reference.as_str(),
            &intent,
            changes.as_slice(),
            landed,
            transaction_id.as_str(),
        );
        report_stderr.push_str(ending.lines.as_str());
        carry_finished = ending.finished;
        if carry_finished {
            reach_phase(Phase::MergeCarriedBack)?;
        }
    }
    if carry_finished {
        // The record's carry has ended, and that is written beside the record before the capture's ref is decided: a kill
        //  between the two steps then leaves a record whose next settle needs nothing from the objects and drops a ref that is
        //  dropped already or still there (`merge_isolation_journal.rs`, `mark_carry_finished`).
        let finish_recorded: bool = match plan.carried {
            Some(_) => {
                match crate::merge_isolation_journal::mark_carry_finished(record.as_path()) {
                    Ok(()) => true,
                    Err(error) => {
                        debug(TAG, format!("the finish of the merge's carried work could not be recorded: {error}").as_str());
                        false
                    }
                }
            }
            None => true,
        };
        // The record was the last thing that named the capture, so its bytes are in the checkout and the ref goes first: a
        //  deletion another command's lock blocked keeps the record, and the next command's recovery retries it instead of the
        //  name being left for a person to clean up (a cross-vendor audit of alpha.12). A finish that could not be recorded
        //  keeps both, so the next command does both again rather than dropping the name this one could not vouch for.
        let gone: bool = match plan.carried {
            Some(capture) if finish_recorded => drop_capture_ref(
                &session.context,
                git_dir,
                root,
                capture,
                Some(record.as_path()),
            ),
            Some(_) => false,
            None => true,
        };
        if gone {
            if let Err(error) = remove_merge_intent(record.as_path()) {
                debug(
                    TAG,
                    format!("the merge record could not be removed: {error}").as_str(),
                );
            }
        } else {
            report_stderr.push_str(a_refused_cleanup(record.as_path()).as_str());
        }
    }
    // The fast-forward carries no `--quiet`, and what Git says of it ("Updating a..b", "Fast-forward", its diffstat) is about
    //  the wrapper's own step, not about the merge the person asked for. The person's streams carry exactly what native Git's
    //  merge prints (the wrapper's lines below), so what Git wrote to standard output about the step is kept in the journal. What
    //  it wrote to standard error (a warning) still reaches the person, as it did before the option was dropped.
    record_streams(
        &["merge", "--ff-only", "--no-edit", new_oid],
        &moved.stdout,
        &[],
    );
    let mut stdout: String = String::new();
    if !shape.quiet {
        stdout.push_str("Merge made by the 'ort' strategy.\n");
        if shape.stat.unwrap_or(plan.config.stat)
            && let Ok(stat) = session.run(&["diff", "--stat", "--summary", old_oid, new_oid])
        {
            stdout.push_str(String::from_utf8_lossy(&stat.stdout).as_ref());
        }
    }
    let mut stderr: String = String::from_utf8_lossy(&moved.stderr).into_owned();
    stderr.push_str(report_stderr.as_str());
    return Ok(IsolatedMerge::Finished {
        code: 0,
        stdout,
        stderr,
    });
}

/// Apply controls stay out of the release executable.
#[cfg(test)]
#[path = "merge_isolation_apply_tests.rs"]
mod tests;
