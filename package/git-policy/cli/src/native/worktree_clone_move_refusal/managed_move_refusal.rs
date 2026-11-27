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
//!  A record filed under the slot's last name for another path refuses only
//!  when Git's own worktree list names no worktree at the move's source,
//!  because a plain worktree that happens to share a bound slot's last name is
//!  a worktree of its own and moving it renames nothing the route binds
//!  (`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`,
//!  "Moving a plain worktree that shares a bound slot's last name").
//!  The refusal reads only the binding record and Git's registration files,
//!  so the mounted and unmounted arms are the same code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = refuseMoveOfABoundSlot(commonDir, sourcePath); // undefined when no record stands
//! ```

use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::registration_home::{
    home_folder,
    slot_of,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    read_binding_reading,
};
use std::path::{
    Path,
    PathBuf,
};

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
    /// A record is filed under the slot's name,
    /// names another path,
    /// and Git's own list names no worktree at the move's source.
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
fn bound_sentence(
    slot: &str,
    clone_dataset: &str,
    phase: Phase,
) -> String {
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
fn unreadable_sentence(
    slot: &str,
    reason: &str,
) -> String {
    return format!(
        "`git worktree move` is refused: a record is filed under the slot name `{slot}` and cannot be read ({reason}), so nothing proves this move would not rename a managed clone's slot. Nothing was moved. Read the record, restore it from a copy or settle it, then run the move again."
    );
}

/// What:
///  The sentence for a record whose slot path is not the move's source and
///  whose source Git's list does not name.
/// Why:
///  A record filed under the same leaf name for another path still leaves two
///  statements that disagree,
///  and a move cannot be proved safe while they do.
///  A source Git's list does name is not this sentence's:
///  that move renames a worktree of its own and is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = anotherPathSentence('topic', source, recorded);
/// ```
fn another_path_sentence(
    slot: &str,
    source: &Path,
    recorded: &Path,
) -> String {
    return format!(
        "`git worktree move` is refused: the record filed under the slot name `{slot}` names the slot {}, not {}, and Git's own list for this repository names no worktree at this path, so nothing proves this move would not rename a managed clone's slot. Nothing was moved. Read the record and run the move again once the record and the path agree; a worktree that belongs to another checkout is Git's to answer.",
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
///  so it is left to Git.
///  A record that names another path refuses only when that path is not the
///  source and Git's own list names no worktree at the source:
///  the record's key is the slot's last name,
///  so a plain worktree that shares that name holds the same record without
///  being the slot,
/// and moving it renames nothing this route binds.
///  A record that names this very slot through another spelling is that slot's
///  record,
/// so the two paths are compared by the file they name and not only
///  by the bytes the record holds.
///
/// # Errors
/// None:
///  every failure refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = refuseMoveOfABoundSlot(commonDir, source);
/// ```
pub fn refuse_move_of_a_bound_slot(
    common_dir: &Path,
    source: &Path,
) -> Option<ManagedMoveRefusal> {
    // `let Ok(x) = ... else` unwraps the slot name or leaves the move to Git: no record can be keyed by a path
    //  whose last component is no plain name.
    let Ok(slot) = slot_of(source) else {
        return None;
    };
    let home: PathBuf = home_folder(common_dir);
    let reading: BindingReading = read_binding_reading(home.as_path(), slot.as_str());
    let cause: ManagedMoveRefusalCause = match reading {
        BindingReading::Absent => return None,
        BindingReading::Unreadable {
            reason,
        } => ManagedMoveRefusalCause::Unreadable {
            reason,
        },
        BindingReading::Record(record) => match record.slot_path() {
            Some(recorded) if names_the_recorded_slot(recorded.as_path(), source) => {
                ManagedMoveRefusalCause::Bound {
                    phase: record.phase,
                    clone_dataset: record.clone_dataset.clone(),
                }
            }
            Some(_recorded) if git_lists_a_worktree_at(common_dir, source) => return None,
            Some(recorded) => ManagedMoveRefusalCause::NamesAnotherPath {
                recorded,
            },
            None => ManagedMoveRefusalCause::Unreadable {
                reason: String::from(
                    "the record's `slot=` is not a path, so the slot it belongs to is unproved",
                ),
            },
        },
    };
    let sentence: String = match &cause {
        ManagedMoveRefusalCause::Bound {
            phase,
            clone_dataset,
        } => bound_sentence(slot.as_str(), clone_dataset.as_str(), *phase),
        ManagedMoveRefusalCause::Unreadable {
            reason,
        } => unreadable_sentence(slot.as_str(), reason.as_str()),
        ManagedMoveRefusalCause::NamesAnotherPath {
            recorded,
        } => another_path_sentence(slot.as_str(), source, recorded.as_path()),
    };
    return Some(ManagedMoveRefusal {
        slot,
        cause,
        sentence,
    });
}

/// What:
///  Whether a record's slot path and the path a move names are one folder.
/// Why:
///  The record is keyed by the slot's last name,
///  so the record's own slot path is what says which of two worktrees sharing
///  that name it belongs to,
///  and a slot reached through a symbolic link or a relative spelling is still
///  that slot:
///  reading the spelling as another path would leave the slot's own move
///  unrefused the moment a person named it the other way.
///  A path that is not there can be compared only as text,
///  which is the whole of the question then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = namesTheRecordedSlot(recorded, source);
/// ```
fn names_the_recorded_slot(
    recorded: &Path,
    source: &Path,
) -> bool {
    return recorded == source || crate::real_git_candidate::same_file(recorded, source);
}

/// What:
///  Whether Git's own worktree list of this repository names a worktree at
///  `source`.
/// Why:
///  A record filed under the slot's last name belongs to the worktree its own
///  slot path names,
///  so the question a move of another path asks is whether Git lists a
///  worktree of this repository at that path at all:
///  the registration's `gitdir` back-link and its administrative folder are what
///  Git's list holds,
///  and a source no list names is Git's to refuse or to move as it always does.
///  The typed operand is not asked here again:
///  `parse_move_source` already resolved it into `source` through the one operand
///  reading the removal shares,
///  and Git's suffix match answers nothing for a path spelling,
/// which is exactly
///  the spelling this allowance exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = registeredWorktree({ commonDir, target: source }) !== undefined;
/// ```
fn git_lists_a_worktree_at(
    common_dir: &Path,
    source: &Path,
) -> bool {
    return crate::worktree_remove_registry::registered_worktree(common_dir, source).is_ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "managed_move_refusal_tests.rs"]
mod tests;
