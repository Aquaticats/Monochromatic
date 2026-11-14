//! A measurement for the record,
//! not a control:
//! what one write of the note a landing keeps for the next service costs on this machine,
//! written on the
//!  landing's own thread and on a thread of its own,
//! timed round by round with the two arms interleaved.
//! `wait_ceilings.rs`'s `LANDING_WATCH_NOTE_CEILING` cites the first arm's spread,
//! and the
//!  report of the change that bounded that wait cites the ratio between the arms.
//! Run:
//! `mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture the_note_a_landing_writes`.
//!
//! Measured on 2026-11-14 on this host,
//! 2,000 notes per arm in each of six runs,
//! four of them on the build volume and two on the file system this user's journals are on:
//! - the note written on the landing's thread:
//!   162 to 180 us at the middle,
//!   271 to 297 us at the 99th percentile,
//!   and 246 ms at the slowest of the 12,000,
//!   one of this host's stalls;
//! - the same note written on a thread of its own:
//!   232 to 247 us at the middle,
//!   365 to 442 us at the 99th percentile,
//!   and 135 ms at the slowest,
//!   which is what a commit pays for the ceiling;
//! - the ratio of the two middles,
//!   interleaved round by round:
//!   1.37 to 1.43.
//! The middle is what a reader compares:
//! the largest samples of both arms move with the host's load and say nothing about the change.
//!
//! The second arm is what a commit pays for the bound,
//! and the first is what it paid before,
//! so the ratio of their middles is the cost of the change on a commit's path.
//! Both arms write the same note,
//! through the same `write_down`,
//! into stores of their own,
//! each naming a directory that is not there yet,
//! so every sample is a note created rather than one read back.
//! The directories named are made beside the stores so that the instance check inside the call answers for a real
//!  directory,
//! as it does in a service.
//! The root is the package's build directory unless `CLI_GIT_WATCH_NOTE_COST_ROOT` names another,
//! which is how the run measured the pair on the file system the journals of this user are on as well.

#![cfg(unix)]

use super::*;
use crate::service_claim_watch_note_within_a_ceiling::{NoteForTheNextService, note_for_the_next_service_within};
use std::time::Instant;

/// How many notes each arm writes,
/// so each middle is worth comparing.
const WRITES: usize = 2000;

/// The ceiling the bounded arm is given:
/// far past what a write that answers takes,
/// so that arm measures the wait and never its own ceiling.
const GENEROUS: Duration = Duration::from_secs(30);

/// The variable that names another root to measure under,
/// which a run sets to measure on another file system.
const ROOT_VARIABLE: &str = "CLI_GIT_WATCH_NOTE_COST_ROOT";

/// An empty directory of this control's own,
/// under the build directory or under the root the variable names.
fn scratch(name: &str) -> PathBuf {
    let base: PathBuf = match std::env::var_os(ROOT_VARIABLE) {
        Some(named) => PathBuf::from(named),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("claim-watch-cost"),
    };
    let directory: PathBuf = base.join(format!("{name}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    return directory;
}

/// A directory one note names,
/// made so the instance check inside the write answers for a real one,
/// and named after the round and the arm so every sample writes a note that is not there yet.
fn a_named_directory(root: &Path, arm: &str, number: usize) -> PathBuf {
    let git_dir: PathBuf = root.join("named").join(format!("{arm}-{number}.git"));
    std::fs::create_dir_all(git_dir.join(WORK_DIRECTORY)).expect("the work directory of the note");
    return git_dir;
}

/// The middle,
/// the 99th percentile and the largest of `micros`,
/// with the count.
fn spread(mut micros: Vec<u128>) -> (u128, u128, u128, usize) {
    micros.sort_unstable();
    let at = |per_thousand: usize| -> u128 { return micros[(micros.len() * per_thousand / 1000).min(micros.len() - 1)]; };
    return (at(500), at(990), *micros.last().expect("at least one sample"), micros.len());
}

/// The text of one arm's spread,
/// in microseconds.
fn words(label: &str, micros: Vec<u128>) -> String {
    let (median, p99, largest, count): (u128, u128, u128, usize) = spread(micros);
    return format!("{label}: n={count} median={median}us p99={p99}us max={largest}us");
}

/// The ratio of two middles,
/// to two decimals,
/// so a cost is read as the multiple it is.
fn ratio(before: u128, after: u128) -> String {
    return format!("{:.2}", after as f64 / before.max(1) as f64);
}

/// What:
///  One write of each arm per round,
/// with the arm that goes first alternating,
/// so a machine that slows or speeds during the run costs each arm the same.
/// Why:
///   The two arms are what the change is,
/// and a ratio of two spreads taken at different times is a ratio of those times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { control, bounded } = rounds(WRITES); // each round writes one note per arm, alternating which goes first
/// ```
fn rounds(writes: usize) -> (Vec<u128>, Vec<u128>) {
    let root: PathBuf = scratch("note-write");
    let control_store: PathBuf = root.join("control").join(WATCH_DIRECTORY);
    let bounded_store: PathBuf = root.join("bounded").join(WATCH_DIRECTORY);
    // One write on the landing's own thread, timed around the call.
    let control_arm = |git_dir: &Path| -> u128 {
        let began: Instant = Instant::now();
        note_for_the_next_service(control_store.as_path(), Some(instance_of(git_dir).as_str()), git_dir);
        return began.elapsed().as_micros();
    };
    // One write handed to a thread of its own, timed around the same call and its wait.
    let bounded_arm = |git_dir: &Path| -> u128 {
        let began: Instant = Instant::now();
        let _the_note: NoteForTheNextService = note_for_the_next_service_within(GENEROUS, bounded_store.as_path(), Some(instance_of(git_dir).as_str()), git_dir);
        return began.elapsed().as_micros();
    };
    let mut control: Vec<u128> = Vec::with_capacity(writes);
    let mut bounded: Vec<u128> = Vec::with_capacity(writes);
    for number in 0..writes {
        let named_control: PathBuf = a_named_directory(root.as_path(), "control", number);
        let named_bounded: PathBuf = a_named_directory(root.as_path(), "bounded", number);
        // The arm that goes first alternates, so neither arm is always the one paying for the write before it.
        if number % 2 == 0 {
            control.push(control_arm(named_control.as_path()));
            bounded.push(bounded_arm(named_bounded.as_path()));
        } else {
            bounded.push(bounded_arm(named_bounded.as_path()));
            control.push(control_arm(named_control.as_path()));
        }
    }
    // What each store answered: one note per write, which is what a bound that gives up on never-written notes would lose.
    let kept: usize = std::fs::read_dir(control_store.as_path()).map_or(0, |listing| { return listing.count(); });
    let kept_bounded: usize = std::fs::read_dir(bounded_store.as_path()).map_or(0, |listing| { return listing.count(); });
    println!("notes in the store of the arm on this thread: {kept}");
    println!("notes in the store of the arm on a thread of its own: {kept_bounded}");
    let _removed = std::fs::remove_dir_all(root.as_path());
    return (control, bounded);
}

/// What one note costs on the landing's own thread,
/// which is the wait the ceiling had to cover,
/// and on a thread of its own,
/// which is what a commit pays for that ceiling.
#[test]
#[ignore = "a measurement for the record, not a control"]
fn the_note_a_landing_writes_costs_this_on_this_machine() {
    let (control, bounded): (Vec<u128>, Vec<u128>) = rounds(WRITES);
    let control_words: String = words("the note written on the landing's thread", control.clone());
    let bounded_words: String = words("the note written on a thread of its own", bounded.clone());
    let (control_middle, control_p99, control_largest, _): (u128, u128, u128, usize) = spread(control);
    let (bounded_middle, bounded_p99, bounded_largest, _): (u128, u128, u128, usize) = spread(bounded);
    println!("{control_words}");
    println!("{bounded_words}");
    println!("cost of the bound, median: {}", ratio(control_middle, bounded_middle));
    println!("cost of the bound, p99: {}", ratio(control_p99, bounded_p99));
    println!("cost of the bound, largest: {}", ratio(control_largest, bounded_largest));
}
