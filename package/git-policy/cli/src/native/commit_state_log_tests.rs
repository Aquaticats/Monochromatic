//! What:
//!  Controls of revision 13's state log: a hash-chained append-only log with checkpoint rotation, recovered as the longest valid prefix.
//! Why:
//!  Two unsynced overwrites of an alternating pair can tear both generations (R12-4); a log is never overwritten, a record that follows
//!  a damaged one is ignored however valid it looks, and a rotation interrupted at any step leaves a valid log. The waiting path of a
//!  plain commit still makes no sync: appending is visible, the syncer or a departure makes it durable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a torn newer record keeps the longest valid prefix', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::model_fixture::scratch;
use git_policy_cli::commit_identity::{start_events, take_events};
use git_policy_cli::commit_state_log::{StateError, append, complete_rotation, read, rotate, rotation_number};
use std::path::PathBuf;

/// The number of syncs in `events`.
fn syncs(events: &[String]) -> usize {
    return events.iter().filter(|event| { return event.starts_with("file-sync:") || event.starts_with("dir:"); }).count();
}

/// Appends are visible, read back as the newest generation, and make no sync (the waiting path of a plain commit).
#[test]
fn appending_to_the_state_log_makes_no_sync_and_reads_back_the_newest() {
    let dir: PathBuf = scratch("log-append");
    start_events();
    append(&dir, 1, b"adopted").expect("append");
    append(&dir, 2, b"ready").expect("append");
    append(&dir, 3, b"published").expect("append");
    assert_eq!(syncs(&take_events()), 0);
    assert_eq!(read(&dir).expect("readable").expect("state"), (3, b"published".to_vec()));
}

/// A torn newer record, cut at every byte, leaves the longest valid prefix: the previous generation.
#[test]
fn a_torn_newer_record_keeps_the_longest_valid_prefix() {
    let dir: PathBuf = scratch("log-torn");
    append(&dir, 1, b"first").expect("append");
    let after_first: u64 = std::fs::metadata(dir.join("state.log.1")).expect("size").len();
    append(&dir, 2, b"second").expect("append");
    let full: Vec<u8> = std::fs::read(dir.join("state.log.1")).expect("read");
    for cut in after_first as usize..full.len() {
        std::fs::write(dir.join("state.log.1"), &full[..cut]).expect("torn");
        assert_eq!(read(&dir).expect("readable").expect("prefix"), (1, b"first".to_vec()), "cut {}", cut);
    }
}

/// A record that follows a damaged one is ignored however valid it looks: the chain, not the checksum, decides.
#[test]
fn a_valid_looking_record_after_a_damaged_one_is_ignored() {
    let dir: PathBuf = scratch("log-chain");
    append(&dir, 1, b"first").expect("append");
    let boundary: usize = std::fs::metadata(dir.join("state.log.1")).expect("size").len() as usize;
    append(&dir, 2, b"second").expect("append");
    append(&dir, 3, b"third").expect("append");
    let mut bytes: Vec<u8> = std::fs::read(dir.join("state.log.1")).expect("read");
    bytes[boundary + 6] ^= 1;
    std::fs::write(dir.join("state.log.1"), &bytes).expect("damage the second record");
    assert_eq!(read(&dir).expect("readable").expect("prefix"), (1, b"first".to_vec()));
}

/// A rotation interrupted at each step: the new file empty or cut at every byte of its first record keeps the old log as the state; the
/// complete new file is the state; after `complete_rotation` the old file is gone, removed only after the new file and its directory
/// entry were synced.
#[test]
fn a_rotation_interrupted_at_each_step_keeps_a_valid_log() {
    let dir: PathBuf = scratch("log-rotate");
    append(&dir, 1, b"first").expect("append");
    append(&dir, 2, b"second").expect("append");
    assert_eq!(rotate(&dir, 3, b"the whole document").expect("rotate"), 2);
    let new_path: PathBuf = dir.join("state.log.2");
    let complete: Vec<u8> = std::fs::read(&new_path).expect("read");
    for cut in 0..complete.len() {
        std::fs::write(&new_path, &complete[..cut]).expect("interrupted");
        assert_eq!(read(&dir).expect("readable").expect("old log"), (2, b"second".to_vec()), "cut {}", cut);
    }
    std::fs::write(&new_path, &complete).expect("complete");
    assert_eq!(read(&dir).expect("readable").expect("new log"), (3, b"the whole document".to_vec()));
    assert!(dir.join("state.log.1").exists(), "the old file stays until the new one is durable");
    start_events();
    complete_rotation(&dir).expect("complete");
    let events: Vec<String> = take_events();
    assert_eq!(
        events,
        vec![
            format!("file-sync:{}", new_path.display()),
            format!("dir:{}", dir.display()),
            format!("remove:{}", dir.join("state.log.1").display()),
        ]
    );
    assert!(!dir.join("state.log.1").exists());
    assert_eq!(read(&dir).expect("readable").expect("after"), (3, b"the whole document".to_vec()));
    assert_eq!(rotation_number(&dir).expect("listing"), Some(2));
    append(&dir, 4, b"later").expect("append");
    assert_eq!(read(&dir).expect("readable").expect("later"), (4, b"later".to_vec()));
}

/// Review 5, item 3.5: absence, damage and an I/O error are three answers. A directory with no log is absent, not an error.
#[test]
fn a_directory_with_no_log_is_absent() {
    let dir: PathBuf = scratch("log-absent");
    assert!(read(&dir).expect("an empty directory is readable").is_none());
}

/// A log whose bytes validate nowhere is damaged, not empty: recovery must not read it as "no state", and an append must not truncate
/// it (the bytes are the only evidence of what was there).
#[test]
fn a_log_with_no_valid_record_is_damaged_and_an_append_leaves_it_alone() {
    let dir: PathBuf = scratch("log-damaged");
    std::fs::write(dir.join("state.log.1"), b"not a record at all\n").expect("write garbage");
    assert!(matches!(read(&dir), Err(StateError::Damaged)));
    let refused = append(&dir, 1, b"first");
    assert!(refused.is_err(), "an append over a damaged log must refuse");
    assert_eq!(std::fs::read(dir.join("state.log.1")).expect("read"), b"not a record at all\n".to_vec(), "the evidence is untouched");
}

/// An unreadable newest log (here a directory in its place, which fails for every user) is an I/O error: recovery must not fall back
/// silently to the older generation, and an append must not write a newer generation beside it.
#[test]
fn an_unreadable_newest_log_is_an_io_error_not_an_older_generation() {
    let dir: PathBuf = scratch("log-unreadable");
    append(&dir, 1, b"first").expect("append");
    std::fs::create_dir(dir.join("state.log.2")).expect("a directory where the newest log should be");
    assert!(matches!(read(&dir), Err(StateError::Io(_))), "the newest log cannot be read");
    assert!(append(&dir, 2, b"second").is_err());
    assert_eq!(std::fs::read_dir(dir.join("state.log.2")).expect("still a directory").count(), 0);
}

/// A newest file whose first record is torn falls back to the previous file (it exists until the new one is durable); that fallback is
/// not an error.
#[test]
fn a_torn_newest_file_still_falls_back_to_the_previous_one() {
    let dir: PathBuf = scratch("log-fallback");
    append(&dir, 1, b"first").expect("append");
    std::fs::write(dir.join("state.log.2"), b"rec 2 ").expect("a torn new file");
    assert_eq!(read(&dir).expect("readable").expect("fallback"), (1, b"first".to_vec()));
}
