//! What:
//!  Removing one directory a pass chose,
//! and writing the audit line that says what went.
//! Why:
//!  The directory that was judged is the only thing removed:
//! a name that now stands for another directory is kept and said,
//! a link is never entered,
//! a failure part way is audited as partial and reported,
//! and a directory that cannot be removed is left with a debug line.
//! A line per directory,
//! written once it is gone,
//! means a kill between two removals leaves a line for each directory that
//! went and a whole directory for each that did not
//! (`discarded_prune.rs` runs the pass and `discarded_prune_listing.rs` read the folder).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = removeEntry(context, entry, path, reason);
//! ```

/// The debug log that says what was kept or only partly removed.
use crate::diagnostic_log::debug;
/// How long the audit file's lines are kept.
use crate::discard_audit::AUDIT_RETENTION;
/// The directory a removal reports and the line it writes.
use crate::discarded_prune_audit_line::{
    Removed,
    audit_line,
};
/// The limits the audit line states.
use crate::discarded_prune_limits::Limits;
/// Why a directory was chosen,
/// and what a pass says went wrong.
use crate::discarded_prune_plan::Reason;
use crate::discarded_prune_problems::{
    Problem,
    ProblemKind,
};
/// A directory's name as a time.
use crate::discarded_prune_stamps::Stamped;
/// The opened folder,
/// a directory's contents and what a walk left.
use crate::discarded_store::{
    Interrupted,
    Store,
    Tally,
};
/// One audit line appended under the record's own append rule.
use crate::log_lines_appended_without_a_lock::append_line;
/// `OsStr` is a borrowed file name in the system's own bytes.
use std::ffi::OsStr;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// Module tag for diagnostics.
const TAG: &str = "discardedPrune";
/// What a pass did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PruneOutcome {
    /// The directories removed,
    /// in order.
    pub removed: Vec<Removed>,
    /// What could not be done.
    pub problems: Vec<Problem>,
}

/// A problem with the directory at `path`,
/// of the given kind.
fn problem_at(
    path: &Path,
    kind: ProblemKind,
    detail: String,
) -> Problem {
    return Problem {
        path: path.to_path_buf(),
        kind,
        detail,
    };
}

/// Whether every file now in a directory was in the pass's listing of it.
/// A directory the listing does not know has none.
fn all_were_listed(
    context: &Removal<'_>,
    directory_name: &str,
    now: &[String],
) -> bool {
    let listed = context.listed_files.get(directory_name);
    for file in now {
        let was_listed: bool = match listed {
            Some(files) => files.contains(file),
            None => false,
        };
        if !was_listed {
            return false;
        }
    }
    return true;
}

/// What a removal needs besides the entry:
/// the folder,
/// where the audit goes,
/// the limits and the numbers its line states.
/// Its fields are `pub(crate)` because the pass in `discarded_prune.rs` fills one in for each chosen directory.
pub(crate) struct Removal<'pass> {
    /// The folder this pass opened.
    pub(crate) store: &'pass Store,
    /// The audit file.
    pub(crate) audit_path: &'pass Path,
    /// The limits the pass judges by.
    pub(crate) limits: Limits,
    /// The size of the folder before the pass.
    pub(crate) folder_bytes: u128,
    /// The time of the pass.
    pub(crate) now_millis: i64,
    /// The files each directory held when the pass listed it.
    pub(crate) listed_files:
        &'pass std::collections::HashMap<String, std::collections::HashSet<String>>,
}

/// Writes the audit line of a directory of which `tally` went;
/// said as a problem when the line cannot be written.
fn write_audit(
    context: &Removal,
    outcome_word: &str,
    removed: &Removed,
    tally: &Tally,
    outcome: &mut PruneOutcome,
) {
    let line: String = audit_line(
        outcome_word,
        removed,
        tally.names.as_slice(),
        context.limits,
        context.folder_bytes,
        context.now_millis,
    );
    if let Err(error) = append_line(context.audit_path, line.as_bytes(), &AUDIT_RETENTION) {
        debug(
            TAG,
            &format!(
                "the audit record of {} could not be written: {error}",
                removed.path.display()
            ),
        );
        outcome.problems.push(Problem {
            path: removed.path.clone(),
            kind: ProblemKind::RemovedWithoutAudit,
            detail: error.to_string(),
        });
    }
}

/// What:
///  Removes one chosen directory through the open folder:
/// opens it by name without following a link,
/// checks that it is the
///  directory the listing saw,
/// removes its files and subdirectories through its descriptor,
/// then the empty directory,
/// and
///  audits what went.
/// Why:
///   The directory that was judged is the only thing removed:
/// a name that now stands for another directory is kept and said,
///  a link is never entered,
/// and a failure part way is audited as partial and reported,
/// never counted as a removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeEntry(context, entry, path, reason, outcome)
/// ```
pub(crate) fn remove_entry(
    context: &mut Removal,
    entry: &Stamped,
    path: PathBuf,
    reason: Reason,
    outcome: &mut PruneOutcome,
) {
    let name: &OsStr = OsStr::new(entry.name.as_str());
    let directory = match context.store.open_directory(name) {
        Ok(Some(opened)) => opened,
        // Gone: another actor removed it, which is what this pass was going to do.
        Ok(None) => return,
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => {
            outcome.problems.push(problem_at(
                &path,
                ProblemKind::EntryChanged,
                String::from("the name is now a link or a file"),
            ));
            return;
        }
        Err(error) => {
            outcome.problems.push(problem_at(
                &path,
                ProblemKind::NotRemoved,
                error.to_string(),
            ));
            return;
        }
    };
    match directory.identity() {
        // The same directory the listing saw: device and inode alone say so, never the change time. Another pass
        //  removing its files moves that time, and a file added inside the tick the listing's stamp came from leaves
        //  it where it was, because this host's ZFS advances file times in one-millisecond steps
        //  (`doc/troubleshooting/openzfs-file-times-advance-in-one-millisecond-steps.md`). Only a directory that
        //  gained nothing is the directory that was judged, so what it holds is compared with the listing every
        //  time, and a directory that gained a file is kept and said (issue 6544).
        Ok(found)
            if found.device == entry.identity.device && found.inode == entry.identity.inode =>
        {
            match directory.survey() {
                Ok(now) if all_were_listed(context, entry.name.as_str(), &now.names) => {}
                Ok(_) => {
                    outcome.problems.push(problem_at(
                        &path,
                        ProblemKind::EntryChanged,
                        String::from("a file was added to it after the pass listed it"),
                    ));
                    return;
                }
                // Removed by another pass while it was looked at.
                Err(Interrupted {
                    error,
                    ..
                }) if error.kind() == std::io::ErrorKind::NotFound => return,
                Err(Interrupted {
                    error,
                    ..
                }) => {
                    outcome.problems.push(problem_at(
                        &path,
                        ProblemKind::NotRemoved,
                        error.to_string(),
                    ));
                    return;
                }
            }
        }
        Ok(_) => {
            outcome.problems.push(problem_at(
                &path,
                ProblemKind::EntryChanged,
                String::from("the name now stands for another directory"),
            ));
            return;
        }
        Err(error) => {
            outcome.problems.push(problem_at(
                &path,
                ProblemKind::NotRemoved,
                error.to_string(),
            ));
            return;
        }
    }
    let mut gone: Removed = Removed {
        path: path.clone(),
        bytes: 0,
        reason,
    };
    let tally: Tally = match directory.empty() {
        Ok(emptied) => emptied,
        Err(Interrupted {
            tally,
            error,
        }) => {
            if tally.names.is_empty() {
                outcome.problems.push(problem_at(
                    &path,
                    ProblemKind::NotRemoved,
                    error.to_string(),
                ));
            } else {
                gone.bytes = tally.bytes;
                write_audit(context, "partial", &gone, &tally, outcome);
                outcome.problems.push(problem_at(
                    &path,
                    ProblemKind::PartiallyRemoved,
                    error.to_string(),
                ));
            }
            return;
        }
    };
    gone.bytes = tally.bytes;
    match context.store.remove_empty_directory(name, entry.identity) {
        Ok(true) => {}
        Ok(false) => match context.store.directory_identity(name) {
            // Another pass removed the emptied directory first. It audits what it removed; this pass audits what it removed.
            Ok(None) if tally.names.is_empty() => return,
            Ok(None) => {}
            _ => outcome.problems.push(problem_at(&path, ProblemKind::EntryChanged, String::from("the name now stands for another directory, which was kept; the copies judged are gone"))),
        },
        Err(error) => {
            write_audit(context, "partial", &gone, &tally, outcome);
            outcome.problems.push(problem_at(&path, ProblemKind::PartiallyRemoved, format!("its copies are gone but the empty directory could not be removed: {error}")));
            return;
        }
    }
    write_audit(context, "pruned", &gone, &tally, outcome);
    outcome.removed.push(gone);
}
