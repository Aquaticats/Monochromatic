//! What:
//!  The one function `doctor` and the removal call to tell the wrapper's managed Git lock from a person's
//!  and from a lock nothing explains.
//! Why:
//!  The design says the classification is one function with the states of "How doctor and removal tell the states
//!  apart" as its answer,
//!  so no caller invents its own order
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "How doctor and removal tell the states apart" and
//!  "Where the code goes").
//!  The order is the design's:
//!  the record's nonce,
//!  then the registration by `registration=` or by the finder,
//!  then the lock's bytes,
//!  and the answer is the state the three make.
//!  A record that gave no nonce makes every lock S3,
//!  because no sentence can be recomposed to compare,
//!  and a registration the record does not explain is S7 or S9 rather than a guess.
//!  Nothing here writes,
//!  removes or releases:
//!  the install is [`super::lock_installation`],
//!  the release is [`super::lock_release`],
//!  and both need a caller that holds the slot's claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const classification = classifyTheManagedLock(commonDir, slot, slotPath, mounts);
//! // { state: { kind: 'managed' | ... }, pointer }
//! ```
//!
//! The mount table is the caller's,
//! because the slot's identity is compared at the recorded path only while nothing
//! covers it,
//! and a mount covering the slot is compared through the clone the record names
//! (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//! "What proves ownership",
//! and measurement `M2`).

use super::lock_and_pointer_readings::{read_the_lock, read_the_slot_pointer};
use super::lock_state::{Classification, ManagedLockState, TheWordsCause};
use super::reading_a_bounded_file::Reading;
use super::recorded_lock_facts::{RecordedLockFacts, read_the_recorded_facts};
use super::registration_finding::{TheRegistration, find_the_slot_registration};
use super::registration_proof_two_facts::{TwoFactRefusal, prove_the_registration};
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::lock_words::{LOCK_WORDS_MAXIMUM_BYTES, LockStanding, lock_standing_of};
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_binding_record::registration_resume::EntryDecision;
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::path::{Path, PathBuf};

/// The first line of one lock's bytes,
/// or the words an empty file earns,
/// which is what S3's parenthetical prints.
fn first_line_of(bytes: &[u8]) -> String {
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes);
    let line: &str = text.lines().next().unwrap_or("");
    if line.is_empty() {
        return String::from("an empty file, which is also what a person's `git worktree lock` with no reason leaves");
    }
    return String::from(line);
}

/// What:
///  The state of a lock in a registration the two-fact proof has just proved.
/// Why:
///  The lock's bytes are the last fact:
///  the whole sentence is S1,
///  a covering prefix is S2,
///  no file is S4,
///  and anything else is S3,
///  with the read's own refusal named rather than folded into a person's words.
fn state_of_a_proved_registration(registration: &Path, nonce: &RequestNonce) -> ManagedLockState {
    return match read_the_lock(registration) {
        Reading::Absent => ManagedLockState::NoLock { nonce: nonce.clone(), registration: registration.to_path_buf() },
        Reading::Refused(cause) => ManagedLockState::NotTheAddsWords { cause: TheWordsCause::TheReadRefused { cause } },
        Reading::Bytes(bytes) => match lock_standing_of(bytes.as_slice(), nonce) {
            LockStanding::Verified => ManagedLockState::Managed { nonce: nonce.clone(), registration: registration.to_path_buf() },
            LockStanding::CutShort { at } => ManagedLockState::CutShort { nonce: nonce.clone(), registration: registration.to_path_buf(), at, of: LOCK_WORDS_MAXIMUM_BYTES },
            LockStanding::Foreign => ManagedLockState::NotTheAddsWords { cause: TheWordsCause::AnothersWords { first_line: first_line_of(bytes.as_slice()) } },
        },
    };
}

/// The state one refused two-fact proof leaves,
/// which is never the wrapper's own lock.
fn state_of_a_refused_proof(registration: &Path, refusal: &TwoFactRefusal) -> ManagedLockState {
    return match refusal {
        TwoFactRefusal::GitdirNamesAnotherPath { .. } | TwoFactRefusal::CommondirNamesAnotherRepository { .. } => ManagedLockState::SomebodyElsesWorktree {
            registration: registration.to_path_buf(),
            found: refusal.words(registration),
        },
        TwoFactRefusal::NoGitdir
        | TwoFactRefusal::TheSlotIsNotTheRecordedFolder { .. }
        | TwoFactRefusal::AMountThatIsNotTheRecords { .. }
        | TwoFactRefusal::TheMountTableWasNotRead { .. }
        | TwoFactRefusal::NoRecordedSlotIdentity
        | TwoFactRefusal::Unreadable { .. } => ManagedLockState::NoBindingExplains { found: refusal.words(registration) },
    };
}

/// What:
///  The state of a registration the record names,
///  or nothing when the finder must answer instead.
/// Why:
///  A record's `registration=` names the entry the add wrote,
///  and the proof decides whether it is still this slot's;
///  a registration with no `gitdir` file is no complete entry,
///  which the finder's rows name better than this path can.
fn state_of_the_recorded_registration(facts: &RecordedLockFacts, repository: &Path, registration: &Path, mounts: &dyn MountedDatasets) -> Option<ManagedLockState> {
    return match prove_the_registration(facts, repository, registration, mounts) {
        Ok(()) => Some(state_of_a_proved_registration(registration, &facts.nonce)),
        Err(TwoFactRefusal::NoGitdir) => None,
        Err(refusal) => Some(state_of_a_refused_proof(registration, &refusal)),
    };
}

/// What:
///  The state one finder decision leaves,
///  with the registration proved before the lock is read.
/// Why:
///  A complete entry is the add's only when the proof agrees,
///  and a half-made one is the state S8 rather than a lock's state,
///  because the add's own resume clears it.
fn state_of_a_found_entry(name: &str, path: &Path, decision: &EntryDecision, facts: &RecordedLockFacts, repository: &Path, mounts: &dyn MountedDatasets) -> ManagedLockState {
    return match decision {
        EntryDecision::Clear { row } => ManagedLockState::HalfMade { nonce: facts.nonce.clone(), name: String::from(name), row: row.clone() },
        EntryDecision::Adopt => match prove_the_registration(facts, repository, path, mounts) {
            Ok(()) => state_of_a_proved_registration(path, &facts.nonce),
            Err(refusal) => state_of_a_refused_proof(path, &refusal),
        },
        // A refusal over an entry with no back-link names the half-made row it is,
        // because the lock's own state cannot be read without one.
        EntryDecision::Refuse { row, .. } => match prove_the_registration(facts, repository, path, mounts) {
            Err(TwoFactRefusal::NoGitdir) => ManagedLockState::HalfMade { nonce: facts.nonce.clone(), name: String::from(name), row: row.clone() },
            Ok(()) => state_of_a_proved_registration(path, &facts.nonce),
            Err(refusal) => state_of_a_refused_proof(path, &refusal),
        },
        EntryDecision::Leave { row } => ManagedLockState::NoBindingExplains { found: format!("the finder left the entry at row {row:?}") },
    };
}

/// What:
///  The state the finder's walk of this slot's candidate names leaves.
/// Why:
///  The finder is the binding's,
///  so its adoption,
///  its rows and its two-match refusal read here as they read there,
///  and a walk that found nothing is the state S6 with the path the record named.
fn state_of_the_found_registration(repository: &Path, slot: &str, slot_path: &Path, facts: &RecordedLockFacts, mounts: &dyn MountedDatasets) -> ManagedLockState {
    let finding = find_the_slot_registration(repository, slot, slot_path, &facts.nonce);
    return match finding.registration {
        TheRegistration::One { name, path, decision } => state_of_a_found_entry(name.as_str(), path.as_path(), &decision, facts, repository, mounts),
        TheRegistration::Two { names } => ManagedLockState::NoBindingExplains {
            found: format!("two entries attribute themselves to this add (`{}`), and only a copy of the lock's sentence could make two", names.join("`, `")),
        },
        TheRegistration::None => ManagedLockState::NoRegistration { nonce: facts.nonce.clone(), recorded: facts.registration.clone() },
    };
}

/// What:
///  Classify one slot's managed Git lock,
///  with the slot's pointer beside it.
/// Why:
///  This is the one order `doctor` and the removal share,
///  and its answer carries the words each prints,
///  so the two cannot disagree about whether a lock is the wrapper's own.
///  The slot's path is the caller's,
///  because a record that cannot be read still leaves a slot to look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const classification = classifyTheManagedLock(commonDir, slot, slotPath, mounts);
/// ```
pub fn classify_the_managed_lock(repository: &Path, slot: &str, slot_path: &Path, mounts: &dyn MountedDatasets) -> Classification {
    return classify_with_a_known_slot_identity(repository, slot, slot_path, mounts, None);
}

/// What:
///  The same classification,
///  with the slot folder's identity the caller already holds,
///  for a record that has not written its own yet.
/// Why:
///  The add records the slot's identity in a write of its own,
///  and one whose add was killed before that write stands at phase `claimed`,
///  so the resume that made the folder is the one caller that knows the identity
///  the proof compares.
///  Every other reader passes nothing,
///  and a record with no identity is then refused rather than guessed at.
pub fn classify_with_a_known_slot_identity(repository: &Path, slot: &str, slot_path: &Path, mounts: &dyn MountedDatasets, known_slot_identity: Option<FolderIdentity>) -> Classification {
    let pointer: Reading = read_the_slot_pointer(slot_path);
    let mut facts: RecordedLockFacts = match read_the_recorded_facts(repository, slot) {
        Ok(found) => found,
        Err(refusal) => {
            let home: PathBuf = home_folder(repository);
            let cause: TheWordsCause = TheWordsCause::NoRecordToRecompose { reason: refusal.words(slot, home.as_path()) };
            return Classification { state: ManagedLockState::NotTheAddsWords { cause }, pointer };
        }
    };
    if facts.slot_identity.is_none() {
        facts.slot_identity = known_slot_identity;
    }
    let state: ManagedLockState = match facts.registration.as_ref() {
        Some(recorded) if recorded.is_dir() => match state_of_the_recorded_registration(&facts, repository, recorded.as_path(), mounts) {
            Some(answered) => answered,
            None => state_of_the_found_registration(repository, slot, slot_path, &facts, mounts),
        },
        Some(_) | None => state_of_the_found_registration(repository, slot, slot_path, &facts, mounts),
    };
    return Classification { state, pointer };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lock_classification_tests.rs"]
mod tests;
