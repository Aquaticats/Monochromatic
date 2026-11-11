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
use crate::merge_isolation_carry_apply::{CarryFailure, LockWait, carry, changes_between, clear};
/// What Git's own update left at each path,
/// and the read of it.
use crate::merge_isolation_carry_landed::{LandedFile, landed_of, write_landed_record};
use crate::merge_isolation_carry_notice::report_lines;
/// The words a merge that could not be finished now prints,
/// which its own file holds so this one stays inside the code-line limit.
use crate::merge_isolation_apply_findings::{a_carry_that_stopped, a_deferred_save_finding, a_merge_that_could_not_be_saved};
use crate::merge_isolation_apply_stops::{a_clear_that_met_newer_bytes, settle_a_failed_step};
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
use crate::index_holder_wait::{IndexWait, wait_for_index_release_before_a_merge_that_clears_a_dead_lock};
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
/// A span of time the wait lasted.
use std::time::Duration;
/// The clock reading a side ref's number starts from.
use std::time::SystemTime;

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
            // A capture whose record could not be written names the merge nothing runs, so it goes.
            if let Some(capture) = plan.carried {
                drop_capture_ref(&session.context, git_dir, root, capture);
            }
            return Err(Bail::from(format!("the merge record could not be written: {error}")));
        }
    };
    // The bytes the person's paths hold are objects on a ref of their own now, and the record names them; nothing of the
    //  checkout has changed yet, which is the instant a kill leaves the command exactly as it was.
    reach_phase(Phase::MergeCarried)?;
    reach_phase(Phase::MergeIntent)?;
    // Git's fast-forward takes `index.lock` before it changes anything and dies when another program holds it. The merge waits
    //  for the holder here, bounded and naming it; a lock taken in the instant between is met by the round made again. The
    //  record is written first, so a wait that ends with the lock still held leaves the next command the work of finishing
    //  this very merge.
    let waited: IndexWait = wait_for_index_release_before_a_merge_that_clears_a_dead_lock(git_dir.join("index").as_path(), &mut std::io::stderr());
    // Some when the wait reached its ceiling; nothing when the lock is one this command may not clear at all.
    let mut held_for: Option<Duration> = None;
    let mut still_held: bool = false;
    match waited {
        IndexWait::NotHeld | IndexWait::Released(_) => {}
        IndexWait::GaveUp(waited_for) => {
            held_for = Some(waited_for);
            still_held = true;
        }
        IndexWait::OwnerGone(_) => {
            // A lock a gone process left is removed on the proof the commit service acts on, so a merge repairs what a commit
            //  repairs (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`, finding 3). A lock that proof does
            //  not cover (it holds content, it stands behind a service's marker, a process has it open) stays where it is.
            still_held = !matches!(
                remove_if_the_owner_is_gone(&SYSTEM, git_dir.join("index").as_path(), &scan_platform_holders_bounded),
                Unheld::Removed(_)
            );
        }
    }
    if still_held {
        // The owner's answer for a live foreign `index.lock` (`SPEC.md`, "Merges and commits never fail"): the merge saves its
        //  result on a side ref, exits 0, and the next command finishes the checkout when the lock is free.
        return save_for_a_later_finish(session, plan, &intent, record.as_path(), reference.as_str(), held_for);
    }
    let place: RecoveryPlace = RecoveryPlace::settling(&session.context, git_dir, root);
    let changes: Vec<Change> = if plan.carried.is_some() {
        match changes_between(&session.context, root, old_oid, new_oid) {
            Ok(found) => found,
            Err(reason) => {
                return Err(settle_a_failed_step(&place, record.as_path(), &intent, reference.as_str(), reason.as_str(), false, 2, b""));
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
        let detail: String = failure_text(&failure);
        let lock_taken: bool = matches!(failure, CarryFailure::IndexHeld(_));
        return Err(settle_a_failed_step(&place, record.as_path(), &intent, reference.as_str(), detail.as_str(), lock_taken, 2, b""));
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
        return Err(settle_a_failed_step(
            &place,
            record.as_path(),
            &intent,
            reference.as_str(),
            detail.as_str(),
            lock_taken,
            moved.code.unwrap_or(2),
            moved.stderr.as_slice(),
        ));
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
    if let Some(capture) = plan.carried {
        match carry(&session.context, root, git_dir, changes.as_slice(), capture, landed, transaction_id.as_str(), LockWait::WhileFinishing, &mut std::io::stderr()) {
            Ok(report) => {
                for line in report_lines(reference.as_str(), &report) {
                    report_stderr.push_str(line.as_str());
                    report_stderr.push('\n');
                }
            }
            Err(failure) => {
                carry_finished = false;
                report_stderr.push_str(a_carry_that_stopped(reference.as_str(), &failure, capture).as_str());
            }
        }
        if carry_finished {
            reach_phase(Phase::MergeCarriedBack)?;
        }
    }
    if carry_finished {
        if let Err(error) = remove_merge_intent(record.as_path()) {
            debug(TAG, format!("the merge record could not be removed: {error}").as_str());
        }
        // The record was the last thing that named the capture, so its bytes are in the checkout and the ref can go.
        if let Some(capture) = plan.carried {
            drop_capture_ref(&session.context, git_dir, root, capture);
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
///  How many names are tried when the merge is kept on a side ref.
/// Why:
///   A name a landing already kept is never written over;
/// the next number is tried instead,
/// and a host that
///  took all of them is a refusal rather than a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIDE_REF_ATTEMPTS = 8;
/// ```
const SIDE_REF_ATTEMPTS: u128 = 8;

/// What:
///  Keep the merge commit on `refs/cli-git/kept/<number>`,
/// which only a deletion removes,
/// and return the name it took.
/// Why:
///   The record that names the merge is not enough:
/// `git gc` collects a commit nothing points at,
/// and a
///  deferred merge must still be finishable after the lock frees.
/// The write is an `update-ref` whose old value is all zeros,
/// so a name
///  another command kept is never written over,
/// and the walk to the next number is bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepOnASideRef(session, mergeOid): string | undefined; // update-ref --no-deref refs/cli-git/kept/<ms+n> <oid> 0000...
/// ```
fn keep_on_a_side_ref(session: &Session, merge_oid: &str) -> Option<String> {
    let zeros: String = "0".repeat(merge_oid.len());
    let first: u128 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok()?.as_millis();
    for step in 0..SIDE_REF_ATTEMPTS {
        let name: String = format!("refs/cli-git/kept/{}", first + step);
        let made: GitOutput = session.run(&["update-ref", "--no-deref", name.as_str(), merge_oid, zeros.as_str()]).ok()?;
        if made.succeeded() {
            return Some(name);
        }
    }
    return None;
}

/// What:
///  Save a merge that met an index lock it may not clear:
/// keep its commit on a side ref,
/// record that the checkout is still to be finished,
/// say so,
/// and end
///  with status 0.
/// Why:
///   The owner's answer for a live foreign `index.lock` (`SPEC.md`,
/// "Merges and commits never fail") is that a merge
///  saves its result on a side ref,
/// exits 0,
/// and finishes the checkout when the lock frees.
/// A second record,
///  created once like every other,
/// names the side ref and the merge,
/// so the next command finishes this very merge even though nothing of it is in the
///  checkout yet.
/// A merge whose commit cannot be kept that way is refused instead,
/// with nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function saveForALaterFinish({ session, plan, intent, record, reference, heldFor }): Promise<IsolatedMerge>;
/// ```
fn save_for_a_later_finish(
    session: &Session,
    plan: &Plan<'_>,
    intent: &MergeIntent,
    record: &Path,
    reference: &str,
    held_for: Option<Duration>,
) -> Result<IsolatedMerge, Bail> {
    let lock: PathBuf = plan.git_dir.join("index.lock");
    let Some(side_ref) = keep_on_a_side_ref(session, plan.new_oid) else {
        return Err(refused(2, a_merge_that_could_not_be_saved(reference, lock.as_path())));
    };
    let mut later: MergeIntent = intent.clone();
    later.owner.transaction_id = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(Bail::from(error.0)),
    };
    later.side_ref = Some(side_ref.clone());
    if let Err(error) = write_merge_intent(plan.git_dir, &later) {
        return Err(refused(
            2,
            format!(
                "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({}), and the merge \
                 could not be recorded for a later finish ({error}); its commit is kept on {side_ref}. Nothing was changed. Run the merge \
                 again once that program is done.\n",
                lock.display()
            ),
        ));
    }
    debug(TAG, format!("the merge is saved on {side_ref} beside its first record {}", record.display()).as_str());
    return Ok(IsolatedMerge::Finished { code: 0, stdout: String::new(), stderr: a_deferred_save_finding(reference, lock.as_path(), side_ref.as_str(), held_for) });
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
