//! What:
//!  The bound on `<common git dir>/monochromatic-git-policy-cli/discarded/`:
//! copies older than a number of days are removed,
//! and the folder is
//!  kept under a size by removing the oldest copies first,
//! never one younger than a day and never the copy just written.
//!  Each removed copy leaves one line in the audit file,
//! and `git monochromatic-git-policy-cli doctor` reports the folder.
//! Why:
//!  The copies of conflicted files were never removed,
//! so the folder grew for as long as the repository lived.
//! They are the
//!  only remaining content of a file a command replaced,
//! so the bound errs toward keeping:
//! both limits are settings of
//!  `monochromatic-git-policy-cli.config.jsonc` (`discardedCopies.maxAgeDays`,
//! `discardedCopies.maxSize`,
//! read by `config_discarded.rs`),
//! a copy
//!  younger than a day is out of reach of both,
//! and the pass runs only after a new copy is complete and durable (so a kill while copying costs the older copies
//!  nothing),
//! and touches only directories named like the wrapper's own (`2026-11-02T14-01-07`,
//! with an optional
//!  `-<number>`).
//! The folder is opened once without following a link and everything is read and removed through that
//!  descriptor (`discarded_store.rs`).
//! No lock is taken:
//! passes may overlap,
//! and a file is counted and audited by the one pass
//!  whose removal of it succeeded,
//! so overlapping passes split a directory's files between their audit lines and every copy
//!  is audited once.
//! A directory another pass removed first is not a problem.
//! No commit waits on a pass.
//! What a pass cannot do
//!  is kept and said,
//! never skipped silently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removed = await pruneDiscarded({ commonDir, nowMillis, limits, keep: newestDirectoryName });
//! ```

use crate::conflict_sides::SAVED_DIRECTORY;
use crate::diagnostic_log::debug;
use crate::discard_audit::{AUDIT_FILE, append_line, check_appendable};
use crate::discarded_store::{Interrupted, OpenFailure, Store, Tally};use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub use crate::discarded_prune_audit_line::{Removed, audit_line, prune_line};
pub use crate::discarded_prune_limits::{DEFAULT_MAX_AGE_DAYS, DEFAULT_MAX_BYTES, Limits, parse_size};
pub use crate::discarded_prune_plan::{Reason, plan};
pub use crate::discarded_prune_problems::{Listing, Problem, ProblemKind};
pub use crate::discarded_prune_stamps::{STAMP_RESOLUTION_MILLISECONDS, Stamped, YOUNG_FLOOR_MILLISECONDS, parse_stamp};
pub use crate::discarded_store::Identity;

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

/// What:
///  Every directory of the folder at `relative` below a Git directory,
/// oldest first,
/// with its size and what it was when
///  listed,
/// and what could not be read.
/// Why:
///   The folder is opened without following a link and each directory is opened through it,
/// so the sizes are those of the
///  directories that a later removal will meet by identity.
/// A directory that cannot be read has no size to judge by,
/// so it
///  is left out of the entries and named in the problems;
/// a folder that is absent is simply empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, problems } = await listStampedBelow(commonDir, SAVED_DIRECTORY);
/// ```
pub(crate) fn list_stamped_below(common_dir: &Path, relative: &str) -> Listing {
    return match Store::open(common_dir, relative) {
        Ok(Some(store)) => list_in(&store, common_dir.join(relative).as_path()),
        Ok(None) => Listing::default(),
        Err(failure) => Listing { problems: vec![unusable(common_dir.join(relative).as_path(), &failure)], ..Listing::default() },
    };
}

/// What:
///  Every directory of the folder that is named like the wrapper's own,
/// oldest first,
/// with its size and what it was when
///  listed,
/// and what could not be read.
/// Why:
///  The pruning pass works on the folder the current name holds,
/// because it is the pass that may write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, problems } = await listStamped(commonDir);
/// ```
pub fn list_stamped(common_dir: &Path) -> Listing {
    return list_stamped_below(common_dir, SAVED_DIRECTORY);
}

/// The problem that says why the folder cannot be used.
fn unusable(saved_root: &Path, failure: &OpenFailure) -> Problem {
    let detail: String = match failure {
        OpenFailure::Link => String::from("a symbolic link, or something that is not a directory, stands at it or at its parent `monochromatic-git-policy-cli`, and a removal never follows one"),
        OpenFailure::Refused(error) => format!("it cannot be opened: {error}"),
    };
    return Problem { path: saved_root.to_path_buf(), kind: ProblemKind::StoreUnusable, detail };
}

/// The text of an input or output error,
/// for the places that carry an error as words.
fn io_text<T>(result: std::io::Result<T>) -> Result<T, String> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(error.to_string()),
    };
}

/// Order two listed directories by the time in their names,
/// a tie broken by name.
/// `sort_by` takes this function by path.
fn compare_by_time_then_name(left: &Stamped, right: &Stamped) -> std::cmp::Ordering {
    return left.millis.cmp(&right.millis).then(left.name.cmp(&right.name));
}

/// A problem with the directory at `path`,
/// of the given kind.
fn problem_at(path: &Path, kind: ProblemKind, detail: String) -> Problem {
    return Problem { path: path.to_path_buf(), kind, detail };
}

/// Whether every file now in a directory was in the pass's listing of it.
/// A directory the listing does not know has none.
fn all_were_listed(context: &Removal<'_>, directory_name: &str, now: &[String]) -> bool {
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

/// What:
///  Something told for each chosen directory after the listing and before its removal.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The controls change the folder at the moment a pass is most exposed,
/// and the shipped pass changes nothing;
/// a value stands
///  where a callback stood,
/// and a closure of the right shape is one too,
/// so a control passes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface BeforeRemoval { before(path: string): void; }
/// ```
pub trait BeforeRemoval {
    /// Called with the path of the directory about to be removed.
    fn before(&mut self, candidate: &Path);
}

/// Every function-like value with the right shape is such a step:
/// `before` just calls it.
impl<Step: FnMut(&Path)> BeforeRemoval for Step {
    fn before(&mut self, candidate: &Path) {
        self(candidate);
    }
}

/// The step of the shipped pass,
/// which does nothing.
struct NothingBeforeRemoval;

impl BeforeRemoval for NothingBeforeRemoval {
    fn before(&mut self, _candidate: &Path) {}
}

/// The listing of an opened folder;
/// `saved_root` is the path the problems and the removal records show.
fn list_in(store: &Store, saved_root: &Path) -> Listing {
    let mut listing: Listing = Listing::default();
    let names = match store.names() {
        Ok(found) => found,
        Err(error) => {
            listing.problems.push(Problem { path: saved_root.to_path_buf(), kind: ProblemKind::StoreUnusable, detail: format!("it cannot be read: {error}") });
            return listing;
        }
    };
    for name in names {
        let text: String = name.to_string_lossy().into_owned();
        let Some(millis) = parse_stamp(text.as_str()) else { continue };
        let path: PathBuf = saved_root.join(text.as_str());
        match survey_entry(store, name.as_os_str()) {
            Ok(None) => {}
            Ok(Some((identity, tally))) => {
                listing.files += tally.names.len();
                listing.entries.push(Stamped { name: text.clone(), millis, bytes: tally.bytes, identity });
                listing.listed_files.insert(text, tally.names.into_iter().collect());
            }
            Err(detail) => listing.problems.push(Problem { path, kind: ProblemKind::EntryUnreadable, detail }),
        }
    }
    listing.entries.sort_by(compare_by_time_then_name);
    return listing;
}

/// The identity and contents of the directory `name`,
/// nothing when `name` is not a directory (a link or a file),
/// or the
/// reason it cannot be read.
fn survey_entry(store: &Store, name: &OsStr) -> Result<Option<(Identity, Tally)>, String> {
    let Some(seen) = io_text(store.directory_identity(name))? else {
        return Ok(None);
    };
    let Some(directory) = io_text(store.open_directory(name))? else {
        return Ok(None);
    };
    let opened: Identity = io_text(directory.identity())?;
    if opened != seen {
        return Err(String::from("it was replaced while it was being read"));
    }
    return match directory.survey() {
        Ok(tally) => Ok(Some((opened, tally))),
        // Removed by another pass while it was read: nothing to count.
        Err(Interrupted { error, .. }) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(Interrupted { error, .. }) => Err(error.to_string()),
    };
}

/// What:
///  Removes what `plan` chooses from the folder,
/// one directory at a time,
/// and writes one audit line after each removal.
/// Why:
///   A line per directory,
/// written once it is gone,
/// means a kill between two removals leaves a line for each directory that
///  went and a whole directory for each that did not;
/// a directory that cannot be removed is left,
/// said as a debug line,
/// and
///  has no line.
/// No lock is taken and no commit waits on this;
/// a directory that another pass removes first is that pass's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = await pruneFolder({ commonDir, nowMillis, limits, keep })
/// ```
pub fn prune(common_dir: &Path, now_millis: i64, limits: Limits, keep: Option<&str>) -> PruneOutcome {
    return prune_with(common_dir, now_millis, limits, keep, &mut NothingBeforeRemoval);
}

/// `prune` with a step run for each chosen directory after the listing and before its removal:
/// the controls use it to
/// change the folder at the moment a pass is most exposed;
/// `prune` passes a step that does nothing.
pub fn prune_with(common_dir: &Path, now_millis: i64, limits: Limits, keep: Option<&str>, before_removal: &mut dyn BeforeRemoval) -> PruneOutcome {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    let mut outcome: PruneOutcome = PruneOutcome::default();
    let store: Store = match Store::open(common_dir, SAVED_DIRECTORY) {
        Ok(Some(opened)) => opened,
        Ok(None) => return outcome,
        Err(failure) => {
            outcome.problems.push(unusable(saved_root.as_path(), &failure));
            return outcome;
        }
    };
    let listing: Listing = list_in(&store, saved_root.as_path());
    outcome.problems.extend(listing.problems);
    let doomed: Vec<(usize, Reason)> = plan(listing.entries.as_slice(), now_millis, limits, keep);
    if doomed.is_empty() {
        return outcome;
    }
    let audit_path: PathBuf = common_dir.join(AUDIT_FILE);
    // Every removal is followed by its line, so a file that cannot take one stops the pass before the first removal.
    if let Err(error) = check_appendable(audit_path.as_path()) {
        outcome.problems.push(Problem { path: audit_path, kind: ProblemKind::AuditUnwritable, detail: error.to_string() });
        return outcome;
    }
    let mut folder_bytes: u128 = 0;
    for entry in &listing.entries {
        folder_bytes += entry.bytes;
    }
    for (index, reason) in doomed {
        let entry: &Stamped = &listing.entries[index];
        let path: PathBuf = saved_root.join(entry.name.as_str());
        before_removal.before(path.as_path());
        let mut context: Removal = Removal { store: &store, audit_path: audit_path.as_path(), limits, folder_bytes, now_millis, listed_files: &listing.listed_files };
        remove_entry(&mut context, entry, path, reason, &mut outcome);
    }
    return outcome;
}

/// What a removal needs besides the entry:
/// the folder,
/// where the audit goes,
/// the limits and the numbers its line states.
struct Removal<'pass> {
    /// The folder this pass opened.
    store: &'pass Store,
    /// The audit file.
    audit_path: &'pass Path,
    /// The limits the pass judges by.
    limits: Limits,
    /// The size of the folder before the pass.
    folder_bytes: u128,
    /// The time of the pass.
    now_millis: i64,
    /// The files each directory held when the pass listed it.
    listed_files: &'pass std::collections::HashMap<String, std::collections::HashSet<String>>,
}

/// Writes the audit line of a directory of which `tally` went;
/// said as a problem when the line cannot be written.
fn write_audit(context: &Removal, outcome_word: &str, removed: &Removed, tally: &Tally, outcome: &mut PruneOutcome) {
    let line: String = audit_line(outcome_word, removed, tally.names.as_slice(), context.limits, context.folder_bytes, context.now_millis);
    if let Err(error) = append_line(context.audit_path, line.as_bytes()) {
        debug(TAG, &format!("the audit record of {} could not be written: {error}", removed.path.display()));
        outcome.problems.push(Problem { path: removed.path.clone(), kind: ProblemKind::RemovedWithoutAudit, detail: error.to_string() });
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
fn remove_entry(context: &mut Removal, entry: &Stamped, path: PathBuf, reason: Reason, outcome: &mut PruneOutcome) {
    let name: &OsStr = OsStr::new(entry.name.as_str());
    let directory = match context.store.open_directory(name) {
        Ok(Some(opened)) => opened,
        // Gone: another actor removed it, which is what this pass was going to do.
        Ok(None) => return,
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => {
            outcome.problems.push(problem_at(&path, ProblemKind::EntryChanged, String::from("the name is now a link or a file")));
            return;
        }
        Err(error) => {
            outcome.problems.push(problem_at(&path, ProblemKind::NotRemoved, error.to_string()));
            return;
        }
    };
    match directory.identity() {
        Ok(found) if found == entry.identity => {}
        // The same directory whose change time moved: another pass removing its files does that, and so does a file added after
        //  the listing. Only a directory that gained nothing is the directory that was judged.
        Ok(found) if found.device == entry.identity.device && found.inode == entry.identity.inode => match directory.survey() {
            Ok(now) if all_were_listed(context, entry.name.as_str(), &now.names) => {}
            Ok(_) => {
                outcome.problems.push(problem_at(&path, ProblemKind::EntryChanged, String::from("a file was added to it after the pass listed it")));
                return;
            }
            // Removed by another pass while it was looked at.
            Err(Interrupted { error, .. }) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(Interrupted { error, .. }) => {
                outcome.problems.push(problem_at(&path, ProblemKind::NotRemoved, error.to_string()));
                return;
            }
        },
        Ok(_) => {
            outcome.problems.push(problem_at(&path, ProblemKind::EntryChanged, String::from("the name now stands for another directory")));
            return;
        }
        Err(error) => {
            outcome.problems.push(problem_at(&path, ProblemKind::NotRemoved, error.to_string()));
            return;
        }
    }
    let mut gone: Removed = Removed { path: path.clone(), bytes: 0, reason };
    let tally: Tally = match directory.empty() {
        Ok(emptied) => emptied,
        Err(Interrupted { tally, error }) => {
            if tally.names.is_empty() {
                outcome.problems.push(problem_at(&path, ProblemKind::NotRemoved, error.to_string()));
            } else {
                gone.bytes = tally.bytes;
                write_audit(context, "partial", &gone, &tally, outcome);
                outcome.problems.push(problem_at(&path, ProblemKind::PartiallyRemoved, error.to_string()));
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

/// The name of the directory under the folder that holds a copy,
/// or nothing for a path outside the folder.
pub fn directory_name_of(saved_root: &Path, copy: &Path) -> Option<String> {
    let first = copy.strip_prefix(saved_root).ok()?.components().next()?;
    return Some(first.as_os_str().to_string_lossy().into_owned());
}

/// What the folder of saved copies holds,
/// for `git monochromatic-git-policy-cli doctor`,
/// and the report's own text.
pub use crate::discarded_report::{Report, doctor_section, report};

/// Pruning is controlled in the test module.
#[cfg(test)]
#[path = "discarded_prune_tests.rs"]
mod tests;

/// What a pass may touch and what it must say is controlled in the guard test module.
#[cfg(test)]
#[path = "discarded_prune_guard_tests.rs"]
mod guard_tests;
