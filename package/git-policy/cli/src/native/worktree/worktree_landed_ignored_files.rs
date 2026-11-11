//! What:
//!  The ignored entries of a worktree whose bytes nothing else in the repository holds.
//! Why:
//!   The sweep removes a worktree by itself,
//!  and every route it has deletes what Git ignores:
//!  the btrfs destroy takes the whole directory,
//!  and Git's own removal deletes the files Git ignores without a word.
//!  The owner's rule is that no command of this program discards a person's data,
//!  so an ignored file the person wrote there after the add must keep the worktree.
//!  Nothing in the worktree says who wrote a file,
//!  and the ignored state the add carried is the source's own,
//!  so the one proof available is the source:
//!  a file the worktree the command runs in holds with the same kind,
//!  mode and bytes is state the repository already has,
//!  and a file it does not hold,
//!  or holds differently,
//!  may be the person's.
//!  Ignored directories are not read:
//!  a build writes them into the worktree alone,
//!  which is the state this sweep exists to give back,
//!  and reading every file below them would cost the whole tree on every merge.
//!  A directory the person made under an ignored name goes with the worktree,
//!  and the removal line says that directories go.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const alone = ignoredEntriesAloneHere(git, worktree, admin, source); // { names, more }
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_copy_compare::entry_matches;
use crate::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry, retained_mode};
use crate::worktree_copy_repository_path::filesystem_path;
use crate::worktree_copy_vanished_entry::lstat_or_absent;
use crate::worktree_ladder_target::checkout_environment;
use crate::worktree_snapshot_source::plain;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The query that names every ignored entry Git sees in one worktree,
///  with each whole ignored directory as one name.
/// Why:
///   The restore of a snapshot asks Git the same question with these words
///  (`worktree_dirty_restore.rs`),
/// so the ignore rules that decide what the sweep may not take are Git's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IGNORED_WORDS = ['ls-files', '--others', '--ignored', '--exclude-standard', '--directory', '--no-empty-directory', '-z'];
/// ```
const IGNORED_WORDS: [&str; 7] = ["ls-files", "--others", "--ignored", "--exclude-standard", "--directory", "--no-empty-directory", "-z"];

/// How many names the sentence of a kept worktree shows before it counts the rest.
const SHOWN_NAMES: usize = 3;

/// The ignored entries of one worktree that the source does not hold,
/// or holds with other bytes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct AloneHere {
    /// Their names,
    /// as Git listed them,
    /// at most `SHOWN_NAMES` of them.
    pub(crate) shown: Vec<String>,
    /// How many of them there were,
    /// the shown ones included.
    pub(crate) count: usize,
}

impl AloneHere {
    /// Whether the worktree holds any,
    /// so the sweep must keep it.
    pub(crate) fn is_empty(&self) -> bool {
        return self.count == 0;
    }

    /// What:
    ///  The names for a person reading why the worktree stays.
    /// Why:
    ///   A person decides about their own files,
    ///  so the files are named,
    ///  and a list longer than a few is counted rather than printed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// alone.list() // '.env and 2 more'
    /// ```
    pub(crate) fn list(&self) -> String {
        let mut parts: Vec<String> = self.shown.clone();
        let rest: usize = self.count - self.shown.len();
        return match rest {
            0 => parts.join(", "),
            1 => {
                parts.push(String::from("1 more"));
                parts.join(", ")
            }
            _ => {
                parts.push(format!("{rest} more"));
                parts.join(", ")
            }
        };
    }
}

/// What:
///  The ignored entries of `worktree` that `source` does not hold with the same kind,
///  mode and bytes,
/// named as far as a sentence needs them.
/// Why:
///   A file the source holds identically came from the add or from the repository,
///  so removing this worktree takes a copy and nothing else;
///  anything else may be the person's and keeps the worktree.
///  The question is asked of Git,
///  so an entry Git does not ignore is never judged here:
///  a tracked modification or an untracked file already makes Git refuse the removal
///  in Git's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ignoredEntriesAloneHere(git, worktree, admin, source) // throws the reason
/// ```
///
/// # Errors
///
/// The words for a query or a comparison this cannot make,
/// which leaves the worktree kept rather than read as empty.
pub(crate) fn ignored_entries_alone_here(git: &GitContext, worktree: &Path, admin: &Path, source: &Path) -> Result<AloneHere, String> {
    let request: GitRequest = GitRequest {
        cwd: worktree.to_path_buf(),
        arguments: IGNORED_WORDS.iter().map(OsString::from).collect(),
        without_prefix: true,
        environment: checkout_environment(admin, worktree),
        ..GitRequest::default()
    };
    let output: GitOutput = match run_git(git, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("the ignored files in {} cannot be asked of Git ({error})", worktree.display())),
    };
    if output.code != Some(0) {
        let said: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned();
        return Err(format!("Git cannot list the ignored files in {} ({said})", worktree.display()));
    }
    let mut alone: AloneHere = AloneHere::default();
    for name in output.stdout.split(|byte: &u8| *byte == 0).filter(|name: &&[u8]| !name.is_empty()) {
        match alone_in_the_worktree(worktree, source, name) {
            Ok(true) => {
                alone.count += 1;
                if alone.shown.len() < SHOWN_NAMES {
                    alone.shown.push(plain(String::from_utf8_lossy(name).as_ref()));
                }
            }
            Ok(false) => {}
            Err(reason) => return Err(reason),
        }
    }
    return Ok(alone);
}

/// What:
///  Whether one listed ignored entry is state `source` does not hold identically.
/// Why:
///   Directories are the build state the sweep gives back and are never read;
///  a file or a link is compared with the source by kind,
///  mode and bytes,
///  and an entry that went away while it was read has nothing left to lose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aloneInTheWorktree(worktree, source, name) // boolean, or the reason
/// ```
fn alone_in_the_worktree(worktree: &Path, source: &Path, name: &[u8]) -> Result<bool, String> {
    // The listing names a whole ignored directory with the separator that ends it, which names no entry of its own.
    let entry_name: &[u8] = name.strip_suffix(b"/").unwrap_or(name);
    let inside: PathBuf = filesystem_path(worktree, entry_name).map_err(|error| format!("{} names no path this program can read ({})", String::from_utf8_lossy(entry_name), error.0))?;
    let here = match lstat_or_absent(inside.as_path()) {
        Ok(Some(metadata)) => metadata,
        Ok(None) => return Ok(false),
        Err(error) => return Err(format!("the ignored entry {} cannot be read ({})", String::from_utf8_lossy(entry_name), error.0)),
    };
    if here.is_dir() {
        return Ok(false);
    }
    let kind: EntryKind = if here.file_type().is_symlink() {
        EntryKind::Symlink
    } else if here.is_file() {
        EntryKind::File
    } else {
        // A socket, a FIFO or a device node holds no bytes a removal can lose.
        return Ok(false);
    };
    let entry: WorktreeCopyEntry = WorktreeCopyEntry { kind, mode: retained_mode(&here), relative_path: entry_name.to_vec() };
    return match entry_matches(worktree, source, &entry) {
        Ok(same) => Ok(!same),
        Err(error) => Err(format!("the ignored entry {} cannot be compared with the worktree this command runs in ({})", String::from_utf8_lossy(entry_name), error.0)),
    };
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_landed_ignored_files_tests.rs"]
mod tests;
