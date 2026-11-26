//! What:
//!  Every state that prevents the cancel command,
//!  with its exit code and the sentence a person reads,
//!  and every state that prevents a removal's own cancellation.
//! Why:
//!  The design lists the refusals one by one and gives the two whose state the open item asks about
//!  their own words
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "What it refuses" and "Exit codes and streams").
//!  A caller's mistake exits `2`,
//!  a state that prevents the command exits `1`,
//!  and every sentence names the state it found,
//!  what it changed,
//!  and the step that repairs it,
//!  so no person has to read the code to learn what to do.
//!  One value carries them all,
//!  so the command has one place to print from and cannot print half a refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = { kind: 'unknown-outcome'; ... } | ...;
//! console.error(refusal.message()); process.exit(refusal.exitCode());
//! ```

use crate::worktree_clone_mount::clone_slot_git_pointer::GitPointerRefusal;
use std::path::PathBuf;

/// The exit code of a caller's mistake.
pub const CALLERS_MISTAKE: i32 = 2;

/// The exit code of a state that prevents the command.
pub const THE_OPERATION_FAILED: i32 = 1;

/// What:
///  One state in which a removal is not reversed.
/// Why:
///  Each arm names the record or the object it read and the step that repairs the state,
///  because the command changes nothing it could not prove and never attaches over an unknown
///  outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'not-being-removed'; slot } | { kind: 'live-removal'; owner } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CancelRefusal {
    /// Nothing about this slot is a removal.
    NotBeingRemoved {
        /// The slot that was named.
        slot: String,
        /// The records that were read and are not there.
        found: String,
    },
    /// The slot's pointer names another repository's registration.
    ASlotOfAnotherRepository {
        /// The slot that was named.
        slot: String,
        /// What the slot's pointer names.
        points_at: String,
        /// The registration this repository's slot would have.
        this_repository: String,
    },
    /// The slot's `.git` pointer could not be read,
    /// is not one `gitdir:` line,
    /// or names no registration of this repository.
    ThePointerIsNotOneGitdirLine {
        /// The slot that was named.
        slot: String,
        /// What the read answered.
        said: String,
    },
    /// The add's binding record is absent or unreadable.
    NoBinding {
        /// The slot the record is filed under.
        slot: String,
        /// What the read answered.
        reason: String,
    },
    /// The binding record names another slot or lacks a field the proof needs.
    TheBindingIsNotThisSlots {
        /// The slot the command proves.
        slot: String,
        /// What the record says instead.
        said: String,
    },
    /// Two records that must name one clone name different values.
    TheRecordsDisagree {
        /// The field or the pair of records that disagree.
        what: String,
        /// The first reading.
        first: String,
        /// The second reading.
        second: String,
    },
    /// A live removal holds the clone's claim.
    ALiveRemoval {
        /// The owner the claim names.
        owner: String,
    },
    /// The removal claim could not be taken at all.
    TheClaimCouldNotBeTaken {
        /// What the claim's own mechanism answered.
        said: String,
    },
    /// The clone's dataset does not answer the pool.
    TheCloneDoesNotAnswer {
        /// The dataset the records name.
        dataset: String,
        /// What the read answered.
        reason: String,
    },
    /// The clone answers with another identity than the records name.
    TheCloneIsNotTheRecordedOne {
        /// The field that differs.
        field: &'static str,
        /// What the record says.
        recorded: String,
        /// What the pool answers.
        live: String,
    },
    /// The clone's own desired-state property is no local request.
    TheIntentIsNotALocalRequest {
        /// The clone.
        clone: String,
        /// What the property holds instead.
        said: String,
    },
    /// The clone's snapshots could not be listed completely.
    TheInventoryIsUnreadable {
        /// The clone.
        clone: String,
        /// What the read answered.
        reason: String,
    },
    /// The plan records a snapshot whose outcome cannot be settled.
    AnUnknownOutcome {
        /// The snapshot's recorded name.
        snapshot: String,
        /// The snapshot's recorded GUID.
        guid: String,
        /// What proves the outcome unknown,
        /// in the plan's or the pool's own words.
        said: String,
    },
    /// A live snapshot carries the recorded name with another GUID.
    ASnapshotNameIsReused {
        /// The recorded name.
        name: String,
        /// The recorded GUID.
        recorded_guid: String,
        /// The live GUID.
        live_guid: String,
    },
    /// The clone's head is gone,
    /// so nothing can attach it again.
    TheHeadIsGone {
        /// The slot whose clone is gone.
        slot: String,
        /// The clone the records name.
        dataset: String,
        /// Its recorded GUID.
        guid: String,
        /// What proves the absence.
        said: String,
    },
    /// The destruction plan could not be read.
    ThePlanIsUnreadable {
        /// The plan's file.
        path: PathBuf,
        /// What the read answered.
        said: String,
    },
    /// The plan that stands names another clone than the records do.
    ThePlanIsOfAnotherClone {
        /// The plan's file.
        path: PathBuf,
        /// The clone's GUID the plan carries.
        plan_guid: String,
        /// The GUID the records carry.
        recorded_guid: String,
    },
    /// The one plan a cancellation removes could not be removed.
    ThePlanCouldNotGo {
        /// The plan's file.
        path: PathBuf,
        /// What the unlink answered.
        said: String,
    },
    /// The `mount` request could not be written.
    TheMountRequestCouldNotBeWritten {
        /// What the writer answered.
        said: String,
    },
    /// The handler answered the `mount` request with a refusal,
    /// a failure or a word this route does not read.
    TheHandlerRefused {
        /// The word the answer carried.
        outcome: String,
        /// The step the answer names,
        /// where it names one.
        step: String,
        /// The errno the answer names,
        /// where it names one.
        errno: String,
    },
    /// The `mount` request's answer never arrived.
    NoAnswerAtTheDeadline {
        /// What the wait last saw,
        /// in its own words.
        said: String,
    },
    /// The clone's staged state cannot be settled safely.
    TheStagedStateIsKept {
        /// The step that kept it.
        step: String,
        /// What refused.
        said: String,
    },
    /// The clone's mount state could not be read or is not the one the answer claims.
    TheMountStateIsNotTheAnswers {
        /// What the read or the answer said.
        said: String,
    },
}

impl CancelRefusal {
    /// What:
    ///  The status the command exits with.
    /// Why:
    ///  Only the two caller mistakes exit `2`;
    ///  every other arm is a state that prevents the command and exits `1`,
    ///  which is what the repository's exit codes mean by an operation that failed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const code = refusal.exitCode(); // 1 or 2
    /// ```
    pub fn exit_code(&self) -> i32 {
        return match self {
            CancelRefusal::NotBeingRemoved { .. } => CALLERS_MISTAKE,
            CancelRefusal::ASlotOfAnotherRepository { .. } => CALLERS_MISTAKE,
            _ => THE_OPERATION_FAILED,
        };
    }

    /// What:
    ///  The sentence a person reads for one refusal.
    /// Why:
    ///  Each says what was found,
    ///  that nothing was changed unless an earlier step changed something,
    ///  and the step that repairs it,
    ///  because a refused command changes no intent,
    ///  no folder and no plan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// console.error(refusal.message());
    /// ```
    pub fn message(&self) -> String {
        return match self {
            CancelRefusal::NotBeingRemoved { slot, found } => format!(
                "the slot `{slot}` is not being removed: {found} No destruction plan, no staged-folder record and no local `unmount` request stands for it, so there is nothing to cancel. Nothing was read beyond those records and nothing was changed. Ways forward: a pending add is this state and not a removal, so run the add's own removal command for it; where the slot was a mistyped name, run the command with the slot you meant."
            ),
            CancelRefusal::ASlotOfAnotherRepository { slot, points_at, this_repository } => format!(
                "the slot `{slot}` belongs to another checkout: its `.git` pointer names `{points_at}`, and this repository's slot is registered at `{this_repository}`. Nothing was changed. Ways forward: run the command inside the repository that owns the slot, or name a slot of this repository."
            ),
            CancelRefusal::ThePointerIsNotOneGitdirLine { slot, said } => format!(
                "the removal of the slot `{slot}` cannot be reversed: its `.git` pointer could not be read ({said}). Nothing was mounted and nothing was changed. Ways forward: an administrator restores the slot's pointer, or repairs the registration the slot names; a damaged pointer is never treated as a removal's."
            ),
            CancelRefusal::NoBinding { slot, reason } => format!(
                "the removal of the slot `{slot}` cannot be reversed: the binding the add wrote for it could not be read ({reason}), so nothing proves which clone, origin or registration this command is about. Nothing was mounted and nothing was changed. Ways forward: an administrator's adoption is the step that repairs a damaged binding; a command never rebuilds or removes one."
            ),
            CancelRefusal::TheBindingIsNotThisSlots { slot, said } => format!(
                "the removal of the slot `{slot}` cannot be reversed: its binding record {said}. Nothing was mounted and nothing was changed. Ways forward: an administrator's adoption is the step that repairs a damaged binding, and a command never rewrites one to match the slot it was given."
            ),
            CancelRefusal::TheRecordsDisagree { what, first, second } => format!(
                "the removal cannot be reversed: two records disagree about what this clone is, and {what}: one says `{first}` and the other `{second}`. Nothing was mounted and nothing was changed, because a disagreement is never resolved in favour of either record. Ways forward: read the plan, the binding and the clone's property, and run `git monochromatic-git-policy-cli doctor`; where the records cannot be made to agree, an administrator recovers the lineage."
            ),
            CancelRefusal::ALiveRemoval { owner } => format!(
                "the removal cannot be reversed while it runs: its claim names the process {owner}. Nothing was mounted and nothing was changed. Ways forward: wait for that removal to finish, or end it, and run the command again once its process is gone."
            ),
            CancelRefusal::TheClaimCouldNotBeTaken { said } => format!(
                "the removal cannot be reversed: the clone's removal claim could not be taken ({said}). Nothing was mounted and nothing was changed. Ways forward: read the claim file the message names, and run the command again once the folder is usable."
            ),
            CancelRefusal::TheCloneDoesNotAnswer { dataset, reason } => format!(
                "the removal cannot be reversed: the clone `{dataset}` does not answer the pool ({reason}). Nothing was mounted and nothing was changed. Ways forward: run the command again once the pool answers for that name."
            ),
            CancelRefusal::TheCloneIsNotTheRecordedOne { field, recorded, live } => format!(
                "the removal cannot be reversed: the clone answers with {field} `{live}`, and the records say `{recorded}`, so the object standing there is not the one this removal proved. Nothing was mounted and nothing was changed. Ways forward: an administrator has to recover the lineage; a command never attaches a clone whose identity differs from the one its records prove."
            ),
            CancelRefusal::TheIntentIsNotALocalRequest { clone, said } => format!(
                "the removal of `{clone}` cannot be reversed: the clone's own request property reads {said}, and the removal's request is not there. Nothing was mounted and nothing was changed. Ways forward: read the clone's property, run `git monochromatic-git-policy-cli doctor`, and run `git worktree remove` again to resume the removal, which writes its own request."
            ),
            CancelRefusal::TheInventoryIsUnreadable { clone, reason } => format!(
                "the removal of `{clone}` cannot be reversed: its live snapshots could not be listed completely ({reason}), and a list this route could not read is a refusal rather than an empty history. Nothing was mounted and nothing was changed. Ways forward: run the command again once the pool answers; an outcome this route could not settle is never treated as destroyed or as intact."
            ),
            CancelRefusal::AnUnknownOutcome { snapshot, guid, said } => format!(
                "cannot cancel the removal: the destruction plan records a snapshot whose outcome cannot be settled. `{snapshot}` (GUID `{guid}`) is marked as still being requested, and {said}, so its outcome is unknown. Nothing was mounted and nothing was changed. Ways forward: run `git worktree remove` again for this slot, which settles the plan before anything else, and read `git monochromatic-git-policy-cli doctor`; where that removal also refuses, an administrator recovers the lineage."
            ),
            CancelRefusal::ASnapshotNameIsReused { name, recorded_guid, live_guid } => format!(
                "the removal cannot be reversed: the recorded snapshot name `{name}` now carries GUID `{live_guid}`, and the plan recorded GUID `{recorded_guid}`, so the name was reused for another object and the recorded outcome cannot be settled. Nothing was mounted and nothing was changed. Ways forward: compare the two identities with `zfs get -H -p -o value guid,createtxg <name>`; no wrapper command settles a reused name."
            ),
            CancelRefusal::TheHeadIsGone { slot, dataset, guid, said } => format!(
                "cannot cancel the removal of the slot `{slot}`: its clone is gone. The dataset `{dataset}` (GUID `{guid}`) does not answer and {said}, so the head was destroyed. A cancelled removal cannot bring it back, and nothing was mounted or changed. Ways forward: run `git worktree remove` again for this slot, which settles the registration, the underlay and the origin cleanup; `git monochromatic-git-policy-cli doctor` lists what is still owed."
            ),
            CancelRefusal::ThePlanIsUnreadable { path, said } => format!(
                "the removal cannot be reversed: the destruction plan at {} could not be read ({said}). Nothing was mounted and nothing was changed, because a plan this route cannot read may describe a destroy nobody can settle. Ways forward: read the plan's file and `git monochromatic-git-policy-cli doctor`; a plan is never removed on a read that failed.",
                path.display()
            ),
            CancelRefusal::ThePlanIsOfAnotherClone { path, plan_guid, recorded_guid } => format!(
                "the removal cannot be reversed: the destruction plan at {} records the clone GUID `{plan_guid}` and the binding records `{recorded_guid}`, so the two records describe different clones. Nothing was mounted and nothing was changed. Ways forward: read both records and `git monochromatic-git-policy-cli doctor`; where they cannot be made to agree, an administrator recovers the lineage.",
                path.display()
            ),
            CancelRefusal::ThePlanCouldNotGo { path, said } => format!(
                "the removal could not be reversed: the plan at {} witnesses nothing and could not be removed ({said}), so the removal's own records are kept as they stand and nothing further was done. Ways forward: fix what refused the unlink and run the command again; the plan is removed only after the clone and the plan are proved.",
                path.display()
            ),
            CancelRefusal::TheMountRequestCouldNotBeWritten { said } => format!(
                "the removal cannot be reversed: the `mount` request could not be written on the clone, and its gate runs before any request ({said}). Nothing was mounted and nothing was changed. Ways forward: read the clone's own request property before any retry, as the writer's sentence says, and run the command again once the machine's ZFS compatibility answers."
            ),
            CancelRefusal::TheHandlerRefused { outcome, step, errno } => format!(
                "the removal was not reversed: the handler answered `{outcome}` at the step `{step}` with errno {errno}. Every record is kept as it stands, the intent still asks for the mount, and a later trigger answers it. Ways forward: read the handler's journal with `journalctl -u reconcile-git-worktree-clone-mounts.service`, fix what the step names, and run the command again."
            ),
            CancelRefusal::NoAnswerAtTheDeadline { said } => format!(
                "the removal was not reversed within the deadline: {said} The `mount` request still stands on the clone, so a later trigger may still answer it, and every record is kept as it stands. Ways forward: run `git monochromatic-git-policy-cli doctor` to read the state, and run the command again once the handler answers; a deadline is not a cancellation."
            ),
            CancelRefusal::TheStagedStateIsKept { step, said } => format!(
                "the removal was not reversed: the clone's staging state could not be settled ({step}: {said}). The folder, its record and its claim are kept and reported, because a folder that holds anything or carries a mount is never removed. Ways forward: read the staging folder in the account's cache and run `git worktree remove` again, whose sweep settles the folder once its cause is gone."
            ),
            CancelRefusal::TheMountStateIsNotTheAnswers { said } => format!(
                "the removal was not reversed: {said} Nothing further was done, because a mount this command cannot see is not one it treats as the answer's. Ways forward: check the clone's mount with `findmnt` in the namespace that holds it, and run the command again."
            ),
        };
    }
}

/// What:
///  The pointer refusal as this command's,
///  with the slot it named.
/// Why:
///  The reader's own arms carry the details a person needs,
///  and this command's exit code follows the design's split:
///  another registration is the caller's mistake,
///  and a damaged pointer is the operation failing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = ofPointer(slot, pointerRefusal);
/// ```
pub fn of_pointer(slot: &str, refusal: &GitPointerRefusal) -> CancelRefusal {
    return match refusal {
        GitPointerRefusal::AnotherRegistration { points_at, wanted, .. } => CancelRefusal::ASlotOfAnotherRepository {
            slot: String::from(slot),
            points_at: points_at.display().to_string(),
            this_repository: wanted.display().to_string(),
        },
        other => CancelRefusal::ThePointerIsNotOneGitdirLine { slot: String::from(slot), said: other.message() },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "cancel_refusal_tests.rs"]
mod tests;
