//! What:
//!  The batch whose write of its own scratch reference was not proven:
//!  what the landing does with it,
//!  and the one line the person reads about it.
//! Why:
//!   The scratch reference `refs/worktree/cli-git-service/<batch>` is what keeps a batch's objects out of a
//!  `git gc` while the landing waits for the index lock and moves the branch
//!  (`service_land_objects_of_batch.rs`).
//!  Real Git needs no such reference:
//!  one `git commit` writes the commit's objects and moves the branch itself,
//!  so the objects are named by the branch the same instant they exist.
//!  A wrapper commit whose reference the write never proved was answered as not made,
//!  and kept on `refs/cli-git/kept/<batch>` for the person to find,
//!  which made a commit real Git would have made a failure
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  the section of 2026-11-12T10:27:45-05:00,
//!  item 1.5:
//!  "Commit and say so").
//!  The batch now lands as real Git lands it,
//!  and the person is told once,
//!  on the command that made the commit,
//!  that the extra protection was not in place for it.
//!  A batch whose reference the write did prove is told nothing new.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rememberWhyTheReferenceWasNotProven('Git refused it'); /* the batch lands */ sayOnceWhatTheWindowWas(); // and, for a held batch: keepTheCommitOnItsKeptName()
//! ```

use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use crate::service_index_lock::{Announce, Announcer};
use crate::service_land_context::LandContext;
use crate::service_land_outcomes::Outcome;
use crate::service_land_side_reference_write::{keep_the_batches_commit, the_batch_reference_names};
use crate::service_protocol::CommitRequest;
use crate::service_scratch_ref_of_a_dropped_batch::{kept_ref, scratch_ref};
use crate::transaction_git::GitContext;
use std::path::Path;

/// What:
///  Remembers why this landing's batch has no scratch reference,
///  where the landing that answers the batch reads it.
/// Why:
///  A write that is not proven is no reason to fail the commit,
///  and the person still has to be told what it cost:
///  the step that writes the reference answers its caller with an enumerated outcome alone,
///  so the reason travels beside the batch in the landing's own state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landing.withoutItsScratchReference = 'Git refused the write of refs/worktree/cli-git-service/1 (cannot lock ref ...)'
/// ```
pub(crate) fn remember_why_it_has_none(why: String) {
    on_the_landing_state(why, keep_the_reason);
}

/// Keeps the reason in the landing's state.
fn keep_the_reason(state: &LandingState, why: String) {
    *state.without_its_scratch_reference.borrow_mut() = Some(why);
}

/// What:
///  Forgets a reason an earlier attempt of this batch recorded.
/// Why:
///   A batch whose branch moved under it is built again from the same requests,
///  and that attempt writes the reference for itself:
///  what the attempt before it could not prove says nothing about the batch that lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landing.withoutItsScratchReference = undefined
/// ```
pub(crate) fn forget_why_it_had_none() {
    on_the_landing_state((), clear_the_reason);
}

/// Clears the reason in the landing's state.
fn clear_the_reason(state: &LandingState, (): ()) {
    *state.without_its_scratch_reference.borrow_mut() = None;
}

/// What:
///  The line of the branch write that removes the batch's scratch reference,
///  or nothing when this attempt never wrote it.
/// Why:
///   The branch write removes that reference in its own transaction,
///  and a name this landing never wrote is not its to remove:
///  the delete is left out of the transaction,
///  because the name may hold another command's commit,
///  and that commit may be the only thing holding it
///  (the owner's rule "never discard user data").
///  A reference a wait left running may finish after the branch moved and recreate that name;
///  it then names the commit the branch already has,
///  which costs space and never the person's data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = proven ? `delete ${scratch}\n` : ''
/// ```
pub(crate) fn the_removal_of_the_scratch_reference(scratch: &str) -> String {
    return match on_the_landing_state((), the_reference_is_proven) {
        true => format!("delete {scratch}\n"),
        false => String::new(),
    };
}

/// Whether the state holds no reason,
/// which is the reference having been proven.
fn the_reference_is_proven(state: &LandingState, (): ()) -> bool {
    return state.without_its_scratch_reference.borrow().is_none();
}

/// What became of a batch whose reference the write did not prove:
/// the commit that names it,
/// and whether a branch names it already.
type Made = (String, bool);

/// What:
///  Says what the write that was not proven cost,
///  and keeps a held commit on its kept name,
///  for a batch that was made anyway.
/// Why:
///   The owner's answer for such a write is that the commit is made and the person is told
///  (`doc/handover/cli-git-rust-implementation.md`,
///  2026-11-12T10:27:45-05:00,
///  item 1.5),
///  so the person has to read what the missing reference exposed and that it is over:
///  a commit that no reference named while it was landing is one a `git gc --prune=now` started in that window
///  could have taken,
///  and nothing can take it once a branch names it.
///  A held commit is named by its branch only when the lock goes,
///  so it is kept on `refs/cli-git/kept/<batch>` when the batch's own reference is not proven:
///  the settlement that moves the branch needs the objects to be there.
///  A batch that was not made is left alone:
///  the ending that answered it names its commit where a commit that did not land is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const why = takeWhy(); if (!why) return; const made = madeOf(outcomes); if (!made) return; announce(theLine(why)); if (!made.named) keepIt()
/// ```
pub(crate) fn say_what_it_cost_and_keep_a_held_commit(
    context: &LandContext,
    announce: &dyn Announcer,
    requests: &[CommitRequest],
    seq: u64,
    outcomes: Option<&[Outcome]>,
) {
    let Some(why) = on_the_landing_state((), take_the_reason) else { return };
    let Some(made) = made_of(outcomes) else { return };
    announce.announce(Announce::Waiting(the_line(why.as_str())));
    if made.1 {
        return;
    }
    let Some(first) = requests.first() else { return };
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    let repository: &Path = first.repository.as_path();
    let scratch: String = scratch_ref(seq);
    // A write that finished after its wait ended names the commit already, and the reference it wrote is what holds the commit
    //  until the branch moves; nothing is kept beside it.
    if the_batch_reference_names(&git, repository, scratch.as_str(), made.0.as_str()) {
        return;
    }
    let kept: String = kept_ref(seq);
    match keep_the_batches_commit(&git, repository, seq, made.0.as_str()) {
        None => announce.announce(Announce::Waiting(format!(
            "monochromatic-git-policy-cli: the commit of this held batch is kept at {kept} (see it with `git log {kept}`), because the batch's own reference {scratch} was not written, so a maintenance that drops what no reference names cannot take it while the branch waits.\n"
        ))),
        // Nothing of this repository names the commit now, so the one thing this landing can still give the person is the
        //  object's own name with what to do with it before a maintenance that drops what no reference names takes it.
        Some(words) => announce.announce(Announce::Waiting(format!(
            "monochromatic-git-policy-cli: the commit of this held batch, {commit}, could not be kept on a reference either ({words}), {stays}.\n",
            commit = made.0,
            stays = the_words_for_a_commit_no_reference_names(made.0.as_str())
        ))),
    }
}

/// What a batch that was made answers the line with:
/// the commit a held batch keeps,
/// and whether a branch names it.
fn made_of(outcomes: Option<&[Outcome]>) -> Option<Made> {
    let answered: &[Outcome] = outcomes?;
    for outcome in answered {
        if let Outcome::Landed { commit } = outcome {
            return Some((commit.clone(), true));
        }
    }
    for outcome in answered {
        if let Outcome::Held { commit, .. } = outcome {
            return Some((commit.clone(), false));
        }
    }
    return None;
}

/// Takes the reason the batch has no scratch reference,
/// once.
fn take_the_reason(state: &LandingState, (): ()) -> Option<String> {
    return state.without_its_scratch_reference.take();
}

/// What:
///  The clause a sentence about a commit ends with when no reference of the repository is proven to name it.
/// Why:
///   A landed batch is named by its branch and a kept one by `refs/cli-git/kept/<batch>`,
///  so a commit that no reference names is an ending the wrapper has to say out loud:
///  a maintenance that drops what no reference names takes it,
///  and the one thing left to give the person is the object's own name and the command that makes a reference of it
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `and no reference is proven to name it, ... keep it with \`git update-ref refs/heads/<the name you want> ${tip}\``
/// ```
pub(crate) fn the_words_for_a_commit_no_reference_names(tip: &str) -> String {
    return format!(
        "and no reference is proven to name it, so a maintenance that drops what no reference names would take it; keep it with `git update-ref refs/heads/<the name you want> {tip}`"
    );
}

/// What:
///  The one line the person reads about a commit that was made without its scratch reference.
/// Why:
///   The decision is "commit and say so",
///  and what there is to say has three parts:
///  the commit was made,
///  the reference that would have kept a `git gc` off its objects while it was landing was not written and why,
///  and no later maintenance can take it now.
///  The words name the reference and Git's own refusal,
///  because the person's next question is what refused it,
///  and they never say the commit failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: your commit was made, and ... was not written (${why}), so ... The branch names the commit now ...`
/// ```
fn the_line(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: your commit was made, and the reference that keeps a commit out of a `git gc` started while it is \
being landed was not written for it ({why}), so a `git gc --prune=now` or `git prune --expire=now` running in that window could have taken \
its objects. The branch names the commit now, so no later maintenance takes it.\n"
    );
}
