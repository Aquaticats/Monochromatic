//! Controls for the claim:
//! the first claim,
//! a live owner,
//! a released claim,
//! an owner that died,
//! an owner whose PID was
//!  reused,
//! several claimants at once,
//! the removal of older generations,
//! and records that are not claims.

use super::*;
use crate::process_identity::current_birth_identity;
use std::ffi::OsStr;
use crate::test_support::{fixture, remove};

/// The prefix the controls use.
const PREFIX_TEXT: &str = "item.owner";

/// The prefix the controls use,
/// as the claim functions take it.
fn prefix() -> &'static OsStr {
    return OsStr::new(PREFIX_TEXT);
}

/// The generation of the first claim on an item.
const FIRST: u64 = 1;

/// The birth identity of a process that never ran:
/// the largest start tick a 64-bit number holds,
/// written as this program writes one,
/// and no process is born at it.
/// Why:
///   A control needs a birth a live process can never have.
/// A text that is not a birth at all is no record this reader can compare,
/// which is what `a_live_owner_whose_birth_is_spelled_another_way_keeps_the_item` plants instead.
const NO_SUCH_BIRTH: &str = "linux:18446744073709551615";

/// The held claim of an outcome.
fn held(outcome: Result<ClaimOutcome, MigrateError>) -> OwnerClaim {
    return match outcome.expect("no failure") {
        ClaimOutcome::Held(claim) => claim,
        ClaimOutcome::Busy(owner) => panic!("expected a held claim, found the owner {owner:?}"),
    };
}

/// The live owner of an outcome.
fn busy(outcome: Result<ClaimOutcome, MigrateError>) -> BusyOwner {
    return match outcome.expect("no failure") {
        ClaimOutcome::Busy(owner) => owner,
        ClaimOutcome::Held(claim) => panic!("expected a live owner, took {claim:?}"),
    };
}

/// The generations of the controls' item in a directory.
fn present(directory: &Path) -> Vec<u64> {
    return generations(directory, prefix()).expect("listing");
}

/// The first claim is generation one,
/// and it names this process.
#[test]
fn the_first_claim_is_generation_one() {
    let directory: PathBuf = fixture("claim-first");
    let claim: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    assert_eq!(present(directory.as_path()), vec![FIRST]);
    let text: String = std::fs::read_to_string(&claim.path).expect("claim file");
    assert!(text.contains(format!("pid={}\n", std::process::id()).as_str()), "{text}");
    remove(directory.as_path());
}

/// A live owner stops the next claimant,
/// which is told the owner's process id and file,
/// creates nothing and does not wait.
#[test]
fn a_live_owner_stops_the_next_claimant() {
    let directory: PathBuf = fixture("claim-live");
    let first: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    let owner: BusyOwner = busy(take_claim(directory.as_path(), prefix()));
    assert_eq!(owner, BusyOwner { pid: i64::from(std::process::id()), path: first.path.clone() });
    assert_eq!(present(directory.as_path()), vec![FIRST], "the refused claimant created nothing");
    remove(directory.as_path());
}

/// A released claim,
/// by `release` or by dropping,
/// is taken by the next claimant as the next generation,
/// and the older
///  generation is removed.
#[test]
fn a_released_claim_is_replaced_by_the_next_generation() {
    let directory: PathBuf = fixture("claim-released");
    let mut first: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    first.release().expect("released");
    let second: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    assert_eq!(present(directory.as_path()), vec![FIRST + 1], "the older generation is gone");
    drop(second);
    let third: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    assert_eq!(present(directory.as_path()), vec![FIRST + 2]);
    drop(third);
    assert_eq!(present(directory.as_path()), vec![FIRST + 2], "the newest generation is never removed, even when released");
    remove(directory.as_path());
}

/// An owner that was killed leaves its claim file;
/// the next claimant sees that no process has that id and takes over.
#[test]
fn an_owner_that_died_is_replaced() {
    let directory: PathBuf = fixture("claim-died");
    plant_claim_for_tests(directory.as_path(), prefix(), FIRST, unused_pid_for_tests(), NO_SUCH_BIRTH);
    let claim: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    assert_eq!(present(directory.as_path()), vec![FIRST + 1]);
    drop(claim);
    remove(directory.as_path());
}

/// A claim whose process id names a live process with another birth (the id was reused) is a dead owner.
#[test]
fn an_owner_whose_process_id_was_reused_is_replaced() {
    let directory: PathBuf = fixture("claim-reused");
    plant_claim_for_tests(directory.as_path(), prefix(), FIRST, i64::from(std::process::id()), NO_SUCH_BIRTH);
    let claim: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    assert_eq!(present(directory.as_path()), vec![FIRST + 1]);
    drop(claim);
    remove(directory.as_path());
}

/// Refused:
///  a live owner whose claim spells its own birth another way keeps the item,
///  and the claimant says what it could not read instead of taking over.
/// Why:
///   The claim names its owner by PID and process birth because the number alone is reused,
///  and the birth is compared with the identity the process table reports now (`live_owner`).
/// Two spellings of one start tick read as two processes,
/// so a live owner's claim is replaced,
/// and two commands then work on one item:
/// they write one journal,
/// share one token on a resume and exchange one directory back (`single_owner_claim.rs`,
/// its module description).
///  The spellings are the ones a person or a program can produce without changing the value:
/// a leading zero on the start tick,
/// a space on either side of the birth,
/// and the birth's tag in capitals.
/// A record whose birth cannot be compared is a question this claimant cannot answer,
/// so it stops and names the file,
/// which keeps the item for a person to look at.
///  A birth in the spelling this program writes stops the claimant too,
/// as `a_live_owner_stops_the_next_claimant` shows,
/// which is the control of this control.
#[test]
fn a_live_owner_whose_birth_is_spelled_another_way_keeps_the_item() {
    let birth: String = current_birth_identity().expect("this process has an identity");
    let Some((tag, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} of this process carries no tag this control can respell");
    };
    let own: i64 = i64::from(std::process::id());
    let arms: [(&str, String); 4] = [
        ("a leading zero on the start tick", format!("{tag}:0{tick}")),
        ("a space before the birth", format!(" {birth}")),
        ("a space after the birth", format!("{birth} ")),
        ("the birth's tag in capitals", format!("{}:{tick}", tag.to_uppercase())),
    ];
    let mut wrong: Vec<String> = Vec::new();
    for (index, (what, respelled)) in arms.into_iter().enumerate() {
        let directory: PathBuf = fixture(format!("claim-birth-respelled-{index}").as_str());
        plant_claim_for_tests(directory.as_path(), prefix(), FIRST, own, respelled.as_str());
        let outcome: Result<ClaimOutcome, MigrateError> = take_claim(directory.as_path(), prefix());
        let generations_after: Vec<u64> = present(directory.as_path());
        remove(directory.as_path());
        match outcome {
            Ok(ClaimOutcome::Held(claim)) => {
                wrong.push(format!("{what}: the claimant took the item from the live owner of the claim reading {respelled}, into {:?}", claim.path));
            }
            Ok(ClaimOutcome::Busy(owner)) => {
                assert_eq!(owner.pid, own, "{what}: the owner a claim reading {respelled} names is this process");
                assert_eq!(generations_after, vec![FIRST], "{what}: the claimant created no generation of its own");
            }
            Err(refusal) => {
                assert!(format!("{refusal:?}").contains(respelled.as_str()), "{what}: the refusal names the birth it could not compare: {refusal:?}");
                assert_eq!(generations_after, vec![FIRST], "{what}: the claimant created no generation of its own");
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Claimants that arrive together:
/// exactly one takes the claim,
/// and every other is told that it is busy.
#[test]
fn only_one_of_many_simultaneous_claimants_holds_the_item() {
    let directory: PathBuf = fixture("claim-simultaneous");
    plant_claim_for_tests(directory.as_path(), prefix(), FIRST, unused_pid_for_tests(), NO_SUCH_BIRTH);
    let claimants: usize = std::thread::available_parallelism().map_or(2, std::num::NonZero::get);
    let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(claimants));
    let handles: Vec<std::thread::JoinHandle<Option<OwnerClaim>>> = (0..claimants)
        .map(|_index: usize| {
            let (claimant_directory, claimant_barrier) = (directory.clone(), std::sync::Arc::clone(&barrier));
            return std::thread::spawn(move || {
                claimant_barrier.wait();
                return match take_claim(claimant_directory.as_path(), prefix()).expect("no failure") {
                    ClaimOutcome::Held(claim) => Some(claim),
                    ClaimOutcome::Busy(_owner) => None,
                };
            });
        })
        .collect();
    let winners: Vec<OwnerClaim> = handles.into_iter().filter_map(|handle: std::thread::JoinHandle<Option<OwnerClaim>>| { return handle.join().expect("claimant"); }).collect();
    assert_eq!(winners.len(), 1, "exactly one claimant holds the item");
    remove(directory.as_path());
}

/// Claims of other items and files that only resemble a claim are not read as this item's generations.
#[test]
fn only_this_items_numbered_files_are_generations() {
    let directory: PathBuf = fixture("claim-names");
    for name in ["item.owner", "item.owner.", "item.owner.x", "item.owner.1.x", "item.owner.-1", "other.owner.9", "xitem.owner.5"] {
        std::fs::write(directory.join(name), b"unrelated").expect("file");
    }
    let claim: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    assert_eq!(present(directory.as_path()), vec![FIRST]);
    drop(claim);
    remove(directory.as_path());
}

/// A newest generation that is not a claim cannot be judged:
/// the claimant stops with the file named and changes nothing.
#[test]
fn a_file_that_is_not_a_claim_stops_the_claimant_untouched() {
    let directory: PathBuf = fixture("claim-garbage");
    std::fs::write(path_of(directory.as_path(), prefix(), FIRST), b"somebody else's text").expect("file");
    let message: String = take_claim(directory.as_path(), prefix()).expect_err("cannot be judged").to_string();
    assert!(message.contains("is not a claim file") && message.contains(format!("{}.{FIRST}", PREFIX_TEXT).as_str()), "{message}");
    assert_eq!(std::fs::read(path_of(directory.as_path(), prefix(), FIRST)).expect("file"), b"somebody else's text");
    assert_eq!(present(directory.as_path()), vec![FIRST]);
    remove(directory.as_path());
}

/// Found by the audit of the names the rename changed:
/// a claim file an earlier release wrote starts with its own spelling of the
///  first line,
/// and the files are named as before (`cli-git-convert.owner.N`),
/// so a released or dead claim of the earlier release
///  must be replaced like any other,
/// and a live one must still stop the claimant.
/// Writing stays in the current spelling.
#[test]
fn a_claim_an_earlier_release_wrote_is_read_as_a_claim() {
    let earlier = |text: String| -> String { return text.replacen("monochromatic-git-policy-cli ", "cli-git ", 1); };
    let released: String = earlier(encode(&ClaimRecord::Released));
    assert!(released.starts_with("cli-git owner-claim v1\n"), "{released}");
    assert_eq!(decode(released.as_str()), Some(ClaimRecord::Released));
    let owned: ClaimRecord = ClaimRecord::Owned { pid: i64::from(std::process::id()), birth: String::from("linux:42"), namespace: None };
    assert_eq!(decode(earlier(encode(&owned)).as_str()), Some(owned));
    assert!(encode(&ClaimRecord::Released).starts_with("monochromatic-git-policy-cli owner-claim v1\n"), "new claims are written in the current spelling");
    let directory: PathBuf = fixture("claim-earlier-release");
    std::fs::write(path_of(directory.as_path(), prefix(), FIRST), released).expect("an earlier release's released claim");
    let claim: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
    assert_eq!(present(directory.as_path()), vec![FIRST + 1], "it is replaced by the next generation");
    drop(claim);
    remove(directory.as_path());
}

/// A claim from a PID namespace this process is not in counts as alive while its file is young.
#[test]
fn an_owner_in_another_pid_namespace_is_alive_while_its_claim_is_young() {
    let directory: PathBuf = fixture("claim-foreign");
    let record: ClaimRecord = ClaimRecord::Owned { pid: unused_pid_for_tests(), birth: String::from(NO_SUCH_BIRTH), namespace: Some(String::from("pid:[not-this-namespace]")) };
    std::fs::write(path_of(directory.as_path(), prefix(), FIRST), encode(&record)).expect("planted claim");
    let owner: BusyOwner = busy(take_claim(directory.as_path(), prefix()));
    assert_eq!(owner.pid, unused_pid_for_tests());
    remove(directory.as_path());
}

/// The text of every kind of claim record decodes to the record,
/// and text that is not a claim decodes to nothing.
#[test]
fn the_text_of_a_claim_round_trips() {
    for record in [
        ClaimRecord::Released,
        ClaimRecord::Owned { pid: i64::from(std::process::id()), birth: String::from("linux:42"), namespace: None },
        ClaimRecord::Owned { pid: i64::from(std::process::id()), birth: String::from("linux:43"), namespace: Some(String::from("pid:[1]")) },
    ] {
        assert_eq!(decode(encode(&record).as_str()), Some(record));
    }
    for text in ["", "monochromatic-git-policy-cli owner-claim v2\nreleased\n", "monochromatic-git-policy-cli owner-claim v1\npid=0\nbirth=x\nnamespace=-\n", "monochromatic-git-policy-cli owner-claim v1\npid=1\nbirth=\nnamespace=-\n"] {
        assert_eq!(decode(text), None, "{text:?}");
    }
}

/// A claimant whose listing went stale,
/// because another command took a newer generation before this one published,
/// withdraws
///  what it published and reports the live owner instead of holding the item beside it.
#[test]
fn a_claimant_with_a_stale_listing_withdraws_when_a_newer_generation_exists() {
    let directory: PathBuf = fixture("claim-stale-listing");
    let newer: u64 = FIRST + 2;
    let mut acted: bool = false;
    let outcome = take_claim_seamed(directory.as_path(), prefix(), &mut || {
        if !acted {
            acted = true;
            plant_claim_for_tests(directory.as_path(), prefix(), newer, i64::from(std::process::id()), current_birth_identity().expect("identity").as_str());
        }
    });
    assert_eq!(busy(outcome).path, path_of(directory.as_path(), prefix(), newer));
    assert_eq!(present(directory.as_path()), vec![newer], "the stale claimant left nothing of its own");
    remove(directory.as_path());
}

/// A claim of another PID namespace whose file time is later than this reader's clock has no age:
/// it is refused with the claim's name and the `rm` instead of reading as a live owner's for ever
/// (audit of 2026-11-09,
/// finding 2).
/// Before the fix the claim answered "a process is migrating this checkout,
/// wait for it to finish" about a process that no longer exists,
/// with no age and no `rm`.
#[test]
fn a_claim_of_another_namespace_whose_time_is_ahead_is_refused_and_named_and_nothing_changes() {
    let directory: PathBuf = fixture("claim-foreign-ahead");
    let record: ClaimRecord = ClaimRecord::Owned { pid: unused_pid_for_tests(), birth: String::from(NO_SUCH_BIRTH), namespace: Some(String::from("pid:[not-this-namespace]")) };
    let claim: PathBuf = path_of(directory.as_path(), prefix(), FIRST);
    std::fs::write(&claim, encode(&record)).expect("planted claim");
    std::fs::File::options().write(true).open(&claim).expect("open").set_modified(std::time::SystemTime::now() + std::time::Duration::from_secs(7200)).expect("move the time ahead");
    let refused = take_claim(directory.as_path(), prefix());
    let Err(MigrateError::Journal(text)) = refused else { panic!("a claim whose age cannot be measured is refused, not taken for a live owner's: {refused:?}") };
    assert!(text.contains(claim.to_string_lossy().as_ref()) && text.contains("pid:[not-this-namespace]") && text.contains(unused_pid_for_tests().to_string().as_str()), "{text}");
    assert!(text.contains("later than this reader's clock") && text.contains("another container") && text.contains("rm -- ") && text.contains("left running"), "{text}");
    assert_eq!(present(directory.as_path()), vec![FIRST], "no newer generation was made and the claim stays");
    remove(directory.as_path());
}

/// A claim of another PID namespace older than the patience is neither alive nor dead (owner,
/// 2026-11-04,
/// "Refuse and name it
/// (Recommended)"):
/// taking the claim stops with an error that names the claim file,
/// the owner and its namespace,
/// its age and the
/// `rm`,
/// and the claim is left as it was,
/// with no newer generation made.
#[test]
fn a_claim_of_another_namespace_older_than_the_patience_is_refused_and_named_and_nothing_changes() {
    let directory: PathBuf = fixture("claim-foreign-expired");
    let record: ClaimRecord = ClaimRecord::Owned { pid: unused_pid_for_tests(), birth: String::from(NO_SUCH_BIRTH), namespace: Some(String::from("pid:[not-this-namespace]")) };
    let claim: PathBuf = path_of(directory.as_path(), prefix(), FIRST);
    std::fs::write(&claim, encode(&record)).expect("planted claim");
    std::fs::File::options().write(true).open(&claim).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age");
    let refused = take_claim(directory.as_path(), prefix());
    let Err(MigrateError::Journal(text)) = refused else { panic!("an expired foreign claim is refused: {refused:?}") };
    assert!(text.contains(claim.to_string_lossy().as_ref()) && text.contains("pid:[not-this-namespace]") && text.contains(unused_pid_for_tests().to_string().as_str()), "{text}");
    assert!(text.contains("minutes old") && text.contains("another container") && text.contains("rm -- ") && text.contains("left running"), "{text}");
    assert_eq!(present(directory.as_path()), vec![FIRST], "no newer generation was made and the claim stays");
    remove(directory.as_path());
}

/// What:
///  Whether the claim's own writes asked the shared step to flush,
///  and which answer it gave.
/// Why:
///   A regular file reads back from the page cache whether or not its flush ran,
/// so the claim's two filesystem commits (the record and the directory that holds its name)
///  show only through the step's own count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// syncCounts.take(); const claim = takeClaim(dir); return syncCounts.take();
/// ```
fn syncs_of(work: impl FnOnce()) -> crate::subvolume_file_write_steps::SyncCounts {
    let _taken = crate::subvolume_file_write_steps::take_sync_counts();
    work();
    return crate::subvolume_file_write_steps::take_sync_counts();
}

/// A claim taken and given back on a thread that keeps its writes in memory makes no filesystem commit,
/// and asks the shared step for both of them.
/// The negative control below is what says the count can move.
#[test]
fn a_claim_kept_in_memory_flushes_nothing_and_asks_the_shared_step_twice() {
    let directory: PathBuf = fixture("claim-kept-in-memory");
    let counts = syncs_of(|| {
        let _kept = crate::subvolume_file_write_steps::WritesKeptInMemory::for_this_thread();
        let mut claim: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
        claim.release().expect("released");
    });
    assert_eq!(counts.made, 0, "no flush reached the kernel: {counts:?}");
    assert!(counts.skipped >= 4, "the record and the directory of the take and of the release each asked the step: {counts:?}");
    remove(directory.as_path());
}

/// The same claim on a thread that flushes makes both commits,
/// so the count above is a measurement and not a constant.
#[test]
fn a_claim_on_a_flushing_thread_flushes_its_record_and_its_directory() {
    let directory: PathBuf = fixture("claim-flushing");
    let counts = syncs_of(|| {
        let mut claim: OwnerClaim = held(take_claim(directory.as_path(), prefix()));
        claim.release().expect("released");
    });
    assert!(counts.made >= 4, "the record and the directory of the take and of the release each reached the disk: {counts:?}");
    assert_eq!(counts.skipped, 0, "nothing was skipped where the thread flushes: {counts:?}");
    remove(directory.as_path());
}

/// The answer one row of the identity table requires.
/// Why:
///   This caller answers in three values
///  (`Ok(Some(pid))` for a live owner,
/// `Ok(None)` for a gone one,
/// and an error for a question it cannot answer),
/// so the table can state all three states of the world,
/// unlike the held push's two-valued reader (`landing/held_push_holder_and_verdict_tests.rs`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Required {
    /// Nothing of the owner runs: the claimant takes the item.
    Gone,
    /// The owner is the process that has the number now: the claimant is told who it is.
    Alive,
    /// The reader cannot tell: the claimant stops and names the file without taking anything.
    Unknown,
}

/// The birth `birth` with a leading zero on its start tick,
/// which is the same tick written the way a program that pads numbers writes it.
/// Why:
///   A start tick is a number, and one number has one value however many zeros lead it.
fn birth_with_a_leading_zero(birth: &str) -> String {
    let Some((prefix, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} is no start tick this control can pad");
    };
    return format!("{prefix}:0{tick}");
}

/// Refused:
///  no pairing of what the process table says about a claim's number with what the claim says about its birth
///  takes an item from an owner that runs,
///  and no pairing keeps an item whose owner cannot run.
/// Why:
///  The two rules pull against each other.
/// Nothing may take an item from a live owner,
/// so a birth that is not one value in the spelling this program writes it answers unknown,
/// which refuses the claimant and names the file.
/// And an owner that cannot run must not be able to refuse every later claimant,
/// so the birth decides only where there is a process to compare it with:
/// a number no process has ends the claim whatever its birth says,
/// and only a live process with that number is asked about the birth.
///  The birth texts are the value as the writer writes it,
/// that value in another spelling (a leading zero on its start tick),
/// nothing but a space,
/// the empty text,
/// and a text that claims this platform's tag and names no tick.
/// The number states are a number no process has,
/// the number of the process that wrote the claim,
/// and the number of a later process,
/// which is a live number with a birth the claim says belongs to an earlier owner.
#[test]
fn every_pairing_of_a_number_state_and_a_birth_state_gives_the_answer_the_rule_requires() {
    let own: i64 = i64::from(std::process::id());
    let live_birth: String = current_birth_identity().expect("this process has an identity");
    let live_birth_respelled: String = birth_with_a_leading_zero(live_birth.as_str());
    let earlier_birth: String = String::from(NO_SUCH_BIRTH);
    let earlier_birth_respelled: String = birth_with_a_leading_zero(NO_SUCH_BIRTH);
    let blank: &str = " ";
    let empty: &str = "";
    let unparseable: String = format!("{}not-a-tick", crate::process_identity::LINUX_IDENTITY_PREFIX);
    let free: i64 = unused_pid_for_tests();
    let rows: [(&str, i64, &str, Required); 15] = [
        ("a free number and the birth as the writer writes it", free, live_birth.as_str(), Required::Gone),
        ("a free number and the birth in another spelling", free, live_birth_respelled.as_str(), Required::Gone),
        ("a free number and a blank birth", free, blank, Required::Gone),
        ("a free number and an empty birth", free, empty, Required::Gone),
        ("a free number and an unparseable birth", free, unparseable.as_str(), Required::Gone),
        ("the writer's own number and the birth as the writer writes it", own, live_birth.as_str(), Required::Alive),
        ("the writer's own number and the birth in another spelling", own, live_birth_respelled.as_str(), Required::Unknown),
        ("the writer's own number and a blank birth", own, blank, Required::Unknown),
        ("the writer's own number and an empty birth", own, empty, Required::Unknown),
        ("the writer's own number and an unparseable birth", own, unparseable.as_str(), Required::Unknown),
        ("a later process's number and an earlier birth as the writer writes it", own, earlier_birth.as_str(), Required::Gone),
        ("a later process's number and that earlier birth in another spelling", own, earlier_birth_respelled.as_str(), Required::Gone),
        ("a later process's number and a blank birth", own, blank, Required::Unknown),
        ("a later process's number and an empty birth", own, empty, Required::Unknown),
        ("a later process's number and an unparseable birth", own, unparseable.as_str(), Required::Unknown),
    ];
    // Every row is asked in a fixture of its own, and every wrong answer is kept, so one run of this table names each
    //  pairing that broke a rule.
    let mut wrong: Vec<String> = Vec::new();
    for (index, (what, number, birth, required)) in rows.into_iter().enumerate() {
        let directory: PathBuf = fixture(format!("claim-identity-table-{index}").as_str());
        plant_claim_for_tests(directory.as_path(), prefix(), FIRST, number, birth);
        let outcome: Result<ClaimOutcome, MigrateError> = take_claim(directory.as_path(), prefix());
        let generations_after: Vec<u64> = present(directory.as_path());
        remove(directory.as_path());
        let expected_generations: Vec<u64> = match required {
            Required::Gone => vec![FIRST + 1],
            Required::Alive | Required::Unknown => vec![FIRST],
        };
        match (outcome, required) {
            (Ok(ClaimOutcome::Held(_claim)), Required::Gone) => {}
            (Ok(ClaimOutcome::Held(_claim)), _) => {
                wrong.push(format!("{what}: the claimant took the item, and the rule requires {required:?}"));
            }
            (Ok(ClaimOutcome::Busy(owner)), Required::Alive) if owner.pid == number => {}
            (Ok(ClaimOutcome::Busy(owner)), Required::Alive) => {
                wrong.push(format!("{what}: the claimant named process {} instead of {number}", owner.pid));
            }
            (Ok(ClaimOutcome::Busy(_owner)), _) => {
                wrong.push(format!("{what}: the claimant answered with a live owner, and the rule requires {required:?}"));
            }
            (Err(refusal), Required::Unknown) if format!("{refusal:?}").contains(birth) => {}
            (Err(refusal), Required::Unknown) => {
                wrong.push(format!("{what}: the refusal does not name the birth {birth:?}: {refusal:?}"));
            }
            (Err(_refusal), _) => {
                wrong.push(format!("{what}: the claimant refused, and the rule requires {required:?}"));
            }
        }
        if generations_after != expected_generations {
            wrong.push(format!("{what}: the item holds generations {generations_after:?} afterwards, and {expected_generations:?} is what this row leaves"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
