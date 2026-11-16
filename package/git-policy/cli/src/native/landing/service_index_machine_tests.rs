//! Controls for the machine ID in a marker:
//! a lock or claim whose maker is on another machine is never read as gone,
//! and a
//!  reboot of the same machine still ends every process in it.

use super::*;
use crate::claim_time::current_machine_id;
use crate::process_identity::ProcessIdentityError;
use super::claim_tests::{claim_at, gone_marker, index_in, names_in};
use crate::service_index_marker::own_identity;
use crate::service_index_marker::{HolderState, LOCK_MARKER, ReaderView, holder_state, holder_state_in, identity_in_view_on};
use crate::test_support::remove;

/// A machine ID that is no host's here.
const MACHINE_A: &str = "0123456789abcdef0123456789abcdef";
const MACHINE_B: &str = "fedcba9876543210fedcba9876543210";
const BOOT_A: &str = "00112233445566778899aabbccddeeff";
const BOOT_B: &str = "ffeeddccbbaa99887766554433221100";
const SCOPE: &str = "4026532000";

/// Gives this thread's reader the machine ID `machine` (none for a host with no valid one),
/// whatever the host has.
fn on_machine(machine: Option<&'static str>) {
    crate::claim_time::test_machine::inject(machine.unwrap_or(""));
}

/// A marker of this release for a gone process,
/// with the identity `birth` (the text after the process number).
fn v2_lock_text(birth: &str) -> String {
    let gone: String = gone_marker(5);
    let line: &str = gone.trim_end();
    let without_birth: &str = line.strip_suffix("gone").expect("a gone birth");
    return format!("{without_birth}{birth}\n");
}

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

/// Allowed:
/// a boot that ended on the same machine ends every process in it.
#[test]
fn the_same_machine_in_another_boot_is_gone() {
    let holder = holder_on(BOOT_A, Some(MACHINE_A));
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, Some(MACHINE_A)), &nothing_runs), HolderState::Gone);
}

/// Refused:
/// a boot ID that differs says nothing about a maker on another machine,
/// which may run there;
/// and a reader that has no
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

/// Refused:
/// a machine ID in a marker that is not 32 lowercase hexadecimal digits,
/// or is all zero,
/// compares with nothing.
#[test]
fn a_recorded_machine_id_that_is_not_valid_is_unknown() {
    let malformed: [&str; 4] = ["0123456789ABCDEF0123456789ABCDEF", "00000000000000000000000000000000", "0123", "uninitialized"];
    for recorded in malformed {
        let holder = holder_on(BOOT_A, Some(recorded));
        assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, Some(recorded)), &nothing_runs), HolderState::Unknown, "{recorded}");
    }
}

/// A boot ID as the kernel prints it,
/// with the dashes a stored identity never carries.
const A_BOOT_WITH_THE_KERNEL_DASHES: &str = "e044d6c2-e7f5-41bd-aff8-9ac57532f7ff";

/// Refused:
/// a text whose parts are not the kinds a writer of this release leaves is never read as a boot that ended.
/// A writer that cannot read the boot ID of its host leaves that part out,
/// so the number of its PID namespace stands where the boot belongs,
/// and then nothing says which namespace its process number belongs to.
/// A machine part with nothing in it names no machine,
/// so a reader cannot tell this machine from another.
/// Read as ours,
/// either text would end a holder that runs,
/// and a lock of a writer that runs is removed.
#[test]
fn a_text_whose_parts_are_not_the_kinds_a_writer_leaves_is_never_gone() {
    let asked = std::cell::Cell::new(0_usize);
    let counting = |_: i64| -> Result<Option<String>, ProcessIdentityError> {
        asked.set(asked.get() + 1);
        return Ok(None);
    };
    // The boot with the kernel's dashes is no longer one of these: where the marker names this PID namespace, an absent
    //  number ends the maker whatever its boot part says (finding E of the third review,
    //  `a_marker_whose_boot_or_namespace_is_written_another_way_is_still_gone_by_its_free_number`).
    let texts: [(&str, String); 2] = [
        ("a boot the writer could not read is left out, so its PID namespace stands in the boot's place", format!("linux:777#{SCOPE}")),
        ("a machine part with nothing in it", format!("linux:777#{BOOT_A}#{SCOPE}#")),
    ];
    for (what, text) in texts {
        // A reader with no machine ID is a container's, and one with MACHINE_A is this host's; neither may call it gone.
        for machine in [None, Some(MACHINE_A)] {
            // `text.clone()` copies the text into the holder the reader is given, which owns the identity it explains.
            let holder: (i64, String) = (100, text.clone());
            assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, machine), &counting), HolderState::Unknown, "{what}: {holder:?}");
        }
    }
    assert_eq!(asked.get(), 0, "no process is asked about on a text whose parts cannot be read like for like");
}

/// A marker of this release naming the process number `pid_field` and the identity `identity`,
/// in whatever spelling the control gives them.
/// Why:
///   A control that respells one field must keep every other byte a writer would write,
///  or it tests a text no command leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// markerNaming('0' + pid, identity) // `cli-git-service v2 0 0<pid> <identity>\n`
/// ```
fn marker_naming(pid_field: &str, identity: &str) -> String {
    return format!("{LOCK_MARKER}{}0 {pid_field} {identity}\n", crate::service_index_marker::MARKER_VERSION);
}

/// The birth `linux:8423337` with a leading zero on its start tick,
/// which is the same tick written the way a program that pads numbers writes it.
/// Why:
///   A start tick is a number, and one number has one value however many zeros lead it.
fn birth_with_a_leading_zero(birth: &str) -> String {
    let Some((prefix, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} is no start tick this control can pad");
    };
    assert!(!tick.starts_with('0'), "the birth {birth} already leads its tick with a zero");
    return format!("{prefix}:0{tick}");
}

/// The same birth with the letters of its tag in capitals,
/// which is the same tag written the way a program that capitalizes a field writes it.
/// Why:
///   A tag's case is part of how it is written and no part of what it names,
/// so a record whose tag is capitalized names the same start tick another way.
fn birth_with_a_capitalized_tag(birth: &str) -> String {
    let Some((prefix, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} carries no tag this control can capitalize");
    };
    return format!("{}:{tick}", prefix.to_uppercase());
}

/// What:
///  The identity text cut where its parts begin:
///  the birth,
///  the boot,
///  the PID namespace,
///  and the rest,
///  which is the machine part with its separator or nothing.
/// Why:
///   A control that respells one part must leave every other byte as the writer wrote it,
///  and the machine is the part a host may not name at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [birth, boot, scope, ...rest] = identity.split('#'); const tail = rest.length === 0 ? '' : '#' + rest.join('#');
/// ```
fn split_birth_parts(identity: &str) -> (&str, &str, &str, &str) {
    let mut parts = identity.split('#');
    let (Some(birth), Some(boot), Some(scope)) = (parts.next(), parts.next(), parts.next()) else {
        panic!("the identity {identity} names no boot and PID namespace to respell");
    };
    let head: String = format!("{birth}#{boot}#{scope}");
    let tail: &str = identity.strip_prefix(head.as_str()).expect("the identity holds its own three parts");
    return (birth, boot, scope, tail);
}

/// Refused:
/// a live holder whose marker spells one part of its identity another way is never judged gone,
/// and its lock stays.
/// Why:
///   Six spellings a person or another program can produce without changing the value:
///  the boot's letters in capitals,
///  a leading zero on the birth's start tick,
///  a space before the birth,
///  the birth's tag in capitals,
///  a leading zero on the process number,
///  and a leading zero on the PID namespace.
///  Two spellings of one boot or one start tick are no proof of another,
///  and a padded number is the same number,
///  so a reader that cannot compare them like for like answers unknown or running,
///  never gone.
///  Read as another boot or another process, the lock of a Git that runs is removed.
#[test]
fn a_live_holders_identity_spelled_another_way_is_never_gone() {
    let identity: String = own_identity().expect("this process has an identity");
    let (birth, boot, scope, tail) = split_birth_parts(identity.as_str());
    let uppercase_boot: String = boot.to_uppercase();
    assert_ne!(uppercase_boot, boot, "this host's boot ID {boot} has no letter whose case can change");
    let padded_tick: String = birth_with_a_leading_zero(birth);
    let pid: i64 = i64::from(std::process::id());
    let pid_field: String = pid.to_string();
    let padded_pid: String = format!("0{pid_field}");
    let padded_scope: String = format!("0{scope}");
    let respelled_boot: String = format!("{birth}#{uppercase_boot}#{scope}{tail}");
    let respelled_tick: String = format!("{padded_tick}#{boot}#{scope}{tail}");
    let respelled_tag: String = format!("{}#{boot}#{scope}{tail}", birth_with_a_capitalized_tag(birth));
    let respelled_scope: String = format!("{birth}#{boot}#{padded_scope}{tail}");
    // A double space after the process number leaves the space in the birth field, which is how a person or a program
    //  that padded the fields writes the same birth.
    let respelled_space: String = format!(" {identity}");
    // Finding A of the third review: a birth in no form this platform writes proves nothing, so a live holder whose
    //  marker carries one is never judged gone. Each of these is one text a damaged or hand-written marker can hold.
    let bare_tick: String = String::from(birth.strip_prefix("linux:").expect("a Linux birth"));
    let quoted_birth: String = format!("\"{birth}\"");
    let with_byte_order_mark: String = format!("\u{feff}{birth}");
    let garbage_birth: String = String::from("not a birth at all");
    let respelled_bare_tick: String = format!("{bare_tick}#{boot}#{scope}{tail}");
    let respelled_quoted: String = format!("{quoted_birth}#{boot}#{scope}{tail}");
    let respelled_mark: String = format!("{with_byte_order_mark}#{boot}#{scope}{tail}");
    let respelled_garbage: String = format!("{garbage_birth}#{boot}#{scope}{tail}");
    let arms: [(&str, &str, &str, &str, HolderState); 10] = [
        ("the boot in capitals", "uppercase-boot", pid_field.as_str(), respelled_boot.as_str(), HolderState::Unknown),
        ("a leading zero on the start tick", "padded-tick", pid_field.as_str(), respelled_tick.as_str(), HolderState::Unknown),
        ("the birth's tag in capitals", "capitalized-tag", pid_field.as_str(), respelled_tag.as_str(), HolderState::Unknown),
        ("a leading zero on the process number", "padded-pid", padded_pid.as_str(), identity.as_str(), HolderState::Running),
        ("a leading zero on the PID namespace", "padded-scope", pid_field.as_str(), respelled_scope.as_str(), HolderState::Unknown),
        ("a space before the birth", "space-before-birth", pid_field.as_str(), respelled_space.as_str(), HolderState::Unknown),
        ("the start tick on its own, with no tag", "bare-tick", pid_field.as_str(), respelled_bare_tick.as_str(), HolderState::Unknown),
        ("the birth inside double quotes", "quoted-birth", pid_field.as_str(), respelled_quoted.as_str(), HolderState::Unknown),
        ("the birth behind a byte order mark", "byte-order-mark", pid_field.as_str(), respelled_mark.as_str(), HolderState::Unknown),
        ("a text that is no birth", "garbage-birth", pid_field.as_str(), respelled_garbage.as_str(), HolderState::Unknown),
    ];
    // Every arm is asked and every wrong answer is kept, so one run of the control names each spelling that was read
    //  as another process instead of the first one that was.
    let mut wrong: Vec<String> = Vec::new();
    for (what, fixture_name, number, respelled, expected) in arms {
        let (root, _index, lock) = index_in(fixture_name);
        let text: String = marker_naming(number, respelled);
        std::fs::write(&lock, text.as_bytes()).expect("the marker of a live holder, one part respelled");
        let marked = crate::service_index_marker::marker_of(lock.as_path()).expect("a marker");
        let state: HolderState = holder_state(marked.holder.as_ref());
        if state != expected {
            wrong.push(format!("{what}: the reader answered {state:?} for {respelled}, and {expected:?} is what a live holder's respelling gets"));
        }
        let outcome: Reclaimed = reclaim(&SYSTEM, &lock);
        if outcome != Reclaimed::Kept || !lock.exists() {
            wrong.push(format!("{what}: the lock of a live holder was {outcome:?}, and it exists afterwards: {}", lock.exists()));
        }
        remove(root.as_path());
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A marker that names no machine keeps its reading for a reader that has none either:
/// another boot is gone.
#[test]
fn a_marker_without_a_machine_id_keeps_the_reading_for_a_reader_without_one() {
    let holder = holder_on(BOOT_A, None);
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, None), &nothing_runs), HolderState::Gone);
}

/// The identity a process writes ends in the machine ID when the host has one,
/// and is as before when it has none.
#[test]
fn the_identity_names_the_machine_when_there_is_one() {
    let own: u32 = std::process::id();
    let record: String = format!("{own} (cli) S 1 2 3");
    let with = identity_in_view_on(record.as_bytes(), own, BOOT_A, Some(SCOPE), Some(MACHINE_A)).expect("an identity");
    assert!(with.ends_with(&format!("#{BOOT_A}#{SCOPE}#{MACHINE_A}")), "{with}");
    let without = identity_in_view_on(record.as_bytes(), own, BOOT_A, Some(SCOPE), None).expect("an identity");
    assert!(without.ends_with(&format!("#{BOOT_A}#{SCOPE}")), "{without}");
}

/// This process writes the machine ID of its host (given by the control),
/// and writes none where the host has none.
#[test]
fn this_process_writes_its_hosts_machine_id() {
    on_machine(Some(MACHINE_A));
    let identity: String = own_identity().expect("an identity");
    assert!(identity.ends_with(&format!("#{MACHINE_A}")), "{identity}");
    on_machine(None);
    let bare: String = own_identity().expect("an identity");
    assert!(!bare.contains(MACHINE_A) && bare.split('#').count() == 3, "{bare}");
}

/// The real host:
/// the machine ID this host has is the one the process writes.
/// Skipped,
/// loudly,
/// where the host has none.
#[test]
fn this_process_writes_the_real_hosts_machine_id() {
    let machine: &str = current_machine_id();
    if machine.is_empty() {
        eprintln!("SKIPPED: this host has no valid machine ID, so the real-host control cannot run; the injected-view controls cover the rule");
        return;
    }
    let identity: String = own_identity().expect("an identity");
    assert!(identity.ends_with(&format!("#{machine}")), "{identity}");
}

/// A lock text for a gone process of the machine `machine` in the boot `boot`.
fn lock_text_on(machine: &str, boot: &str) -> String {
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    return v2_lock_text(&format!("gone#{boot}#{scope}#{machine}"));
}

/// Refused:
/// a stale-looking lock of a service on another machine is not removed,
/// because its maker may run there;
/// the answer
///  carries the lock and its marker so the person is told how to remove it.
#[test]
fn a_lock_of_another_machine_is_left_and_named() {
    on_machine(Some(MACHINE_A));
    let (root, _index, lock) = index_in("machine-lock");
    let text: String = lock_text_on(MACHINE_B, crate::claim_time::current_boot_id());
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

/// Allowed:
/// the lock of a service on this machine in a boot that has ended is removed (a reboot ends every process).
#[test]
fn a_lock_of_this_machine_in_an_earlier_boot_is_removed() {
    on_machine(Some(MACHINE_A));
    let (root, _index, lock) = index_in("machine-same-lock");
    std::fs::write(&lock, lock_text_on(MACHINE_A, BOOT_B)).expect("a lock of an earlier boot");
    assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Removed);
    assert!(!lock.exists());
    remove(root.as_path());
}

/// Allowed:
/// a host that has no machine ID reads a marker that names none as before:
/// another boot is gone.
#[test]
fn a_reader_with_no_machine_id_keeps_the_earlier_reading_of_a_marker_with_none() {
    on_machine(None);
    let (root, _index, lock) = index_in("machine-none-lock");
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    std::fs::write(&lock, v2_lock_text(&format!("gone#{BOOT_B}#{scope}"))).expect("a lock of an earlier boot");
    assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Removed);
    remove(root.as_path());
}

/// Refused:
/// a reader that has a machine ID cannot tell a marker that names none (an earlier release's,
/// or a writer with no
///  machine ID) of another boot from a live service on another machine,
/// so it leaves the lock and names it.
#[test]
fn a_marker_with_no_machine_id_of_another_boot_is_left_by_a_reader_that_has_one() {
    on_machine(Some(MACHINE_A));
    let holder = holder_on(BOOT_A, None);
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_B, Some(MACHINE_A)), &nothing_runs), HolderState::Unknown);
    assert_eq!(holder_state_in(Some(&holder), &view(BOOT_A, Some(MACHINE_A)), &nothing_runs), HolderState::Gone, "the same boot is judged by the process");
    let (root, _index, lock) = index_in("machine-legacy-boot");
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    std::fs::write(&lock, v2_lock_text(&format!("gone#{BOOT_B}#{scope}"))).expect("a lock of another boot");
    assert!(matches!(reclaim(&SYSTEM, &lock), Reclaimed::Foreign { .. }));
    assert!(lock.exists());
    remove(root.as_path());
}

/// Refused:
/// a marker that names no process says nothing about whether anything runs,
/// so it is never gone.
#[test]
fn a_marker_that_names_no_process_is_never_gone() {
    for machine in [Some(MACHINE_A), None] {
        assert_eq!(holder_state_in(None, &view(BOOT_A, machine), &nothing_runs), HolderState::Unknown);
    }
    on_machine(Some(MACHINE_A));
    let (root, _index, lock) = index_in("machine-holderless");
    std::fs::write(&lock, format!("{LOCK_MARKER}5\n")).expect("a holderless lock");
    assert!(matches!(reclaim(&SYSTEM, &lock), Reclaimed::Foreign { .. }));
    assert!(lock.exists());
    remove(root.as_path());
}

/// Refused:
/// a claim whose maker is on another machine is not passed over;
/// the reclaimer leaves the stale lock and the claim and
///  names the claim.
#[test]
fn a_claim_of_another_machine_is_left_and_named() {
    on_machine(Some(MACHINE_A));
    let (root, _index, lock) = index_in("machine-claim");
    let claim: PathBuf = claim_at(&lock, 0);
    let text: String = lock_text_on(MACHINE_B, crate::claim_time::current_boot_id());
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

/// The words for a file of another machine name the file,
/// the batch and the process,
/// say that the maker is not proven gone,
/// and
///  give the removal.
#[test]
fn the_words_for_a_file_of_another_machine_give_the_removal() {
    let marked = parse_marker("cli-git-service v2 7 4242 linux:1#a#b#c\n").expect("a marker");
    let words: String = foreign_words(Path::new("/repo/.git/index.lock"), &marked);
    for needed in ["/repo/.git/index.lock", "batch 7", "process 4242", "another machine", "rm -- '/repo/.git/index.lock'"] {
        assert!(words.contains(needed), "missing {needed}: {words}");
    }
}

/// A landing that meets a lock of another machine waits to its ceiling,
/// withdraws,
/// leaves the lock,
/// and tells the person.
#[test]
fn a_landing_names_the_lock_of_another_machine() {
    on_machine(Some(MACHINE_A));
    let (root, index, lock) = index_in("machine-landing");
    std::fs::write(&lock, lock_text_on(MACHINE_B, crate::claim_time::current_boot_id())).expect("a lock of another machine");
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
    assert!(lines.contains("another machine") && lines.contains(&format!("rm -- '{}'", lock.display())), "{lines}");
    remove(root.as_path());
}

/// Refused:
///  a service whose marker spells its birth as nothing but a space is still proven gone by its absent process number,
///  and its index lock is removed.
/// Why:
///  A marker's holder is decided by every proof that stands on its own.
/// The birth is compared only beside a process that has the number,
/// so a birth this reader cannot compare is no proof that the maker lives,
/// and a number the process table answers nothing for ends the maker whatever its birth field holds.
///  Without that order a dead service leaves a marker nothing can settle:
/// the lock stays,
/// every later landing meets it,
/// and a person has to remove it by hand.
///  The canonical arm is the control of the control:
/// the same marker text with the birth this program writes is gone,
/// from the same one probe,
/// so the difference between the arms is the birth alone.
#[test]
fn a_marker_whose_birth_is_blank_is_still_proven_gone_by_its_free_number() {
    let identity: String = own_identity().expect("this process has an identity");
    let (_birth, boot, scope, tail) = split_birth_parts(identity.as_str());
    let host_tail: String = format!("#{boot}#{scope}{tail}");
    let free: i64 = crate::single_owner_claim::unused_pid_for_tests();
    let pid_field: String = free.to_string();
    let arms: [(&str, String, &str); 2] = [
        ("the birth this program writes", format!("linux:1{host_tail}"), "written"),
        ("a birth of nothing but a space", format!(" {host_tail}"), "blank"),
    ];
    let mut wrong: Vec<String> = Vec::new();
    for (what, arm_identity, fixture_name) in arms {
        // The probe is counted, so a reader that never asks about a number no process has is named by this arm.
        let asked = std::cell::Cell::new(0_usize);
        let nothing = |_: i64| -> Result<Option<String>, ProcessIdentityError> {
            asked.set(asked.get() + 1);
            return Ok(None);
        };
        let marked = (free, arm_identity.clone());
        let answered: HolderState = holder_state_in(Some(&marked), &ReaderView::current(), &nothing);
        if answered != HolderState::Gone {
            wrong.push(format!("{what}: the reader answered {answered:?} for {arm_identity}, and Gone is what a number no process has gives"));
        }
        if asked.get() != 1 {
            wrong.push(format!("{what}: the reader asked the process table {} times for a number no process has, and once is what that proof costs", asked.get()));
        }
        // The real reader over the real marker text, and the real reclaiming of the lock it sits in.
        let (root, _index, lock) = index_in(format!("blank-birth-{fixture_name}").as_str());
        let text: String = marker_naming(pid_field.as_str(), arm_identity.as_str());
        std::fs::write(&lock, text.as_bytes()).expect("a marker of a maker whose number no process has");
        let holder = crate::service_index_marker::marker_of(lock.as_path()).expect("a marker").holder;
        let marked_state: HolderState = holder_state(holder.as_ref());
        if marked_state != HolderState::Gone {
            wrong.push(format!("{what}: the reader answered {marked_state:?} for the marker text {text:?}, and Gone is what a number no process has gives"));
        }
        let outcome: Reclaimed = reclaim(&SYSTEM, &lock);
        if outcome != Reclaimed::Removed || lock.exists() {
            wrong.push(format!("{what}: the lock of a maker whose number no process has was {outcome:?}, and it exists afterwards: {}", lock.exists()));
        }
        remove(root.as_path());
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Refused:
///  a marker whose boot or PID namespace part is written another way,
///  and whose number no process has,
///  is still proven gone by the number.
/// Why:
///  Finding E of the third review: the boot part was checked for readability before the number was asked about,
/// so a dead service whose marker spells its boot with the kernel's dashes, in capitals, as garbage or with a
/// trailing space was `UNKNOWN` for ever and its index lock was never reclaimed by the marker's own proof.
/// The number is the proof that stands apart from the boot:
/// the marker names this machine and this PID namespace, so a number no process has on this machine is the maker's
/// own end whatever its boot part says.
///  The padded PID namespace arm is the same rule for the other part: a namespace written with a leading zero is the
/// same namespace, so the number is still ours to read.
/// A marker that names another machine or another PID namespace keeps its `UNKNOWN`,
/// which is the control of this control: that number is not this machine's to read.
#[test]
fn a_marker_whose_boot_or_namespace_is_written_another_way_is_still_gone_by_its_free_number() {
    // The marker text is read by this host's own reader, so its identity names this host's boot, namespace and machine.
    let boot: &str = crate::claim_time::current_boot_id();
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    let machine: &str = crate::claim_time::current_machine_id();
    let free: i64 = crate::single_owner_claim::unused_pid_for_tests();
    let pid_field: String = free.to_string();
    let view = ReaderView { boot, scope: Some(scope), machine: Some(machine), procfs_agrees: true };
    let arms: [(&str, String, HolderState); 5] = [
        ("the boot with the kernel's dashes", format!("linux:1#{}#{scope}#{machine}", A_BOOT_WITH_THE_KERNEL_DASHES), HolderState::Gone),
        ("the boot in capitals", format!("linux:1#{}#{scope}#{machine}", BOOT_A.to_uppercase()), HolderState::Gone),
        ("the boot as garbage", format!("linux:1#a boot no kernel writes#{scope}#{machine}"), HolderState::Gone),
        ("the boot with a trailing space", format!("linux:1#{BOOT_A} #{scope}#{machine}"), HolderState::Gone),
        // The number of another PID namespace is not this machine's to read, so nothing is proven about its maker,
        //  even though the marker names this boot: the number after it belongs to another numbering.
        ("another PID namespace", format!("linux:1#{boot}#{}#{machine}", "4026532999"), HolderState::Unknown),
    ];
    let mut wrong: Vec<String> = Vec::new();
    for (what, identity, expected) in arms {
        let asked = std::cell::Cell::new(0_usize);
        let nothing = |_: i64| -> Result<Option<String>, ProcessIdentityError> {
            asked.set(asked.get() + 1);
            return Ok(None);
        };
        let holder = (free, identity.clone());
        let state: HolderState = holder_state_in(Some(&holder), &view, &nothing);
        if state != expected {
            wrong.push(format!("{what}: the reader answered {state:?} for {identity}, and {expected:?} is what a free number gives"));
        }
        if expected == HolderState::Gone && asked.get() == 0 {
            wrong.push(format!("{what}: the reader never asked the process table about a number it could have settled"));
        }
        // And the real marker text, so the words a marker carries are the ones judged here.
        let (_root, _index, lock) = index_in(format!("written-another-way-{}", what.replace(' ', "-")).as_str());
        let text: String = marker_naming(pid_field.as_str(), identity.as_str());
        std::fs::write(&lock, text.as_bytes()).expect("a marker whose number no process has");
        let marked_holder = crate::service_index_marker::marker_of(lock.as_path()).expect("a marker").holder;
        let read: HolderState = holder_state(marked_holder.as_ref());
        if read != expected {
            wrong.push(format!("{what}: the marker text {text:?} was answered {read:?}, and {expected:?} is what a free number gives"));
        }
        let outcome: Reclaimed = reclaim(&SYSTEM, &lock);
        if expected == HolderState::Gone && (outcome != Reclaimed::Removed || lock.exists()) {
            wrong.push(format!("{what}: the lock of a maker whose number no process has was {outcome:?}, and it exists afterwards: {}", lock.exists()));
        }
        remove(_root.as_path());
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
