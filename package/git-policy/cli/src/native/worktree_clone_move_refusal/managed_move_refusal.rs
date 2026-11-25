//! What:
//!  The refusal of one `git worktree move` that would rename a managed clone's
//!  slot,
//!  mounted or not,
//!  and the sentence that says why the slot is fixed.
//! Why:
//!  The design refuses `git worktree move` for a bound managed clone whether it
//!  is mounted or not,
//!  and reports that the slot is fixed and must not be renamed
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The kernel refuses a move only while the clone is mounted
//!  (`Device or resource busy`),
//!  so an unmounted slot has no such protection and the wrapper is the only
//!  refusal there;
//!  a guest arm shows both.
//!  A record that cannot be read refuses too:
//!  nothing then proves the move would not rename a managed clone's slot,
//!  and the design's answer to an unreadable record is to act on nothing.
//!  The refusal reads only the binding record,
//!  so the mounted and unmounted arms are the same code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = refuseMoveOfABoundSlot(commonDir, sourcePath); // undefined when no record stands
//! ```

use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::registration_home::{home_folder, slot_of};
use crate::worktree_clone_slot_state::slot_state_readings::{BindingReading, read_binding_reading};
use std::path::{Path, PathBuf};

/// What:
///  Why one move is refused.
/// Why:
///  Each arm names a different thing a reader must go and look at,
///  and the sentence is built from it,
///  so the reason and the words a person reads cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagedMoveRefusalCause = { kind: 'bound'; phase; cloneDataset } | { kind: 'unreadable'; reason } | { kind: 'names-another-path'; recorded };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagedMoveRefusalCause {
    /// A binding record stands for the slot the move would rename.
    Bound {
        /// How far the record says the add got.
        phase: Phase,
        /// The clone dataset the record names.
        clone_dataset: String,
    },
    /// A record is filed under the slot's name and cannot be read.
    Unreadable {
        /// Why the record was not read.
        reason: String,
    },
    /// A record is filed under the slot's name and names another path.
    NamesAnotherPath {
        /// The slot path the record names.
        recorded: PathBuf,
    },
}

/// What:
///  One refused move:
///  its cause and the sentence the caller prints before Git runs.
/// Why:
///  The wrapper prints one sentence and exits,
///  and a control asserts the cause rather than matching prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagedMoveRefusal = { slot: string; cause: ManagedMoveRefusalCause; sentence: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedMoveRefusal {
    /// The slot the move would rename.
    pub slot: String,
    /// Why the move is refused.
    pub cause: ManagedMoveRefusalCause,
    /// The sentence a person reads.
    pub sentence: String,
}

/// What:
///  The sentence for a bound slot.
/// Why:
///  It says the slot is fixed,
///  why renaming the folder does not move the clone,
///  and what moving the work would need,
///  including that this release has no removal for a clone,
///  so a person is not left with a refusal and a command that does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = boundSentence('topic', 'pool/clones/topic', 'filled');
/// ```
fn bound_sentence(slot: &str, clone_dataset: &str, phase: Phase) -> String {
    return format!(
        "`git worktree move` is refused: the slot `{slot}` is fixed because a managed clone is bound to it at `phase={}`. The clone's dataset is `{clone_dataset}`, and this route derives a clone's slot from that dataset's own name, so renaming the folder does not move the clone and leaves the binding record and the Git registration naming different paths. Nothing was moved. Moving the work means removing the clone and settling its registration first, then adding the worktree where you want it; {}.",
        phase.word(),
        crate::worktree_clone_binding_record::NO_REMOVAL_WORDS
    );
}

/// What:
///  The sentence for a record no reader may act on.
/// Why:
///  A record that cannot be read may name this very slot,
///  so the refusal says what refused and what to read before trying again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = unreadableSentence('topic', 'permission denied');
/// ```
fn unreadable_sentence(slot: &str, reason: &str) -> String {
    return format!(
        "`git worktree move` is refused: a record is filed under the slot name `{slot}` and cannot be read ({reason}), so nothing proves this move would not rename a managed clone's slot. Nothing was moved. Read the record, restore it from a copy or settle it, then run the move again."
    );
}

/// What:
///  The sentence for a record whose slot path is not the move's source.
/// Why:
///  A record filed under the same leaf name for another path still leaves two
///  statements that disagree,
///  and a move cannot be proved safe while they do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = anotherPathSentence('topic', source, recorded);
/// ```
fn another_path_sentence(slot: &str, source: &Path, recorded: &Path) -> String {
    return format!(
        "`git worktree move` is refused: the record filed under the slot name `{slot}` names the slot {}, not {}, so the record and this path disagree and nothing proves this move would not rename a managed clone's slot. Nothing was moved. Read the record and run the move again once the record and the path agree.",
        recorded.display(),
        source.display()
    );
}

/// What:
///  Whether one `git worktree move` would rename a bound managed clone's slot,
///  and the sentence to print when it would.
/// Why:
///  The wrapper's `worktree move` handling calls this before Git runs,
///  and the answer depends only on the record filed under the source's slot
///  name,
///  which is what keeps the unmounted arm as covered as the mounted one.
///  A source whose last component is no plain slot name cannot hold a record,
///  so it is left to Git,
///  and a record that names another path still refuses,
///  because two disagreeing statements prove nothing.
///
/// # Errors
/// None:
///  every failure refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = refuseMoveOfABoundSlot(commonDir, source);
/// ```
pub fn refuse_move_of_a_bound_slot(common_dir: &Path, source: &Path) -> Option<ManagedMoveRefusal> {
    // `let Ok(x) = ... else` unwraps the slot name or leaves the move to Git: no record can be keyed by a path
    //  whose last component is no plain name.
    let Ok(slot) = slot_of(source) else {
        return None;
    };
    let home: PathBuf = home_folder(common_dir);
    let reading: BindingReading = read_binding_reading(home.as_path(), slot.as_str());
    let cause: ManagedMoveRefusalCause = match reading {
        BindingReading::Absent => return None,
        BindingReading::Unreadable { reason } => ManagedMoveRefusalCause::Unreadable { reason },
        BindingReading::Record(record) => match record.slot_path() {
            Some(recorded) if recorded == source => ManagedMoveRefusalCause::Bound { phase: record.phase, clone_dataset: record.clone_dataset.clone() },
            Some(recorded) => ManagedMoveRefusalCause::NamesAnotherPath { recorded },
            None => ManagedMoveRefusalCause::Unreadable { reason: String::from("the record's `slot=` is not a path, so the slot it belongs to is unproved") },
        },
    };
    let sentence: String = match &cause {
        ManagedMoveRefusalCause::Bound { phase, clone_dataset } => bound_sentence(slot.as_str(), clone_dataset.as_str(), *phase),
        ManagedMoveRefusalCause::Unreadable { reason } => unreadable_sentence(slot.as_str(), reason.as_str()),
        ManagedMoveRefusalCause::NamesAnotherPath { recorded } => another_path_sentence(slot.as_str(), source, recorded.as_path()),
    };
    return Some(ManagedMoveRefusal { slot, cause, sentence });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "managed_move_refusal_tests.rs"]
mod tests;
