//! What:
//!  A record whose directory entry a power loss took: the file was synced, its name in the directory was not, so the
//!  record is absent after the crash although it was written. The outcome of the next wrapped commands must equal the
//!  outcome with the record never written.
//! Why:
//!  The directory syncs after `preparing.json`, `index-lock-1.json` and `ref-updated.json`, and the sync of the
//!  `.pending` candidate directory, are dropped. They rest on a file sync persisting the file's own name, which no
//!  filesystem promise in this repository backs, so this module simulates the loss: it deletes the file the way the
//!  tearing sweep's delete damage does, and compares the result with a state built as if the record was never written.
//!  The first barrier that makes the names of the transaction directory durable is the directory sync of
//!  `landing-1.json`, which comes before the branch moves, so the losses are reachable only at the phases before it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lost = await classify(await pausedWithout(phase, 'preparing.json'));
//! expect(lost).toEqual(await classify(await pausedAsIfNeverWritten(phase, 'preparing.json')));
//! ```

use super::{Paused, Probe, classify, pause_at, probe};
use crate::commit::remove_settled;
use git_policy_cli::transaction_registry::is_transaction_id;
use std::path::{Path, PathBuf};

/// The phases a commit reaches before the directory sync of `landing-1.json`.
const PHASES_BEFORE_THE_LANDING_BARRIER: [&str; 4] =
    ["capture-locked", "preparation-done", "landing-locked", "objects-migrated"];

/// The phases a commit reaches after the branch moved, where `landing-1.json` is durable.
const PHASES_AFTER_THE_BRANCH_MOVED: [&str; 2] = ["ref-updated", "index-installed"];

/// What:
///  The one published transaction directory a killed commit left.
/// Why:
///   Every case works inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(registry)).filter(isTransactionId)[0]
/// ```
fn transaction_directory(git_dir: &Path) -> PathBuf {
    let found: Vec<PathBuf> = std::fs::read_dir(git_dir.join("cli-git-transactions"))
        .expect("registry")
        .flatten()
        .filter(|entry| { return entry.file_name().to_str().is_some_and(is_transaction_id); })
        .map(|entry| { return entry.path(); })
        .collect();
    assert_eq!(found.len(), 1, "exactly one transaction directory: {found:?}");
    return found[0].clone();
}

/// What:
///  Remove a record the way a lost directory entry does.
/// Why:
///   The record is absent after the crash although it was written, and the records written after it stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(join(directory, name));
/// ```
fn lose_name(directory: &Path, name: &str) {
    std::fs::remove_file(directory.join(name)).unwrap_or_else(|error| { panic!("{name} is present to lose: {error}"); });
}

/// What:
///  What the next wrapped commands did, with the first line of what the person was told.
/// Why:
///   Two states are equal when the recorded word and the failure the person reads are the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ cell, note })
/// ```
#[derive(Debug, Eq, PartialEq)]
struct Outcome {
    /// The classified word of the tearing table.
    cell: String,
    /// The first line of the standard error of the first probe.
    note: String,
}

/// What:
///  Classify a paused repository and remove its fixture.
/// Why:
///   Each case starts from a fresh killed commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await classify(paused); await remove(paused);
/// ```
fn settle(paused: Paused) -> Outcome {
    let (cell, note): (String, String) = classify(&paused);
    remove_settled(&paused.fixture);
    return Outcome { cell, note };
}

/// The names `preparing.json` loses, compared at each phase before the landing barrier with a transaction directory
///  that holds the owner record only and a repository with no shadow: a commit killed before it wrote `preparing.json`.
#[test]
fn a_lost_preparing_name_equals_a_preparing_record_never_written() {
    for phase in PHASES_BEFORE_THE_LANDING_BARRIER {
        let lost: Paused = pause_at(phase);
        let directory: PathBuf = transaction_directory(lost.repo.join(".git").as_path());
        lose_name(directory.as_path(), "preparing.json");
        let lost_outcome: Outcome = settle(lost);
        let never: Paused = pause_at(phase);
        let never_directory: PathBuf = transaction_directory(never.repo.join(".git").as_path());
        for entry in std::fs::read_dir(never_directory.as_path()).expect("transaction directory").flatten() {
            if entry.file_name() != "owner.json" {
                let path: PathBuf = entry.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(path).expect("remove a later directory");
                } else {
                    std::fs::remove_file(path).expect("remove a later record");
                }
            }
        }
        std::fs::remove_dir_all(never.repo.join(".git/cli-git/shadow")).expect("remove the shadow");
        let never_outcome: Outcome = settle(never);
        eprintln!("{phase}: lost {lost_outcome:?}; never written {never_outcome:?}");
        assert_eq!(lost_outcome.cell, never_outcome.cell, "{phase}: {lost_outcome:?} against {never_outcome:?}");
        assert!(
            lost_outcome.cell.starts_with("discards"),
            "{phase}: a lost preparing.json discards the commit without sticking: {lost_outcome:?}"
        );
    }
}

/// The name `index-lock-1.json` loses, compared at each phase that holds the real index lock with a repository where
///  the lock exists and neither its record nor its PID file does: a commit killed between creating the lock and writing
///  the record.
///  Equal does not mean converged: both cells are `discards+residue!commit-exit-2`, a stale `index.lock` that fails every
///  later commit until it is removed by hand, and the window for it is wider than before the directory sync was dropped.
#[test]
fn a_lost_index_lock_name_equals_a_lock_record_never_written() {
    for phase in ["landing-locked", "objects-migrated"] {
        let lost: Paused = pause_at(phase);
        let directory: PathBuf = transaction_directory(lost.repo.join(".git").as_path());
        lose_name(directory.as_path(), "index-lock-1.json");
        let lost_outcome: Outcome = settle(lost);
        let never: Paused = pause_at(phase);
        let never_directory: PathBuf = transaction_directory(never.repo.join(".git").as_path());
        lose_name(never_directory.as_path(), "index-lock-1.json");
        std::fs::remove_file(never.repo.join(".git/index~pid.lock")).expect("the PID file is announced after the record");
        let never_outcome: Outcome = settle(never);
        eprintln!("{phase}: lost {lost_outcome:?}; never written {never_outcome:?}");
        assert_eq!(lost_outcome.cell, never_outcome.cell, "{phase}: {lost_outcome:?} against {never_outcome:?}");
    }
}

/// The name `ref-updated.json` loses once the branch moved: the landing record is durable, so recovery finishes the
///  landing exactly as it does when the record was never written because the kill came right after the move.
#[test]
fn a_lost_ref_updated_name_still_completes_the_landing() {
    for phase in PHASES_AFTER_THE_BRANCH_MOVED {
        let lost: Paused = pause_at(phase);
        let directory: PathBuf = transaction_directory(lost.repo.join(".git").as_path());
        lose_name(directory.as_path(), "ref-updated.json");
        let outcome: Outcome = settle(lost);
        assert_eq!(outcome.cell, "completes", "{phase}: {outcome:?}");
    }
}

/// What:
///  The standard error of the next guarded command after the owner record's name was lost, and its exit code.
/// Why:
///   This is the one lost name that does not leave the state of a record never written, so the finding needs the text
///  the person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await run('git status') // { code, stderr }
/// ```
fn status_after_losing_the_owner_name(phase: &str) -> Probe {
    let paused: Paused = pause_at(phase);
    let directory: PathBuf = transaction_directory(paused.repo.join(".git").as_path());
    lose_name(directory.as_path(), "owner.json");
    let first: Probe = probe(&paused.fixture, paused.repo.as_path(), &["status", "--porcelain=v1"]);
    remove_settled(&paused.fixture);
    return first;
}

/// The owner record's name lost from the published directory (the candidate directory sync dropped): every phase before
///  the landing barrier sticks, and the failure names the directory.
///  This pins a known gap, not a goal: the repository is stuck until the directory is removed by hand
///  (`doc/planning/cli-git-spawn-and-sync-reduction.md`, section "J4 outcome"); a recovery that converges here should
///  change this test.
#[test]
fn a_lost_owner_name_pins_the_known_gap_of_a_stuck_repository_naming_the_directory() {
    for phase in PHASES_BEFORE_THE_LANDING_BARRIER {
        let first: Probe = status_after_losing_the_owner_name(phase);
        assert_eq!(first.code, Some(2), "{phase}: {}", first.stderr);
        eprintln!("{phase}: lost owner name, first status: code {:?}, {}", first.code, first.stderr);
        assert!(first.stderr.contains("Transaction owner record is missing"), "{phase}: {}", first.stderr);
        assert!(first.stderr.contains("cli-git-transactions"), "{phase}: {}", first.stderr);
    }
}
