//! What:
//!  Controls of one slot's managed-lock line:
//!  the wrapper's verified lock,
//!  its cut-short twin,
//!  a person's words,
//!  no lock,
//!  the slot that shows no pointer beside it,
//!  no registration,
//!  a registration no binding of the slot explains,
//!  a half-made one,
//!  and another worktree's registration.
//! Why:
//!  Plan step 13's acceptance asks `doctor` to report each of these states with
//!  the exact words,
//!  each from a fixture,
//!  with a healthy slot as the control
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  `T9`).
//!  The exact words are the words module's,
//!  so a control here shows this check prints that module's sentence for the
//!  state it was handed,
//!  in the order a person reads it,
//!  and names the half of the pair the state is about.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkManagedLock({ slot: SLOT, classification: verified() })).toMatchObject({ kind: 'pass' });
//! expect(checkManagedLock({ slot: SLOT, classification: lost() })).toMatchObject({ kind: 'finding', field: 'lock' });
//! ```

use super::{ManagedLockFacts, check_managed_lock};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::worktree_clone_binding_record::EntryRow;
use crate::worktree_clone_binding_record::lock_words::LOCK_WORDS_MAXIMUM_BYTES;
use crate::worktree_clone_managed_lock::{ARefusedReading, Classification, ManagedLockState, Reading, TheWordsCause};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::PathBuf;

/// The slot every sentence below names.
const SLOT: &str = "one";

/// The add nonce every sentence below names.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The registration the sentences name.
const REGISTRATION: &str = "/home/user/Monochromatic/.git/worktrees/one";

/// One slot's name and the state a control writes.
fn facts(state: ManagedLockState, pointer: Reading) -> ManagedLockFacts {
    return ManagedLockFacts {
        slot: String::from(SLOT),
        classification: Classification { state, pointer },
    };
}

/// The nonce every sentence names.
fn nonce() -> RequestNonce {
    return RequestNonce::parse(NONCE).expect("the fixture nonce reads");
}

/// The registration path every sentence names.
fn registration() -> PathBuf {
    return PathBuf::from(REGISTRATION);
}

/// A pointer a healthy slot shows,
/// which is a `.git` file naming its registration.
fn the_slots_pointer() -> Reading {
    return Reading::Bytes(Vec::from(format!("gitdir: {REGISTRATION}\n").into_bytes()));
}

/// The wrapper's verified lock passes,
/// with the sentence that names the nonce and the settle,
/// and the missing pointer's line stays out.
#[test]
fn the_verified_lock_passes() {
    let answer: CheckOutcome = check_managed_lock(&facts(ManagedLockState::Managed { nonce: nonce(), registration: registration() }, the_slots_pointer()));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("the verified lock is the state the route intends: {answer:?}");
    };
    assert!(detail.contains("the clone slot one holds the managed Git lock its add wrote for add nonce 9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"), "{detail}");
    assert!(detail.contains("the settle releases it after the clone is destroyed"), "{detail}");
    assert!(!detail.contains("shows no .git"), "{detail}");
}

/// A lock a killed write cut short is a finding naming its bytes and the next
/// claim holder.
#[test]
fn a_lock_cut_short_is_a_finding() {
    let state: ManagedLockState = ManagedLockState::CutShort { nonce: nonce(), registration: registration(), at: 40, of: LOCK_WORDS_MAXIMUM_BYTES };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a lock cut short is not the verified one: {answer:?}");
    };
    assert_eq!(field, "lock");
    assert!(detail.contains(&format!("cut short at 40 of {LOCK_WORDS_MAXIMUM_BYTES} bytes")), "{detail}");
    assert!(detail.contains("the next holder of the slot's claim completes it"), "{detail}");
}

/// Words the add did not write are a finding naming the words and the command
/// that unlocks the slot.
#[test]
fn a_foreign_lock_is_a_finding() {
    let state: ManagedLockState = ManagedLockState::NotTheAddsWords { cause: TheWordsCause::AnothersWords { first_line: String::from("initializing") } };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a person's lock is not the add's: {answer:?}");
    };
    assert_eq!(field, "lock");
    assert!(detail.contains("words its add did not write (initializing)"), "{detail}");
    assert!(detail.contains("`git worktree unlock one`"), "{detail}");
}

/// A lock whose own read refused is the same state,
/// and the sentence names the refusal rather than calling it words.
#[test]
fn a_lock_the_read_refused_is_a_finding_naming_the_refusal() {
    let state: ManagedLockState = ManagedLockState::NotTheAddsWords { cause: TheWordsCause::TheReadRefused { cause: ARefusedReading::ALink } };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a lock nothing read is not the add's: {answer:?}");
    };
    assert_eq!(field, "lock");
    assert!(detail.contains("a symbolic link stands at the name"), "{detail}");
}

/// A lost lock is a finding naming the nonce and the command that writes it
/// again.
#[test]
fn a_lost_lock_is_a_finding() {
    let state: ManagedLockState = ManagedLockState::NoLock { nonce: nonce(), registration: registration() };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a lost lock is not the verified one: {answer:?}");
    };
    assert_eq!(field, "lock");
    assert!(detail.contains("holds no managed Git lock for add nonce 9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"), "{detail}");
    assert!(detail.contains("writes the lock again and prints one line saying it did"), "{detail}");
}

/// A slot that shows no pointer prints the missing pointer's sentence after the
/// lock's own,
/// in that order,
/// because the two are one answer a person reads.
#[test]
fn a_slot_with_no_pointer_prints_its_own_sentence_after_the_lock() {
    let state: ManagedLockState = ManagedLockState::NoLock { nonce: nonce(), registration: registration() };
    let answer: CheckOutcome = check_managed_lock(&facts(state, Reading::Absent));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a lost lock is not the verified one: {answer:?}");
    };
    assert_eq!(field, "lock");
    let lock_at: usize = detail.find("holds no managed Git lock").unwrap_or_else(|| panic!("the lock's own sentence is missing: {detail}"));
    let pointer_at: usize = detail.find("shows no .git").unwrap_or_else(|| panic!("the pointer's sentence is missing: {detail}"));
    assert!(lock_at < pointer_at, "the lock's sentence stands first: {detail}");
    assert!(detail.ends_with("the managed Git lock is what stops it"), "{detail}");
}

/// A registration the record names and the pool lost is a finding naming the
/// path and both ways on.
#[test]
fn a_lost_registration_is_a_finding() {
    let state: ManagedLockState = ManagedLockState::NoRegistration { nonce: nonce(), recorded: Some(registration()) };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a lost registration is not a fact that holds: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains(&format!("the binding names `{REGISTRATION}`")), "{detail}");
    assert!(detail.contains("an explicit re-registration at the commit a person names"), "{detail}");
}

/// A registration no binding of the slot explains is a finding that names what
/// was found.
#[test]
fn a_registration_no_binding_explains_is_a_finding() {
    let state: ManagedLockState = ManagedLockState::NoBindingExplains { found: String::from("the folder is not the recorded one") };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("an unexplained registration is not a fact that holds: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains("a registration no binding of this slot explains (the folder is not the recorded one)"), "{detail}");
    assert!(detail.contains("nothing here changes it"), "{detail}");
}

/// A half-made registration is a finding naming its entry and its row.
#[test]
fn a_half_made_registration_is_a_finding() {
    let state: ManagedLockState = ManagedLockState::HalfMade { nonce: nonce(), name: String::from("one"), row: EntryRow::LockOnly };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a half-made registration is not a fact that holds: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains("a half-made registration, one, for add nonce"), "{detail}");
    assert!(detail.contains("in row 2"), "{detail}");
    assert!(detail.contains("the add's own resume clears it"), "{detail}");
}

/// Another worktree's registration is a finding naming it.
#[test]
fn another_worktrees_registration_is_a_finding() {
    let state: ManagedLockState = ManagedLockState::SomebodyElsesWorktree { registration: PathBuf::from("/elsewhere/.git/worktrees/topic"), found: String::from("another path") };
    let answer: CheckOutcome = check_managed_lock(&facts(state, the_slots_pointer()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("another worktree's registration is not this slot's: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains("a registration that is not this slot's (`/elsewhere/.git/worktrees/topic`: another path)"), "{detail}");
}

/// The state decides the field,
/// so the lock's states and the registration's are told apart in the report,
/// and the healthy state is the only pass.
#[test]
fn the_field_follows_the_half_of_the_pair_the_state_is_about() {
    let lock_states: [ManagedLockState; 3] = [
        ManagedLockState::CutShort { nonce: nonce(), registration: registration(), at: 40, of: LOCK_WORDS_MAXIMUM_BYTES },
        ManagedLockState::NotTheAddsWords { cause: TheWordsCause::AnothersWords { first_line: String::from("initializing") } },
        ManagedLockState::NoLock { nonce: nonce(), registration: registration() },
    ];
    for state in lock_states {
        let answer: CheckOutcome = check_managed_lock(&facts(state.clone(), the_slots_pointer()));
        let CheckOutcome::Finding { field, .. } = answer else {
            panic!("{state:?} is not the verified lock: {answer:?}");
        };
        assert_eq!(field, "lock", "{state:?}");
    }
    let registration_states: [ManagedLockState; 4] = [
        ManagedLockState::NoRegistration { nonce: nonce(), recorded: Some(registration()) },
        ManagedLockState::NoBindingExplains { found: String::from("the folder is not the recorded one") },
        ManagedLockState::HalfMade { nonce: nonce(), name: String::from("one"), row: EntryRow::LockOnly },
        ManagedLockState::SomebodyElsesWorktree { registration: PathBuf::from("/elsewhere/.git/worktrees/topic"), found: String::from("another path") },
    ];
    for state in registration_states {
        let answer: CheckOutcome = check_managed_lock(&facts(state.clone(), the_slots_pointer()));
        let CheckOutcome::Finding { field, .. } = answer else {
            panic!("{state:?} is not the verified lock: {answer:?}");
        };
        assert_eq!(field, "registration", "{state:?}");
    }
}
