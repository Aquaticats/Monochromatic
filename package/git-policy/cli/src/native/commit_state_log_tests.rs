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
use git_policy_cli::commit_state_log::{append, complete_rotation, read, rotate, rotation_number};
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
    assert_eq!(read(&dir).expect("state"), (3, b"published".to_vec()));
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
        assert_eq!(read(&dir).expect("prefix"), (1, b"first".to_vec()), "cut {}", cut);
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
    assert_eq!(read(&dir).expect("prefix"), (1, b"first".to_vec()));
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
        assert_eq!(read(&dir).expect("old log"), (2, b"second".to_vec()), "cut {}", cut);
    }
    std::fs::write(&new_path, &complete).expect("complete");
    assert_eq!(read(&dir).expect("new log"), (3, b"the whole document".to_vec()));
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
    assert_eq!(read(&dir).expect("after"), (3, b"the whole document".to_vec()));
    assert_eq!(rotation_number(&dir), Some(2));
    append(&dir, 4, b"later").expect("append");
    assert_eq!(read(&dir).expect("later"), (4, b"later".to_vec()));
}
