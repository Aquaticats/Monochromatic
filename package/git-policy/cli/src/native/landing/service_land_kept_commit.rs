//! What:
//!  What a recovery keeps of a batch whose branch does not show its commit:
//!  the commit itself,
//!  on `refs/cli-git/kept/<number>`.
//! Why:
//!   The batch's intent is about to go,
//!  and it is the last record that names the commit;
//!  a commit this wrapper built and did not put on a branch has no other name,
//!  so it is kept here and the person is told the name.
//!  Every batch holds its commits on a scratch branch until its branch write,
//!  a commit Git built included,
//!  so the drop keeps the commit here and then removes that branch
//!  (`service_scratch_ref_of_a_dropped_batch.rs`).
//!  A kept ref that already names another commit is work an earlier batch was kept under,
//!  so it is left alone and said;
//!  only a number the clock and the journal deal twice can bring that about.
//!  A write that cannot be made keeps nothing,
//!  and it does not hold up the drop either:
//!  the drop is what releases the worktree's index lock and forgets the batch,
//!  so a commit whose ref cannot be written is still the commit the person runs again.
//!  It sits in a file of its own because the recovery's file is at the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! keepWhatTheBranchDoesNotShow(intent) // write refs/cli-git/kept/<seq>, or say why it could not be written
//! ```

use crate::service_journal::Intent;
use crate::service_land_git_steps_and_import_stream::keep_on_a_side_ref;
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, kept_ref, value_of};
use crate::transaction_git::GitContext;

/// What:
///  Keeps the commit of a batch that recovery drops on `refs/cli-git/kept/<number>`,
/// the ref the wrapper names for a commit it did not land,
/// before the batch's intent goes,
/// and never replaces a kept ref that already names another commit.
/// Why:
///   A service stopped inside the branch write never cuts that write:
/// it ends at its bound with the write still running,
/// so the landing cannot unwind and keep its commit,
/// and a service killed between its intent and its branch move leaves the same shape.
///   Every batch holds its commits on a scratch branch until its branch write,
/// a commit Git built included,
/// so the drop keeps the commit here and then removes that branch
/// (`service_scratch_ref_of_a_dropped_batch.rs`).
///   A kept ref that already names another commit is work an earlier batch was kept under,
/// so it is left alone and said;
/// only a number the clock and the journal deal twice can bring that about.
/// A write that cannot be made keeps nothing,
/// and it does not hold up the drop either:
/// the drop is what releases the worktree's index lock and forgets the batch,
/// so a commit whose ref cannot be written is still the commit the person runs again.
pub(crate) fn keep_what_the_branch_does_not_show(git: &GitContext, intent: &Intent) {
    let kept: String = kept_ref(intent.seq);
    match value_of(git, intent.repository.as_path(), kept.as_str()) {
        RefValue::Names(named) if named == intent.new => {
            crate::service_log!("batch {} is not on its branch, and its commit {} is already kept at {kept}", intent.seq, intent.new);
        }
        RefValue::Names(named) => crate::service_error!(
            "batch {} is not on its branch, and {kept} names {named}, so its commit {} is not kept there and is left where the landing put it",
            intent.seq,
            intent.new
        ),
        RefValue::Unasked => crate::service_error!(
            "batch {} is not on its branch, and {kept} cannot be read, so its commit {} is left where the landing put it",
            intent.seq,
            intent.new
        ),
        RefValue::Absent => match keep_on_a_side_ref(git, intent.repository.as_path(), intent.seq, intent.new.as_str()) {
            None => crate::service_log!("batch {} is not on its branch, so its commit {} is kept at {kept}", intent.seq, intent.new),
            Some(words) => crate::service_error!(
                "batch {} is not on its branch, and its commit {} could not be kept at {kept} ({words}), so the commit is left where the landing put it",
                intent.seq,
                intent.new
            ),
        },
    }
}
