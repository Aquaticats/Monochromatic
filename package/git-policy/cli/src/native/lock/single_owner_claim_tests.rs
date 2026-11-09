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

/// The birth identity of a process that never ran.
const NO_SUCH_BIRTH: &str = "linux:not-a-birth";

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
