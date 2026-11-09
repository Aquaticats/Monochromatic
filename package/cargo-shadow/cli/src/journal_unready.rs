//! What:
//! The recovery of journals that never became ready,
//! and the check for children of a killed run that may still be writing.
//! Why:
//! Round 6 (fifth review 5b2 1 to 4):
//! a survivor holds back every recovery and every new transaction,
//! an unreadable record is
//! uncertainty,
//! and a journal without a ready record is removed only when every payload is redundant and the live files are durable.
//! Split out of `journal_recover.rs`,
//! which holds the recovery of ready journals.
//! The assessment of one journal's records is split into `journal_unready_assessment.rs`,
//! to keep this file under the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function survivorNote(directory: string): string | undefined
//! export function recoverUnready(root: string, directory: string, messages: string[]): void
//! ```

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports the journal file names and the durability barrier for this file.
use crate::journal::{ACKNOWLEDGED_FILE, ACKNOWLEDGED_HEADER, make_durable, sync_directory};

/// Imports `crate::journal_transaction::finish_journal` for this file.
use crate::journal_transaction::finish_journal;

/// Imports the assessment of one journal's records for this file.
use crate::journal_unready_assessment::{Finding, assess};

/// The verdict on a journal that has no ready record (split out of the journal unready module).
pub(crate) use crate::journal_unready_verdict::{is_allocatable_process_id, sync_opened, unready_verdict};

/// The test hook that runs between the queries of a check and its judgement of unsettled scopes (split out of the
/// journal unready module).
#[cfg(test)]
pub(crate) use crate::journal_unready_assessment::after_query_for_test;

/// What:
/// `fn recover_unready(root: &Path, directory: &Path, messages: &mut Vec<String>)` removes a journal that never became ready
/// when every payload in it is redundant and the live files that make it redundant are durable,
/// and keeps it otherwise.
/// Why:
/// Deleting a durable copy while its live equal is still in the page cache could lose both in a power cut (round 6,
/// 5b2 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverUnready(root: string, directory: string, messages: string[]): void
/// ```
pub(crate) fn recover_unready(root: &Path, directory: &Path, messages: &mut Vec<String>, mandatory: &mut bool) {
    let live_files: Vec<String> = match unready_verdict(root, directory) {
        Ok(found) => found,
        Err(reason) => {
            messages.push(format!("{}: a journal without a ready record was kept: {reason}", directory.display()));
            return;
        }
    };
    for live in &live_files {
        #[cfg(test)]
        let outcome: io::Result<()> = if crate::failpoints::fires(crate::failpoints::Point::RecoverDurability) {
            Err(io::Error::other("injected recovery durability failure"))
        } else {
            make_durable(Path::new(live))
        };
        #[cfg(not(test))]
        let outcome: io::Result<()> = make_durable(Path::new(live));
        if let Err(error) = outcome {
            messages.push(format!("{}: a journal without a ready record was kept because {live} could not be made durable ({error})", directory.display()));
            *mandatory = true;
            return;
        }
    }
    if let Err(error) = finish_journal(directory) {
        messages.push(format!("{}: a journal that never became ready could not be removed ({error}); it will be tried again", directory.display()));
        if !error.cleanup_only() {
            *mandatory = true;
        }
    }
}

/// What:
/// `fn is_acknowledged(directory: &Path) -> bool` says whether a person acknowledged this journal.
/// Why:
/// An unreadable or partial file does not count:
/// an acknowledgement interrupted while it was written leaves the journal unacknowledged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAcknowledged(directory: string): boolean
/// ```
pub(crate) fn is_acknowledged(directory: &Path) -> bool {
    let file: PathBuf = directory.join(ACKNOWLEDGED_FILE);
    let Ok(text) = fs::read_to_string(&file) else {
        return false;
    };
    // The whole versioned record: the header, a process id, a time, three lines, each ended by a line feed.
    let lines: Vec<&str> = text.lines().collect();
    let complete: bool = text.ends_with('\n')
        && lines.len() == 3
        && lines[0] == ACKNOWLEDGED_HEADER
        && lines[1].parse::<u32>().is_ok_and(is_allocatable_process_id)
        && lines[2].parse::<u64>().is_ok();
    if !complete {
        return false;
    }
    // A name that is visible is not yet an acknowledgement that survives a crash: the reader makes the file and its directory entry
    // durable before it relies on them, and an acknowledgement it cannot make durable does not count (round 9, 8a).
    // The file first, then the directory entry that names it; a failure of either one means the acknowledgement does not count.
    // Each barrier is the real operation, and a test replaces its result, so a failure takes the path of a failure of the kernel
    // (sixth pass, group 13).
    #[cfg(test)]
    let file_durable: bool = if crate::failpoints::fires(crate::failpoints::Point::AcknowledgementFileSync) {
        false
    } else {
        fs::File::open(&file).and_then(sync_opened).is_ok()
    };
    #[cfg(not(test))]
    let file_durable: bool = fs::File::open(&file).and_then(sync_opened).is_ok();
    if !file_durable {
        return false;
    }
    #[cfg(test)]
    let directory_durable: bool = if crate::failpoints::fires(crate::failpoints::Point::AcknowledgementDirectorySync) {
        false
    } else {
        sync_directory(directory).is_ok()
    };
    #[cfg(not(test))]
    let directory_durable: bool = sync_directory(directory).is_ok();
    return directory_durable;
}

/// What:
/// `pub(crate) fn survivor_note(directory: &Path) -> Option<String>` says why a journal's recorded children may still be running,
/// or `None` when the records show they stopped.
/// Why:
/// The end of the wrapper's lock does not say whether its cargo ended.
/// A live recorded scope or process group always blocks.
/// Any other uncertainty about the containment (a weaker-path run,
/// a missing,
/// unreadable or damaged record,
/// a recorded scope that
/// cannot be examined,
/// another mount view) blocks until a person acknowledges the journal
/// (`cargo-shadow acknowledge`),
/// who thereby asserts that every process or worker of the run has stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function survivorNote(directory: string): string | undefined
/// ```
pub(crate) fn survivor_note(directory: &Path) -> Option<String> {
    return match assess(directory) {
        None => None,
        Some(Finding::Live(message)) => Some(message),
        Some(Finding::Uncertain(message)) => {
            if is_acknowledged(directory) {
                None
            } else {
                Some(message)
            }
        }
    };
}
