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
//!  The stops are one type here,
//!  and the sentence each prints sits in the file beside this one,
//!  so a reader can hold every way the command stops at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Stop = { kind: 'the-clone-still-stands'; clone: string } | ...;
//! throw new Error(stop.sentence());
//! ```

use std::path::PathBuf;

/// The sentence a person reads for each stop.
#[path = "settling_refusal_sentences.rs"]
mod settling_refusal_sentences;

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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_refusal_tests.rs"]
mod tests;
