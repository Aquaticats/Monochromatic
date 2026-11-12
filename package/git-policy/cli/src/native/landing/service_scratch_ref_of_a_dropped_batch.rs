//! What:
//!  The names of the two side references a landing uses
//! and of the reference a removal keeps a held commit on
//!  when one of those cannot be written,
//! and what a landing or a restart does with the scratch reference of a batch that will not land.
//! Why:
//!  A landing keeps the objects of its batch reachable with `refs/worktree/cli-git-service/<seq>`,
//! and the branch write that
//!  moves the branch deletes that reference in the same transaction.
//! A service killed inside the batch never reaches
//!  the deletion,
//! so the reference stays,
//! holding objects no branch names.
//! A batch whose landing ends without recording itself in the journal,
//! and a batch a restart finds open with the branch
//!  unmoved,
//! will never land,
//! so each keeps the commit the reference holds on `refs/cli-git/kept/<seq>`
//!  (where the ceiling of an ended step keeps a commit that was not made) and only then removes the scratch
//!  reference:
//! no commit is lost,
//! and no reference is left that pins objects for ever.
//! Each removal has two faces:
//! the synchronous one a caller on a thread asks,
//! and
//!  the awaited one a task of the runtime asks (`clear_after_dropped_batch_awaiting`,
//! `clear_after_a_failed_landing_awaiting`),
//! which
//!  awaits every Git child on the runtime and reads the batch's journal record as a job of the blocking pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await keepThenRemove(scratchRef(seq), keptRef(seq)); // only for a batch that will not land
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::service_journal::{OPEN_EXTENSION, batch_path};
use crate::service_land_context::LandContext;
use crate::service_protocol::CommitRequest;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, NO_REFLOG_SETTING, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::LANDING_STATE_CEILING;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The exit status `git rev-parse --verify --quiet` gives for a name that is no object:
/// the one answer that means "not there".
const NAME_IS_NO_OBJECT: i32 = 1;

/// What:
///  The per-worktree reference a landing keeps the objects of batch `seq` on until its branch write removes it.
/// Why:
///  The name is read by earlier releases and by the transaction rules that recognize the wrapper's own side-ref
///  writes,
/// so it is spelled in this one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `refs/worktree/cli-git-service/${seq}`
/// ```
pub(crate) fn scratch_ref(seq: u64) -> String {
    return format!("refs/worktree/cli-git-service/{seq}");
}

/// What:
///  The reference a commit that did not land is kept on,
/// named for its batch.
/// Why:
///   A commit this wrapper built and did not put on a branch is kept where its batch number names it
/// (`keep_unlanded`,
/// which tells the person the name).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `refs/cli-git/kept/${seq}`
/// ```
pub(crate) fn kept_ref(seq: u64) -> String {
    return format!("refs/cli-git/kept/{seq}");
}

/// What:
///  The reference that keeps a held commit a removal may destroy
///  when that batch's own kept reference cannot be written.
/// Why:
///   A lock standing on `refs/cli-git/kept/<batch>` refuses every write to that exact name,
///  and the removal that follows deletes the worktree's own Git directory
///  and the per-worktree reference that are the only other holders of the commit:
///  `git gc --prune=now` then takes it,
///  which loses a commit the wrapper said it kept
///  (the owner's rule "never discard user data").
///  A name beside the batch's own is free of that lock,
///  and it stays in the kept family,
/// where anyone who looks for a commit the wrapper kept
///  looks (`worktree_removal_held_commits.rs`).
///  `is_side_ref_name` in `transaction_git_step_rerun_and_end_rules.rs` spells the same suffix,
///  as it already spells these prefixes,
/// because that folder may not use this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `refs/cli-git/kept/${seq}-rescue`
/// ```
pub(crate) fn rescue_ref(seq: u64) -> String {
    return format!("refs/cli-git/kept/{seq}-rescue");
}

/// What:
///  What became of the scratch reference of a batch that will not land.
/// Why:
///   A caller that keeps a record of the reference (`service_scratch_ref_marker_of_a_batch.rs`) may drop that record only
///  when the reference is gone,
/// and a caller that announces the batch says what it left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScratchRefRemoval = 'removed' | 'left'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScratchRefRemoval {
    /// The reference is gone,
    /// or the batch never made one.
    Removed,
    /// The reference is left where it is,
    /// because the commit it holds could not be proven safe elsewhere.
    Left,
}

/// What Git said about a reference,
/// read as the reference's own value.
/// Why:
///   A reference that is not there,
/// a reference that names an object,
/// a reference whose own value is the name of another reference and a question Git gave no answer to are four
///  different answers,
/// and removing a reference on any of the last two would be removing it on no proof.
/// The restart that keeps a batch's commit asks the same question about the kept reference,
/// so it never replaces a commit another batch was kept under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefValue = { names: string } | { symbolic: string } | 'absent' | 'unasked'
/// ```
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RefValue {
    /// The reference names this object.
    Names(String),
    /// The reference is symbolic: its own value is the name of this other reference,
    /// never an object.
    /// Why:
    ///   A reference the wrapper writes is written with `--no-deref`,
    ///  so a symbolic reference at such a name was made by another command,
    ///  and the object it leads to is held by that other reference,
    ///  which its owner can move.
    Symbolic(String),
    /// Git said for certain that the reference is not there.
    Absent,
    /// Git gave no answer.
    Unasked,
}

/// What:
///  Whether `name` is a symbolic reference,
/// and what it leads to when it is.
/// Why:
///  A symbolic reference names another reference, not an object,
///  and every reference the wrapper asks about is one it writes with `--no-deref`:
///  its own value is the only value that says what this command put there,
///  and what the target names can change without this reference moving
///  (the owner's rule "never discard user data":
///  a commit that only the target names is one the command that moves that target takes with it).
///  A read that followed the symbolic reference answered a refused write as written
///  (a review of the side-reference write of the landing).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASymbolicReference = { pointsAt: string } | 'not-symbolic' | 'unasked'
/// ```
enum ASymbolicReference {
    /// The reference is symbolic and leads to this reference.
    PointsAt(String),
    /// The reference is not symbolic, so its own value is an object name.
    NotSymbolic,
    /// Git gave no answer.
    Unasked,
}

/// What:
///  The question that says whether `name` is a symbolic reference.
/// Why:
///  `symbolic-ref` reads the reference itself,
/// so it answers with the reference's own value where `rev-parse` answers with the object its target names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['symbolic-ref', '--quiet', name] };
/// ```
fn symbolic_request(repository: &Path, name: &str) -> GitRequest {
    return GitRequest::new(repository, &["symbolic-ref", "--quiet", name]);
}

/// What:
///  What Git's answer to `symbolic_request` says about the reference.
/// Why:
///   `--quiet` makes a name that is not a symbolic reference exit 1 with nothing on either stream,
///  whether it is a direct reference or no reference at all;
/// every other answer (a lock,
/// a broken reference file,
/// a store Git cannot open) is no answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function symbolicAnsweredBy(answered: GitOutput | Error): ASymbolicReference;
/// ```
fn symbolic_answered_by(answered: std::io::Result<GitOutput>) -> ASymbolicReference {
    let Ok(output) = answered else {
        return ASymbolicReference::Unasked;
    };
    if output.succeeded() {
        return ASymbolicReference::PointsAt(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
    }
    return if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() { ASymbolicReference::NotSymbolic } else { ASymbolicReference::Unasked };
}

fn symbolic_reference(git: &GitContext, repository: &Path, name: &str) -> ASymbolicReference {
    return symbolic_answered_by(run_git(git, &symbolic_request(repository, name)));
}

/// What:
///  `symbolic_reference` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function symbolicReferenceAwaiting(git, repository, name): Promise<ASymbolicReference>;
/// ```
async fn symbolic_reference_awaiting(git: &GitContext, repository: &Path, name: &str) -> ASymbolicReference {
    return symbolic_answered_by(run_git_awaiting(git, &symbolic_request(repository, name)).await);
}

/// What:
///  The value of a reference,
/// its own value,
/// as Git reports it now.
/// Why:
///   `--quiet` makes a missing reference exit 1 with nothing on the standard error;
/// every other failure (a lock,
/// a stuck
///  mount,
/// a store Git cannot open,
/// a reference file it cannot read) is no answer.
///  A symbolic reference is asked about first,
/// because `rev-parse` follows one and answers what its target names
/// (`ASymbolicReference`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function valueOf(git, repository, name) { const symbolic = await git(['symbolic-ref', '--quiet', name]); if (symbolic.code === 0) return { symbolic: symbolic.out.trim() }; if (symbolic.code !== 1 || symbolic.err) return 'unasked'; const asked = await git(['rev-parse', '--verify', '--quiet', name]); return asked.code === 0 ? { names: asked.out.trim() } : asked.code === 1 && !asked.err ? 'absent' : 'unasked'; }
/// ```
pub(crate) fn value_of(git: &GitContext, repository: &Path, name: &str) -> RefValue {
    return match symbolic_reference(git, repository, name) {
        ASymbolicReference::PointsAt(target) => RefValue::Symbolic(target),
        ASymbolicReference::Unasked => RefValue::Unasked,
        ASymbolicReference::NotSymbolic => value_answered_by(run_git(git, &value_request(repository, name))),
    };
}

/// `value_of` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function valueOfAwaiting(git, repository, name): Promise<RefValue>;
/// ```
pub(crate) async fn value_of_awaiting(git: &GitContext, repository: &Path, name: &str) -> RefValue {
    return match symbolic_reference_awaiting(git, repository, name).await {
        ASymbolicReference::PointsAt(target) => RefValue::Symbolic(target),
        ASymbolicReference::Unasked => RefValue::Unasked,
        ASymbolicReference::NotSymbolic => value_answered_by(run_git_awaiting(git, &value_request(repository, name)).await),
    };
}

/// What:
///  The question `value_of` asks Git about `name`.
/// Why:
///  Both faces ask the one question,
/// so neither can read another reference than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['rev-parse', '--verify', '--quiet', name] };
/// ```
fn value_request(repository: &Path, name: &str) -> GitRequest {
    return GitRequest::new(repository, &["rev-parse", "--verify", "--quiet", name]);
}

/// What:
///  What Git's answer to `value_request` says about the reference.
/// Why:
///  Both faces read Git's answer through this one function,
/// so a reference that is not there,
/// a reference that names an
///  object and a question Git gave no answer to stay three different answers for both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function valueAnsweredBy(answered: GitOutput | Error): RefValue;
/// ```
fn value_answered_by(answered: std::io::Result<GitOutput>) -> RefValue {
    let Ok(output) = answered else {
        return RefValue::Unasked;
    };
    if output.succeeded() {
        return RefValue::Names(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
    }
    return if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() { RefValue::Absent } else { RefValue::Unasked };
}

/// What:
///  Runs one `update-ref` of a side reference,
/// saying whether Git took it.
/// Why:
///  Both writes of this file are the wrapper's own side references:
/// no reflog is written for them and no
///  symbolic reference is followed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['-c', 'core.logAllRefUpdates=false', 'update-ref', ...arguments])
/// ```
fn write_ref(git: &GitContext, repository: &Path, arguments: &[&str]) -> bool {
    return git_took_it(run_git(git, &GitRequest::new(repository, arguments)));
}

/// `write_ref` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeRefAwaiting(git, repository, arguments): Promise<boolean>;
/// ```
async fn write_ref_awaiting(git: &GitContext, repository: &Path, arguments: &[&str]) -> bool {
    return git_took_it(run_git_awaiting(git, &GitRequest::new(repository, arguments)).await);
}

/// What:
///  Whether Git took a side-reference write.
/// Why:
///  Both faces read the answer through this one function,
/// so a write that could not be started and a write Git refused are the
///  same answer for both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitTookIt(answered: GitOutput | Error): boolean;
/// ```
fn git_took_it(answered: std::io::Result<GitOutput>) -> bool {
    return matches!(answered, Ok(output) if output.succeeded());
}

/// What:
///  Says why the scratch reference of a batch is left where it is,
/// in the service's log.
/// Why:
///  A reference that is kept must be found by the person who looks for the commit it holds,
/// and a reference
///  that could not be removed must not go unmentioned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.warn(`batch ${seq}: ${reason}`)
/// ```
fn left(seq: u64, reason: &str) {
    crate::service_warn!("batch {seq}: {reason}");
}

/// What:
///  Removes the scratch reference of a batch that will not land,
/// after the commit it holds is on the batch's
///  kept reference.
/// Why:
///   The caller has decided that the batch never landed and never will (the branch is neither at the batch's commit
///  nor below it,
/// or the intent that would let a restart land it was not written),
/// so the only thing the scratch reference still does is keep the batch's objects alive.
/// Its commit is
///  kept first,
/// because a reference that is the only holder of a commit is not removed:
/// a keep Git refused,
/// a kept
///  reference that cannot be read,
/// and a kept reference that already names another commit all leave the scratch reference
///  where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tip = await valueOf(scratch); if (tip === 'absent') return 'removed'; if (tip === 'unasked') return 'left'; if (!await keep(tip)) return 'left'; return await git(['update-ref', '-d', '--no-deref', scratch]) ? 'removed' : 'left'
/// ```
pub(crate) fn clear_after_dropped_batch(git: &GitContext, repository: &Path, seq: u64) -> ScratchRefRemoval {
    let scratch: String = scratch_ref(seq);
    let kept: String = kept_ref(seq);
    let tip: String = match value_of(git, repository, scratch.as_str()) {
        RefValue::Names(found) => found,
        // No scratch reference at all: the landing made no objects, or its branch write removed it.
        RefValue::Absent => return ScratchRefRemoval::Removed,
        // A symbolic reference is not one of the wrapper's writes, so what it holds is not proven to be the batch's commit: nothing
        //  is removed on it (`RefValue::Symbolic`).
        RefValue::Symbolic(target) => {
            left(seq, format!("{scratch} is a symbolic reference to {target}, not a reference of the wrapper's own, so nothing is removed").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(seq, format!("{scratch} cannot be read, so nothing is removed").as_str());
            return ScratchRefRemoval::Left;
        }
    };
    let already: RefValue = value_of(git, repository, kept.as_str());
    match &already {
        RefValue::Names(named) if *named == tip => {}
        RefValue::Names(named) => {
            left(seq, format!("{kept} names {named} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        // The commit is held by the reference this one leads to, which its owner can move: the batch's commit is not proven to be
        //  safe elsewhere, so the scratch reference stays.
        RefValue::Symbolic(target) => {
            left(seq, format!("{kept} is a symbolic reference to {target} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(seq, format!("{kept} cannot be read, so the scratch reference {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Absent => {
            // The empty old value asks Git for a write of a name that names nothing: a reference another command writes between this
            //  read and the write is never replaced (`keep_on_a_side_ref`, the owner's rule "never discard user data").
            if !write_ref(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", kept.as_str(), tip.as_str(), ""]) {
                left(
                    seq,
                    format!("the commit {tip} of a batch that did not land could not be kept at {kept}, so its scratch reference {scratch} is left as it is").as_str(),
                );
                return ScratchRefRemoval::Left;
            }
            crate::service_log!("batch {}: the commit {tip} of a batch that did not land is kept at {}", seq, kept);
        }
    }
    if write_ref(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch.as_str()]) {
        crate::service_log!("batch {}: the scratch reference {scratch} of a batch that did not land is removed", seq);
        return ScratchRefRemoval::Removed;
    }
    left(seq, format!("the scratch reference {scratch} could not be removed; its commit {tip} is kept at {kept}").as_str());
    return ScratchRefRemoval::Left;
}

/// What:
///  `clear_after_dropped_batch` with every Git step of it awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and the removal is a step of the
///  landing once that landing is a task.
/// The order of the steps,
/// the three answers of a reference that is not there,
/// names another commit or gives no answer,
/// and the words each of them says are the synchronous face's,
/// pinned by
///  `the_awaited_clearing_answers_what_the_synchronous_one_answers`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = await clearAfterDroppedBatchAwaiting(git, repository, seq)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn clear_after_dropped_batch_awaiting(git: &GitContext, repository: &Path, seq: u64) -> ScratchRefRemoval {
    let scratch: String = scratch_ref(seq);
    let kept: String = kept_ref(seq);
    let tip: String = match value_of_awaiting(git, repository, scratch.as_str()).await {
        RefValue::Names(found) => found,
        // No scratch reference at all: the landing made no objects, or its branch write removed it.
        RefValue::Absent => return ScratchRefRemoval::Removed,
        // A symbolic reference is not one of the wrapper's writes, so what it holds is not proven to be the batch's commit: nothing
        //  is removed on it (`RefValue::Symbolic`).
        RefValue::Symbolic(target) => {
            left(seq, format!("{scratch} is a symbolic reference to {target}, not a reference of the wrapper's own, so nothing is removed").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(seq, format!("{scratch} cannot be read, so nothing is removed").as_str());
            return ScratchRefRemoval::Left;
        }
    };
    let already: RefValue = value_of_awaiting(git, repository, kept.as_str()).await;
    match &already {
        RefValue::Names(named) if *named == tip => {}
        RefValue::Names(named) => {
            left(seq, format!("{kept} names {named} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        // The commit is held by the reference this one leads to, which its owner can move: the batch's commit is not proven to be
        //  safe elsewhere, so the scratch reference stays.
        RefValue::Symbolic(target) => {
            left(seq, format!("{kept} is a symbolic reference to {target} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(seq, format!("{kept} cannot be read, so the scratch reference {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Absent => {
            // The empty old value asks Git for a write of a name that names nothing: a reference another command writes between this
            //  read and the write is never replaced (`keep_on_a_side_ref`, the owner's rule "never discard user data").
            if !write_ref_awaiting(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", kept.as_str(), tip.as_str(), ""]).await {
                left(
                    seq,
                    format!("the commit {tip} of a batch that did not land could not be kept at {kept}, so its scratch reference {scratch} is left as it is").as_str(),
                );
                return ScratchRefRemoval::Left;
            }
            crate::service_log!("batch {}: the commit {tip} of a batch that did not land is kept at {}", seq, kept);
        }
    }
    if write_ref_awaiting(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch.as_str()]).await {
        crate::service_log!("batch {}: the scratch reference {scratch} of a batch that did not land is removed", seq);
        return ScratchRefRemoval::Removed;
    }
    left(seq, format!("the scratch reference {scratch} could not be removed; its commit {tip} is kept at {kept}").as_str());
    return ScratchRefRemoval::Left;
}

/// What:
///  Clears the scratch reference of the batch a failed landing left,
/// when no open intent names the batch.
/// Why:
///   A landing that ends before its intent reaches the journal leaves the batch's objects on the scratch reference and
///  nothing that names them,
/// so no restart can land the batch and nothing else would remove the reference.
/// A failed landing that did write its intent leaves its batch to the restart,
/// which settles it from the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = requests[0]; if (!first || openIntent(journal, seq)) return 'removed'; return await clearAfterDroppedBatch(git, first.repository, seq)
/// ```
pub(crate) fn clear_after_a_failed_landing(context: &LandContext, requests: &[CommitRequest], seq: u64) -> ScratchRefRemoval {
    // No request is no import, so the batch made no reference at all.
    let Some(first) = requests.first() else { return ScratchRefRemoval::Removed };
    // An open intent names the batch's own worktree, and the restart that settles it removes the reference then.
    if batch_path(context.journal.as_path(), seq, OPEN_EXTENSION).exists() {
        return ScratchRefRemoval::Removed;
    }
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    return clear_after_dropped_batch(&git, first.repository.as_path(), seq);
}

/// The one path one open-intent question names,
/// owned so that a job of the blocking pool can take it.
struct OpenIntentQuestion {
    /// The journal record of the batch's intent.
    path: PathBuf,
}

/// The look one job of the pool does:
/// whether the journal holds an open intent for the batch.
fn an_intent_is_open(question: OpenIntentQuestion) -> bool {
    return question.path.exists();
}

/// What:
///  Whether the journal holds an open intent for batch `seq`,
/// as a job of the blocking pool awaited no longer than
///  `LANDING_STATE_CEILING`:
/// `Some(true)` or `Some(false)` when the look answered,
/// and nothing when it gave no answer.
/// Why:
///   The look is a `stat` of a file in the instance's journal,
/// which waits in the kernel on a mount that stopped answering,
/// and a
///  task of the runtime may not wait there.
/// A look that gave nothing in time is told apart from "no intent",
/// because
///  "no intent" sets the removal of the reference in motion and a look nobody answered must set nothing in motion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const open = await anOpenIntentAwaiting(journal, seq); // null when the look gave no answer
/// ```
async fn an_open_intent_awaiting(journal: &Path, seq: u64) -> Option<bool> {
    return an_open_intent_within(LANDING_STATE_CEILING, journal, seq).await;
}

/// `an_open_intent_awaiting` with the wait `ceiling` grants.
async fn an_open_intent_within(ceiling: Duration, journal: &Path, seq: u64) -> Option<bool> {
    let question: OpenIntentQuestion = OpenIntentQuestion { path: batch_path(journal, seq, OPEN_EXTENSION) };
    return match run_a_job_awaited_no_longer_than(ceiling, question, an_intent_is_open).await {
        TheAnswerOfAJobAwaited::Ran(open) => Some(open),
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}

/// What:
///  `clear_after_a_failed_landing` with the look at the intent a job of the pool and every Git step of it awaited
///  on the runtime.
/// Why:
///   A task of the runtime may wait on neither a file nor a child it starts itself,
/// so a landing that becomes a task
///  does this through this face.
/// A look that gave no answer leaves the reference where it is and says why,
/// so
///  no commit is lost and nothing claims to have removed what it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = await clearAfterAFailedLandingAwaiting(context, requests, seq)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn clear_after_a_failed_landing_awaiting(context: &LandContext, requests: &[CommitRequest], seq: u64) -> ScratchRefRemoval {
    // No request is no import, so the batch made no reference at all.
    let Some(first) = requests.first() else { return ScratchRefRemoval::Removed };
    // An open intent names the batch's own worktree, and the restart that settles it removes the reference then.
    match an_open_intent_awaiting(context.journal.as_path(), seq).await {
        Some(true) => return ScratchRefRemoval::Removed,
        Some(false) => {}
        None => {
            left(seq, "the record of this batch's intent cannot be read, so its scratch reference is left as it is and a restart settles the batch");
            return ScratchRefRemoval::Left;
        }
    }
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    return clear_after_dropped_batch_awaiting(&git, first.repository.as_path(), seq).await;
}

/// Controls for the awaited faces of these removals stay out of the release executable.
#[cfg(test)]
#[path = "service_scratch_ref_awaiting_tests.rs"]
mod awaiting_tests;
