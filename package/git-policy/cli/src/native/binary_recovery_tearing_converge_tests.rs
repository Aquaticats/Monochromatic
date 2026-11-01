//! What:
//!  A lost `owner.json` in a published transaction directory, which once left a repository stuck: a fresh directory
//!  still stops every guarded command with a message that names the way out, an old one that no process uses is
//!  recovered like a dead owner's, and an old one that a process uses keeps the stop.
//! Why:
//!  The wrapper must never leave a repository stuck, so the loss has to end in a state where `git status` and
//!  `git commit` succeed without a hand removal. No record names the writer of a directory whose `owner.json` is gone,
//!  so the rule is an age bound (the patience the wrapper gives an owner it cannot probe) plus a veto from any process
//!  that has the directory open or as its working directory. The lost lock record is in
//!  `binary_recovery_tearing_lock_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await loseName(phase, 'owner.json'); await ageDirectory(); expect(await classify(phase)).toBe('discards');
//! ```

use super::names::{
    Outcome, PHASES_AFTER_THE_BRANCH_MOVED, PHASES_BEFORE_THE_LANDING_BARRIER, lose_name, settle, transaction_directory,
};
use super::{Paused, Probe, pause_at, probe};
use crate::commit::remove_settled;
use crate::support::Sleeper;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// What:
///  Date every entry of a directory two hours ago, children before their directories.
/// Why:
///   A directory is abandoned once nothing in it changed for the patience, and waiting an hour is no test.
///  A link would keep its own date, so one is a failure of the test, not a silent miss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of walk(directory).reverse()) await utimes(path, longAgo, longAgo);
/// ```
pub(super) fn age_tree(directory: &Path) {
    let long_ago: SystemTime = SystemTime::now() - Duration::from_secs(2 * 3600);
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    let mut ordered: Vec<PathBuf> = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata = std::fs::symlink_metadata(&path).expect("entry to age");
        assert!(!metadata.file_type().is_symlink(), "a link keeps its date: {}", path.display());
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&path).expect("directory to age").flatten() {
                pending.push(entry.path());
            }
        }
        ordered.push(path);
    }
    for path in ordered.iter().rev() {
        std::fs::File::open(path).expect("open").set_modified(long_ago).expect("age");
    }
}

/// What:
///  The first status of a paused repository, and its standard error.
/// Why:
///   The finding is the text the person reads and whether the state is still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await run('git status') // { code, stderr }
/// ```
pub(super) fn status(paused: &Paused) -> Probe {
    return probe(&paused.fixture, paused.repo.as_path(), &["status", "--porcelain=v1"]);
}

/// A lost `owner.json` in a directory that changed minutes ago stays a stop that names the directory, why nothing is
///  recovered, when the wrapper recovers it by itself and the one safe removal by hand. The repository is not stuck for
///  good: the next test shows the same directory converging once it is old.
#[test]
fn a_lost_owner_name_stops_a_fresh_directory_and_names_the_way_out() {
    for phase in PHASES_BEFORE_THE_LANDING_BARRIER {
        let paused: Paused = pause_at(phase);
        let directory: PathBuf = transaction_directory(paused.repo.join(".git").as_path());
        lose_name(directory.as_path(), "owner.json");
        let first: Probe = status(&paused);
        let second: Probe = status(&paused);
        let context: String = format!("{phase}: {}", first.stderr);
        assert_eq!(first.code, Some(2), "{context}");
        assert_eq!(second.code, Some(2), "{context}");
        assert!(first.stderr.contains("Transaction owner record is missing"), "{context}");
        assert!(first.stderr.contains(directory.to_string_lossy().as_ref()), "the directory is named: {context}");
        assert!(first.stderr.contains(format!("rm -r {}", directory.display()).as_str()), "the removal is exact: {context}");
        assert!(first.stderr.contains("Its newest entry changed"), "why it is not recovered: {context}");
        assert!(first.stderr.contains("60 minutes"), "when the wrapper acts by itself: {context}");
        assert!(first.stderr.contains("next guarded command"), "which command recovers it: {context}");
        assert!(first.stderr.contains("landing-<n>.json"), "when removal by hand loses work: {context}");
        assert!(directory.exists(), "{context}");
        remove_settled(&paused.fixture);
    }
}

/// A lost `owner.json` in a directory nothing changed for the patience and no process uses converges at every phase:
///  the next command recovers it as a dead owner's, and the commit after it lands. Before the branch moved the commit
///  is discarded without residue; after it moved, the landing is completed.
#[test]
fn a_lost_owner_name_in_an_aged_directory_converges() {
    for phase in PHASES_BEFORE_THE_LANDING_BARRIER.iter().chain(PHASES_AFTER_THE_BRANCH_MOVED.iter()) {
        let paused: Paused = pause_at(phase);
        let directory: PathBuf = transaction_directory(paused.repo.join(".git").as_path());
        lose_name(directory.as_path(), "owner.json");
        age_tree(directory.as_path());
        let outcome: Outcome = settle(paused);
        let expected: &str = if PHASES_AFTER_THE_BRANCH_MOVED.contains(phase) { "completes" } else { "discards" };
        assert_eq!(outcome.cell, expected, "{phase}: {outcome:?}");
    }
}

/// A process with its working directory in an old directory is a use: the stop stays and names the process, and the
///  same directory converges once the process is gone. The sleeper stands in for a live wrapper that lost its name.
#[test]
fn a_live_process_in_an_aged_directory_keeps_the_stop_until_it_is_gone() {
    for phase in ["capture-locked", "landing-locked"] {
        let paused: Paused = pause_at(phase);
        let directory: PathBuf = transaction_directory(paused.repo.join(".git").as_path());
        lose_name(directory.as_path(), "owner.json");
        age_tree(directory.as_path());
        let holder: Sleeper = Sleeper::start(directory.as_path(), None);
        let blocked: Probe = status(&paused);
        let context: String = format!("{phase}: {}", blocked.stderr);
        assert_eq!(blocked.code, Some(2), "{context}");
        assert!(blocked.stderr.contains(format!("PID {} (sleep)", holder.pid()).as_str()), "the process is named: {context}");
        assert!(directory.exists(), "{context}");
        drop(holder);
        let outcome: Outcome = settle(paused);
        assert_eq!(outcome.cell, "discards", "{phase}: {outcome:?}");
    }
}
