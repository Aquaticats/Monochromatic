//! What:
//!  The managed Git lock of a ZFS clone worktree,
//!  after the add wrote it:
//!  the states `doctor` and the removal tell apart,
//!  the write that puts a lost lock back,
//!  and the one release the registration-cleanup step may make.
//! Why:
//!  The add writes the lock with its registration,
//!  and everything that happens to it afterwards is plan step 13's
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 13",
//!  and `doc/planning/cli-git-zfs-add-time-git-lock-design.md`).
//!  Its own step wires it in:
//!  the add's own resume asks the classification before it adopts a
//!  registration and writes a lost lock again
//!  (`worktree_clone_add_route::add_registration`).
//!  The release stays unreached until plan step 16's registration-cleanup step
//!  calls it,
//!  and `doctor`'s own lines are plan step 18's.
//!  The sentence a lock holds and the comparison that proves one the wrapper's
//!  landed in step 10 with the write path,
//!  so this folder reads them and restates none of them.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `reading_a_bounded_file.rs`:
//!   the bounded,
//!   no-follow read of a small file,
//!   and the causes its refusal names.
//! - `lexical_path_resolution.rs`:
//!   one path with `.` and `..` resolved,
//!   and nothing read from the filesystem.
//! - `lock_and_pointer_readings.rs`:
//!   one registration's `locked` and one slot's `.git`,
//!   each with its own bound.
//! - `binding_record_fields.rs`:
//!   a record's field lines,
//!   whatever phase the record stands at,
//!   and the refusals they owe.
//! - `recorded_lock_facts.rs`:
//!   the facts a lock's proof needs,
//!   read from a record whatever its phase or refusal.
//! - `dataset_guid_reads.rs`:
//!   the pool read the mounted arm compares a record's `cloneGuid=` with.
//! - `registration_proof_two_facts_refusals.rs`:
//!   which fact of the two-fact proof differed,
//!   and the sentence each answers.
//! - `registration_proof_two_facts.rs`:
//!   the proof of a registration that needs no pointer.
//! - `registration_entry_reading.rs`:
//!   one `worktrees/<name>` entry as the finder's facts.
//! - `registration_finding.rs`:
//!   the walk of a slot's candidate names,
//!   and the entry that attributes itself.
//! - `lock_state.rs`:
//!   the states S1 to S9,
//!   and the classes a caller acts on.
//! - `lock_state_words.rs`:
//!   the sentence each state prints.
//! - `lock_classification.rs`:
//!   the one order `doctor` and the removal share.
//! - `lock_installation.rs`:
//!   the exclusive write that puts a lost lock back,
//!   and the one line it prints.
//! - `clone_destruction_proof.rs`:
//!   the proof that the recorded clone is gone,
//!   as a value no caller can write.
//! - `lock_release.rs`:
//!   the release,
//!   open only to a caller holding that proof.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { classifyTheManagedLock, writeTheLostLock, proveTheRecordedCloneIsGone, releaseTheManagedLock } from '@monochromatic-dev/git-policy-cli/worktree-clone-managed-lock';
//! ```

pub mod binding_record_fields;
pub mod clone_destruction_proof;
pub mod dataset_guid_reads;
pub mod lexical_path_resolution;
pub mod lock_and_pointer_readings;
pub mod lock_classification;
pub mod lock_installation;
pub mod lock_release;
pub mod lock_state;
pub mod lock_state_words;
pub mod reading_a_bounded_file;
pub mod recorded_lock_facts;
pub mod registration_entry_reading;
pub mod registration_finding;
pub mod registration_proof_two_facts;
pub mod registration_proof_two_facts_refusals;

pub use binding_record_fields::RecordedFactsRefusal;
pub use clone_destruction_proof::{
    CloneDestructionProof,
    DestructionRefusal,
    TheLiveCloneAnswer,
    prove_the_recorded_clone_is_gone,
};
pub use dataset_guid_reads::DatasetGuidReads;
pub use lexical_path_resolution::the_lexical_path;
pub use lock_and_pointer_readings::{
    LOCK_FILE_NAME,
    SLOT_POINTER_NAME,
    lock_file_path,
    read_the_lock,
    read_the_slot_pointer,
    slot_pointer_path,
};
pub use lock_classification::{
    classify_the_managed_lock,
    classify_with_a_known_slot_identity,
};
pub use lock_installation::{
    InstallAnswer,
    line_of_an_installed_lock,
    write_the_lost_lock,
};
pub use lock_release::{
    ReleaseOutcome,
    ReleaseRefusal,
    release_the_managed_lock,
};
pub use lock_state::{
    Classification,
    LockOwnership,
    ManagedLockState,
    TheWordsCause,
};
pub use lock_state_words::{
    line_of,
    line_of_an_absent_pointer,
    the_lines,
};
pub use reading_a_bounded_file::{
    ARefusedReading,
    Reading,
    read_a_bounded_file,
};
pub use recorded_lock_facts::{
    RecordedClone,
    RecordedLockFacts,
    read_the_recorded_facts,
};
pub use registration_entry_reading::read_one_entry;
pub use registration_finding::{
    TheRegistration,
    WhatTheWalkFound,
    find_the_slot_registration,
    the_slots_the_home_names,
};
pub use registration_proof_two_facts::{
    GITDIR_FILE_NAME,
    prove_the_registration,
};
pub use registration_proof_two_facts_refusals::TwoFactRefusal;

/// The throwaway repository and slot the controls are built in.
#[cfg(test)]
pub mod test_fixtures;
