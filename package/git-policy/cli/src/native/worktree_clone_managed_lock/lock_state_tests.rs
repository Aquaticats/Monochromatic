//! What:
//!  Controls for the classes a caller acts on:
//!  the wrapper's managed lock,
//!  a person's lock,
//!  an unexplained lock,
//!  no lock,
//!  and no registration.
//! Why:
//!  These five are what `doctor` and the removal branch on,
//!  so each state must fall in exactly one class and the slot's pointer must travel beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(classification.ownership()).toBe('the-wrappers-managed');
//! ```

use super::{
    Classification,
    LockOwnership,
    ManagedLockState,
    TheWordsCause,
};
use crate::worktree_clone_binding_record::EntryRow;
use crate::worktree_clone_managed_lock::ARefusedReading;
use crate::worktree_clone_managed_lock::Reading;
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::PathBuf;

/// The nonce every state below is built from.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        .expect("the fixture nonce reads");
}

/// One classification of a state,
/// with the slot's pointer standing.
fn standing(state: ManagedLockState) -> Classification {
    return Classification {
        state,
        pointer: Reading::Bytes(Vec::from(b"gitdir: /somewhere\n")),
    };
}

/// The two states of the add's own lock are the managed class.
#[test]
fn the_adds_own_lock_is_the_managed_class() {
    let registration: PathBuf = PathBuf::from("/repository/.git/worktrees/locked-slot");
    let verified: ManagedLockState = ManagedLockState::Managed {
        nonce: nonce(),
        registration: registration.clone(),
    };
    let cut: ManagedLockState = ManagedLockState::CutShort {
        nonce: nonce(),
        registration,
        at: 40,
        of: 149,
    };
    assert_eq!(
        standing(verified).ownership(),
        LockOwnership::TheWrappersManaged
    );
    assert_eq!(standing(cut).ownership(), LockOwnership::TheWrappersManaged);
}

/// Words this add did not write are a person's lock,
/// whatever the file holds.
#[test]
fn anothers_words_are_a_persons_lock() {
    let states: [ManagedLockState; 3] = [
        ManagedLockState::NotTheAddsWords {
            cause: TheWordsCause::AnothersWords {
                first_line: String::from("initializing"),
            },
        },
        ManagedLockState::NotTheAddsWords {
            cause: TheWordsCause::NoRecordToRecompose {
                reason: String::from("the record is absent"),
            },
        },
        ManagedLockState::NotTheAddsWords {
            cause: TheWordsCause::TheReadRefused {
                cause: ARefusedReading::ALink,
            },
        },
    ];
    for state in states {
        assert_eq!(standing(state).ownership(), LockOwnership::AnOperatorsLock);
    }
}

/// A registration nothing explains is the unexplained class,
/// and so is a half-made one and one that is another worktree's.
#[test]
fn nothing_explained_is_the_unexplained_class() {
    let states: [ManagedLockState; 3] = [
        ManagedLockState::NoBindingExplains {
            found: String::from("the folder is not the recorded one"),
        },
        ManagedLockState::HalfMade {
            nonce: nonce(),
            name: String::from("locked-slot"),
            row: EntryRow::LockOnly,
        },
        ManagedLockState::SomebodyElsesWorktree {
            registration: PathBuf::from("/elsewhere/.git/worktrees/topic"),
            found: String::from("another path"),
        },
    ];
    for state in states {
        assert_eq!(standing(state).ownership(), LockOwnership::Unexplained);
    }
}

/// A registration with no lock is its own class,
/// and one with no registration is not a lock's question at all.
#[test]
fn no_lock_and_no_registration_are_their_own_classes() {
    let registration: PathBuf = PathBuf::from("/repository/.git/worktrees/locked-slot");
    assert_eq!(
        standing(ManagedLockState::NoLock {
            nonce: nonce(),
            registration
        })
        .ownership(),
        LockOwnership::NoLockStands
    );
    assert_eq!(
        standing(ManagedLockState::NoRegistration {
            nonce: nonce(),
            recorded: Some(PathBuf::from("/repository/.git/worktrees/locked-slot"))
        })
        .ownership(),
        LockOwnership::NoRegistrationStands
    );
    assert_eq!(
        standing(ManagedLockState::NoRegistration {
            nonce: nonce(),
            recorded: None
        })
        .ownership(),
        LockOwnership::NoRegistrationStands
    );
}

/// A missing pointer is a fact beside the state,
/// and it is only ever the absence of the file.
#[test]
fn a_missing_pointer_travels_beside_the_state() {
    let state: ManagedLockState = ManagedLockState::NoRegistration {
        nonce: nonce(),
        recorded: None,
    };
    let absent: Classification = Classification {
        state: state.clone(),
        pointer: Reading::Absent,
    };
    let refused: Classification = Classification {
        state,
        pointer: Reading::Refused(ARefusedReading::ADirectory),
    };
    assert!(absent.the_slot_shows_no_pointer());
    assert!(!refused.the_slot_shows_no_pointer());
}
