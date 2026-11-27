//! What:
//!  The writes that keep a commit on a reference of the wrapper's own:
//!  the reference a commit that did not land is kept on (`refs/cli-git/kept/<number>`),
//!  the shared reference of a batch (`refs/worktree/cli-git-service/<seq>`),
//!  and a named side reference a caller hands in.
//! Why:
//!   One place names each of those references and writes it,
//!  so the side ref of a landing that gave up at a step and the side ref a restart writes for a batch its branch does
//!  not show cannot drift apart;
//!  every one of them is a create,
//!  so a name another command wrote is never replaced,
//!  and every one of them is written on both faces,
//!  the synchronous one and the one a task of the runtime awaits,
//!  from the same attempts and the same read of the reference.
//!  They sit in a file of their own because the landing's side reference file was over the limit the repository linter
//!  enforces (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(await keepOnASideRef(git, repo, name, tip)).toBeUndefined()
//! ```

use crate::service_scratch_ref_of_a_dropped_batch::kept_ref;
use crate::the_write_of_a_side_reference::{
    ASideRefWrite,
    write_until_the_reference_names,
    write_until_the_reference_names_awaiting,
};
use crate::transaction_git::GitContext;
use std::path::Path;

/// What:
///  Keeps the commit of batch `seq` on `refs/cli-git/kept/<number>`,
/// the ref the wrapper keeps a commit it did not land on,
/// and returns nothing when the reference names it or Git's own words when it does not.
/// Why:
///   One place names that ref and writes it,
/// so the side ref of a landing that gave up at a step and the side ref the restart writes for a batch its branch does
/// not show cannot drift apart;
/// the write follows no symbolic ref,
/// and it passes the setting that keeps Git from making a reflog for a ref that has none
/// (`SPEC.md`,
/// a setting that does not stop an append to a log another Git made).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepTheBatchesCommit(git, repo, 1, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_the_batches_commit(
    git: &GitContext,
    repository: &Path,
    seq: u64,
    tip: &str,
) -> Option<String> {
    return keep_on_a_side_ref(git, repository, kept_ref(seq).as_str(), tip);
}

/// What:
///  `keep_the_batches_commit` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a commit a landing could not put on a branch is kept on its
///  own kept reference once the landing is a task;
/// the name and the write are the synchronous face's,
/// so a commit one face keeps is kept where the other keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepTheBatchesCommitAwaiting(git, repo, 1, tip)).toBeUndefined()
/// ```
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn keep_the_batches_commit_awaiting(
    git: &GitContext,
    repository: &Path,
    seq: u64,
    tip: &str,
) -> Option<String> {
    return keep_on_a_side_ref_awaiting(git, repository, kept_ref(seq).as_str(), tip).await;
}

/// What:
///  Writes the per-worktree reference `refs/worktree/cli-git-service/<seq>` at `tip`,
///  the reference that keeps the objects of a batch reachable until its branch write,
///  and answers what became of the write.
/// Why:
///  This is the write that keeps a commit Git built reachable while its branch write meets a lock
///  (`service_land_objects_of_batch.rs`),
///  and it is the one the release gate of 0.6.1-alpha.11 read as a failed write when its wait ran out;
///  it is written here so that reading is answered by the reference
///  (`write_until_the_reference_names`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnTheScratchReference(git, repo, scratchRef(seq), tip)).toEqual('written')
/// ```
pub(crate) fn keep_on_the_scratch_reference(
    git: &GitContext,
    repository: &Path,
    scratch: &str,
    tip: &str,
) -> ASideRefWrite {
    return write_until_the_reference_names(git, repository, scratch, tip);
}

/// What:
///  `keep_on_the_scratch_reference` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the reference that keeps a batch's objects reachable is written
///  from the landing once it is a task;
/// the write,
/// its two attempts and the read that answers a wait which ended are the
///  synchronous face's,
/// so the two cannot disagree about what the reference names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnTheScratchReferenceAwaiting(git, repo, scratchRef(seq), tip)).toEqual('written')
/// ```
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn keep_on_the_scratch_reference_awaiting(
    git: &GitContext,
    repository: &Path,
    scratch: &str,
    tip: &str,
) -> ASideRefWrite {
    return write_until_the_reference_names_awaiting(git, repository, scratch, tip).await;
}

/// What:
///  Writes the reference `name` at `tip`
/// only while that name names nothing,
/// and answers nothing when the reference names `tip`
///  or Git's own words when it does not.
/// Why:
///   A command that removes a linked worktree keeps a held commit on a reference of the common Git directory
///  before Git removes the only other ones (`worktree_removal_held_commits.rs`),
///  and a name that another command already wrote must never be replaced:
///  replacing it would forget the commit that reference was made for,
///  which is the owner's rule "never discard user data" again.
///  The empty old value is Git's `--stdin`-free way to ask for exactly that:
///  `update-ref <ref> <new> ""` refuses when the name exists at all,
///  whether this command read it first or another one wrote it in between,
///  and it is what every attempt of `write_until_the_reference_names` asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnASideRef(git, repo, name, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_on_a_side_ref(
    git: &GitContext,
    repository: &Path,
    name: &str,
    tip: &str,
) -> Option<String> {
    return write_until_the_reference_names(git, repository, name, tip).refusal();
}

/// What:
///  `keep_on_a_side_ref` with every Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a commit a landing could not put on a branch is kept on its
///  side reference once the landing is a task;
/// the name,
/// the create-only write and the words a refusal gives are the synchronous face's,
/// so a commit is kept at one name either way,
///  and a name another command wrote is replaced by neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await keepOnASideRefAwaiting(git, repo, name, tip);
/// ```
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn keep_on_a_side_ref_awaiting(
    git: &GitContext,
    repository: &Path,
    name: &str,
    tip: &str,
) -> Option<String> {
    return write_until_the_reference_names_awaiting(git, repository, name, tip)
        .await
        .refusal();
}
