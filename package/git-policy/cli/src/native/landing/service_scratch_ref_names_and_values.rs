//! What:
//!  Everything about one side reference that is not the decision to remove it:
//! the names a landing writes,
//! how a reference's value is read
//! (its own object,
//! the reference a symbolic one leads to,
//! that it is not there,
//! and that Git gave no answer),
//! the one write of a side reference,
//! the line that says a reference was left where it is,
//! and what became of a reference a removal was asked about.
//! Why:
//!  The two faces of that removal
//! (`service_scratch_ref_of_a_dropped_batch.rs` and
//! `service_scratch_ref_of_a_dropped_batch_awaiting.rs`)
//! ask and write through these one each,
//! so neither can read another reference than the other,
//! and they sit in a file that decides nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const value = valueOf(git, repository, scratchRef(seq)); // {names} | {symbolic} | 'absent' | 'unasked'
//! ```

use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::path::Path;

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
    /// The reference is symbolic:
    /// its own value is the name of this other reference,
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
///  A symbolic reference names another reference,
/// not an object,
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
    /// The reference is not symbolic,
    /// so its own value is an object name.
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
fn symbolic_request(
    repository: &Path,
    name: &str,
) -> GitRequest {
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
        return ASymbolicReference::PointsAt(
            String::from_utf8_lossy(output.stdout.as_slice())
                .trim()
                .to_string(),
        );
    }
    return if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() {
        ASymbolicReference::NotSymbolic
    } else {
        ASymbolicReference::Unasked
    };
}

fn symbolic_reference(
    git: &GitContext,
    repository: &Path,
    name: &str,
) -> ASymbolicReference {
    return symbolic_answered_by(run_git(git, &symbolic_request(repository, name)));
}

/// What:
///  `symbolic_reference` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function symbolicReferenceAwaiting(git, repository, name): Promise<ASymbolicReference>;
/// ```
async fn symbolic_reference_awaiting(
    git: &GitContext,
    repository: &Path,
    name: &str,
) -> ASymbolicReference {
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
pub(crate) fn value_of(
    git: &GitContext,
    repository: &Path,
    name: &str,
) -> RefValue {
    return match symbolic_reference(git, repository, name) {
        ASymbolicReference::PointsAt(target) => RefValue::Symbolic(target),
        ASymbolicReference::Unasked => RefValue::Unasked,
        ASymbolicReference::NotSymbolic => {
            value_answered_by(run_git(git, &value_request(repository, name)))
        }
    };
}

/// `value_of` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function valueOfAwaiting(git, repository, name): Promise<RefValue>;
/// ```
pub(crate) async fn value_of_awaiting(
    git: &GitContext,
    repository: &Path,
    name: &str,
) -> RefValue {
    return match symbolic_reference_awaiting(git, repository, name).await {
        ASymbolicReference::PointsAt(target) => RefValue::Symbolic(target),
        ASymbolicReference::Unasked => RefValue::Unasked,
        ASymbolicReference::NotSymbolic => {
            value_answered_by(run_git_awaiting(git, &value_request(repository, name)).await)
        }
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
fn value_request(
    repository: &Path,
    name: &str,
) -> GitRequest {
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
        return RefValue::Names(
            String::from_utf8_lossy(output.stdout.as_slice())
                .trim()
                .to_string(),
        );
    }
    return if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() {
        RefValue::Absent
    } else {
        RefValue::Unasked
    };
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
pub(crate) fn write_ref(
    git: &GitContext,
    repository: &Path,
    arguments: &[&str],
) -> bool {
    return git_took_it(run_git(git, &GitRequest::new(repository, arguments)));
}

/// `write_ref` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeRefAwaiting(git, repository, arguments): Promise<boolean>;
/// ```
pub(crate) async fn write_ref_awaiting(
    git: &GitContext,
    repository: &Path,
    arguments: &[&str],
) -> bool {
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
/// console.warn(`batch ${seq} in the repository at ${repo}: ${reason}`)
/// ```
pub(crate) fn left(
    seq: u64,
    repository: &Path,
    reason: &str,
) {
    crate::service_warn!(
        "batch {seq} {}: {reason}",
        crate::service_timing::in_the_repository_at(repository)
    );
}
