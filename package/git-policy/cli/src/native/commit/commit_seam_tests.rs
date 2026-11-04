//! What:
//!  Ordering and race controls of the durable-record components that need the test seams (the event log and the fault points).
//! Why:
//!  A directory trace cannot show that a saved file is synced before a ledger is truncated, and a barrier cannot force the unsafe
//!  interleaving; the events (file syncs, directory syncs, truncation, each recorded after it succeeded) and the fault points do.
//!  The seams exist only with the Cargo feature `test-seams`, so this module is compiled only then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the saved tail is synced before the ledger is truncated', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::model_fixture::{owned_append, owned_repair, scratch};
use git_policy_cli::commit_identity::{create_dir_all_durable, init_root, set_fault_point, start_events, take_events};
use git_policy_cli::commit_ledger::LedgerEntry;
use git_policy_cli::commit_spool::{SpoolError, write_spool};
use std::path::{Path, PathBuf};

/// A 32-character identity made of one repeated digit.
fn identity(digit: char) -> String {
    return digit.to_string().repeat(32);
}

/// A ledger entry for identity digit `digit`.
fn entry(digit: char) -> LedgerEntry {
    return LedgerEntry { id: identity(digit), outcome: "published".to_string(), commit: "a".repeat(40), time: "2026-11-03T12:00:00-05:00".to_string() };
}

/// The directories synced in `events`, in order.
fn directories(events: &[String]) -> Vec<PathBuf> {
    return events.iter().filter_map(|event| { return event.strip_prefix("dir:").map(PathBuf::from); }).collect();
}

/// Every created directory is reported, and the chain through the anchor is synced on every call, the second included.
#[test]
fn durable_directory_creation_syncs_the_chain_through_the_anchor_every_time() {
    let root: PathBuf = scratch("seam-chain");
    start_events();
    let created: Vec<PathBuf> = create_dir_all_durable(&root, &root.join("a/b/c")).expect("create");
    let first: Vec<PathBuf> = directories(&take_events());
    assert_eq!(created, vec![root.join("a"), root.join("a/b"), root.join("a/b/c")]);
    assert_eq!(first, vec![root.join("a/b/c"), root.join("a/b"), root.join("a"), root.clone()]);
    start_events();
    assert!(create_dir_all_durable(&root, &root.join("a/b/c")).expect("again").is_empty());
    assert_eq!(directories(&take_events()), first, "a chain that merely exists is synced again");
}

/// An append to a ledger that already exists syncs the directory chain through the anchor, last.
#[test]
fn an_append_to_an_existing_ledger_syncs_the_chain_through_the_anchor() {
    let root: PathBuf = scratch("seam-resend");
    let ledger: PathBuf = root.join("ledger");
    owned_append(&root, &ledger, &entry('3')).expect("append");
    start_events();
    owned_append(&root, &ledger, &entry('4')).expect("append");
    assert_eq!(directories(&take_events()).last(), Some(&root));
}

/// Review 3: the saved tail's file sync, then its directory sync, then the truncation, in that order and nothing destructive earlier.
#[test]
fn a_repair_syncs_the_saved_tail_and_its_directory_before_it_truncates() {
    let directory: PathBuf = scratch("seam-repair");
    let ledger: PathBuf = directory.join("ledger");
    owned_append(&directory, &ledger, &entry('1')).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"abc");
    std::fs::write(&ledger, &bytes).expect("torn");
    start_events();
    owned_repair(ledger.parent().expect("parent"), &ledger).expect("repair");
    let events: Vec<String> = take_events().into_iter().filter(|event| { return !event.starts_with("read-bytes:"); }).collect();
    let torn: String = format!("file-sync:{}", directory.join("ledger.torn").display());
    let parent: String = format!("dir:{}", directory.display());
    let truncate: String = format!("truncate:{}", ledger.display());
    // The parent chain through the anchor is made durable first (owner repair); from the saved tail's sync on, the order is fixed.
    let from: usize = events.iter().position(|event| { return *event == torn; }).expect("the saved tail is synced");
    assert_eq!(events[from..from + 3], [torn, parent, truncate]);
}

/// Review 3: with a fault point at the truncation, the saved bytes are already on disk when the writer stops there.
#[test]
fn a_writer_stopped_before_the_truncation_has_already_saved_the_tail() {
    static AT_TRUNCATION: std::sync::Barrier = std::sync::Barrier::new(2);
    static RELEASED: std::sync::Barrier = std::sync::Barrier::new(2);
    fn pause(point: &str) {
        if point == "ledger:before-truncate" {
            AT_TRUNCATION.wait();
            RELEASED.wait();
        }
    }
    let directory: PathBuf = scratch("seam-truncate");
    let ledger: PathBuf = directory.join("ledger");
    owned_append(&directory, &ledger, &entry('1')).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"abc");
    std::fs::write(&ledger, &bytes).expect("torn");
    let ledger_clone: PathBuf = ledger.clone();
    let writer = std::thread::spawn(move || {
        set_fault_point(Some(pause));
        return owned_repair(ledger_clone.parent().expect("parent"), &ledger_clone).expect("repair");
    });
    AT_TRUNCATION.wait();
    assert_eq!(std::fs::read(directory.join("ledger.torn")).expect("saved"), b"abc".to_vec());
    assert_eq!(std::fs::read(&ledger).expect("untouched"), bytes);
    RELEASED.wait();
    assert_eq!(writer.join().expect("join").cut, 3);
}

/// Review 3: a bare relative ledger name repairs and appends (the empty parent is the current directory), and the chain is made
/// durable before the repair.
#[test]
fn a_bare_relative_ledger_name_repairs_and_appends() {
    let directory: PathBuf = scratch("seam-relative");
    let previous: PathBuf = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&directory).expect("chdir");
    let outcome = (|| -> Result<(), String> {
        owned_append(Path::new("."), Path::new("ledger"), &entry('1')).map_err(|e| { return format!("{:?}", e); })?;
        let mut bytes: Vec<u8> = std::fs::read("ledger").map_err(|e| { return e.to_string(); })?;
        bytes.extend_from_slice(b"abc");
        std::fs::write("ledger", &bytes).map_err(|e| { return e.to_string(); })?;
        owned_append(Path::new("."), Path::new("ledger"), &entry('2')).map_err(|e| { return format!("{:?}", e); })?;
        return Ok(());
    })();
    std::env::set_current_dir(previous).expect("chdir back");
    outcome.expect("relative ledger");
    assert_eq!(std::fs::read(directory.join("ledger.torn")).expect("saved"), b"abc".to_vec());
}

/// Review 3: the anchor is never created silently, a path outside it or with `..` is refused before anything is created, and the
/// initialized-root constructor makes the root and then its known-durable parent durable.
#[test]
fn the_root_is_initialized_through_a_known_durable_parent_and_paths_are_contained() {
    let parent: PathBuf = scratch("seam-root");
    let root: PathBuf = parent.join("state");
    assert!(create_dir_all_durable(&root, &root.join("a")).is_err(), "a missing anchor is not created");
    assert!(!root.exists());
    start_events();
    let made: PathBuf = init_root(&parent, &root).expect("init");
    assert_eq!(directories(&take_events()), vec![made.clone(), parent.clone()]);
    assert_eq!(init_root(&parent, &root).expect("again"), made, "an interrupted creator's root is accepted and synced again");
    assert!(init_root(&parent, &parent.join("x/y")).is_err());
    assert!(create_dir_all_durable(&root, &parent.join("outside")).is_err());
    assert!(!parent.join("outside").exists(), "refused before creation");
    assert!(create_dir_all_durable(&root, &root.join("a/../../escape")).is_err());
    assert!(!parent.join("escape").exists());
}

/// Review 4 P1: a root that is a symlink is refused; its target's ancestry is not what the known-durable parent makes durable.
#[test]
fn a_symlinked_root_is_refused() {
    let parent: PathBuf = scratch("seam-symlink-root");
    let elsewhere: PathBuf = scratch("seam-symlink-target");
    std::os::unix::fs::symlink(&elsewhere, parent.join("state")).expect("symlink");
    assert!(init_root(&parent, &parent.join("state")).is_err());
}

/// Review 4 P2: an append to a clean ledger reads only its tail, not the whole ledger.
#[test]
fn an_append_to_a_clean_ledger_reads_only_the_tail() {
    let directory: PathBuf = scratch("seam-tail-read");
    let ledger: PathBuf = directory.join("ledger");
    for number in 0..200 {
        let digit: char = char::from_digit(number % 10, 10).expect("digit");
        owned_append(&directory, &ledger, &entry(digit)).expect("append");
    }
    let size: usize = std::fs::metadata(&ledger).expect("size").len() as usize;
    start_events();
    owned_append(&directory, &ledger, &entry('9')).expect("append");
    let events: Vec<String> = take_events();
    let reads: Vec<usize> =
        events.iter().filter_map(|event| { return event.strip_prefix("read-bytes:").and_then(|n| { return n.parse::<usize>().ok(); }); }).collect();
    assert!(!reads.is_empty(), "positive control: the repair records what it read");
    let read: usize = reads.iter().sum();
    assert!(read <= 1024, "read {} of {} bytes", read, size);
}

/// Review 6, 4.5: a resend that finds every final spool file in place wants no temporary file (a full disk must not fail it), and a
/// conflicting resend is still refused without writing one. The first write wants a temporary file, so the fault point is shown live.
#[test]
fn a_spool_resend_creates_no_temporary_file() {
    static TEMPORARY_WANTED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    fn note(point: &str) {
        if point == "spool:before-temporary" {
            TEMPORARY_WANTED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let root: PathBuf = scratch("seam-spool-resend");
    let id: String = identity('6');
    let files: Vec<Vec<u8>> = vec![b"new bytes".to_vec()];
    let olds: Vec<Vec<u8>> = vec![Vec::new()];
    set_fault_point(Some(note));
    write_spool(&root, &id, &files, &olds).expect("first write");
    let first: usize = TEMPORARY_WANTED.load(std::sync::atomic::Ordering::SeqCst);
    assert!(first > 0, "the fault point never fired");
    write_spool(&root, &id, &files, &olds).expect("resend");
    assert!(matches!(write_spool(&root, &id, &[b"other".to_vec()], &olds), Err(SpoolError::Conflict)));
    set_fault_point(None);
    assert_eq!(TEMPORARY_WANTED.load(std::sync::atomic::Ordering::SeqCst), first, "a resend wanted a temporary file");
}
