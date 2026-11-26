//! What:
//!  Controls for the words each state prints,
//!  which are the design's own sentences with the slot and the numbers filled in.
//! Why:
//!  `doctor` prints these on standard error with its exit status unchanged,
//!  and a person acts on the sentence:
//!  it must name the slot,
//!  the nonce,
//!  the command that unlocks,
//!  and the row a half-made registration is in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(rows(classification, 'slot')).toContain('the clone slot slot holds the managed Git lock');
//! ```

use super::{
    line_of,
    line_of_an_absent_pointer,
    the_lines,
};
use crate::worktree_clone_binding_record::EntryRow;
use crate::worktree_clone_binding_record::lock_words::LOCK_WORDS_MAXIMUM_BYTES;
use crate::worktree_clone_managed_lock::test_fixtures::SLOT;
use crate::worktree_clone_managed_lock::{
    ARefusedReading,
    Classification,
    ManagedLockState,
    Reading,
    TheWordsCause,
};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::PathBuf;

/// The nonce every sentence below names.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        .expect("the fixture nonce reads");
}

/// One line of a state.
fn line(state: &ManagedLockState) -> String {
    return line_of(state, SLOT);
}

/// S1 and S2 name the lock,
/// the nonce,
/// and what a settle and a next claim holder do with it.
#[test]
fn the_two_managed_states_name_their_holder() {
    let registration: PathBuf = PathBuf::from("/repository/.git/worktrees/locked-slot");
    let verified: String = line(&ManagedLockState::Managed {
        nonce: nonce(),
        registration: registration.clone(),
    });
    assert!(verified.contains("the clone slot locked-slot holds the managed Git lock its add wrote for add nonce 9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"), "{verified}");
    assert!(
        verified.contains("the settle releases it after the clone is destroyed"),
        "{verified}"
    );
    let cut: String = line(&ManagedLockState::CutShort {
        nonce: nonce(),
        registration,
        at: 40,
        of: LOCK_WORDS_MAXIMUM_BYTES,
    });
    assert!(
        cut.contains(&format!(
            "cut short at 40 of {LOCK_WORDS_MAXIMUM_BYTES} bytes"
        )),
        "{cut}"
    );
    assert!(
        cut.contains("the next holder of the slot's claim completes it"),
        "{cut}"
    );
}

/// S3 names the words found and the command a person runs,
/// and each cause reads as its own clause.
#[test]
fn the_foreign_state_names_the_words_and_the_unlock() {
    let person: String = line(&ManagedLockState::NotTheAddsWords {
        cause: TheWordsCause::AnothersWords {
            first_line: String::from("initializing"),
        },
    });
    assert!(
        person.contains("words its add did not write (initializing)"),
        "{person}"
    );
    assert!(
        person.contains(&format!("git worktree unlock {SLOT}")),
        "{person}"
    );
    let empty: String = line(&ManagedLockState::NotTheAddsWords {
        cause: TheWordsCause::AnothersWords {
            first_line: String::from("an empty file"),
        },
    });
    assert!(empty.contains("(an empty file)"), "{empty}");
    let link: String = line(&ManagedLockState::NotTheAddsWords {
        cause: TheWordsCause::TheReadRefused {
            cause: ARefusedReading::ALink,
        },
    });
    assert!(
        link.contains("a symbolic link stands at the name"),
        "{link}"
    );
    let no_record: String = line(&ManagedLockState::NotTheAddsWords {
        cause: TheWordsCause::NoRecordToRecompose {
            reason: String::from("the record is absent"),
        },
    });
    assert!(no_record.contains("the record is absent"), "{no_record}");
}

/// S4 names the nonce and the command that puts the lock back.
#[test]
fn the_unlocked_state_names_the_next_claim_holder() {
    let unlocked: String = line(&ManagedLockState::NoLock {
        nonce: nonce(),
        registration: PathBuf::from("/repository/.git/worktrees/locked-slot"),
    });
    assert!(
        unlocked.contains("holds no managed Git lock for add nonce"),
        "{unlocked}"
    );
    assert!(
        unlocked.contains("writes the lock again and prints one line saying it did"),
        "{unlocked}"
    );
}

/// S5 is one more sentence,
/// printed after the lock's own and only when the slot shows no `.git`.
#[test]
fn the_missing_pointer_is_a_sentence_of_its_own() {
    let sentence: String = line_of_an_absent_pointer(SLOT);
    assert!(sentence.contains("shows no .git"), "{sentence}");
    assert!(
        sentence.contains("the managed Git lock is what stops it"),
        "{sentence}"
    );
    let state: ManagedLockState = ManagedLockState::NoLock {
        nonce: nonce(),
        registration: PathBuf::from("/repository/.git/worktrees/locked-slot"),
    };
    let with_pointer: Vec<String> = the_lines(
        &Classification {
            state: state.clone(),
            pointer: Reading::Bytes(Vec::from(b"gitdir: /x\n")),
        },
        SLOT,
    );
    let without: Vec<String> = the_lines(
        &Classification {
            state,
            pointer: Reading::Absent,
        },
        SLOT,
    );
    assert_eq!(with_pointer.len(), 1);
    assert_eq!(without.len(), 2);
    assert_eq!(without[1], sentence);
}

/// S6 names what the binding records and both ways on.
#[test]
fn the_missing_registration_names_the_ways_on() {
    let recorded: String = line(&ManagedLockState::NoRegistration {
        nonce: nonce(),
        recorded: Some(PathBuf::from("/repository/.git/worktrees/locked-slot")),
    });
    assert!(
        recorded.contains("the binding names `/repository/.git/worktrees/locked-slot`"),
        "{recorded}"
    );
    assert!(
        recorded.contains("an explicit re-registration at the commit a person names"),
        "{recorded}"
    );
    let unknown: String = line(&ManagedLockState::NoRegistration {
        nonce: nonce(),
        recorded: None,
    });
    assert!(
        unknown.contains("the binding names no registration"),
        "{unknown}"
    );
}

/// S7 and S8 name what was found and the row,
/// and S9 names the registration that is not this slot's.
#[test]
fn the_unexplained_states_name_what_they_found() {
    let unexplained: String = line(&ManagedLockState::NoBindingExplains {
        found: String::from("the folder is not the recorded one"),
    });
    assert!(
        unexplained.contains(
            "a registration no binding of this slot explains (the folder is not the recorded one)"
        ),
        "{unexplained}"
    );
    assert!(
        unexplained.contains("nothing here changes it"),
        "{unexplained}"
    );
    let half: String = line(&ManagedLockState::HalfMade {
        nonce: nonce(),
        name: String::from("locked-slot"),
        row: EntryRow::LockOnly,
    });
    assert!(
        half.contains("a half-made registration, locked-slot"),
        "{half}"
    );
    assert!(half.contains("in row 2"), "{half}");
    assert!(
        half.contains("the add's own resume clears it with the binding's recipe"),
        "{half}"
    );
    let another: String = line(&ManagedLockState::SomebodyElsesWorktree {
        registration: PathBuf::from("/elsewhere/.git/worktrees/topic"),
        found: String::from("another path"),
    });
    assert!(
        another.contains("a registration that is not this slot's"),
        "{another}"
    );
    assert!(
        another.contains("/elsewhere/.git/worktrees/topic"),
        "{another}"
    );
}
