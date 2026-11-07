//! A measurement for the handover,
//! not a control:
//! how long the checked rename holds a name (the claim,
//! the check that the name is
//!  free,
//! and the rename),
//! once alone and with threads publishing beside it.
//! `HELD_NAME_WAIT` cites its result.
//!  Run:
//! `mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture measure_how_long_a_name_is_held`.
#![cfg(unix)]

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;
use std::time::Instant;

/// The longest hold of a name seen in any run of the measurements below,
/// in microseconds:
/// one publish of 20,000 by one thread took
///  this long on 2026-11-07 on this host's btrfs while the load average of the host was about 70 (median 181 us,
/// 99.9th percentile
///  312143 us;
/// three later runs of 4,000 under a load average of 15 to 22 saw at most 296227 us).
/// `HELD_NAME_WAIT` must outlast it.
const LONGEST_HOLD_SEEN_MICROS: u64 = 6_031_986;

/// The multiple of the longest hold seen that the wait for a held name covers:
/// the owner's pick,
/// 2026-11-07,
/// "3x worst case",
/// when
///  asked how much longer than the measured worst case (6.03 s) the wrapper waits before giving up.
const OWNER_MARGIN: u32 = 3;

/// The wait for a held name is the longest hold ever measured times the owner's margin,
/// exactly:
/// a thread that waits for a name whose
///  holder is delayed in a slow rename does not give up while the holder is within the owner's multiple of the worst case,
/// and no
///  longer than it,
/// because giving up publishes nothing and "nothing should ever wait forever or indefinitely".
#[test]
fn the_wait_for_a_held_name_is_the_longest_hold_ever_measured_times_the_owners_margin() {
    let covered: Duration = Duration::from_micros(LONGEST_HOLD_SEEN_MICROS) * OWNER_MARGIN;
    assert_eq!(HELD_NAME_WAIT, covered);
}

/// How many publishes the alone measurement times.
const ALONE_PUBLISHES: usize = 4_000;

/// How many threads publish at once in the crowded measurement,
/// and how many publishes each makes.
const CROWD_THREADS: usize = 8;

/// Publishes of each thread of the crowd.
const CROWD_PUBLISHES_EACH: usize = 250;

/// Stages one small file at `directory/<name>.staged` and returns its path.
fn stage(directory: &Path, name: &str) -> PathBuf {
    let staged: PathBuf = directory.join(format!("{name}.staged"));
    std::fs::write(&staged, b"x").expect("the staged file");
    return staged;
}

/// Prints the spread of `micros`:
/// how many,
/// the middle,
/// the 99th and 99.9th percentile and the largest,
/// in microseconds.
fn print_spread(label: &str, mut micros: Vec<u128>) {
    micros.sort_unstable();
    let at = |per_thousand: usize| -> u128 { return micros[(micros.len() * per_thousand / 1000).min(micros.len() - 1)]; };
    println!(
        "{label}: n={} median={}us p99={}us p99.9={}us max={}us",
        micros.len(),
        at(500),
        at(990),
        at(999),
        micros.last().copied().unwrap_or(0)
    );
}

/// Times the call of the checked rename (claim,
/// check,
/// rename) for publishes of fresh names made by one thread.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_how_long_a_name_is_held_by_the_checked_rename() {
    let root: PathBuf = fixture("hold-time-measurement");
    let alone: PathBuf = root.join("alone");
    std::fs::create_dir(&alone).expect("the directory");
    let mut alone_micros: Vec<u128> = Vec::with_capacity(ALONE_PUBLISHES);
    for number in 0..ALONE_PUBLISHES {
        let staged: PathBuf = stage(alone.as_path(), format!("a{number}").as_str());
        let target: PathBuf = alone.join(format!("a{number}.final"));
        let started: Instant = Instant::now();
        publish_by_checked_rename(staged.as_path(), target.as_path()).expect("the publish");
        alone_micros.push(started.elapsed().as_micros());
    }
    print_spread("alone", alone_micros);
    remove(root.as_path());
}

/// Times the call of the checked rename for publishes of fresh names made by a crowd of threads at once.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_how_long_a_name_is_held_by_the_checked_rename_with_a_crowd() {
    let root: PathBuf = fixture("hold-time-measurement-crowd");
    let crowd: PathBuf = root.join("crowd");
    std::fs::create_dir_all(&crowd).expect("the directory");
    let workers: Vec<_> = (0..CROWD_THREADS)
        .map(|thread| {
            let directory: PathBuf = crowd.clone();
            return std::thread::spawn(move || {
                let mut micros: Vec<u128> = Vec::with_capacity(CROWD_PUBLISHES_EACH);
                for number in 0..CROWD_PUBLISHES_EACH {
                    let staged: PathBuf = stage(directory.as_path(), format!("t{thread}-{number}").as_str());
                    let target: PathBuf = directory.join(format!("t{thread}-{number}.final"));
                    let started: Instant = Instant::now();
                    publish_by_checked_rename(staged.as_path(), target.as_path()).expect("the publish");
                    micros.push(started.elapsed().as_micros());
                }
                return micros;
            });
        })
        .collect();
    let mut crowd_micros: Vec<u128> = Vec::new();
    for worker in workers {
        crowd_micros.extend(worker.join().expect("a worker"));
    }
    print_spread("crowd of eight threads", crowd_micros);
    remove(root.as_path());
}
