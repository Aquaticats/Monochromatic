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
//!
//! This file holds the face a caller on a thread asks,
//! beside the names and the reader of a reference's value
//! (`service_scratch_ref_names_and_values.rs`)
//! and the face a task of the runtime awaits
//! (`service_scratch_ref_of_a_dropped_batch_awaiting.rs`).

//! What:
//!  The face a caller on a thread asks of the removal of a batch's scratch reference:
//! keep the commit the reference holds on the batch's kept reference,
//! and only then remove the scratch reference,
//! for a batch that will not land
//! and for one a failed landing left with no open intent.
//! Why:
//!  A reference that is the only holder of a commit is never removed,
//! so every step here reads first and removes only what it proved,
//! and the face a task of the runtime awaits
//! (`service_scratch_ref_of_a_dropped_batch_awaiting.rs`)
//! answers the same way,
//! pinned by `the_awaited_clearing_answers_what_the_synchronous_one_answers`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removal = clearAfterDroppedBatch(git, repository, seq) // 'removed' | 'left'
//! ```

use crate::service_journal::{
    OPEN_EXTENSION,
    batch_path,
};
use crate::service_land_context::LandContext;
use crate::service_protocol::CommitRequest;
/// What:
///  The names and the reader of a reference's value that this file serves,
/// declared beside it and kept at the paths its callers already name.
pub(crate) use crate::service_scratch_ref_names_and_values::{
    RefValue,
    ScratchRefRemoval,
    kept_ref,
    rescue_ref,
    scratch_ref,
    value_of,
    value_of_awaiting,
};
use crate::service_scratch_ref_names_and_values::{
    left,
    write_ref,
};
use crate::transaction_git::{
    GitContext,
    NO_REFLOG_SETTING,
};
use std::path::Path;

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
pub(crate) fn clear_after_dropped_batch(
    git: &GitContext,
    repository: &Path,
    seq: u64,
) -> ScratchRefRemoval {
    let scratch: String = scratch_ref(seq);
    let kept: String = kept_ref(seq);
    let tip: String = match value_of(git, repository, scratch.as_str()) {
        RefValue::Names(found) => found,
        // No scratch reference at all: the landing made no objects, or its branch write removed it.
        RefValue::Absent => return ScratchRefRemoval::Removed,
        // A symbolic reference is not one of the wrapper's writes, so what it holds is not proven to be the batch's commit: nothing
        //  is removed on it (`RefValue::Symbolic`).
        RefValue::Symbolic(target) => {
            left(seq, repository, format!("{scratch} is a symbolic reference to {target}, not a reference of the wrapper's own, so nothing is removed").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(
                seq,
                repository,
                format!("{scratch} cannot be read, so nothing is removed").as_str(),
            );
            return ScratchRefRemoval::Left;
        }
    };
    let already: RefValue = value_of(git, repository, kept.as_str());
    match &already {
        RefValue::Names(named) if *named == tip => {}
        RefValue::Names(named) => {
            left(seq, repository, format!("{kept} names {named} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        // The commit is held by the reference this one leads to, which its owner can move: the batch's commit is not proven to be
        //  safe elsewhere, so the scratch reference stays.
        RefValue::Symbolic(target) => {
            left(seq, repository, format!("{kept} is a symbolic reference to {target} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(
                seq,
                repository,
                format!(
                    "{kept} cannot be read, so the scratch reference {scratch} is left as it is"
                )
                .as_str(),
            );
            return ScratchRefRemoval::Left;
        }
        RefValue::Absent => {
            // The empty old value asks Git for a write of a name that names nothing: a reference another command writes between this
            //  read and the write is never replaced (`keep_on_a_side_ref`, the owner's rule "never discard user data").
            if !write_ref(
                git,
                repository,
                &[
                    "-c",
                    NO_REFLOG_SETTING,
                    "update-ref",
                    "--no-deref",
                    kept.as_str(),
                    tip.as_str(),
                    "",
                ],
            ) {
                left(
                    seq,
                    repository,
                    format!("the commit {tip} of a batch that did not land could not be kept at {kept}, so its scratch reference {scratch} is left as it is").as_str(),
                );
                return ScratchRefRemoval::Left;
            }
            crate::service_log!(
                "batch {} {}: the commit {tip} of a batch that did not land is kept at {kept}",
                seq,
                crate::service_timing::in_the_repository_at(repository)
            );
        }
    }
    // The old value is the one just read: a reference another command wrote in between is never removed, because the commit it
    //  names may be the only thing naming it (the owner's rule "never discard user data").
    if write_ref(
        git,
        repository,
        &[
            "-c",
            NO_REFLOG_SETTING,
            "update-ref",
            "-d",
            "--no-deref",
            scratch.as_str(),
            tip.as_str(),
        ],
    ) {
        crate::service_log!(
            "batch {} {}: the scratch reference {scratch} of a batch that did not land is removed",
            seq,
            crate::service_timing::in_the_repository_at(repository)
        );
        return ScratchRefRemoval::Removed;
    }
    left(seq, repository, format!("the scratch reference {scratch} could not be removed; its commit {tip} is kept at {kept}").as_str());
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
pub(crate) fn clear_after_a_failed_landing(
    context: &LandContext,
    requests: &[CommitRequest],
    seq: u64,
) -> ScratchRefRemoval {
    // No request is no import, so the batch made no reference at all.
    let Some(first) = requests.first() else {
        return ScratchRefRemoval::Removed;
    };
    // An open intent names the batch's own worktree, and the restart that settles it removes the reference then.
    if batch_path(context.journal.as_path(), seq, OPEN_EXTENSION).exists() {
        return ScratchRefRemoval::Removed;
    }
    let git: GitContext = GitContext {
        common_dir: Some(first.common_dir.clone()),
        ..context.git.clone()
    };
    return clear_after_dropped_batch(&git, first.repository.as_path(), seq);
}
