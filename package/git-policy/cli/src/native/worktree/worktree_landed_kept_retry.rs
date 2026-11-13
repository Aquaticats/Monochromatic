//! What:
//!  The worktrees one landed sweep kept,
//!  remembered so the next sweep examines them again with nobody asking.
//! Why:
//!  A sweep keeps a landed worktree for a reason that often goes away by itself:
//!  a process finishes,
//!  a person commits or removes the file that stood in the way,
//!  a reference lock is dropped.
//!  A later merge cannot find that worktree again:
//!  a sweep's candidates are the branches this merge brought in,
//!  and the branch landed in the earlier merge,
//!  so without a record the worktree would sit there for ever,
//!  holding a snapshot's worth of space nobody can see.
//!  The record is one file per kept worktree in the repository's common Git directory,
//!  written with the words that kept it,
//!  and it is removed as soon as a later sweep finds the worktree gone,
//!  or decides it may not remove it at all.
//!  A record whose worktree no longer exists is dropped by the same pass:
//!  nothing else would ever look at it.
//!  The worktree is written and read as text,
//!  so a path that is not valid UTF-8 names no record a sweep acts on:
//!  a sweep reads such a worktree as one no registration names and drops the record,
//!  which leaves the worktree where it is rather than removing another directory.
//!  The record is private (`0600`) and written before the sweep's line is printed,
//!  so a command killed in between leaves a statement rather than silence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rememberKept(commonDir, worktree, branch, why); const kept = rememberedKept(commonDir); forgetKept(commonDir, worktree);
//! ```

use crate::diagnostic_log::debug;
use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::push_failure_notice::fnv_hex;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "landedKeptRetry";

/// The folder under the common Git directory that holds one record per landed worktree a sweep kept.
pub const KEPT_RETRY_FOLDER: &str = "monochromatic-git-policy-cli/kept-landed-worktrees";

/// The first line of every record,
/// which tells it from anything else written there.
const MAGIC: &str = "monochromatic-git-policy-cli kept landed worktree 1";

/// The line that names the worktree the record is about.
const WORKTREE_FIELD: &str = "worktree: ";

/// The line that names the branch it had checked out.
const BRANCH_FIELD: &str = "branch: ";

/// The end of a record's own name.
const RECORD_SUFFIX: &str = ".txt";

/// One worktree a sweep kept,
/// as a later sweep reads it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RememberedKept {
    /// The worktree it names.
    pub worktree: PathBuf,
    /// The branch that worktree had checked out,
    /// as a full ref name.
    pub branch: String,
}

/// What:
///  Remember one worktree a sweep kept,
///  and why.
/// Why:
///  The next sweep needs the worktree and its branch to examine it again,
///  and the reason is what a person reads when the folder is opened.
///  A record that cannot be written costs the retry and nothing else:
///  the sweep's own line still says the worktree stayed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberKept(commonDir, worktree, 'refs/heads/task-z', 'process 4242 is working in it')
/// ```
pub fn remember_kept(common_dir: &Path, worktree: &Path, branch: &str, why: &str) {
    if let Err(error) = written(common_dir, worktree, branch, why) {
        debug(TAG, format!("the record of the kept worktree {} could not be written: {error}", worktree.display()).as_str());
    }
}

/// What:
///  Write the record,
///  or the reason it could not be written.
/// Why:
///  The folder and the file are both private,
///  and the file is written afresh so a second sweep of the same landing writes one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// written(commonDir, worktree, branch, why) // throws the reason
/// ```
fn written(common_dir: &Path, worktree: &Path, branch: &str, why: &str) -> Result<(), String> {
    let directory: PathBuf = kept_retry_directory(common_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the folder failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(worktree));
    let text: String = format!("{MAGIC}\n{WORKTREE_FIELD}{}\n{BRANCH_FIELD}{branch}\n\n{why}\n", worktree.display());
    let _replaced = std::fs::remove_file(path.as_path());
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("writing the record failed: {error}")),
    };
}

/// What:
///  The records of the worktrees earlier sweeps kept,
///  in the order their names sort.
/// Why:
///  A pass that walked a directory in the reader's order would examine the same worktrees in another order on
///  another day,
///  and a reader comparing two runs could not tell one worktree from another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberedKept(commonDir) // [{ worktree, branch }]
/// ```
pub fn remembered_kept(common_dir: &Path) -> Vec<RememberedKept> {
    let directory: PathBuf = kept_retry_directory(common_dir);
    let listing = match std::fs::read_dir(directory.as_path()) {
        Ok(listed) => listed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            debug(TAG, format!("the kept worktrees in {} cannot be listed: {error}", directory.display()).as_str());
            return Vec::new();
        }
    };
    let mut named: Vec<PathBuf> = listing.flatten().map(|entry| return entry.path()).collect();
    named.sort();
    let mut found: Vec<RememberedKept> = Vec::new();
    for path in named {
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(RECORD_SUFFIX.trim_start_matches('.')) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path.as_path()) else {
            continue;
        };
        if let Some(one) = parsed(text.as_str()) {
            found.push(one);
        }
    }
    return found;
}

/// What:
///  The parts of one record,
///  or nothing when the file is not one this release wrote.
/// Why:
///  A file that is not a record names no worktree,
///  and a reader that guessed would remove a folder this program never kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parsed('monochromatic-git-policy-cli kept landed worktree 1\nworktree: /x\nbranch: refs/heads/y\n\nwhy\n')
/// ```
fn parsed(text: &str) -> Option<RememberedKept> {
    let mut lines = text.splitn(4, '\n');
    if lines.next()? != MAGIC {
        return None;
    }
    let worktree: &str = lines.next()?.strip_prefix(WORKTREE_FIELD)?;
    let branch: &str = lines.next()?.strip_prefix(BRANCH_FIELD)?;
    if lines.next()?.trim().is_empty() {
        return None;
    }
    return Some(RememberedKept { worktree: PathBuf::from(worktree), branch: branch.to_owned() });
}

/// What:
///  Drop the record of one worktree,
///  once a sweep decided what becomes of it.
/// Why:
///  A worktree that went,
///  one no registration names any more,
///  and one this program cannot prove it made are each answered for good:
///  a record kept for them would have every later sweep look at them again for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetKept(commonDir, worktree)
/// ```
pub fn forget_kept(common_dir: &Path, worktree: &Path) {
    let path: PathBuf = kept_retry_directory(common_dir).join(record_name(worktree));
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => debug(TAG, format!("the record of the kept worktree {} could not be removed: {error}", worktree.display()).as_str()),
    }
}

/// The folder the records live in,
/// through the reader of names an earlier release spelled differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptRetryDirectory(commonDir) // `${commonDir}/monochromatic-git-policy-cli/kept-landed-worktrees`
/// ```
fn kept_retry_directory(common_dir: &Path) -> PathBuf {
    return path_for_reading(common_dir, KEPT_RETRY_FOLDER);
}

/// The record's own name,
/// which is the path's hash so two worktrees never share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordName('/x/y') // '0123abcd.txt'
/// ```
fn record_name(worktree: &Path) -> String {
    return format!("{}{RECORD_SUFFIX}", fnv_hex(worktree.to_string_lossy().as_ref()));
}

/// Controls of the kept worktrees a later sweep examines stay out of the release executable.
#[cfg(test)]
#[path = "worktree_landed_kept_retry_tests.rs"]
mod tests;
