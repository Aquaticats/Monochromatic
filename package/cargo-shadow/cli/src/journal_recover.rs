//! What:
//! The recovery of a run that was killed:
//! take the lock of its journal,
//! decide per root whether the leftover lockfile is kept or the
//! snapshot put back,
//! and put back the manifests of a killed `--breaking` run.
//! Why:
//! `journal.rs` holds the recording side;
//! the recovery side was split out to keep each file under the line limit.
//! What happens to one journal was split out of this file into `journal_recover_one.rs`,
//! for the same reason.
//! A journal is recovered only when its owner's lock can be taken (the owner is gone) and only after the same lock file is
//! confirmed to be the one at the journal's path.
//! A root that a live transaction covers,
//! a root whose lockfile could not be settled and a root with no decision are never
//! restored,
//! so a lockfile and its manifests are never split.
//! Nothing here waits:
//! a held lock is an answer,
//! and the journal stays for a later run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-recover.ts
//! export function recover(root: string, judge: Judge): string[]
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::fs::{self, File}` for this file.
use std::fs;
/// Imports `std::io` for this file.
use std::io;
// Imports `std::os::unix::fs::MetadataExt`.
/// Imports `std::os::unix::fs::MetadataExt` for this file.
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{
    Path,
    PathBuf,
};

// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
/// Imports the lockfile snapshot the judgement reads for this file.
use crate::crates::LockSnapshot;
// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, J...`.
/// Imports the journal directory name and the no-op test hook for this file.
use crate::journal::{
    JOURNAL_DIR,
    no_hook,
};
// Imports `crate::journal_transaction::{mark_done_and_...`.
/// Imports `crate::journal_transaction::{finish_journal, take_workspace}` for this file.
use crate::journal_transaction::{
    finish_journal,
    take_workspace,
};
/// Imports the process-group probe and the error code that says a group is empty.
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Re-exports the survivor check,
/// which lives in its own file.
pub(crate) use crate::journal_unready::survivor_note;

/// Imports the recovery of one journal,
/// which lives in its own file (split out of this module).
use crate::journal_recover_one::recover_one;

/// The ready record of a journal that only its tests read.
#[cfg(test)]
pub(crate) use crate::journal_directory_readers::read_ready;
/// Reads of a journal directory:
/// index,
/// ready record and listing (split out of the journal recovery module).
pub(crate) use crate::journal_directory_readers::{
    JournalListing,
    list_journals,
    read_journal,
};

/// What:
/// `pub(crate) const KEPT_JOURNAL: &str` is the clause both refusals carry about the journal they kept:
/// what became of it,
/// which later runs settle it,
/// which leave it,
/// and what a run that cannot show what this run left asks a person to run.
/// Why:
/// The install and the update say the same thing about one journal,
/// and a copy in each place would drift;
/// it names no scope,
/// because a stop of a bare process group keeps the same journal and records a process group instead,
/// and it names the runs by the rule that covers them rather than by a list of kinds,
/// so a command added later is covered or excluded by what it does.
/// Measured against a refusal staged on a test build (`refusal_journal_next_runs_tests`,
/// 2026-11-27):
/// `cargo check`,
/// `cargo metadata`,
/// `cargo update`,
/// `cargo-shadow install`,
/// `cargo-shadow update`,
/// `install --dry-run` and `update --dry-run` settle the journal;
/// `cargo fmt`,
/// `--version`,
/// a `CI` run,
/// `install --frozen-lockfile`,
/// a run outside every catalog repository,
/// and a run on the fast path leave it and print nothing about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const KEPT_JOURNAL = 'the journal of this run was kept, and ...'
/// ```
pub(crate) const KEPT_JOURNAL: &str = "the journal of this run was kept, and any later run of this catalog that checks or updates a lockfile settles it once it can show that nothing this run started is still running (`cargo check`, `cargo metadata`, `cargo update`, `cargo-shadow install` or `cargo-shadow update`, a `--dry-run` included); a frozen run (`--locked`, `--frozen`, or `CI`), a run whose lockfile and remembered policy pass are already fresh (the fast path), a run that resolves nothing (`cargo fmt`, `cargo --version`), and a run outside this catalog repository leave it as it is and print nothing about it; a run that cannot show what this run left says so, names the journal, and prints the `cargo-shadow acknowledge <journal directory>` command to run once every process or worker of this run that can modify the workspace has stopped";

/// What:
/// `pub fn recover(root, judge) -> Vec<String>` finishes the journals of runs that no longer exist.
/// Why:
/// It is `recover_with` with no test hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recover(root: string, judge: Judge): string[]
/// ```
pub fn recover(
    root: &Path,
    judge: &dyn Judge,
) -> Vec<String> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return recover_with(root, judge, &no_hook);
}

/// What a recovery says and whether the command that ran it may go on.
#[derive(Debug)]
pub struct RecoveryReport {
    /// What to tell the person,
    /// one warning per entry.
    pub messages: Vec<String>,
    /// Whether a mandatory phase of finishing a recovered journal failed (the output barrier before the marker,
    /// or the durable
    /// completion marker itself):
    /// the journal is not finished,
    /// so the command must stop.
    /// A failed removal of a journal that is
    /// already marked done is not one;
    /// it only warns.
    pub mandatory_failure: bool,
}

/// What:
/// `pub trait Judge` is the judgement a recovery asks of the policy about one root:
/// the findings that would refuse a killed run's lockfile.
/// `judge(old, live)` takes the lockfile as the killed run found it and as it stands now,
/// and returns the refusals (empty when it passes).
/// Why:
/// A recovery takes the judgement as a value instead of a closure.
/// Every function with the shape `Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>` is a `Judge` too,
/// so a test passes a closure and the installer passes a `PolicyJudge`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Judge { judge(old: LockSnapshot[], live: LockSnapshot[]): string[]; }
/// ```
pub trait Judge {
    /// What:
    /// `judge(&self, old, live)` returns the refusals for one root.
    /// Why:
    /// An empty list lets the recovery restore or keep the lockfile.
    fn judge(
        &self,
        old: &[LockSnapshot],
        live: &[LockSnapshot],
    ) -> Vec<String>;
}

/// What:
/// The implementation of `Judge` for every function with the judgement's shape.
/// Why:
/// Tests and callers that already hold such a function need no wrapper.
impl<F: Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>> Judge for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the judgement.
    fn judge(
        &self,
        old: &[LockSnapshot],
        live: &[LockSnapshot],
    ) -> Vec<String> {
        return self(old, live);
    }
}

/// What:
/// `pub fn recover_report(root, judge) -> RecoveryReport` is `recover` that also says whether a mandatory phase of finishing failed.
/// Why:
/// The callers of a recovery stop on a failed mandatory phase and go on after a cleanup warning (certification,
/// second pass,
/// V5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverReport(root: string, judge: Judge): { messages: string[]; mandatoryFailure: boolean }
/// ```
pub fn recover_report(
    root: &Path,
    judge: &dyn Judge,
) -> RecoveryReport {
    return recover_report_with(root, judge, &no_hook);
}

/// What:
/// `pub fn recover_with(root, judge, between) -> Vec<String>` is `recover` with a hook that runs before each replacement.
/// Why:
/// Tests stop the recovery at the point where another process could interfere.
/// The workspace lock is taken first and the journals are listed only afterwards,
/// so a journal published while this run waited for the lock is seen,
/// and a surviving child of any journal holds back every recovery of the workspace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverWith(root: string, judge: Judge, between: (path: string) => void): string[]
/// ```
pub fn recover_with(
    root: &Path,
    judge: &dyn Judge,
    between: &dyn Fn(&Path),
) -> Vec<String> {
    return recover_report_with(root, judge, between).messages;
}

/// What:
/// `pub fn recover_report_with(root, judge, between) -> RecoveryReport` is `recover_with` that also reports a mandatory failure.
/// Why:
/// It is the one implementation of the recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverReportWith(root: string, judge: Judge, between: (path: string) => void): RecoveryReport
/// ```
pub fn recover_report_with(
    root: &Path,
    judge: &dyn Judge,
    between: &dyn Fn(&Path),
) -> RecoveryReport {
    let mut mandatory: bool = false;
    let messages: Vec<String> = recover_messages(root, judge, between, &mut mandatory);
    return RecoveryReport {
        messages,
        mandatory_failure: mandatory,
    };
}

/// What:
/// `fn recover_messages(root, judge, between, mandatory) -> Vec<String>` runs the recovery and sets `mandatory` when a mandatory
/// phase of finishing a journal failed.
/// Why:
/// The early returns of the recovery keep their form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverMessages(root: string, judge: Judge, between: (path: string) => void, mandatory: { value: boolean }): string[]
/// ```
fn recover_messages(
    root: &Path,
    judge: &dyn Judge,
    between: &dyn Fn(&Path),
    mandatory: &mut bool,
) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let journals: PathBuf = root.join(JOURNAL_DIR);
    // A metadata failure is not "no journals" (round 7, 6b2 1).
    match fs::metadata(&journals) {
        Ok(found) if found.is_dir() => {}
        Ok(_) => {
            messages.push(format!(
                "{} is not a directory; the journals of earlier runs were left for a later run",
                journals.display()
            ));
            return messages;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return messages,
        Err(error) => {
            messages.push(format!("{} could not be examined ({error}); the journals of earlier runs were left for a later run", journals.display()));
            return messages;
        }
    }
    let _workspace = match take_workspace(root) {
        Ok(Ok(held)) => held,
        Ok(Err(holder)) => {
            messages.push(format!(
                "{holder}; the journals of earlier runs were left for a later run"
            ));
            return messages;
        }
        Err(error) => {
            messages.push(format!("the workspace lock could not be tried ({error}); the journals of earlier runs were left for a later run"));
            return messages;
        }
    };
    // An entry that cannot be listed or examined stops the recovery: the set of journals, and so of surviving children, is unknown
    // (round 7, 6b2 1).
    let listing: JournalListing = match list_journals(&journals) {
        Ok(Some(found)) => found,
        Ok(None) => return messages,
        Err(error) => {
            messages.push(format!("{} could not be listed ({error}); the journals of earlier runs were left for a later run", journals.display()));
            return messages;
        }
    };
    for directory in listing.done {
        // A journal already marked done still has its completion barrier to make: the earlier rename may not be durable, so the parent
        // is synchronized again here, and a failure of that is a mandatory failure, not a cleanup (certification, third pass).
        if let Err(error) = finish_journal(&directory) {
            messages.push(format!("{}: a finished journal could not be completed or removed ({error}); it will be tried again", directory.display()));
            if !error.cleanup_only() {
                *mandatory = true;
            }
        }
    }
    let open: Vec<PathBuf> = listing.open;
    let mut survivors: Vec<String> = Vec::new();
    for directory in &open {
        if let Some(note) = survivor_note(directory) {
            survivors.push(note);
        }
    }
    if !survivors.is_empty() {
        messages.extend(survivors);
        messages.push("no journal of this workspace was recovered while a child of an earlier run may still be writing".to_string());
        return messages;
    }
    for directory in open {
        recover_one(root, &directory, judge, between, &mut messages, mandatory);
    }
    return messages;
}
