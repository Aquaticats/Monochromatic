//! What:
//!  Keeping every worktree entry a discarding form would overwrite,
//!  whole,
//!  before Git runs.
//! Why:
//!  A forced tree write replaces an untracked file,
//!  turns a file into a directory,
//!  and deletes the work inside a nested repository at a path it writes,
//!  and none of that is uncommitted work of the outer repository,
//!  so the status the guard reads never names it.
//!  The owner's answer for uncommitted work is "save,
//!  then allow" (`conflict_side_guard.rs`),
//!  and a refusal here would only send the person to move the file by hand:
//!  the file is copied first, the copy is named,
//!  and the command runs as typed.
//!  A file or a symbolic link is copied, because the command may still refuse it and the worktree
//!  must stay as it was when it does;
//!  a directory is moved whole,
//!  because nothing copies a nested repository's history cheaply and a rename keeps every byte,
//!  and only the forms that write over a path whatever the worktree holds are asked about one.
//!  A copy that cannot be made stops the command with the reason,
//!  which is the one refusal this save has.
//!  The saved entries live where the package's other copies live
//!  (`monochromatic-git-policy-cli/discarded`),
//!  under the age and size limits those copies already have.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const saved = keepWhatTheFormWouldOverwrite({ facts, command, location, region, limits });
//! // saved === { kind: 'saved', copies, notice } | { kind: 'nothing' } | { kind: 'refused', message, copies }
//! ```

/// The builder of the written paths,
///  which decides whether this form overwrites anything at all.
use crate::command_written_paths::{WrittenPaths, written_paths};
/// The bound on the folder of saved copies applied after a save.
use crate::conflict_side_guard::prune_after_save;
/// The files a save copies,
///  the place they go,
///  and the move of a directory the command would delete.
use crate::conflict_sides::{SAVED_DIRECTORY, SavedCopy, save_copies};
use crate::discarded_nested_repository::set_nested_repositories_aside;
use crate::discarded_prune::Limits;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// The guarded form being let through.
use crate::rule_linked_worktree::GuardedCommand;
/// The Git directories of an identity,
///  for naming the place saved copies live and the worktree the paths are below.
use crate::worktree_identity::{common_directory, worktree_root};
use crate::repository_location::RepositoryLocation;
/// The sentence the person reads before Git runs,
///  and the rejection when a copy could not be made.
use crate::rule_linked_worktree_saved_overwrite_messages::{
    NotSaved, overwritten_saved_notice, overwritten_not_saved_message,
};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  What keeping the entries of one form left to do.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   Nothing in the way lets the command run exactly as it did before,
///  a save lets it run with one notice per entry and an audit line naming the places,
///  and a save that failed stops it with the worktree as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Overwritten = { kind: 'nothing' } | { kind: 'saved'; copies; notice } | { kind: 'refused'; message; copies };
/// ```
pub(crate) enum Overwritten {
    /// Nothing stands where the form writes.
    Nothing,
    /// Each entry is below the saved copies,
    ///  and the notice names where.
    Saved {
        /// Where each entry went,
        /// for the audit line.
        copies: Vec<SavedCopy>,
        /// What the person reads before Git runs.
        notice: String,
    },
    /// An entry could not be kept,
    ///  the sentence says why and what to do,
    ///  and `copies` names every entry this save already kept.
    Refused {
        /// What the person reads instead of the command running.
        message: String,
        /// The entries the stopped save had already kept.
        copies: Vec<SavedCopy>,
    },
}

/// What:
///  Everything one save is asked with.
///  A `struct` is a record with named fields.
/// Why:
///   The written paths, the form, where it runs and the limits of the folder all decide what to save,
///  and one record keeps the caller from passing them in different orders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SaveRequest<'a> = { facts; command; location; region; limits; milliseconds };
/// ```
pub(crate) struct SaveRequest<'a> {
    /// The facts every question is asked.
    pub(crate) facts: &'a mut dyn RepositoryFacts,
    /// The guarded form being let through.
    pub(crate) command: GuardedCommand,
    /// Where the command runs.
    pub(crate) location: &'a RepositoryLocation,
    /// The tokens after the subcommand.
    pub(crate) region: &'a [OsString],
    /// The age and size limits of the folder of saved copies.
    pub(crate) limits: Limits,
    /// The clock reading the saved directory is named with.
    pub(crate) milliseconds: i64,
}

/// What:
///  Keep every worktree entry the form would overwrite,
///  then answer what the caller must say and record.
/// Why:
///  The question is asked only for a form that writes a tree or a path of one,
///  because a clean's own answer is Git's dry run
///  and every other form reaches only paths its index already holds.
///  A form whose worktree holds nothing at those paths costs one Git query per listing and no copy,
///  so a clean worktree that merely holds untracked files elsewhere runs as it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepWhatTheFormWouldOverwrite(request): Overwritten;
/// ```
pub(crate) fn keep_what_the_form_would_overwrite(request: SaveRequest<'_>) -> Overwritten {
    // A clean deletes what its own dry run names, and the nested repositories it deletes are saved by that reading.
    if matches!(request.command, GuardedCommand::Clean | GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly) {
        return Overwritten::Nothing;
    }
    let Some(written) = written_paths(request.command, request.region) else {
        return Overwritten::Nothing;
    };
    let paths: Vec<String> = match request.facts.paths_in_the_way(&written) {
        Ok(found) => found,
        Err(message) => return Overwritten::Refused { message, copies: Vec::<SavedCopy>::new() },
    };
    return save_entries(&request, written, paths.as_slice());
}

/// What:
///  Copy each file and move each directory of `paths` below the saved copies,
///  and answer what happened.
/// Why:
///  The files are copied before anything is moved,
///  so a copy that fails leaves every directory where the command found it;
///  a move that fails puts back what it moved (`set_nested_repositories_aside`),
///  and the copies already made stay, because their places are named in the rejection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function saveEntries(request, written, paths: string[]): Overwritten;
/// ```
fn save_entries(request: &SaveRequest<'_>, written: WrittenPaths, paths: &[String]) -> Overwritten {
    if paths.is_empty() {
        return Overwritten::Nothing;
    }
    let Some(root) = worktree_root(&request.location.identity) else {
        return Overwritten::Nothing;
    };
    let Some(common_dir) = common_directory(&request.location.identity) else {
        return Overwritten::Nothing;
    };
    let mut files: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    let mut directories: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in paths {
        let bytes: Vec<u8> = path.as_bytes().to_vec();
        if is_a_directory(root, path) {
            directories.push(bytes);
        } else {
            files.push(bytes);
        }
    }
    let mut copies: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    if !files.is_empty() {
        match save_copies(root, common_dir, files.as_slice(), request.milliseconds) {
            Ok(made) => copies.extend(made),
            Err(failure) => {
                return Overwritten::Refused {
                    message: overwritten_not_saved_message(NotSaved {
                        failure: &failure,
                        saved_root: &common_dir.join(SAVED_DIRECTORY),
                        requested: paths,
                    }),
                    copies,
                };
            }
        }
    }
    if !directories.is_empty() {
        match set_nested_repositories_aside(common_dir, root, directories.as_slice(), request.milliseconds) {
            Ok(made) => copies.extend(made),
            Err(stopped) => {
                copies.extend(stopped.still_moved.iter().cloned());
                return Overwritten::Refused {
                    message: overwritten_not_saved_message(NotSaved {
                        failure: &stopped.failure,
                        saved_root: &common_dir.join(SAVED_DIRECTORY),
                        requested: paths,
                    }),
                    copies,
                };
            }
        }
    }
    let unpruned: String = prune_after_save(common_dir, copies.as_slice(), request.milliseconds, request.limits);
    let revision: Option<&std::ffi::OsStr> = match &written {
        WrittenPaths::Tree { revision } | WrittenPaths::MatchingTree { revision, .. } => revision.as_deref(),
        WrittenPaths::Index { .. } => None,
    };
    let shown: String = match revision {
        Some(named) => named.to_string_lossy().into_owned(),
        None => String::from("the content this command restores"),
    };
    return Overwritten::Saved {
        notice: format!("{}{unpruned}", overwritten_saved_notice(copies.as_slice(), shown.as_str())),
        copies,
    };
}

/// What:
///  Whether the worktree holds a directory at one path,
///  without following a symbolic link.
/// Why:
///   A link is copied as a link and a directory is moved whole,
///  and a link that points at a directory is still a link:
///  following it would move the directory it names,
///  which the command never touches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectory = (root, path) => lstatSync(join(root, path)).isDirectory();
/// ```
fn is_a_directory(root: &Path, path: &str) -> bool {
    let full: PathBuf = root.join(path);
    return match std::fs::symlink_metadata(full.as_path()) {
        Ok(metadata) => metadata.file_type().is_dir(),
        Err(_) => false,
    };
}

/// The entries a form overwrites, the places they go, and the one refusal this save has.
#[cfg(test)]
#[path = "policy_linked_worktree_saved_overwrite_tests.rs"]
mod tests;
