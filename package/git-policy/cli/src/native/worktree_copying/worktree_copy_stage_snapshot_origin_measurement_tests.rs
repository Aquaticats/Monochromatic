//! What:
//!  A measurement for the handover,
//! not a control:
//!  what reading the origin of a snapshot costs an add's path,
//!  in each shape that path has had or could have.
//! Why:
//!  A record written before a snapshot names the source and the instant its creation cannot be older than,
//!  and the question this change had to answer is what recording that costs on the path that waits.
//!  The arms alternate in one process,
//!  and one arm is measured twice as the control for the spread the host alone adds,
//!  because a difference smaller than that spread is not a difference this host can show.
//! Run:
//! `mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture measure_what_reading_the_origin_of_a_snapshot_costs`.
#![cfg(target_os = "linux")]

use super::*;
use crate::test_support::remove;
use std::path::PathBuf;
use std::time::Instant;

/// How many times one arm runs.
const SAMPLES: usize = 200;
/// The middle of `micros`,
/// the slowest sample within the slowest hundredth,
/// and the slowest sample of all.
fn spread(mut micros: Vec<u128>) -> (u128, u128, u128) {
    micros.sort_unstable();
    let middle: u128 = micros[micros.len() / 2];
    let p99: u128 = micros[(micros.len() * 99 / 100).min(micros.len() - 1)];
    let largest: u128 = micros[micros.len() - 1];
    return (middle, p99, largest);
}

/// What reading the origin costs,
/// arm by arm,
/// with a repeated arm as the control for this host's spread.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_what_reading_the_origin_of_a_snapshot_costs() {
    let Some(root) = crate::fs_btrfs_test_support::btrfs_fixture("origin-cost") else { return };
    let source: PathBuf = root.join("source");
    crate::fs_btrfs_ioctl::subvolume_create_at(source.as_path()).expect("the source subvolume");
    std::fs::write(source.join("tracked.txt"), b"tracked").expect("a file");
    let common: PathBuf = source.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    let journal: PathBuf = crate::worktree_copy_journal::journal_root(common.as_path());
    crate::worktree_copy_journal::ensure_journal_root(common.as_path()).expect("the journal root");
    std::fs::write(journal.join("anchor"), b"anchor\n").expect("a journal write");
    let mut once: Vec<u128> = Vec::new();
    let mut twice: Vec<u128> = Vec::new();
    let mut recorded: Vec<u128> = Vec::new();
    let mut again: Vec<u128> = Vec::new();
    for _sample in 0..SAMPLES {
        // The source's record alone.
        let started: Instant = Instant::now();
        crate::fs_btrfs_ioctl::subvolume_info(source.as_path()).expect("the source's record");
        once.push(started.elapsed().as_micros());
        // The source's record and the journal directory's, which is what the record read before this change.
        let started: Instant = Instant::now();
        crate::fs_btrfs_ioctl::subvolume_info(source.as_path()).expect("the source's record");
        crate::fs_btrfs_ioctl::subvolume_info(journal.as_path()).expect("the journal's record");
        twice.push(started.elapsed().as_micros());
        // What the record reads now.
        let started: Instant = Instant::now();
        origin_of_the_source(source.as_path()).expect("the origin");
        recorded.push(started.elapsed().as_micros());
        // The same read again, as the control for the spread.
        let started: Instant = Instant::now();
        origin_of_the_source(source.as_path()).expect("the origin");
        again.push(started.elapsed().as_micros());
    }
    for (arm, samples) in [("source record only", once), ("source and journal records", twice), ("the origin as recorded", recorded), ("the origin again", again)] {
        let (middle, p99, largest) = spread(samples);
        println!("{arm}: median {middle} us, p99 {p99} us, max {largest} us");
    }
    remove(root.as_path());
}
