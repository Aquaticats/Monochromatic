//! What:
//!  The sentences `doctor` prints for each state of a slot's managed Git lock.
//! Why:
//!  The design fixes the words of S1 to S8,
//!  and a person reads them on standard error with `doctor`'s exit status unchanged
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The states, the checks and the words",
//!  and "Where the code goes").
//!  The words live beside the states so no caller writes its own,
//!  and the state S5 is printed after the lock's own line rather than instead of it.
//!  The design gives S9 no sentence,
//!  only that it is printed under the slot it was found for and that nothing is changed,
//!  so this module writes that one,
//!  which the project may reword.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = rows(classification, slot); // [the lock's line, and S5's line when the slot shows no .git]
//! ```

use super::lock_state::{
    Classification,
    ManagedLockState,
    TheWordsCause,
};
use crate::worktree_clone_binding_record::EntryRow;

/// The words of the lock's own cause in S3,
/// as the parenthetical its sentence carries.
fn words_of_the_cause(cause: &TheWordsCause) -> String {
    return match cause {
        TheWordsCause::NoRecordToRecompose {
            reason,
        } => format!(
            "no sentence can be recomposed because the slot's record was not read: {reason}"
        ),
        TheWordsCause::AnothersWords {
            first_line,
        } => first_line.clone(),
        TheWordsCause::TheReadRefused {
            cause: refusal,
        } => refusal.words(),
    };
}

/// What:
///  The binding design's row one finder row answers,
///  which S8's sentence names.
/// Why:
///  The lock design names the rows of
///  `doc/planning/cli-git-zfs-add-binding-record-design.md`,
///  "The states Git's source can leave",
///  and the merged finder's own row words are its reading of them.
///  Rows 1 to 5 are the entries the recipe clears in that order,
///  a complete registration is adopted rather than named by a row,
///  and every other incomplete shape is row 8's,
///  the kill inside Git's own removal of a failed add.
fn row_number(row: &EntryRow) -> u8 {
    return match row {
        EntryRow::Bare => 1,
        EntryRow::LockOnly => 2,
        EntryRow::GitdirOnly => 3,
        EntryRow::PointerOnly => 4,
        EntryRow::Commondir => 5,
        EntryRow::Complete
        | EntryRow::ForeignLock
        | EntryRow::AnotherSlotsRegistration
        | EntryRow::OwnedByAnotherSlot
        | EntryRow::NotACandidate => 8,
    };
}

/// The line one lock state prints.
pub fn line_of(
    state: &ManagedLockState,
    slot: &str,
) -> String {
    return match state {
        ManagedLockState::Managed {
            nonce,
            ..
        } => format!(
            "the clone slot {slot} holds the managed Git lock its add wrote for add nonce {nonce}, so `git worktree prune` keeps its registration; the settle releases it after the clone is destroyed",
            nonce = nonce.as_str()
        ),
        ManagedLockState::CutShort {
            nonce,
            at,
            of,
            ..
        } => format!(
            "the clone slot {slot} holds the managed Git lock its add wrote for add nonce {nonce}, cut short at {at} of {of} bytes, and it still keeps the registration; the next holder of the slot's claim completes it",
            nonce = nonce.as_str()
        ),
        ManagedLockState::NotTheAddsWords {
            cause,
        } => format!(
            "the clone slot {slot} is locked by words its add did not write ({}), so the wrapper will not release it, and a removal refuses until a person unlocks it with `git worktree unlock {slot}`",
            words_of_the_cause(cause)
        ),
        ManagedLockState::NoLock {
            nonce,
            ..
        } => format!(
            "the clone slot {slot} holds no managed Git lock for add nonce {nonce}, so `git worktree prune` can remove its registration while the slot shows no .git and the registration has no index; the next command that holds the slot's claim writes the lock again and prints one line saying it did",
            nonce = nonce.as_str()
        ),
        ManagedLockState::NoRegistration {
            recorded,
            ..
        } => {
            let named: String = match recorded {
                Some(path) => format!("`{}`", path.display()),
                None => String::from("no registration"),
            };
            format!(
                "the clone slot {slot} has no Git registration, and `git worktree repair` cannot rebuild one once it is gone; the binding names {named}, and the ways on are an explicit re-registration at the commit a person names, or the settle against the binding"
            )
        }
        ManagedLockState::NoBindingExplains {
            found,
        } => format!(
            "the clone slot {slot} holds a registration no binding of this slot explains ({found}), so it is not this wrapper's worktree, and nothing here changes it"
        ),
        ManagedLockState::HalfMade {
            nonce,
            name,
            row,
        } => format!(
            "the worktrees directory holds a half-made registration, {name}, for add nonce {nonce}, in row {row}; while a lock stands no prune removes it, and `git worktree list` does not show the ones whose only entry is `locked`; the add's own resume clears it with the binding's recipe and runs its Git step again",
            nonce = nonce.as_str(),
            row = row_number(row)
        ),
        ManagedLockState::SomebodyElsesWorktree {
            registration,
            found,
        } => format!(
            "the clone slot {slot} names a registration that is not this slot's (`{}`: {found}), so its lock is whatever it is, and nothing here changes it",
            registration.display()
        ),
    };
}

/// What:
///  The line the absent pointer prints,
///  which stands after the lock's own.
/// Why:
///  `git worktree prune` removes a slot with no `.git` at once while the registration has no index,
///  and the managed lock is what stops it,
///  so the missing pointer is one more sentence a person reads rather than a state that replaces the lock's.
pub fn line_of_an_absent_pointer(slot: &str) -> String {
    return format!(
        "the clone slot {slot} shows no .git, so `git worktree prune` can remove its registration at once while it has no index; the managed Git lock is what stops it"
    );
}

/// What:
///  Every line one classification prints,
///  in the order a person reads them.
/// Why:
///  `doctor` prints these and changes nothing,
///  and the removal's refusals quote them,
///  so one function gives both the same sentences.
pub fn the_lines(
    classification: &Classification,
    slot: &str,
) -> Vec<String> {
    let mut lines: Vec<String> = vec![line_of(&classification.state, slot)];
    if classification.the_slot_shows_no_pointer() {
        lines.push(line_of_an_absent_pointer(slot));
    }
    return lines;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lock_state_words_tests.rs"]
mod tests;
