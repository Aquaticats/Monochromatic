//! What:
//!  The sentences of the refusals a force count decides:
//!  a plan that cannot be read,
//!  a clone the wrapper cannot prove,
//!  a forced run whose plan cannot be read and whose clone is gone,
//!  a lock two forces could not delete,
//!  and a lock this add did not write.
//! Why:
//!  The owner's answers of 2026-11-28 made the force count decide five of this
//!  removal's refusals,
//!  and each sentence has to say which count passes it and which command a
//!  person runs,
//!  so they are written together rather than scattered among the stops no force
//!  changes
//!  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
//!  `removal_stop_sentences.rs` asks this file first and prints its own arms for
//!  the stops a force does not reach.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sentence = forcedRefusalSentence(stop); // undefined for a stop no force decides
//! ```

use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;

/// What:
///  The sentence one refusal prints when a force count decided it.
/// Why:
///  A stop this file does not know is a stop no force changes,
///  and the caller prints it from its own arms,
///  so `None` is how the two files divide the stops without a second list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = forcedRefusalSentence(stop) ?? theOtherFile(stop);
/// ```
pub(super) fn forced_refusal_sentence(stop: &RemovalStop) -> Option<String> {
    return match stop {
        RemovalStop::PlanUnreadable {
            slot,
            path,
            refusal,
        } => Some(format!(
            "{}\nWays forward: `git monochromatic-git-policy-cli settle-worktree-clone-record {slot} --confirm-removal-of {slot} --confirm-clone-guid <the last four decimal digits of the clone GUID the record holds>` is the command for a slot whose records must be settled, and it prints the plan it finds without acting on one it cannot read. One force goes past this plan: `git worktree remove --force {slot}` proves the clone by the record's GUID against the pool, destroys the clone and its snapshots from the pool's own list, and deletes the plan file.",
            refusal.message(path.as_path())
        )),
        RemovalStop::TheCloneCannotBeProved {
            slot,
            cause,
            recorded_guid,
            answered_guid,
        } => Some(format!(
            "the clone of the slot `{slot}` cannot be proved: {cause} Nothing was destroyed, and no plan was written. Two forces pass this refusal: `git worktree remove --force --force {slot}` destroys the dataset the slot's record names even when the pool's answer does not match the record, which is the record's GUID `{recorded_guid}` against the pool's `{}`. That destroys whatever now stands under that name, and nothing can prove afterwards which clone it was.",
            answered_guid.as_deref().unwrap_or("no answer")
        )),
        RemovalStop::ThePlanIsUnreadableAndTheCloneIsGone {
            slot,
            path,
            reason,
        } => Some(format!(
            "a force was given, but the destruction plan at {} could not be read and the clone the slot's record names does not answer the pool ({reason}), so nothing proves what an earlier run destroyed and a forced removal deletes nothing. Nothing was unmounted, nothing was destroyed, and the plan stands. Ways forward: `git monochromatic-git-policy-cli settle-worktree-clone-record {slot} --confirm-removal-of {slot} --confirm-clone-guid <the last four decimal digits of the clone GUID the record holds>` is the command for a slot whose records must be settled, and it prints the plan it finds; run the removal again once the pool answers for that name.",
            path.display()
        )),
        RemovalStop::TheLockCouldNotGo {
            path,
            said,
        } => Some(format!(
            "this removal carried two forces, which delete a person's lock, and the lock at {} could not be deleted ({said}). Nothing was unmounted, nothing was destroyed, and the lock still stands. Ways forward: read what the message names, remove the lock with `git worktree unlock <slot>` or by hand, and run the removal again.",
            path.display()
        )),
        RemovalStop::TheLocksWordsAreNotThisAdds {
            slot,
            cause,
        } => Some(format!(
            "the Git registration of the slot `{slot}` holds a lock this add cannot attribute to itself ({cause}), so a person's lock may stand there and this removal stops before it unmounts or destroys anything, with no force or one. The slot's claim file is the only record this run wrote, and stopping gives the slot back. Nothing was unmounted, nothing was destroyed, no intent was filed and no plan was written. Ways forward: run `git worktree unlock {slot}` to remove the lock and run the removal again, or run `git worktree remove --force --force {slot}`: two forces delete the lock file and then run this removal as it stands, which is Git's own gradation, `-f -f` among them."
        )),
        _ => None,
    };
}
