//! Controls for the machine ID in a marker: a lock or claim whose maker is on another machine is never read as gone, and a
//!  reboot of the same machine still ends every process in it.

use super::*;
use crate::claim_time::current_machine_id;
use crate::process_identity::ProcessIdentityError;
use super::claim_tests::{claim_at, gone_marker, index_in, names_in};
use crate::service_index_marker::own_identity;
use crate::service_index_marker::{HolderState, ReaderView, holder_state_in, identity_in_view_on};
use crate::test_support::remove;

/// A machine ID that is no host's here.
const MACHINE_A: &str = "0123456789abcdef0123456789abcdef";
const MACHINE_B: &str = "fedcba9876543210fedcba9876543210";
const BOOT_A: &str = "00112233445566778899aabbccddeeff";
const BOOT_B: &str = "ffeeddccbbaa99887766554433221100";
const SCOPE: &str = "4026532000";

fn view(boot: &'static str, machine: Option<&'static str>) -> ReaderView<'static> {
    return ReaderView { boot, scope: Some(SCOPE), machine, procfs_agrees: true };
}

fn holder_on(boot: &str, machine: Option<&str>) -> (i64, String) {
    let tail: String = machine.map_or_else(String::new, |id| { return format!("#{id}"); });
    return (100, format!("linux:777#{boot}#{SCOPE}{tail}"));
}

/// Gone as `ask` says the process does not exist.
fn nothing_runs(_: i64) -> Result<Option<String>, ProcessIdentityError> {
    return Ok(None);
}

/// Allowed: a boot that ended on the same machine ends every process in it.
#[test]
fn the_same_machine_in_another_boot_is_gone() {
    let holder = holder_on(BOOT_A, Some(MACHINE_A));
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, Some(MACHINE_A)), &nothing_runs), HolderState::Gone);
}

/// Refused: a boot ID that differs says nothing about a maker on another machine, which may run there; and a reader that has no
///  machine ID cannot tell its own machine from the maker's.
#[test]
fn another_machine_is_unknown_and_never_gone() {
    let holder = holder_on(BOOT_A, Some(MACHINE_A));
    let asked = std::cell::Cell::new(0_usize);
    let counting = |_: i64| -> Result<Option<String>, ProcessIdentityError> {
        asked.set(asked.get() + 1);
        return Ok(None);
    };
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, Some(MACHINE_B)), &counting), HolderState::Unknown, "another boot of another machine");
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_A, Some(MACHINE_B)), &counting), HolderState::Unknown, "the same boot ID of another machine");
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, None), &counting), HolderState::Unknown, "a reader with no machine ID");
    assert_eq!(asked.get(), 0, "no process is asked about on a machine it does not belong to");
}

/// Refused: a machine ID in a marker that is not 32 lowercase hexadecimal digits, or is all zero, compares with nothing.
#[test]
fn a_recorded_machine_id_that_is_not_valid_is_unknown() {
    let malformed: [&str; 4] = ["0123456789ABCDEF0123456789ABCDEF", "00000000000000000000000000000000", "0123", "uninitialized"];
    for recorded in malformed {
        let holder = holder_on(BOOT_A, Some(recorded));
        assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, Some(recorded)), &nothing_runs), HolderState::Unknown, "{recorded}");
    }
}

/// Allowed: a marker from a release that wrote no machine ID keeps its reading: another boot is gone.
#[test]
fn a_marker_without_a_machine_id_keeps_the_reading_of_earlier_releases() {
    let holder = holder_on(BOOT_A, None);
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, Some(MACHINE_A)), &nothing_runs), HolderState::Gone);
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, None), &nothing_runs), HolderState::Gone);
}

/// The identity a process writes ends in the machine ID when the host has one, and is as before when it has none.
#[test]
fn the_identity_names_the_machine_when_there_is_one() {
    let own: u32 = std::process::id();
    let record: String = format!("{own} (cli) S 1 2 3");
    let with = identity_in_view_on(record.as_bytes(), own, BOOT_A, Some(SCOPE), Some(MACHINE_A)).expect("an identity");
    assert!(with.ends_with(&format!("#{BOOT_A}#{SCOPE}#{MACHINE_A}")), "{with}");
    let without = identity_in_view_on(record.as_bytes(), own, BOOT_A, Some(SCOPE), None).expect("an identity");
    assert!(without.ends_with(&format!("#{BOOT_A}#{SCOPE}")), "{without}");
}

/// This process writes the machine ID of the host it runs on, when the host has a valid one.
#[test]
fn this_process_writes_the_hosts_machine_id() {
    let machine: &str = current_machine_id();
    if machine.is_empty() {
        eprintln!("MACHINE CONTROL NOT RUN: this host has no valid machine ID");
        return;
    }
    let identity: String = own_identity().expect("an identity");
    assert!(identity.ends_with(&format!("#{machine}")), "{identity}");
}

/// A marker for a gone process on a machine, written as the lock of that machine's service would be.
fn lock_text_on(machine: &str, boot: &str) -> String {
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    let gone: String = gone_marker(5);
    let line: &str = gone.trim_end();
    let without_birth: &str = line.strip_suffix("gone").expect("a gone birth");
    return format!("{without_birth}gone#{boot}#{scope}#{machine}\n");
}

/// A machine ID that is not this host's.
fn other_machine() -> &'static str {
    return if current_machine_id() == MACHINE_B { MACHINE_A } else { MACHINE_B };
}

/// Refused: a stale-looking lock of a service on another machine is not removed, because its maker may run there; the answer
///  carries the lock and its marker so the person is told how to remove it.
#[test]
fn a_lock_of_another_machine_is_left_and_named() {
    let (root, _index, lock) = index_in("machine-lock");
    let text: String = lock_text_on(other_machine(), crate::claim_time::current_boot_id());
    std::fs::write(&lock, &text).expect("a lock of another machine");
    let outcome: Reclaimed = reclaim(&SYSTEM, &lock);
    match outcome {
        Reclaimed::Foreign { file, marked } => {
            assert_eq!(file, lock);
            assert_eq!(marked.seq, 5);
        }
        other => panic!("expected the lock to be left as another machine's: {other:?}"),
    }
    assert_eq!(std::fs::read_to_string(&lock).expect("the lock stands"), text);
    assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "no claim was placed");
    remove(root.as_path());
}

/// Allowed: the lock of a service on this machine in a boot that has ended is removed (a reboot ends every process).
#[test]
fn a_lock_of_this_machine_in_an_earlier_boot_is_removed() {
    let machine: &str = current_machine_id();
    if machine.is_empty() {
        eprintln!("MACHINE CONTROL NOT RUN: this host has no valid machine ID");
        return;
    }
    let (root, _index, lock) = index_in("machine-same-lock");
    std::fs::write(&lock, lock_text_on(machine, BOOT_B)).expect("a lock of an earlier boot");
    assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Removed);
    assert!(!lock.exists());
    remove(root.as_path());
}

/// Refused: a claim whose maker is on another machine is not passed over; the reclaimer leaves the stale lock and the claim and
///  names the claim.
#[test]
fn a_claim_of_another_machine_is_left_and_named() {
    let (root, _index, lock) = index_in("machine-claim");
    let claim: PathBuf = claim_at(&lock, 0);
    let text: String = lock_text_on(other_machine(), crate::claim_time::current_boot_id());
    std::fs::write(&claim, &text).expect("a claim of another machine");
    let outcome: Reclaimed = reclaim(&SYSTEM, &lock);
    match outcome {
        Reclaimed::Foreign { file, .. } => assert_eq!(file, claim),
        other => panic!("expected the claim to be left as another machine's: {other:?}"),
    }
    assert!(lock.exists(), "the stale lock stays");
    assert_eq!(std::fs::read_to_string(&claim).expect("the claim stands"), text);
    remove(root.as_path());
}

/// The words for a file of another machine name the file, the batch and the process, say that the maker is not proven gone, and
///  give the removal.
#[test]
fn the_words_for_a_file_of_another_machine_give_the_removal() {
    let marked = parse_marker("cli-git-service 7 4242 linux:1#a#b#c\n").expect("a marker");
    let words: String = foreign_words(Path::new("/repo/.git/index.lock"), &marked);
    for needed in ["/repo/.git/index.lock", "batch 7", "process 4242", "another machine", "rm /repo/.git/index.lock"] {
        assert!(words.contains(needed), "missing {needed}: {words}");
    }
}

/// A landing that meets a lock of another machine waits to its ceiling, withdraws, leaves the lock, and tells the person.
#[test]
fn a_landing_names_the_lock_of_another_machine() {
    let (root, index, lock) = index_in("machine-landing");
    std::fs::write(&lock, lock_text_on(other_machine(), crate::claim_time::current_boot_id())).expect("a lock of another machine");
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 4;
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let began: std::time::Instant = std::time::Instant::now();
    let failure = acquire_within(&SYSTEM, &index, 9, ceiling, &|announce| {
        if let Announce::Waiting(line) = announce {
            said.borrow_mut().push(line);
        }
    })
    .expect_err("the lock is not taken");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    assert!(began.elapsed() >= ceiling, "the wait ended before its ceiling, as for a dead owner");
    assert!(lock.exists());
    let lines: String = said.borrow().join("\n");
    assert!(lines.contains("another machine") && lines.contains(&format!("rm {}", lock.display())), "{lines}");
    remove(root.as_path());
}
