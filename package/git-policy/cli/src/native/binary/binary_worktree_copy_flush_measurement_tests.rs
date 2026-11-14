//! What:
//!  A measurement for the handover,
//!  not a control:
//!  what one flush of each record of an ignored-state copy costs on this host now,
//!  and what an add of a fixture tree costs beside it.
//! Why:
//!  The records an add writes for itself are published by one rename and flushed by nobody,
//!  because a flush of one of them is a commit of the whole btrfs filesystem on the add's waiting path.
//!  Whether one of them,
//!  the journal,
//!  can have its flush back is a question about this host's numbers,
//!  never about how they feel,
//!  so the arms alternate in one process and one binary,
//!  and the same arm measured twice is the control for the spread the host alone adds.
//!  Run:
//! `mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture measure_what_a_flush_of_one_record_costs`.
#![cfg(unix)]

use super::support::{Fixture, Observed, fixture, git, remove, run_direct, run_wrapped, stderr_of};
use git_policy_cli::worktree_copy_journal::{write_atomic, write_atomic_unsynced};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How many records one arm writes.
const SAMPLES: usize = 40;

/// How often an add of the fixture's tree runs,
/// which is the number of paired arms the add comparison rests on.
const ADDS: usize = 3;

/// The bytes of one record,
/// the size a journal of a copy of a few dozen ignored roots holds.
fn record_bytes(named: &str) -> Vec<u8> {
    return format!("{{\"record\":\"{named}\",\"destinationRootHex\":\"2f746d702f74776f\",\"phase\":\"staged\",\"version\":1}}\n").into_bytes();
}

/// The middle of `micros`,
/// and how far the slowest sample is from it.
fn spread(mut micros: Vec<u128>) -> (u128, u128, u128) {
    micros.sort_unstable();
    let middle: u128 = micros[micros.len() / 2];
    let p99: u128 = micros[(micros.len() * 99 / 100).min(micros.len() - 1)];
    let largest: u128 = micros[micros.len() - 1];
    return (middle, p99, largest);
}

/// Write one record by one arm and say how long the call took.
fn one_record(path: &Path, bytes: &[u8], flushed: bool) -> Duration {
    let started: Instant = Instant::now();
    if flushed {
        write_atomic(path, bytes, "measurement record").expect("a flushed record");
    } else {
        write_atomic_unsynced(path, bytes, "measurement record").expect("an unflushed record");
    }
    return started.elapsed();
}

/// What one flush of each record of a copy costs on this host,
/// with the same arm measured twice as the control for the spread,
/// and what an add of the fixture's tree costs beside it.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_what_a_flush_of_one_record_costs() {
    let records_in: Fixture = fixture("record-flush-measurement");
    let root: PathBuf = records_in.root.clone();
    let records: PathBuf = root.join("records");
    std::fs::create_dir_all(records.as_path()).expect("records folder");
    let mut totals: Vec<(String, u128, u128)> = Vec::new();
    for named in ["claim", "journal", "unfilled", "binding"] {
        let bytes: Vec<u8> = record_bytes(named);
        let flushed: PathBuf = records.join(format!("{named}.flushed"));
        let unflushed: PathBuf = records.join(format!("{named}.unflushed"));
        let control: PathBuf = records.join(format!("{named}.control"));
        let mut flushed_micros: Vec<u128> = Vec::new();
        let mut unflushed_micros: Vec<u128> = Vec::new();
        let mut control_micros: Vec<u128> = Vec::new();
        // The arms alternate, so neither sits in one stretch of the host's own weather.
        for _sample in 0..SAMPLES {
            flushed_micros.push(one_record(flushed.as_path(), bytes.as_slice(), true).as_micros());
            unflushed_micros.push(one_record(unflushed.as_path(), bytes.as_slice(), false).as_micros());
            // The control is the unflushed arm measured again: whatever it adds over the first run is the spread this
            // host alone brings.
            control_micros.push(one_record(control.as_path(), bytes.as_slice(), false).as_micros());
        }
        let flushed_spread: (u128, u128, u128) = spread(flushed_micros);
        let unflushed_spread: (u128, u128, u128) = spread(unflushed_micros);
        let control_spread: (u128, u128, u128) = spread(control_micros);
        println!(
            "{named}: flushed median={}us p99={}us max={}us | unflushed median={}us p99={}us max={}us | control median={}us p99={}us max={}us",
            flushed_spread.0, flushed_spread.1, flushed_spread.2, unflushed_spread.0, unflushed_spread.1, unflushed_spread.2, control_spread.0, control_spread.1, control_spread.2
        );
        totals.push((String::from(named), flushed_spread.0, unflushed_spread.0));
    }
    for (named, flushed, unflushed) in &totals {
        println!("{named}: one flush adds {}us over the unflushed write (medians {flushed}us against {unflushed}us)", flushed.saturating_sub(*unflushed));
    }
    // The add the records serve, on the same filesystem and in the same fixture: the ignored files of a checkout with
    // the shape this repository's worktrees have, and the time the add takes with nobody's records flushed.
    let fixture: Fixture = fixture("record-flush-add-measurement");
    let repo: PathBuf = super::support::repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("ignore file");
    std::fs::create_dir_all(repo.join("ignored").as_path()).expect("ignored folder");
    for index in 0..500 {
        std::fs::write(repo.join(format!("ignored/f{index}.txt")), format!("payload {index}\n").repeat(16)).expect("an ignored file");
    }
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignored"]);
    let mut wrapped_micros: Vec<u128> = Vec::new();
    let mut direct_micros: Vec<u128> = Vec::new();
    for attempt in 0..ADDS {
        let wrapped_started: Instant = Instant::now();
        let wrapped_added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", format!("wrapped{attempt}").as_str(), format!("../wrapped{attempt}").as_str()]);
        assert_eq!(wrapped_added.code, Some(0), "{}", stderr_of(&wrapped_added));
        wrapped_micros.push(wrapped_started.elapsed().as_micros());
        let direct_started: Instant = Instant::now();
        let direct_added: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", format!("direct{attempt}").as_str(), format!("../direct{attempt}").as_str()]);
        assert_eq!(direct_added.code, Some(0), "{}", stderr_of(&direct_added));
        direct_micros.push(direct_started.elapsed().as_micros());
    }
    let wrapped_spread: (u128, u128, u128) = spread(wrapped_micros);
    let direct_spread: (u128, u128, u128) = spread(direct_micros);
    println!(
        "an add of 500 ignored files: through the wrapper (copied) median={}us p99={}us max={}us; real Git alone (not copied) median={}us p99={}us max={}us; {ADDS} adds each",
        wrapped_spread.0, wrapped_spread.1, wrapped_spread.2, direct_spread.0, direct_spread.1, direct_spread.2
    );
    remove(&fixture);
}
