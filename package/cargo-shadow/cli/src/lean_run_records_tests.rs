//! The tests of mechanism D:
//! old records go,
//! the oldest go while over the size cap,
//! and a record in use,
//! a recent one,
//! and anything that is not a run record stay.

use std::{fs, path::{Path, PathBuf}, process::{Child, Command}, time::Duration};

use crate::lean_identity::Deadline;
use crate::lean_run_records::{RecordsOutcome, RetentionPolicy, sweep_run_records};
use crate::lean_test_support::{DAY, HOUR, NOW, age_everything, real_environment};
use crate::test_scratch::scratch;

/// A child stopped when the guard drops.
struct Sleeper {
    /// The child.
    child: Child,
}

impl Drop for Sleeper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The retention of these tests.
const POLICY: RetentionPolicy = RetentionPolicy { max_age_seconds: 14 * DAY, max_bytes: 1 << 30, protect_seconds: DAY };

/// Makes a record directory with a file of the given size,
/// aged by `idle` seconds.
fn record(directory: &Path, name: &str, bytes: usize, idle: i64) -> PathBuf {
    let path: PathBuf = directory.join(name);
    fs::create_dir_all(&path).expect("record directory");
    fs::write(path.join("log"), vec![5_u8; bytes]).expect("record file");
    age_everything(&path, NOW - idle);
    return path;
}

/// Runs the sweep for real.
fn sweep(directory: &Path, policy: &RetentionPolicy) -> RecordsOutcome {
    let mut log: Vec<String> = Vec::new();
    return sweep_run_records(directory, policy, &real_environment(), NOW, false, &Deadline::after(Duration::from_secs(60)), &mut log);
}

#[test]
fn a_record_older_than_the_retention_goes_and_a_younger_one_stays() {
    let root = scratch();
    let old: PathBuf = record(root.path(), "campaign-old", 8192, 20 * DAY);
    let young: PathBuf = record(root.path(), "native-young", 8192, 3 * DAY);
    let outcome: RecordsOutcome = sweep(root.path(), &POLICY);
    assert_eq!(outcome.removed, 1, "{outcome:?}");
    assert!(outcome.bytes > 0);
    assert!(!old.exists() && young.exists());
}

#[test]
fn over_the_size_cap_the_oldest_go_first_and_the_directory_stops_at_the_cap() {
    let root = scratch();
    let oldest: PathBuf = record(root.path(), "mutation-1", 100_000, 8 * DAY);
    let middle: PathBuf = record(root.path(), "mutation-2", 100_000, 6 * DAY);
    let newest: PathBuf = record(root.path(), "mutation-3", 100_000, 4 * DAY);
    let policy: RetentionPolicy = RetentionPolicy { max_bytes: 250_000, ..POLICY };
    let outcome: RecordsOutcome = sweep(root.path(), &policy);
    assert_eq!(outcome.removed, 1, "{outcome:?}");
    assert!(!oldest.exists() && middle.exists() && newest.exists());
}

#[test]
fn a_record_written_within_the_protection_window_is_never_removed_even_over_the_cap() {
    let root = scratch();
    let fresh: PathBuf = record(root.path(), "campaign-fresh", 100_000, HOUR);
    let policy: RetentionPolicy = RetentionPolicy { max_bytes: 1, ..POLICY };
    assert_eq!(sweep(root.path(), &policy).removed, 0);
    assert!(fresh.exists());
}

#[test]
fn a_record_a_process_uses_is_kept() {
    let root = scratch();
    let used: PathBuf = record(root.path(), "native-used", 8192, 30 * DAY);
    let other: PathBuf = record(root.path(), "native-unused", 8192, 30 * DAY);
    let child: Child = Command::new("sleep").arg("120").current_dir(&used).spawn().expect("start sleep");
    let _holder: Sleeper = Sleeper { child };
    let outcome: RecordsOutcome = sweep(root.path(), &POLICY);
    assert_eq!((outcome.removed, outcome.kept_in_use), (1, 1), "{outcome:?}");
    assert!(used.exists() && !other.exists());
}

#[test]
fn files_and_directories_that_are_not_run_records_are_never_touched() {
    let root = scratch();
    fs::write(root.path().join("processors-handover.html"), b"for the owner").expect("handover");
    age_everything(&root.path().join("processors-handover.html"), NOW - 90 * DAY);
    let odd: PathBuf = record(root.path(), "interop-keep", 8192, 90 * DAY);
    assert_eq!(sweep(root.path(), &POLICY).removed, 0);
    assert!(root.path().join("processors-handover.html").exists() && odd.exists());
}

#[test]
fn a_dry_run_reports_and_removes_nothing() {
    let root = scratch();
    let old: PathBuf = record(root.path(), "git-image-old", 8192, 20 * DAY);
    let mut log: Vec<String> = Vec::new();
    let outcome: RecordsOutcome = sweep_run_records(root.path(), &POLICY, &real_environment(), NOW, true, &Deadline::after(Duration::from_secs(60)), &mut log);
    assert_eq!(outcome.removed, 1);
    assert!(old.exists());
}
