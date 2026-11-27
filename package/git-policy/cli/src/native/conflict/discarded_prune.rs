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
//! The listing is `discarded_prune_listing.rs`
//! and one directory's removal and audit line is `discarded_prune_removal.rs`,
//! so this file is the pass itself.
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

/// The audit record's own path.
use crate::discard_audit::AUDIT_FILE;
/// The listing of the folder,
/// and why the folder cannot be used.
use crate::discarded_prune_listing::{
    list_in,
    unusable,
};
/// One chosen directory removed,
/// and the pass that removes it.
use crate::discarded_prune_removal::{
    Removal,
    remove_entry,
};
/// The folder of saved copies,
/// opened once without following a link.
use crate::discarded_store::Store;
/// Whether the audit record can take the line a removal is about to write.
use crate::log_lines_appended_without_a_lock::check_appendable;
/// The folder of saved copies inside a Git directory.
use crate::where_discarded_copies_live::SAVED_DIRECTORY;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

pub use crate::discarded_prune_audit_line::{
    Removed,
    audit_line,
    prune_line,
};
pub use crate::discarded_prune_limits::{
    DEFAULT_MAX_AGE_DAYS,
    DEFAULT_MAX_BYTES,
    Limits,
    parse_size,
};
/// The folder's listing,
/// which `discarded_report.rs` reads for the doctor's section.
pub(crate) use crate::discarded_prune_listing::list_stamped_below;
pub use crate::discarded_prune_plan::{
    Reason,
    plan,
};
pub use crate::discarded_prune_problems::{
    Listing,
    Problem,
    ProblemKind,
};
/// What a pass did,
/// which its callers and controls read.
pub use crate::discarded_prune_removal::PruneOutcome;
pub use crate::discarded_prune_stamps::{
    STAMP_RESOLUTION_MILLISECONDS,
    Stamped,
    YOUNG_FLOOR_MILLISECONDS,
    parse_stamp,
};
pub use crate::discarded_store::Identity;

/// What:
///  Every directory of the folder that is named like the wrapper's own,
/// oldest first,
/// with its size and what it was when
///  listed,
/// and what could not be read.
/// Why:
///  The pruning pass works on the folder the current name holds,
/// because it is the pass that may write,
/// so the folder's own name is read here beside the pass that uses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, problems } = await listStamped(commonDir);
/// ```
pub fn list_stamped(common_dir: &Path) -> Listing {
    return list_stamped_below(common_dir, SAVED_DIRECTORY);
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
    fn before(
        &mut self,
        candidate: &Path,
    );
}

/// Every function-like value with the right shape is such a step:
/// `before` just calls it.
impl<Step: FnMut(&Path)> BeforeRemoval for Step {
    fn before(
        &mut self,
        candidate: &Path,
    ) {
        self(candidate);
    }
}

/// The step of the shipped pass,
/// which does nothing.
struct NothingBeforeRemoval;

impl BeforeRemoval for NothingBeforeRemoval {
    fn before(
        &mut self,
        _candidate: &Path,
    ) {
    }
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
pub fn prune(
    common_dir: &Path,
    now_millis: i64,
    limits: Limits,
    keep: Option<&str>,
) -> PruneOutcome {
    return prune_with(
        common_dir,
        now_millis,
        limits,
        keep,
        &mut NothingBeforeRemoval,
    );
}

/// `prune` with a step run for each chosen directory after the listing and before its removal:
/// the controls use it to
/// change the folder at the moment a pass is most exposed;
/// `prune` passes a step that does nothing.
pub fn prune_with(
    common_dir: &Path,
    now_millis: i64,
    limits: Limits,
    keep: Option<&str>,
    before_removal: &mut dyn BeforeRemoval,
) -> PruneOutcome {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    let mut outcome: PruneOutcome = PruneOutcome::default();
    let store: Store = match Store::open(common_dir, SAVED_DIRECTORY) {
        Ok(Some(opened)) => opened,
        Ok(None) => return outcome,
        Err(failure) => {
            outcome
                .problems
                .push(unusable(saved_root.as_path(), &failure));
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
        outcome.problems.push(Problem {
            path: audit_path,
            kind: ProblemKind::AuditUnwritable,
            detail: error.to_string(),
        });
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
        let mut context: Removal = Removal {
            store: &store,
            audit_path: audit_path.as_path(),
            limits,
            folder_bytes,
            now_millis,
            listed_files: &listing.listed_files,
        };
        remove_entry(&mut context, entry, path, reason, &mut outcome);
    }
    return outcome;
}

/// The name of the directory under the folder that holds a copy,
/// or nothing for a path outside the folder.
pub fn directory_name_of(
    saved_root: &Path,
    copy: &Path,
) -> Option<String> {
    let first = copy.strip_prefix(saved_root).ok()?.components().next()?;
    return Some(first.as_os_str().to_string_lossy().into_owned());
}

/// What the folder of saved copies holds,
/// for `git monochromatic-git-policy-cli doctor`,
/// and the report's own text.
pub use crate::discarded_report::{
    Report,
    doctor_section,
    report,
};

/// Pruning is controlled in the test module.
#[cfg(test)]
#[path = "discarded_prune_tests.rs"]
mod tests;

/// What a pass may touch and what it must say is controlled in the guard test module.
#[cfg(test)]
#[path = "discarded_prune_guard_tests.rs"]
mod guard_tests;
