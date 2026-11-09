//! What:
//!  The names of the two side references a landing uses,
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await keepThenRemove(scratchRef(seq), keptRef(seq)); // only for a batch that will not land
//! ```

use crate::service_journal::{OPEN_EXTENSION, batch_path};
use crate::service_land_context::LandContext;
use crate::service_protocol::CommitRequest;
use crate::transaction_git::{GitContext, GitRequest, NO_REFLOG_SETTING, run_git};
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

/// What Git said about a reference.
/// Why:
///   A reference that is not there,
/// a reference that names an object and a question Git gave no answer to are three
///  different answers,
/// and removing a reference on the third would be removing it on no answer.
/// The restart that keeps a batch's commit asks the same question about the kept reference,
/// so it never replaces a commit another batch was kept under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefValue = { names: string } | 'absent' | 'unasked'
/// ```
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RefValue {
    /// The reference names this object.
    Names(String),
    /// Git said for certain that the reference is not there.
    Absent,
    /// Git gave no answer.
    Unasked,
}

/// What:
///  The value of a reference,
/// as Git reports it now.
/// Why:
///   `--quiet` makes a missing reference exit 1 with nothing on the standard error;
/// every other failure (a lock,
/// a stuck
///  mount,
/// a store Git cannot open,
/// a reference file it cannot read) is no answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function valueOf(git, repository, name) { const asked = await git(['rev-parse', '--verify', '--quiet', name]); return asked.code === 0 ? { names: asked.out.trim() } : asked.code === 1 && !asked.err ? 'absent' : 'unasked'; }
/// ```
pub(crate) fn value_of(git: &GitContext, repository: &Path, name: &str) -> RefValue {
    let request: GitRequest = GitRequest::new(repository, &["rev-parse", "--verify", "--quiet", name]);
    let Ok(output) = run_git(git, &request) else {
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
    let request: GitRequest = GitRequest::new(repository, arguments);
    return matches!(run_git(git, &request), Ok(output) if output.succeeded());
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
        RefValue::Unasked => {
            left(seq, format!("{kept} cannot be read, so the scratch reference {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Absent => {
            if !write_ref(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", kept.as_str(), tip.as_str()]) {
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
