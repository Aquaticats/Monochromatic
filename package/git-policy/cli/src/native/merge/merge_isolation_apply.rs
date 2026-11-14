//! What:
//!  The writing half of an isolated merge:
//! the merge tree,
//! the merge commit,
//! and the two-way update that applies it to the checkout.
//! Why:
//!  `merge_isolation.rs` decides and loops over rounds;
//! the steps that write objects and move the checkout are a file of their own that it imports and that imports nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await apply(input, session, plan); // commit-tree, then merge --ff-only
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// Where Git's words about the wrapper's own fast-forward go now that the run carries no `--quiet`.
use crate::inner_git_words::record_streams;
/// The files the word pass merged,
/// and the lines that list them.
use crate::merge_second_pass::Merged;
use crate::journald_native::CLIENT_IDENTIFIER;
use crate::merge_second_pass_tree::announce;
/// The intent record.
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent, write_merge_intent};
/// Recovery of records.
use crate::merge_isolation_recovery::RecoveryPlace;
/// The refusal findings.
use crate::merge_isolation_apply_findings::failure_text;
/// The person's own work in the paths the merge changes:
/// the clear before Git's update,
/// the carry after it,
/// and the ref that keeps their bytes.
use crate::merge_isolation_carry::drop_capture_ref;
use crate::merge_isolation_carry_apply::{CarryFailure, LockWait, changes_between, clear};
/// What Git's own update left at each path,
/// and the read of it.
use crate::merge_isolation_carry_landed::{LandedFile, landed_of, write_landed_record};
/// The words a merge that could not be finished now prints,
/// which its own file holds so this one stays inside the code-line limit.
use crate::merge_isolation_apply_findings::{WhyTheMergeIsSaved, a_refused_cleanup};
/// The save of a merge that met the index lock for a later command,
/// kept in a file of its own so this one stays inside the code-line limit.
use crate::merge_isolation_apply_save::save_for_a_later_finish;
use crate::merge_isolation_apply_stops::{a_clear_that_met_newer_bytes, the_end_of_a_failed_step};
/// The carry of a merge that landed,
/// its lines,
/// and what a branch that moved under it does about them.
use crate::merge_isolation_apply_moved::{CarryEnding, carry_and_settle};
/// The raw differences between the two commits of the merge.
use crate::merge_isolation_changes::Change;
/// The command-line reading.
use crate::merge_options::{MergeShape, Signing};
/// Marker phases for the crash controls.
use crate::transaction_phase::{Phase, reach_transaction_phase};
/// Real Git.
use crate::transaction_git::{GitContext, GitOutput};
/// The owner of the intent record,
/// and its identifier.
use crate::transaction_owner::current_transaction_owner;
/// Unique identifiers and the clock.
use crate::iso_time::now_iso;
use crate::random_id::random_uuid;
/// Waiting for any program's hold on the index lock before the checkout changes,
/// and what the wait proved.
use crate::index_holder_wait::{IndexWait, look_at_the_index_before_a_merge};
/// Removing an index lock a gone process left,
/// on the commit service's proof of disuse.
use crate::service_index_unheld_removal::{Unheld, remove_if_the_owner_is_gone};
/// Scanning every readable process for a descriptor on that lock.
use crate::index_lock_holders::scan_platform_holders_bounded;
/// The platform whose unlink removes the lock.
use crate::service_index_placement::SYSTEM;
/// Owned operating-system text.
use crate::merge_isolation_git_reads::Session;
use crate::merge_isolation_run_types::{Bail, IsolatedMerge, IsolatedMergeInput, MergeConfig, Plan, TAG, refused};
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The finished merge with the lines that list the files the second pass merged by word,
/// ahead of Git's own words.
/// Why:
///   The journal entries and the lines are produced only now,
/// after the merge is in the checkout,
/// so neither says a merge
///  happened that was refused or undone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withSecondPassReport(done: IsolatedMerge, merged: Merged[]): IsolatedMerge;
/// ```
pub fn with_second_pass_report(done: IsolatedMerge, merged: &[Merged]) -> IsolatedMerge {
    match done {
        IsolatedMerge::Finished { code: 0, stdout, stderr } if !merged.is_empty() => {
            let mut report: String = String::new();
            for line in announce("monochromatic-git-policy-cli: ", (CLIENT_IDENTIFIER, "merge"), merged) {
                report.push_str(line.as_str());
                report.push('\n');
            }
            return IsolatedMerge::Finished { code: 0, stdout, stderr: report + stderr.as_str() };
        }
        other => return other,
    }
}

/// The merge tree of an isolated merge and the pass over its conflicts,
/// which moved to a file of their own and keep their names here.
pub use crate::merge_isolation_merge_tree::{merge_tree, merge_tree_awaiting};

/// What:
///  Create the merge commit with its parents in Git's order,
/// signing it as the options and settings ask.
/// Why:
///   `commit-tree` signs only when given `-S` and never reads `commit.gpgSign`;
/// a merge on a signed branch must not
///  be the one unsigned commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ session, shape, config, parents, tree, message }): Promise<string>;
/// ```
pub fn commit_merge(
    session: &Session,
    shape: &MergeShape,
    config: &MergeConfig,
    ends: (&str, &str, &str),
    message: Vec<u8>,
) -> Result<String, String> {
    let (head, target, tree): (&str, &str, &str) = ends;
    let signing: Option<String> = match &shape.signing {
        Signing::Never => None,
        Signing::Sign(None) => Some(String::from("-S")),
        Signing::Sign(Some(key)) => Some(format!("-S{}", String::from_utf8_lossy(key))),
        Signing::Default => if config.sign_commits { Some(String::from("-S")) } else { None },
    };
    let mut arguments: Vec<&str> = vec!["commit-tree", tree, "-p", head, "-p", target];
    if let Some(flag) = signing.as_deref() {
        arguments.push(flag);
    }
    let created: GitOutput = session.run_with_input(arguments.as_slice(), message)?;
    if !created.succeeded() {
        return Err(created.error_text());
    }
    return Ok(String::from_utf8_lossy(&created.stdout).trim().to_owned());
}

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
pub fn apply(input: &IsolatedMergeInput<'_>, session: &Session, plan: &Plan<'_>) -> Result<IsolatedMerge, Bail> {
    let (git_dir, root): (&Path, &Path) = (plan.git_dir, plan.root);
    let (old_oid, new_oid): (&str, &str) = (plan.old_oid, plan.new_oid);
    let shape: &MergeShape = plan.shape;
    let reference: String = shape.reference.to_string_lossy().into_owned();
    let target_ref: Option<String> = {
        let named: GitOutput = session.run(&["symbolic-ref", "--quiet", "HEAD"])?;
        if named.succeeded() { Some(String::from_utf8_lossy(&named.stdout).trim().to_owned()) } else { None }
    };
    let transaction_id: String = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(Bail::from(error.0)),
    };
    let owner = match current_transaction_owner(transaction_id.as_str(), now_iso().as_str()) {
        Ok(found) => found,
        Err(error) => return Err(Bail::from(error.to_string())),
    };
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: root.to_path_buf(),
        target_ref,
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
        side_ref: None,
        carried: plan.carried.cloned(),
    };
    let record: PathBuf = match write_merge_intent(git_dir, &intent) {
        Ok(written) => written,
        Err(error) => {
            // A capture whose record could not be written names nothing a later command would settle, so it goes; a capture whose
            //  ref stays names itself in the refusal, because nothing else does.
            let kept: String = match plan.carried {
                Some(capture) if !drop_capture_ref(&session.context, git_dir, root, capture, None) => match capture.reference.as_deref() {
                    Some(kept) => format!(", and the keep of your edits is still on {kept}, which nothing removes: `git update-ref -d {kept}` deletes it"),
                    None => String::new(),
                },
                _ => String::new(),
            };
            return Err(Bail::from(format!("the merge record could not be written: {error}{kept}")));
        }
    };
    // The bytes the person's paths hold are objects on a ref of their own now, and the record names them; nothing of the
    //  checkout has changed yet, which is the instant a kill leaves the command exactly as it was.
    reach_phase(Phase::MergeCarried)?;
    reach_phase(Phase::MergeIntent)?;
    // Git's fast-forward takes `index.lock` before it changes anything and dies when another program holds it. The look names a
    //  dead owner whose lock this command may clear, and a live one it does not wait for: waiting on another agent's command is
    //  what the owner forbids, and the save below is the answer that needs no wait. The record is written first, so a merge that
    //  saves itself leaves the next command the work of finishing this very merge.
    let looked: IndexWait = look_at_the_index_before_a_merge(git_dir.join("index").as_path(), &mut std::io::stderr());
    let mut why: Option<WhyTheMergeIsSaved> = None;
    match looked {
        IndexWait::NotHeld | IndexWait::Released(_) => {}
        IndexWait::GaveUp(waited_for) => why = Some(WhyTheMergeIsSaved::TheWaitEnded(waited_for)),
        // The look never waits, so a holder it met is one this merge leaves for the next command rather than wait for.
        IndexWait::HeldByAnotherCommand(holder) => why = Some(WhyTheMergeIsSaved::AnotherProgramHoldsIt(holder)),
        IndexWait::OwnerGone(_) => {
            // A lock a gone process left is removed on the proof the commit service acts on, so a merge repairs what a commit
            //  repairs (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`, finding 3). A lock that proof does
            //  not cover (it holds content, it stands behind a service's marker, a process has it open) stays where it is.
            if !matches!(
                remove_if_the_owner_is_gone(&SYSTEM, git_dir.join("index").as_path(), &scan_platform_holders_bounded),
                Unheld::Removed(_)
            ) {
                why = Some(WhyTheMergeIsSaved::NotTheCommandsToClear);
            }
        }
    }
    if let Some(reason) = why {
        // The owner's answer for a live foreign `index.lock` (`SPEC.md`, "Merges and commits never fail"): the merge saves its
        //  result on a side ref, exits 0, and the next command finishes the checkout when the lock is free.
        // Nothing of the clear has run at this instant, so the files hold the person's own bytes where they were.
        return save_for_a_later_finish(session, plan, &intent, record.as_path(), reference.as_str(), reason, false);
    }
    let place: RecoveryPlace = RecoveryPlace::settling(&session.context, git_dir, root);
    let changes: Vec<Change> = if plan.carried.is_some() {
        match changes_between(&session.context, root, old_oid, new_oid) {
            Ok(found) => found,
            Err(reason) => {
                return the_end_of_a_failed_step(session, plan, &intent, record.as_path(), reference.as_str(), reason.as_str(), false, false, 2, b"");
            }
        }
    } else {
        Vec::new()
    };
    // Git's two-way update refuses a path that holds anything but the branch's content, so the person's paths are put back to
    //  the branch before it runs. Their bytes are the objects the record names, so nothing of theirs is lost by it; a path written
    //  after the capture stops the clear with nothing written, and the round is made again on the newer bytes.
    if let Some(capture) = plan.carried
        && let Err(failure) = clear(&session.context, root, git_dir, changes.as_slice(), capture, transaction_id.as_str(), LockWait::BeforeTheApply, &mut std::io::stderr())
    {
        // A path holding bytes newer than the capture is the person's, so the clear wrote nothing to it, the capture goes back
        //  where it can, and the merge is made again on what the file holds now instead of being refused for a save.
        if matches!(failure, CarryFailure::Foreign(_) | CarryFailure::Changed(_)) {
            return Err(a_clear_that_met_newer_bytes(&place, record.as_path(), &intent, reference.as_str(), &failure));
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
        return the_end_of_a_failed_step(session, plan, &intent, record.as_path(), reference.as_str(), detail.as_str(), false, false, 2, b"");
    }
    if plan.carried.is_some() {
        reach_phase(Phase::MergeCleared)?;
    }
    let mut overlay: Vec<(OsString, OsString)> = input.overlay.to_vec();
    overlay.push((OsString::from("GIT_REFLOG_ACTION"), OsString::from(format!("merge {reference}"))));
    let mover: Session = Session {
        context: GitContext { overlay, ..session.context.clone() },
    };
    // A start failure is handled like a refusal: the record settles whatever Git began.
    let moved: GitOutput = match mover.run(&["merge", "--ff-only", "--no-edit", new_oid]) {
        Ok(output) => output,
        Err(reason) => GitOutput { code: None, stdout: Vec::new(), stderr: reason.into_bytes() },
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
        let ending: CarryEnding = carry_and_settle(session, plan, reference.as_str(), &intent, changes.as_slice(), landed, transaction_id.as_str());
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
            Some(_) => match crate::merge_isolation_journal::mark_carry_finished(record.as_path()) {
                Ok(()) => true,
                Err(error) => {
                    debug(TAG, format!("the finish of the merge's carried work could not be recorded: {error}").as_str());
                    false
                }
            },
            None => true,
        };
        // The record was the last thing that named the capture, so its bytes are in the checkout and the ref goes first: a
        //  deletion another command's lock blocked keeps the record, and the next command's recovery retries it instead of the
        //  name being left for a person to clean up (a cross-vendor audit of alpha.12). A finish that could not be recorded
        //  keeps both, so the next command does both again rather than dropping the name this one could not vouch for.
        let gone: bool = match plan.carried {
            Some(capture) if finish_recorded => drop_capture_ref(&session.context, git_dir, root, capture, Some(record.as_path())),
            Some(_) => false,
            None => true,
        };
        if gone {
            if let Err(error) = remove_merge_intent(record.as_path()) {
                debug(TAG, format!("the merge record could not be removed: {error}").as_str());
            }
        } else {
            report_stderr.push_str(a_refused_cleanup(record.as_path()).as_str());
        }
    }
    // The fast-forward carries no `--quiet`, and what Git says of it ("Updating a..b", "Fast-forward", its diffstat) is about
    //  the wrapper's own step, not about the merge the person asked for. The person's streams carry exactly what native Git's
    //  merge prints (the wrapper's lines below), so what Git wrote to standard output about the step is kept in the journal. What
    //  it wrote to standard error (a warning) still reaches the person, as it did before the option was dropped.
    record_streams(&["merge", "--ff-only", "--no-edit", new_oid], &moved.stdout, &[]);
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

/// What:
///  Reach a marker phase,
/// failing the invocation when the marker variable is malformed.
/// Why:
///   The crash controls arm a phase by environment;
/// a malformed value must not be ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase });
/// ```
pub fn reach_phase(phase: Phase) -> Result<(), Bail> {
    return match reach_transaction_phase(phase, None) {
        Ok(()) => Ok(()),
        Err(error) => Err(refused(2, format!("monochromatic-git-policy-cli: {error}\n"))),
    };
}

/// Apply controls stay out of the release executable.
#[cfg(test)]
#[path = "merge_isolation_apply_tests.rs"]
mod tests;
