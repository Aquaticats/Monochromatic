//! Controls of one leaving's attempts:
//! a slot taken by one command only,
//! a slot a gone command left taken over by the next,
//! a slot two commands find dead at once taken over by exactly one,
//! a number a command marked made never taken again,
//! a released slot free, a slot that cannot be read answered as no count,
//! and the budget spent only by attempts that were made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('the attempts one leaving may cost', () => { /* one case per name below */ });
//! ```

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt as _;
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
///  The number one slot was taken for,
///  whichever ending the call gave,
///  so a control can say what it expected in one line.
/// Why:
///   A call answers three things,
///  and every control here is about the number one of them carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numberOf = (attempt) => attempt.kind === 'took' ? attempt.slot.number : attempt.kind;
/// ```
fn number_of(attempt: &Attempt) -> String {
    return match attempt {
        Attempt::Took(slot) => slot.number.to_string(),
        Attempt::Spent => String::from("spent"),
        Attempt::Held => String::from("held"),
        Attempt::Unreadable(reason) => format!("unreadable ({reason})"),
    };
}

/// What:
///  The number an attempt was taken for,
///  announcing an ending that is not a slot as a failure of the control.
/// Why:
///   Every control that is not about an unreadable or a spent answer needs the slot itself,
///  and the words say which ending came instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slotOf = (attempt) => { if (attempt.kind !== 'took') throw new Error(attempt.kind); return attempt.slot; };
/// ```
fn the_slot_of(attempt: Attempt) -> Slot {
    return match attempt {
        Attempt::Took(slot) => slot,
        Attempt::Spent => panic!("the control expected a slot and every attempt was made"),
        Attempt::Held => panic!("the control expected a slot and every number is another command's attempt"),
        Attempt::Unreadable(reason) => panic!("the control expected a slot and nothing could be read: {reason}"),
    };
}

/// What:
///  Two commands take two different numbers,
///  because the reservation is published exclusively.
/// Why:
///  A number read and then written back would let both take the first one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(numberOf(takeAnAttempt(common, id, mine))).toBe('1'); expect(numberOf(takeAnAttempt(common, id, '2:linux:1'))).toBe('2');
/// ```
#[test]
fn two_commands_take_two_numbers_and_a_live_slot_is_left_alone() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-two-numbers");
    let mine: String = the_owner_of_this_command();
    assert_eq!(number_of(&take_an_attempt(common.as_path(), "the-leaving", mine.as_str())), "1", "the first command takes the first number");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 1, "and one slot is taken");
    assert_eq!(
        number_of(&take_an_attempt(common.as_path(), "the-leaving", "2:another-command-that-may-be-running")),
        "2",
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
/// expect(numberOf(takeAnAttempt(common, id, '1:ended'))).toBe('1');
/// expect(numberOf(takeAnAttempt(common, id, mine))).toBe('1');   // taken over, not a new attempt
/// expect(attemptsTaken(common, id)).toBe(1);
/// ```
#[test]
fn a_slot_a_gone_command_left_is_taken_over_with_the_same_number() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-taken-over");
    let mine: String = the_owner_of_this_command();
    assert_eq!(number_of(&take_an_attempt(common.as_path(), "the-leaving", A_COMMAND_THAT_IS_GONE)), "1", "the command that died took the first number");
    let taken_over: Slot = the_slot_of(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()));
    assert_eq!(taken_over.number, 1, "the next command takes that attempt over");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 1, "and the budget stands at one attempt, not two");
    mark_the_attempt_made(&taken_over);
    assert_eq!(number_of(&take_an_attempt(common.as_path(), "the-leaving", mine.as_str())), "2", "a made attempt is spent, so the next number is taken");
    remove(root.as_path());
}

/// What:
///  Two commands that find the same dead slot at the same moment never both take it over.
/// Why:
///  The takeover is one rename of the slot's own name with `RENAME_NOREPLACE`,
///  and the loser's rename cannot succeed once the winner's has moved the name away,
///  so exactly one command makes the attempt.
///  A protocol that publishes each command's own line over the file and reads it back lets both read their own
///  line and both act on one number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantDead('reserved-<id>.1');
/// const [a, b] = await Promise.all([takeAnAttempt(common, id, ownerA), takeAnAttempt(common, id, ownerB)]);
/// expect(new Set([numberOf(a), numberOf(b)]).size).toBe(2);
/// ```
#[test]
fn two_commands_that_find_one_dead_slot_at_once_take_two_different_numbers() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-dead-slot-race");
    let id: &str = "the-leaving";
    let mine: String = the_owner_of_this_command();
    // The same command's line in both threads: what decides the race is the rename, and not whose name stands there.
    let theirs: String = mine.clone();
    // What: The slot a command killed between its publish and its act leaves behind.
    // Why:  A reservation holds the line of the command that published it, and that command is gone.
    let attempts: PathBuf = common.join(ATTEMPTS_FOLDER);
    std::fs::create_dir_all(attempts.as_path()).expect("the attempts folder");
    write_private_file_unsynced(reserved_path(attempts.as_path(), id, 1).as_path(), A_COMMAND_THAT_IS_GONE.as_bytes()).expect("the dead reservation");
    let start: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(3));
    let numbers: Vec<String> = std::thread::scope(|_scope| {
        let mut handles: Vec<std::thread::JoinHandle<Attempt>> = Vec::new();
        for owner in [mine.clone(), theirs.clone()] {
            let shared: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::clone(&start);
            let their_common: PathBuf = common.clone();
            handles.push(std::thread::spawn(move || {
                let _together: std::sync::BarrierWaitResult = shared.wait();
                return take_an_attempt(their_common.as_path(), id, owner.as_str());
            }));
        }
        let _together: std::sync::BarrierWaitResult = start.wait();
        let ran: Vec<Attempt> = handles.into_iter().map(|handle| { return handle.join().expect("a command that ran"); }).collect();
        return ran.iter().map(number_of).collect();
    });
    let answered: Vec<String> = numbers;
    assert_eq!(answered.len(), 2, "two commands ran");
    assert_ne!(answered[0], answered[1], "one dead slot is never taken over by two commands: {answered:?}");
    remove(root.as_path());
}

/// What:
///  A number a command marked made is never taken again,
///  even by a command whose look it could not have seen.
/// Why:
///   The mark of a made attempt and the reservation that named it both stand until the leaving is forgotten,
///  so a command that looked before the mark and publishes afterwards finds the reservation still there
///  and asks for another number instead of acting on one that was already made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = takeAnAttempt(common, id, mine); markTheAttemptMade(slot);
/// expect(exclusiveCreate(reservedPath(common, id, 1))).toThrow('EEXIST');
/// ```
#[test]
fn the_name_of_a_made_attempt_cannot_be_published_again() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-made-name-stays");
    let mine: String = the_owner_of_this_command();
    let slot: Slot = the_slot_of(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()));
    mark_the_attempt_made(&slot);
    let published: std::io::Result<()> = write_private_file_unsynced(reserved_path(common.join(ATTEMPTS_FOLDER).as_path(), "the-leaving", 1).as_path(), mine.as_bytes());
    assert_eq!(
        published.as_ref().err().map(std::io::Error::kind),
        Some(std::io::ErrorKind::AlreadyExists),
        "a command that looked before the mark cannot publish the number it named: {published:?}",
    );
    assert_eq!(number_of(&take_an_attempt(common.as_path(), "the-leaving", mine.as_str())), "2", "so it asks for the next number");
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
/// releaseTheAttempt(takeAnAttempt(common, id, mine));
/// expect(numberOf(takeAnAttempt(common, id, mine))).toBe('1');
/// markTheAttemptMade(takeAnAttempt(common, id, mine)); expect(numberOf(takeAnAttempt(common, id, mine))).toBe('2');
/// ```
#[test]
fn a_released_slot_is_free_and_a_made_slot_is_spent() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-released");
    let mine: String = the_owner_of_this_command();
    let first: Slot = the_slot_of(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()));
    assert_eq!(first.number, 1, "the first number is taken");
    release_the_attempt(&first);
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 0, "a released slot is no attempt");
    let again: Slot = the_slot_of(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()));
    assert_eq!(again.number, 1, "so the first number is free again");
    mark_the_attempt_made(&again);
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 1, "a made attempt is spent");
    assert_eq!(number_of(&take_an_attempt(common.as_path(), "the-leaving", mine.as_str())), "2", "and the next number is taken");
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
/// for (const n of [1, 2, 3]) expect(numberOf(takeAnAttempt(common, id, mine))).toBe(`${n}`);
/// expect(numberOf(takeAnAttempt(common, id, mine))).toBe('held');       // every slot is a live command's attempt
/// for (const slot of slots) markTheAttemptMade(slot);
/// expect(numberOf(takeAnAttempt(common, id, mine))).toBe('spent');      // every attempt was made
/// forgetTheAttempts(common, id); expect(numberOf(takeAnAttempt(common, id, mine))).toBe('1');
/// ```
#[test]
fn every_number_taken_answers_held_and_every_number_made_answers_spent() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-budget");
    let mine: String = the_owner_of_this_command();
    let mut slots: Vec<Slot> = Vec::new();
    for number in 1..=THE_BUDGET {
        let slot: Slot = the_slot_of(take_an_attempt(common.as_path(), "the-leaving", mine.as_str()));
        assert_eq!(slot.number, number, "the budget is taken number by number");
        slots.push(slot);
    }
    assert_eq!(
        number_of(&take_an_attempt(common.as_path(), "the-leaving", mine.as_str())),
        "held",
        "every slot is another command's attempt that command may still make, so nothing says the attempts ran out",
    );
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), THE_BUDGET, "and the budget stands at its bound");
    for slot in &slots {
        mark_the_attempt_made(slot);
    }
    assert_eq!(number_of(&take_an_attempt(common.as_path(), "the-leaving", mine.as_str())), "spent", "every attempt was made, so the budget is spent");
    forget_the_attempts(common.as_path(), "the-leaving");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 0, "forgetting the leaving frees every number");
    assert_eq!(number_of(&take_an_attempt(common.as_path(), "the-leaving", mine.as_str())), "1", "and the first is free again");
    remove(root.as_path());
}

/// What:
///  A slot whose owner line cannot be read is answered as no count at all,
///  with the file it could not read,
///  and the delete it would have counted is never called spent.
/// Why:
///   A crash can leave a slot file empty, and a person or a program can leave one that is not text:
///  reading either as "an attempt was made" spends a budget on nothing,
///  while the words then say the delete was given up while no service ever tried it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(reservedPath(common, id, 1), ''); expect(takeAnAttempt(common, id, mine).kind).toBe('unreadable');
/// ```
#[test]
fn a_slot_that_cannot_be_read_is_no_count_and_names_the_file() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-unreadable");
    let mine: String = the_owner_of_this_command();
    let folder: PathBuf = common.join(ATTEMPTS_FOLDER);
    std::fs::create_dir_all(folder.as_path()).expect("the attempts folder");
    write_private_file_unsynced(reserved_path(folder.as_path(), "the-leaving", 1).as_path(), b"").expect("an empty slot a crash left");
    let attempt: Attempt = take_an_attempt(common.as_path(), "the-leaving", mine.as_str());
    let Attempt::Unreadable(reason) = &attempt else { panic!("an empty slot is no count, and the answer was {}", number_of(&attempt)) };
    assert!(reason.contains("reserved-the-leaving.1"), "the answer names the file it could not read: {reason}");
    std::fs::write(reserved_path(folder.as_path(), "the-leaving", 2).as_path(), b"no command here\n").expect("a slot that names no command");
    let second: Attempt = take_an_attempt(common.as_path(), "the-leaving", mine.as_str());
    assert!(matches!(second, Attempt::Unreadable(_)), "a slot whose line names no command is no count either: {}", number_of(&second));
    remove(root.as_path());
}

/// What:
///  A slot folder that cannot be written is answered as no count at all,
///  and never as a budget that was spent.
/// Why:
///   A full disk or a folder without the write bit is a count that cannot be kept,
///  and the caller must queue the delete anyway and say what could not be written,
///  because a subvolume nothing will delete is the defect the look exists to end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync(attemptsFolder, 0o500); expect(takeAnAttempt(common, id, mine).kind).toBe('unreadable'); chmodSync(attemptsFolder, 0o700);
/// ```
#[test]
fn a_slot_folder_that_cannot_be_written_is_no_count_and_names_the_folder() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-unwritable");
    let mine: String = the_owner_of_this_command();
    let folder: PathBuf = common.join(ATTEMPTS_FOLDER);
    std::fs::create_dir_all(folder.as_path()).expect("the attempts folder");
    // The control's own folder, which it makes unwritable and then writable again to remove what it made.
    let refused: u32 = 0o500;
    let allowed: u32 = 0o700;
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(refused)).expect("a folder that cannot be written");
    let attempt: Attempt = take_an_attempt(common.as_path(), "the-leaving", mine.as_str());
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(allowed)).expect("the folder is writable again for the control's own cleanup");
    let Attempt::Unreadable(reason) = &attempt else { panic!("an unwritable folder is no count, and the answer was {}", number_of(&attempt)) };
    assert!(reason.contains("leaving-attempts"), "the answer names the folder it could not write: {reason}");
    remove(root.as_path());
}

/// What:
///  A wake that reached no service gives every ask back and says the delete stays queued,
///  and a wake that was made marks every ask it was taken for.
/// Why:
///   A wake that reached nobody is not an attempt:
///  counting it would end in the words that say the delete was given up while no service ever tried it,
///  and the delete must stay queued for the next command to ask for
/// (`settle_the_asks`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = wakingThreeAsks(common, id); expect(settleTheAsks(asked, { error: 'no service' })).toContain('stays for the next command');
/// expect(attemptsTaken(common, id)).toBe(0);
/// const made = wakingThreeAsks(common, id); expect(settleTheAsks(made, undefined)).toBeUndefined(); expect(attemptsTaken(common, id)).toBe(3);
/// ```
#[test]
fn a_wake_that_reached_nobody_counts_no_ask_and_says_so() {
    let (root, common): (PathBuf, PathBuf) = a_common_directory("attempts-wake");
    let mine: String = the_owner_of_this_command();
    let asks: Vec<Slot> = (1..=THE_BUDGET).map(|_number| { return the_slot_of(take_an_attempt(common.as_path(), "the-leaving", mine.as_str())); }).collect();
    assert_eq!(asks.len(), usize::try_from(THE_BUDGET).expect("the budget"), "one ask per attempt the leaving may cost");
    let line: Option<String> = settle_the_asks(asks.as_slice(), Err(String::from("no commit service can be started from this session")));
    let Some(said) = line else { panic!("a wake that reached nobody says the delete stays queued") };
    assert!(said.contains("was not reached"), "the line says the service was not reached: {said}");
    assert!(said.contains("stays for the next command"), "and that the delete stays queued: {said}");
    assert!(said.contains("nothing here counts that as an attempt"), "and that nothing counts it: {said}");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), 0, "so no ask is counted");
    let made: Vec<Slot> = (1..=THE_BUDGET).map(|_number| { return the_slot_of(take_an_attempt(common.as_path(), "the-leaving", mine.as_str())); }).collect();
    assert_eq!(settle_the_asks(made.as_slice(), Ok(())), None, "a wake that was made says nothing");
    assert_eq!(attempts_taken(common.as_path(), "the-leaving"), THE_BUDGET, "and every ask it was taken for is counted");
    remove(root.as_path());
}
