//! What:
//!  The two folders every question about an instance's units is asked of,
//!  which worktree a unit instance belongs to,
//!  and whether that worktree is certainly gone.
//! Why:
//!  A unit instance is named by a hash of its worktree's own Git directory,
//!  and a hash names nothing once the directory is gone,
//! so the mapping is read back from this program's own records:
//! the note beside the socket,
//!  the `gitdir=` lines of the service journal,
//! and the notes of its claim watch,
//!  each only when the text hashes to the instance.
//! The proof that a worktree is gone stands beside the mapping because a survey,
//!  a service and a sweep must never disagree about it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const worktree = worktreeOf({ runtimeDir, stateDir }, instance); // { known: gitDir } or 'unmapped'
//! const gone = worktreeIsGone(gitDir);
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::path_bytes_in_records::{
    GITDIR_HEX_PREFIX,
    path_of_hex,
};
use crate::service_claim_watch::{
    WATCH_DIRECTORY,
    named_repositories,
};
use crate::service_files_of_a_stopped_instance::note_path;
use crate::service_instance::instance_of;
use crate::service_journal_folder_of_an_instance::journal_of;
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{
    Path,
    PathBuf,
};

/// Where the units of the user's instances are told apart from one another.
#[derive(Clone, Copy, Debug)]
pub struct UnitFolders<'a> {
    /// The user's runtime directory,
    /// which holds the sockets and the notes.
    pub runtime_dir: &'a Path,
    /// The user's state directory,
    /// whose `monochromatic-git-policy-cli/service/<instance>` folders hold the service journals.
    pub state_dir: Option<&'a Path>,
}

/// What a unit instance's worktree is,
/// as far as the notes and the journals say.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnitWorktree {
    /// The worktree's Git directory,
    /// whose text hashes to the instance.
    Known(PathBuf),
    /// Nothing maps the instance to a worktree.
    Unmapped,
}

/// What:
/// Whether the worktree whose own Git directory is `git_dir` is certainly gone.
/// Why:
///   A Git directory that is missing while the place that holds it is there is the worktree Git removed:
/// a linked worktree's directory is `<common>/worktrees/<id>`,
/// and Git removes `worktrees` itself with the last one,
/// so the common directory is the place that must be there;
/// a main checkout's Git directory is held by the checkout.
/// A directory that is on a drive which is not mounted has a missing Git directory too,
/// and its worktree is not gone,
/// so a place that is missing as well is read by the same proof the pushes waiting for a new try are read by
/// (`unreachable_git_dir_proven_gone.rs`):
/// the nearest directory above the Git directory that exists,
/// which proves the repository deleted when it is the root of a mounted filesystem or holds other entries,
/// and proves nothing when it holds nothing at all.
/// Both questions are asked here,
/// so the service,
/// the survey,
/// the sweep and `doctor` never disagree about a gone worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeIsGone('/repo/.git/worktrees/topic') // true when /repo/.git is a directory and topic is not there
/// ```
pub fn worktree_is_gone(git_dir: &Path) -> bool {
    match std::fs::symlink_metadata(git_dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => return false,
    }
    let Some(parent) = git_dir.parent() else {
        return false;
    };
    let anchor: Option<&Path> = if parent.file_name() == Some(std::ffi::OsStr::new("worktrees")) {
        parent.parent()
    } else {
        Some(parent)
    };
    if let Some(place) = anchor
        && place.is_dir()
    {
        return true;
    }
    // The place that held the Git directory is missing too: the whole repository may have been deleted, or it may be on a
    //  drive that is not mounted. The proof the retry plans use is what tells the two apart, and it reads no worktree of its
    //  own: a worktree that is still there is one whose Git directory moved, which the caller of the proof names where it knows it.
    return matches!(
        crate::unreachable_git_dir_proven_gone::prove(
            git_dir,
            None,
            &crate::unreachable_git_dir_proven_gone::the_mounts()
        ),
        crate::unreachable_git_dir_proven_gone::Proof::Gone(_)
    );
}

/// What:
///  The Git directory a note or a record line names for an instance,
/// when the path it names hashes to that instance.
/// Why:
///   A note of this release holds the bytes of the directory and one line break after them;
/// a note of an earlier release holds its text and nothing after it,
/// so the last byte of that text is the path's even when it is a line break.
/// A note whose bytes do not decode names nothing,
/// and its units stay where they are rather than being stopped on a guess.
fn named_for(
    instance: &str,
    text: &str,
) -> Option<PathBuf> {
    let (bytes, plain): (Option<&str>, Option<&str>) = match text.strip_prefix(GITDIR_HEX_PREFIX) {
        Some(hexadecimal) => (
            Some(hexadecimal.strip_suffix('\n').unwrap_or(hexadecimal)),
            None,
        ),
        None if text.is_empty() => (None, None),
        None => (None, Some(text)),
    };
    let path: PathBuf = path_bytes::path_of_the_two_fields(bytes, plain)
        .ok()
        .flatten()?;
    return (instance_of(path.as_path()) == instance).then_some(path);
}

/// The records of a service journal,
/// sorted by name.
fn journal_records(journal: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(journal) else {
        return Vec::new();
    };
    let mut records: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.extension() == Some(std::ffi::OsStr::new("done")) {
            records.push(path);
        }
    }
    records.sort();
    return records;
}

/// The `gitdir=` text of the first record of the instance's service journal that has one,
/// and names a directory that hashes to the instance.
fn from_journal(
    state_dir: &Path,
    instance: &str,
) -> Option<PathBuf> {
    let journal: PathBuf = state_dir
        .join(SERVICE_UNIT.current)
        .join("service")
        .join(instance);
    for record in journal_records(journal.as_path()) {
        // A journal record that is a named pipe answers as unreadable at once, where a plain read would wait for a writer.
        let Ok(text) = crate::plain_file::read_plain_text(&record) else {
            continue;
        };
        // A line of this release names the bytes of the directory, which is the only spelling that keeps a path that is not
        //  text; a line of an earlier release names its plain text.
        let mut plain: Option<PathBuf> = None;
        for line in text.lines() {
            if let Some(hexadecimal) = line.strip_prefix(GITDIR_HEX_PREFIX) {
                if let Some(found) = path_of_hex(hexadecimal)
                    && instance_of(found.as_path()) == instance
                {
                    return Some(found);
                }
                continue;
            }
            if plain.is_none()
                && let Some(named) = line.strip_prefix("gitdir=")
            {
                plain = named_for(instance, named);
            }
        }
        if plain.is_some() {
            return plain;
        }
    }
    return None;
}

/// What:
/// The worktree of an instance:
/// its note,
/// else the Git directory in a record of its service journal,
/// else a Git directory the notes of its claim watch name,
/// each only when its text hashes to the instance.
/// Why:
///   A text that does not hash to the instance names some other directory,
/// and stopping the units of a worktree on a wrong guess would cut a live one.
/// The claim watch's notes are the third record because a sweep that stopped an instance it could not finish took
/// the note beside the socket with it and kept the journal:
/// a journal kept for the claim watch of a removed worktree then names that worktree in the only file left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeOf(folders, instance) // { known: gitDir } or unmapped
/// ```
pub fn worktree_of(
    folders: &UnitFolders<'_>,
    instance: &str,
) -> UnitWorktree {
    // The note is this program's own record of whose units these are, so a note that is a named pipe names nobody and answers
    // at once instead of holding the command that asks.
    if let Ok(text) = crate::plain_file::read_plain_text(&note_path(folders.runtime_dir, instance))
        && let Some(found) = named_for(instance, text.as_str())
    {
        return UnitWorktree::Known(found);
    }
    if let Some(state) = folders.state_dir {
        if let Some(found) = from_journal(state, instance) {
            return UnitWorktree::Known(found);
        }
        let journal: PathBuf = journal_of(state.join(SERVICE_UNIT.current).as_path(), instance);
        for git_dir in named_repositories(journal.join(WATCH_DIRECTORY).as_path()) {
            if instance_of(git_dir.as_path()) == instance {
                return UnitWorktree::Known(git_dir);
            }
        }
    }
    return UnitWorktree::Unmapped;
}
