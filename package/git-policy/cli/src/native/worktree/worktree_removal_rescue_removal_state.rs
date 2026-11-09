//! What:
//!  Whether the worktree a removal snapshot was taken for is gone,
//! refused,
//! or still on its way,
//! judged from the Git directory,
//! the processes that took the snapshot and the checkout.
//! Why:
//!  `settle_removals` asks this once per snapshot and acts on the answer alone,
//! so the three answers and the evidence they rest on change together.
//! A wrong answer either loses the records the snapshot keeps or holds them
//!  for a removal that will never come.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removalOf(gitDir, snapshot) // 'done' | 'refused' | 'undecided'
//! ```

use super::{ATTEMPTS, GITDIR_HEX_PREFIX, TAG, files_of, name_of, path_bytes};
use crate::diagnostic_log::warn;
use crate::process_identity::process_birth_identity;
use std::path::{Path, PathBuf};
/// What a Git directory,
/// the process that took a snapshot,
/// and the checkout say about a removal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Removal {
    /// The Git directory is gone:
    /// Git removed the worktree.
    Done,
    /// Every command that took the snapshot has ended and the checkout is intact:
    /// Git refused,
    /// or it was a dry run.
    Refused,
    /// A command that took the snapshot may still be removing,
    /// the checkout is missing from a registered worktree,
    /// or the
    ///  evidence cannot be read:
    /// not decided yet.
    Undecided,
}

/// What:
///  Whether a command that took the snapshot may still be running,
/// which a process that cannot be told apart from it counts as.
/// Why:
///   A process number alone can be reused,
/// so the birth identity recorded with it decides.
/// A record that cannot be read,
/// or a
///  process that cannot be inspected,
/// is an owner that may be alive:
/// discarding the snapshot on a guess would lose the records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await attemptsMayBeRunning(snapshot)
/// ```
fn attempt_may_be_running(snapshot: &Path) -> bool {
    // The loop stops at the first attempt that may be running, as `any` did.
    for path in files_of(snapshot.join(ATTEMPTS).as_path()) {
        if attempt_file_may_be_running(path.as_path()) {
            return true;
        }
    }
    return false;
}

/// Whether the command that wrote one attempt file may still be running;
/// a file that cannot be understood counts as running.
fn attempt_file_may_be_running(path: &Path) -> bool {
    let Ok(pid) = name_of(path).parse::<i64>() else { return false };
    let Ok(recorded) = std::fs::read_to_string(path) else { return true };
    if recorded.is_empty() {
        // Being written.
        return true;
    }
    return match process_birth_identity(pid) {
        Ok(Some(now)) => now == recorded,
        Ok(None) => false,
        Err(error) => {
            warn(TAG, format!("the process {pid} of a worktree removal cannot be inspected: {error}").as_str());
            true
        }
    };
}

/// Whether a byte ends a line of the file Git writes.
fn is_line_end(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether the checkout a registered worktree's Git directory names is still there;
/// nothing when the Git directory does not say.
pub(super) fn checkout_present(git_dir: &Path) -> Option<bool> {
    // Git writes this file itself, as the bytes of the checkout's `.git`, so it is read as bytes and never as text.
    let bytes: Vec<u8> = std::fs::read(git_dir.join("gitdir")).ok()?;
    let end: usize = bytes.iter().position(is_line_end).unwrap_or(bytes.len());
    let marker: PathBuf = PathBuf::from(crate::path_bytes::os_from_path_bytes(&bytes[..end]));
    if marker.as_os_str().is_empty() {
        return None;
    }
    return Some(git_dir.join(marker).exists());
}

/// What:
///  The Git directory a snapshot's `gitdir` file names:
/// its bytes when the file is one this release wrote,
/// its plain text when it is an earlier release's.
/// Why:
///   The removal of a worktree is judged by looking at this directory,
/// and a spelling that lost the path's bytes names nothing,
/// which reads as a removal that finished and would promote a live worktree's records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedGitDir = (text) => text.trim().startsWith('gitdirHex=') ? pathOfHex(text.trim().slice(10)) : text.trim() || undefined;
/// ```
pub(super) fn named_git_dir(text: &str) -> Option<PathBuf> {
    let trimmed: &str = text.trim_end_matches('\n');
    return match trimmed.strip_prefix(GITDIR_HEX_PREFIX) {
        Some(hexadecimal) => path_bytes::path_of_hex(hexadecimal),
        None if trimmed.is_empty() => None,
        None => Some(PathBuf::from(trimmed)),
    };
}

/// What:
///  What a snapshot's Git directory,
/// its taker and its checkout show of the removal it was taken for.
/// Why:
///   Git deletes the checkout first and the Git directory last,
/// and refuses before it deletes anything.
/// So a missing Git
///  directory is a removal;
/// an intact checkout whose taker has ended is a refusal;
/// anything else may be a removal in progress
///  or one that was cut short,
/// and deciding on it would either lose the records or keep them for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removalOf(gitDir, snapshot) // 'done' | 'refused' | 'undecided'
/// ```
pub(super) fn removal_of(git_dir: &Path, snapshot: &Path) -> Removal {
    match std::fs::symlink_metadata(git_dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Removal::Done,
        Err(_) => return Removal::Undecided,
        Ok(_) => {}
    }
    if attempt_may_be_running(snapshot) {
        return Removal::Undecided;
    }
    return if checkout_present(git_dir) == Some(true) { Removal::Refused } else { Removal::Undecided };
}
