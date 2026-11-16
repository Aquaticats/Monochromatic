//! Controls of one leaving's attempts:
//! a slot taken by one command only,
//! a slot a gone command left taken over by the next,
//! a made slot spent, a released slot free, and the budget answering nothing once every slot is taken.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// An owner that names a command which cannot be running:
/// process one is the kernel's own and its birth identity is never this word.
const A_COMMAND_THAT_IS_GONE: &str = "1:ended";

/// What:
///  The owner line of this very test process,
///  which a second command must treat as a command that may still be running.
/// Why:
///   A slot a live command took is its own attempt,
///  and the next command must take the next number rather than the slot it cannot prove gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = `${process.pid}:${currentBirthIdentity()}`;
/// ```
fn the_owner_of_this_command() -> String {
    let identity: String = crate::process_identity::current_birth_identity().expect("the identity of this command");
    return format!("{}:{identity}", std::process::id());
}

/// What:
///  A directory the controls take their slots in,
///  which is no repository and needs no filesystem of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const common = fixture('attempts'); // an empty directory
/// ```
fn a_common_directory(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("the common directory");
    return (root, common);
}

/// What:
///  Two commands take two different numbers,
///  because the reservation is published exclusively.
/// Why:
///  A number read and then written back would let both take the first one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(takeAnAttempt(common, id, 'a:1')).toBe(1); expect(takeAnAttempt(common, id, 'b:1')).toBe(2);
/// ```
#[test]
fn two_commands_take_two_numbers_and_a_live_slot_is_left_alone() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-two-numbers");
    let mine: String = the_owner_of_this_command();
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(1), "the first command takes the first number");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 1, "and one slot is taken");
    assert_eq!(
        take_an_attempt(common.as_path(), "the-leaving", "2:another-command-that-may-be-running"),
        Some(2),
        "a slot whose owner may still be running is that command's own, so the next number is taken",
    );
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 2, "and two slots are taken");
    remove(root.as_path());
}

/// What:
///  A slot a command took and never made is taken over by the next command,
///  which keeps the number the dead command left.
/// Why:
///   A kill between the slot and the act must cost the act's lateness and never the delete:
///  the budget must not be spent by an attempt nobody made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(takeAnAttempt(common, id, '1:ended')).toBe(1);
/// expect(takeAnAttempt(common, id, mine)).toBe(1);   // taken over, not a new attempt
/// expect(attemptsTaken(common, id)).toBe(1);
/// ```
#[test]
fn a_slot_a_gone_command_left_is_taken_over_with_the_same_number() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-taken-over");
    let mine: String = the_owner_of_this_command();
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", A_COMMAND_THAT_IS_GONE), Some(1), "the command that died took the first number");
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(1), "the next command takes that attempt over");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 1, "and the budget stands at one attempt, not two");
    mark_the_attempt_made(common.as_path(), "the-leaving", 1);
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(2), "a made attempt is spent, so the next number is taken");
    remove(root.as_path());
}

/// What:
///  A released slot is free again for the next command,
///  and a made slot never is.
/// Why:
///   An act that failed costs nothing,
///  and an act that was made is what the budget counts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(takeAnAttempt(common, id, mine)).toBe(1); releaseTheAttempt(common, id, 1);
/// expect(takeAnAttempt(common, id, mine)).toBe(1);
/// markTheAttemptMade(common, id, 1); expect(takeAnAttempt(common, id, mine)).toBe(2);
/// ```
#[test]
fn a_released_slot_is_free_and_a_made_slot_is_spent() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-released");
    let mine: String = the_owner_of_this_command();
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(1), "the first number is taken");
    release_the_attempt(common.as_path(), "the-leaving", 1);
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 0, "a released slot is no attempt");
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(1), "so the first number is free again");
    mark_the_attempt_made(common.as_path(), "the-leaving", 1);
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 1, "a made attempt is spent");
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(2), "and the next number is taken");
    remove(root.as_path());
}

/// What:
///  Every number taken answers nothing,
///  and forgetting the leaving frees all of them.
/// Why:
///   The budget is the set of numbers,
///  so a command that finds them all taken acts no more,
///  and the next leaving of the same place starts with all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const n of [1, 2, 3]) expect(takeAnAttempt(common, id, mine)).toBe(n);
/// expect(takeAnAttempt(common, id, mine)).toBeUndefined(); forgetTheAttempts(common, id);
/// expect(takeAnAttempt(common, id, mine)).toBe(1);
/// ```
#[test]
fn every_number_taken_answers_nothing_until_the_leaving_is_forgotten() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-budget");
    let mine: String = the_owner_of_this_command();
    for number in 1..=THE_BUDGET {
        assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(number), "the budget is taken number by number");
    }
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), None, "a command that finds every number taken acts no more");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), THE_BUDGET, "and the budget stands at its bound");
    forget_the_attempts(common.as_path(), "the-leaving");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 0, "forgetting the leaving frees every number");
    assert_eq!(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()), Some(1), "and the first is free again");
    remove(root.as_path());
}
