//! The tests of mechanism D:
//! old records go,
//! the oldest go while over the size cap,
//! and a record in use,
//! a recent one,
//! and anything that is not a run record stay.

use std::{fs, path::{Path, PathBuf}, process::{Child, Command}, time::Duration};

use crate::cli::LeanAction;
use crate::lean_command::{absent_notes, directories_from, run_lean_action};
use crate::lean_identity::{Deadline, now_seconds};
use crate::lean_run_records::{RecordsOutcome, RetentionPolicy, is_record_name, sweep_run_records};
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

/// Every name a program in this repository writes under a verification directory is a run record,
/// each beside the program that writes it,
/// so the retention reaches what the tools leave behind.
#[test]
fn every_name_a_producer_in_this_repository_writes_is_a_run_record() {
    const PRODUCED: [(&str, &str); 22] = [
        ("campaign-abc123", "package/git-policy/cli.fuzz/bin/container.ts"),
        ("native-abc123", "package/git-policy/cli/bin/test-native-container.ts"),
        ("native-mutation-abc123", "package/git-policy/cli/bin/native-gate-proof.ts"),
        ("git-image-abc123", "package/git-policy/cli/bin/build-git-test-image.ts"),
        ("mutation-abc123", "package/linter/monochromatic-lint/bin/mutate-container.ts"),
        ("coverage-abc123", "package/mutation-test/cli-cargo-mutants/src/scope-tools.ts"),
        ("processors-mutation-abc123", "package/linter/monochromatic-lint/bin/mutate-processors.ts"),
        ("planted-abc123", "package/linter/monochromatic-lint.fuzz/bin/planted-controls.ts"),
        ("container-abc123", "package/forbidden-strings/cli/bin/container-snapshot.ts"),
        ("clippy-abc123", "package/forbidden-strings/cli/bin/test-container.ts"),
        ("release-test-abc123", "package/forbidden-strings/cli/bin/test-container.ts"),
        ("test-abc123", "package/forbidden-strings/cli/bin/test-container.ts"),
        ("guard-protected-abc123", "package/forbidden-strings/cli/bin/guard-container.ts"),
        ("guard-without-name-catch-abc123", "package/forbidden-strings/cli/bin/guard-container.ts"),
        ("guard-observer-protected-abc123", "package/forbidden-strings/cli/bin/observe-guards-container.ts"),
        ("pathname-mutation-abc123", "package/forbidden-strings/cli/bin/mutate-container.ts"),
        ("pathname-mutation-list-abc123", "package/forbidden-strings/cli/bin/mutate-container.ts"),
        ("startup-mutation-abc123", "package/forbidden-strings/cli/bin/mutate-container.ts"),
        ("embedding-fuzz-abc123", "package/forbidden-strings/cli.fuzz/bin/smoke-container.ts"),
        ("fuzz-clippy-abc123", "package/forbidden-strings/cli.fuzz/bin/smoke-container.ts"),
        ("windows-native-abc123", "a records directory kept from 2026-10-05, which no current program writes"),
        ("windows-baseline-abc123", "a records directory kept from 2026-10-05, which no current program writes"),
    ];
    for (name, producer) in PRODUCED {
        assert!(is_record_name(name), "{producer} writes {name}, which the retention does not sweep");
    }
}

/// What no program writes under a verification directory is not a run record and stays,
/// each beside what it is.
#[test]
fn what_is_not_a_run_record_stays_out_of_the_retention() {
    const NOT_RECORDS: [(&str, &str); 8] = [
        ("docs", "the rendered documentation of package/forbidden-strings/cli/bin/render-docs.ts"),
        ("reconciliation.json", "the planted-controls reconciliation of one task run"),
        ("nextest-cache", "a Cargo nextest cache directory"),
        ("strace-cache", "a strace cache directory"),
        ("processors-handover.html", "the handover rendered for a person by package/linter/monochromatic-lint"),
        ("processors-manifest.json", "the manifest that handover is read with"),
        ("guard-input-comparisons.json", "the fixed name package/forbidden-strings/cli/bin/verify-evidence.ts rewrites"),
        ("guard-source-comparison", "the fixed name package/forbidden-strings/cli/bin/verify-evidence.ts rewrites"),
    ];
    for (name, what) in NOT_RECORDS {
        assert!(!is_record_name(name), "{name} ({what}) would be swept as a run record");
    }
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

/// A record of a producer an earlier list did not name goes by the same retention,
/// so the sweep reaches what the linter and the scanner actually leave behind.
#[test]
fn a_record_of_any_producer_goes_by_the_age_rule() {
    let root = scratch();
    let coverage: PathBuf = record(root.path(), "coverage-7G78qe", 8192, 20 * DAY);
    let clippy: PathBuf = record(root.path(), "clippy-37qD2W", 8192, 20 * DAY);
    let processors: PathBuf = record(root.path(), "processors-mutation-bklcVi", 8192, 20 * DAY);
    let outcome: RecordsOutcome = sweep(root.path(), &POLICY);
    assert_eq!(outcome.removed, 3, "{outcome:?}");
    assert!(!coverage.exists() && !clippy.exists() && !processors.exists());
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

/// What a producing task runs sweeps the directory it names and leaves a sibling directory alone,
/// and a directory that is not there yet is nothing to sweep rather than a failure.
#[test]
fn the_records_action_sweeps_only_the_directories_it_is_given() {
    let root = scratch();
    let produced: PathBuf = root.path().join("produced");
    let sibling: PathBuf = root.path().join("sibling");
    let missing: PathBuf = root.path().join("not-written-yet");
    let now: i64 = now_seconds();
    let old_here: PathBuf = record(&produced, "native-old", 8192, 0);
    let old_elsewhere: PathBuf = record(&sibling, "native-old", 8192, 0);
    age_everything(&old_here, now - 20 * DAY);
    age_everything(&old_elsewhere, now - 20 * DAY);
    let action = LeanAction::Records { directory: vec![produced, missing], dry_run: false, deadline_minutes: Some(1) };

    let code: i32 = run_lean_action(&action);

    assert_eq!(code, 0);
    assert!(!old_here.exists(), "the record of the named directory was kept");
    assert!(old_elsewhere.exists(), "the sweep reached a directory it was not given");
}

/// A directory named relative to the folder a task runs in,
/// reached through a symbolic link or spelled with a `..` step,
/// is resolved to the one path the in-use scan compares with the paths the kernel reports.
#[test]
fn every_spelling_of_the_same_directory_is_resolved_to_one_path() {
    let root = scratch();
    let real: PathBuf = root.path().join("checkout/target/verification");
    fs::create_dir_all(&real).expect("a verification directory");
    let link: PathBuf = root.path().join("link-to-checkout");
    std::os::unix::fs::symlink(root.path().join("checkout"), &link).expect("a symbolic link");
    let through_a_parent_step: PathBuf = PathBuf::from("checkout/target/verification/../verification");
    let canonical: PathBuf = fs::canonicalize(&real).expect("the resolved directory");

    assert_eq!(directories_from(root.path(), &[link.join("target/verification")]), vec![canonical.clone()]);
    assert_eq!(directories_from(root.path(), &[through_a_parent_step]), vec![canonical.clone()]);
    assert_eq!(directories_from(root.path(), &[PathBuf::from("checkout/target/verification")]), vec![canonical]);
    // A directory that is not there yet keeps its joined name, because nothing can be using it.
    assert_eq!(directories_from(root.path(), &[PathBuf::from("not/there")]), vec![root.path().join("not/there")]);
}

/// The proof that a record is in use survives a directory named through a symbolic link.
/// The scan compares each root with the paths the kernel reports,
/// which are resolved,
/// so a record a process holds is kept whatever spelling the task used.
#[test]
fn a_record_a_process_uses_is_kept_when_the_directory_is_named_through_a_link() {
    let root = scratch();
    let checkout: PathBuf = root.path().join("checkout");
    let verification: PathBuf = checkout.join("target/verification");
    let used: PathBuf = record(&verification, "native-used", 8192, 0);
    age_everything(&used, now_seconds() - 30 * DAY);
    let link: PathBuf = root.path().join("link-to-checkout");
    std::os::unix::fs::symlink(&checkout, &link).expect("a symbolic link");
    let child: Child = Command::new("sleep").arg("120").current_dir(&used).spawn().expect("start sleep");
    let _holder: Sleeper = Sleeper { child };
    let action = LeanAction::Records { directory: vec![link.join("target/verification")], dry_run: false, deadline_minutes: Some(1) };

    let code: i32 = run_lean_action(&action);

    assert_eq!(code, 0);
    assert!(used.exists(), "a record a process uses was removed when its directory was named through a link");
}

/// A directory that is not there yet is named and is not a failure;
/// a directory that is there is not named.
#[test]
fn a_directory_that_is_not_there_yet_is_named_and_not_a_failure() {
    let root = scratch();
    let present: PathBuf = root.path().join("present");
    fs::create_dir(&present).expect("a directory");

    let notes: Vec<String> = absent_notes(&[present, root.path().join("not-written-yet")]);

    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].contains("not-written-yet") && notes[0].contains("holds no record"), "{notes:?}");
}
