//! What:
//!  Every way the settling command stops without settling,
//!  and the sentence a person reads.
//! Why:
//!  The command removes evidence,
//!  so each refusal says what it read,
//!  what it kept,
//!  and the one command that goes on
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command" and "What it refuses,
//! changing nothing").
//!  The command is non-interactive:
//!  a stop is a sentence and an exit status,
//!  never a question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Stop = { kind: 'the-clone-still-stands'; clone: string } | ...;
//! throw new Error(stop.sentence());
//! ```

use crate::worktree_clone_record_settling::settling_words::{CONFIRM_GUID_FLAG, CONFIRM_REMOVAL_FLAG, the_usage_line};
use std::path::PathBuf;

/// What:
///  Why one settling command stopped,
///  with the values its sentence names.
/// Why:
///  A control asserts the arm it planted,
///  and every arm's sentence names the file,
///  the object or the value that stopped the run,
///  so a person reads what to inspect rather than a code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stop = { kind: string; ...fields };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlingStop {
    /// The arguments are not this command's grammar.
    Usage {
        /// What the parser refused.
        said: String,
    },
    /// One of the two values is still the placeholder a printed line carries.
    PlaceholderLeft {
        /// The flag whose value carries it.
        flag: String,
        /// The value as typed.
        value: String,
    },
    /// The slot the second flag names is not the slot the argument names.
    TheSlotIsNamedTwice {
        /// The argument's value.
        asked: String,
        /// The flag's value.
        confirmed: String,
    },
    /// The slot the record derives from its own `slot=` is another name.
    TheSlotIsNotThisRecord {
        /// The name the command was asked for.
        asked: String,
        /// The last component of the record's own slot path.
        recorded: String,
    },
    /// The digits the second flag carries are not the record's own.
    TheCloneGuidDiffers {
        /// The digits typed.
        typed: String,
        /// The digits the record's `cloneGuid=` ends with.
        wanted: String,
    },
    /// No binding record stands under the slot's name.
    NoRecord {
        /// The record's file.
        path: PathBuf,
    },
    /// A binding record stands and cannot be read.
    RecordUnreadable {
        /// The record's file.
        path: PathBuf,
        /// What its reader said.
        said: String,
    },
    /// The record names no clone,
    /// so there is no clone whose absence is in question.
    NoCloneRecorded {
        /// The slot's name.
        slot: String,
        /// The phase the record names.
        phase: String,
    },
    /// A record reads but lacks a field this command compares.
    TheRecordIsIncomplete {
        /// What the record's own reader said.
        said: String,
    },
    /// The slot's own folder cannot be read or listed.
    TheSlotCouldNotBeRead {
        /// The slot's path or the file in it.
        path: PathBuf,
        /// What the filesystem said.
        said: String,
    },
    /// Another command holds the slot's claim.
    TheClaimIsHeld {
        /// The slot's name.
        slot: String,
        /// The process that holds it.
        pid: i64,
    },
    /// The claim could not be taken at all.
    TheClaimCannotBeTaken {
        /// The slot's name.
        slot: String,
        /// What the claim's own reader said.
        said: String,
    },
    /// The clone answers under its recorded name with its recorded GUID.
    TheCloneStillStands {
        /// The clone's dataset name.
        clone: String,
        /// Its GUID.
        guid: String,
    },
    /// The recorded name holds an object whose GUID is not the recorded clone's.
    TheNameHoldsAnother {
        /// The dataset name the pool answered.
        name: String,
        /// The GUID it answered with.
        guid: String,
    },
    /// Another name under the pinned parent carries the recorded clone's GUID.
    TheCloneAnswersElsewhere {
        /// The dataset that carries it.
        found: String,
    },
    /// A mount stands at the slot.
    AMountAtTheSlot {
        /// The mount in the mount table's words.
        described: String,
    },
    /// The mount table could not be read,
    /// which proves nothing about the slot.
    TheMountTableCouldNotBeRead {
        /// What the table's reader said.
        said: String,
    },
    /// The slot's `.git` names another registration.
    ThePointerNamesAnother {
        /// The pointer's words.
        found: String,
    },
    /// The registration standing there is not the one the record names.
    TheRegistrationIsAnother {
        /// The identity the record holds.
        recorded: String,
        /// The identity the folder answers.
        live: String,
    },
    /// The registration the record names cannot be identified.
    TheRegistrationDoesNotAnswer {
        /// The registration's path.
        path: PathBuf,
        /// What the filesystem said.
        said: String,
    },
    /// The registration's lock is a person's own words,
    /// not this add's sentence.
    TheLockIsNotThisAdds {
        /// The lock's file.
        path: PathBuf,
    },
    /// A standing origin cleanup record names another object.
    TheOriginCleanupDisagrees {
        /// The field the two records disagree on.
        field: String,
        /// The binding's value.
        recorded: String,
        /// The cleanup record's value.
        standing: String,
    },
    /// A standing origin cleanup record cannot be read.
    TheOriginCleanupCouldNotBeRead {
        /// The record's file.
        path: PathBuf,
        /// What its reader said.
        said: String,
    },
    /// A plan naming this clone cannot be read.
    ThePlanCouldNotBeRead {
        /// The plan's file.
        path: PathBuf,
        /// What its reader said.
        said: String,
    },
    /// A plan naming this clone records its head as gone,
    /// which is the finish pass's case.
    ThePlanIsTheFinishPasss {
        /// The plan's file.
        path: PathBuf,
    },
    /// A plan's recorded identity disagrees with the binding's.
    ThePlanDisagrees {
        /// The field the two records disagree on.
        field: String,
        /// The binding's value.
        binding: String,
        /// The plan's value.
        plan: String,
    },
    /// One of the records could not be removed or proved gone.
    TheRecordCouldNotGo {
        /// The record's file.
        path: PathBuf,
        /// What the removal or the proof said.
        said: String,
    },
    /// The recipe that clears the registration stopped.
    TheRecipeStopped {
        /// What the recipe said.
        said: String,
    },
    /// The claim was released but the release refused.
    TheClaimCannotBeReleased {
        /// The slot's name.
        slot: String,
        /// What the claim's own writer said.
        said: String,
    },
}

impl SettlingStop {
    /// The sentence a person reads,
    /// naming what stopped the run and the way forward.
    pub fn sentence(&self) -> String {
        return match self {
            SettlingStop::Usage { said } => format!("the settling command reads `{}`, and {said}. Nothing was changed.", the_usage_line()),
            SettlingStop::PlaceholderLeft { flag, value } => format!(
                "the value `{value}` given to `{flag}` is the placeholder a printed line carries, not a value read from this slot's record, so nothing was settled. Nothing was changed. Ways forward: read the record's `slot=` and `cloneGuid=` lines and type the two values, or run the add again at the slot and read what it prints."
            ),
            SettlingStop::TheSlotIsNamedTwice { asked, confirmed } => format!(
                "the argument names the slot `{asked}` and `{CONFIRM_REMOVAL_FLAG}` names `{confirmed}`, and the two must be the same slot, so nothing was settled. Nothing was changed. Ways forward: run the command again with one slot in both places."
            ),
            SettlingStop::TheSlotIsNotThisRecord { asked, recorded } => format!(
                "the record filed under `{asked}` derives the slot name `{recorded}` from its own `slot=` line, so this is not `{asked}`'s record and nothing was settled. Nothing was changed. Ways forward: run the command with the slot the record names, or inspect the file if the two disagree."
            ),
            SettlingStop::TheCloneGuidDiffers { typed, wanted } => format!(
                "`{CONFIRM_GUID_FLAG} {typed}` is not the last four decimal digits of this record's own `cloneGuid=`, which are `{wanted}`, so the confirmation names another clone and nothing was settled. Nothing was changed. Ways forward: read the record's `cloneGuid=` line, and type its last four digits if this is the slot you mean."
            ),
            SettlingStop::NoRecord { path } => format!(
                "{} does not exist, so no record stands at this slot and nothing was settled. Nothing was changed. Ways forward: run the add again at the slot when this worktree should be made, or run `git worktree remove` of the slot when it is an ordinary worktree.",
                path.display()
            ),
            SettlingStop::RecordUnreadable { path, said } => format!(
                "{} cannot be read as this wrapper's binding record, so nothing was settled. Nothing was changed. Ways forward: {said}",
                path.display()
            ),
            SettlingStop::NoCloneRecorded { slot, phase } => format!(
                "the record at `{slot}` is at `phase={phase}` and holds no `cloneGuid=`, so it names no clone whose absence could be unproved, and nothing was settled. Nothing was changed. Ways forward: run the add again at `{slot}`, which continues from this phase and needs no clone's identity."
            ),
            SettlingStop::TheRecordIsIncomplete { said } => format!(
                "the binding record stands and lacks a field this command compares: {said} Nothing was settled and nothing was changed. Ways forward: run the add again at the slot, or inspect the record when the add is not the command that should finish this state."
            ),
            SettlingStop::TheSlotCouldNotBeRead { path, said } => format!(
                "{} could not be read ({said}), so nothing proves what the slot holds and nothing was settled. Nothing was changed. Ways forward: inspect the slot and the record folder, then run the command again.",
                path.display()
            ),
            SettlingStop::TheClaimIsHeld { slot, pid } => format!(
                "another command holds the claim on `{slot}` (process {pid}), so nothing was settled and nothing was changed. Ways forward: wait for that command to end, or stop it if it is not running, and run this command again."
            ),
            SettlingStop::TheClaimCannotBeTaken { slot, said } => format!(
                "the claim on `{slot}` cannot be taken ({said}), so nothing was settled and nothing was changed. Ways forward: inspect the record folder and the process table, then run the command again."
            ),
            SettlingStop::TheCloneStillStands { clone, guid } => format!(
                "the clone `{clone}` answers the pool with GUID `{guid}`, so its absence is not proved and nothing was settled. Nothing was changed. Ways forward: remove the clone with `git worktree remove`, or run the add again at the slot, which continues at the step its record names."
            ),
            SettlingStop::TheNameHoldsAnother { name, guid } => format!(
                "the recorded name `{name}` answers the pool with GUID `{guid}`, which is not the clone this record names, so the name was reused and the recorded clone's absence is unproved. Nothing was settled and nothing was changed. Ways forward: inspect `{name}`; a reused name is a refusal by design, because a live dataset holds the name the recorded clone had."
            ),
            SettlingStop::TheCloneAnswersElsewhere { found } => format!(
                "the clone's recorded GUID answers the pool at `{found}` rather than at its recorded name, so the clone stands under another name and nothing proves it gone. Nothing was settled and nothing was changed. Ways forward: inspect `{found}`; a clone renamed under the pinned parent is the same clone, and no command of this wrapper settles a slot whose clone still stands."
            ),
            SettlingStop::AMountAtTheSlot { described } => format!(
                "a mount stands at the slot: {described}. Nothing was settled and nothing was changed. Ways forward: take the mount down through the wrapper's own unmount route, then run this command again."
            ),
            SettlingStop::TheMountTableCouldNotBeRead { said } => format!(
                "this process's mount table could not be read ({said}), so nothing proves the slot is unmounted and nothing was settled. Nothing was changed. Ways forward: inspect the mount table, then run the command again."
            ),
            SettlingStop::ThePointerNamesAnother { found } => format!(
                "the slot's `.git` holds `{found}`, which names another registration, so this is not this add's abandoned slot and nothing was settled. Nothing was changed. Ways forward: inspect the registration it names, and remove the entry by hand if it is not a worktree."
            ),
            SettlingStop::TheRegistrationIsAnother { recorded, live } => format!(
                "the registration the record names answers the identity `{live}` where the record holds `{recorded}`, so it is another folder's and nothing was settled. Nothing was changed. Ways forward: inspect the registration and the record; a registration removed and made again is another identity and no command of this wrapper settles across that."
            ),
            SettlingStop::TheRegistrationDoesNotAnswer { path, said } => format!(
                "the registration {} the record names cannot be read ({said}), so nothing was settled. Nothing was changed. Ways forward: inspect the folder, then run the command again.",
                path.display()
            ),
            SettlingStop::TheLockIsNotThisAdds { path } => format!(
                "{} holds words that are not this add's lock sentence, so it is a person's lock and nothing was settled. Nothing was changed. Ways forward: run `git worktree unlock` of the slot, which removes a lock a person set, then run this command again.",
                path.display()
            ),
            SettlingStop::TheOriginCleanupDisagrees { field, recorded, standing } => format!(
                "a standing origin cleanup record holds `{field}={standing}` where the binding holds `{recorded}`, so the two records name different objects and nothing was settled. Nothing was changed. Ways forward: inspect both records; the origin's cleanup stays the origin look's business, and no command of this wrapper removes a record another one disagrees with."
            ),
            SettlingStop::TheOriginCleanupCouldNotBeRead { path, said } => format!(
                "the origin cleanup record {} cannot be read ({said}), so nothing was settled. Nothing was changed. Ways forward: inspect the file, then run the command again.",
                path.display()
            ),
            SettlingStop::ThePlanCouldNotBeRead { path, said } => format!(
                "the destruction plan {} names this clone and cannot be read ({said}), so nothing was settled. Nothing was changed. Ways forward: inspect the file; a plan a person reads is a record of what a removal destroyed, and no command of this wrapper removes one it cannot read.",
                path.display()
            ),
            SettlingStop::ThePlanIsTheFinishPasss { path } => format!(
                "the destruction plan {} records its head as `gone`, which is the finish pass's case, so the settling command refuses it and nothing was changed. Ways forward: the finish pass removes that plan once the pool answers; nothing at this slot is the settling command's to do.",
                path.display()
            ),
            SettlingStop::ThePlanDisagrees { field, binding, plan } => format!(
                "the destruction plan holds `{field}={plan}` where the binding holds `{binding}`, so the two records name different objects and nothing was settled. Nothing was changed. Ways forward: inspect both records and the clone's own identities in the pool."
            ),
            SettlingStop::TheRecordCouldNotGo { path, said } => format!(
                "{} could not be removed ({said}), so the run stopped there. Ways forward: inspect the file and its folder's permissions, then run the command again; every removal this command makes is one a rerun repeats.",
                path.display()
            ),
            SettlingStop::TheRecipeStopped { said } => format!("the recipe that clears the registration stopped: {said} Nothing else was changed, and a rerun repeats the recipe."),
            SettlingStop::TheClaimCannotBeReleased { slot, said } => format!(
                "the records went and the claim on `{slot}` could not be released ({said}). The claim's own liveness rule lets the next command take the next generation, so nothing is blocked. Ways forward: inspect the record folder when the next command reports it."
            ),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_refusal_tests.rs"]
mod tests;
