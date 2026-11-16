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
use crate::pid_namespace::{UnreadableOwnerStanding, UnreadablePart, unreadable_owner_standing};
use crate::process_identity::{birth_identity_value, process_birth_identity};
use std::path::{Path, PathBuf};
/// What a Git directory,
/// the process that took a snapshot,
/// and the checkout say about a removal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Removal {
    /// The Git directory is gone,
    /// or it is still registered and its checkout is gone:
    /// the worktree is removed,
    /// or never will be finished.
    Done,
    /// Every command that took the snapshot has ended and the checkout is intact:
    /// Git refused,
    /// or it was a dry run.
    Refused,
    /// A command that took the snapshot may still be removing,
    /// or the evidence cannot be read:
    /// not decided yet.
    Undecided,
    /// A taker's record names a birth of no value this command can read and has stood past the patience:
    /// neither a command that runs nor one that ended,
    /// with the words a person reads.
    CannotBeTold(String),
}

/// What:
///  What the attempts of a snapshot say about the commands that took it.
/// Why:
///   A snapshot is kept while one taker may still be removing and discarded once every taker ended,
///  and a taker whose record names a birth of no value is neither:
///  the third answer carries the words a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Attempts = 'ended' | 'runs' | { cannotBeTold: string };
/// ```
pub(super) enum Attempts {
    /// Every taker ended.
    Ended,
    /// A taker may still run, or its record cannot be read.
    Runs,
    /// A taker's birth names no value, and its record has stood past the patience: the words a person reads.
    CannotBeTold(String),
}

/// What:
///  Whether a command that took the snapshot may still be running,
/// which a process that cannot be told apart from it counts as.
/// Why:
///   A process number alone can be reused,
/// so the birth identity recorded with it decides,
/// by the value it names.
/// A record that cannot be read,
/// or a
///  process that cannot be inspected,
/// is an owner that may be alive:
/// discarding the snapshot on a guess would lose the records.
///  A record whose birth names no value is neither a running command's nor an ended one's,
///  and its own age is the bound:
///  inside the patience the snapshot is kept,
///  past it the person reads which record it is and the command that removes it.
///  `pub(super)` lets the controls of this file's parent ask it about a snapshot they build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await attemptsMayBeRunning(snapshot)
/// ```
pub(super) fn attempt_may_be_running(snapshot: &Path) -> Attempts {
    // The loop stops at the first attempt that may be running, as `any` did.
    let mut said: Option<String> = None;
    for path in files_of(snapshot.join(ATTEMPTS).as_path()) {
        match attempt_file_may_be_running(path.as_path()) {
            Attempts::Ended => {}
            Attempts::Runs => return Attempts::Runs,
            Attempts::CannotBeTold(words) => said = Some(words),
        }
    }
    return match said {
        Some(words) => Attempts::CannotBeTold(words),
        None => Attempts::Ended,
    };
}

/// What:
///  What the command that wrote one attempt file says about itself.
/// Why:
///   A file that cannot be understood counts as running,
///  and one whose birth names no value is neither running nor ended:
///  the file is written in the one spelling of a birth,
///  so a text of no value came from a writer that is not this release,
///  and a snapshot taken for a command that may still run is kept.
fn attempt_file_may_be_running(path: &Path) -> Attempts {
    let Ok(pid) = name_of(path).parse::<i64>() else { return Attempts::Ended };
    // A plain file, so a name that holds a named pipe counts as a command that may still be running rather than a wait
    //  with no end.
    let Ok(bytes) = crate::plain_file::read_plain_file(path, crate::plain_file::SMALL_FILE_LIMIT) else { return Attempts::Runs };
    let Ok(recorded) = String::from_utf8(bytes) else { return Attempts::Runs };
    if recorded.is_empty() {
        // Being written.
        return Attempts::Runs;
    }
    return match process_birth_identity(pid) {
        Ok(Some(now)) => match (birth_identity_value(recorded.as_str()), birth_identity_value(now.as_str())) {
            (Some(theirs), Some(live)) => {
                if theirs == live {
                    Attempts::Runs
                } else {
                    Attempts::Ended
                }
            }
            // A birth that names no value is neither a running command's nor an ended one's.
            _ => match unreadable_owner_standing(path, UnreadablePart::Birth(recorded.clone()), pid) {
                UnreadableOwnerStanding::MayRun => Attempts::Runs,
                UnreadableOwnerStanding::Ended => Attempts::Ended,
                UnreadableOwnerStanding::Refused(words) => Attempts::CannotBeTold(words),
            },
        },
        Ok(None) => Attempts::Ended,
        Err(error) => {
            warn(TAG, format!("the process {pid} of a worktree removal cannot be inspected: {error}").as_str());
            Attempts::Runs
        }
    };
}

/// Whether the checkout a registered worktree's Git directory names is still there;
/// nothing when the Git directory does not say.
pub(super) fn checkout_present(git_dir: &Path) -> Option<bool> {
    // Git writes this file itself, as the bytes of the checkout's `.git`, so it is read as bytes and never as text.
    let bytes: Vec<u8> = crate::plain_file::read_plain_file(git_dir.join("gitdir").as_path(), crate::plain_file::SMALL_FILE_LIMIT).ok()?;
    // Git ends the file with one line break, and a checkout's path may hold one of its own:
    //  stopping at the first one examines a directory the path only begins with.
    let recorded: &[u8] = bytes.strip_suffix(b"\n".as_slice()).unwrap_or(bytes.as_slice());
    let marker: PathBuf = PathBuf::from(crate::path_bytes::os_from_path_bytes(recorded));
    if marker.as_os_str().is_empty() {
        return None;
    }
    return Some(git_dir.join(marker).exists());
}

/// What:
///  The Git directory a snapshot's `gitdir` file names:
/// its bytes when the file is one this release wrote,
/// its text when it is an earlier release's.
/// Why:
///   The removal of a worktree is judged by looking at this directory,
/// and a spelling that lost the path's bytes names nothing,
/// which reads as a removal that finished and would promote a live worktree's records.
/// Only the line break this release writes after the bytes is removed:
/// an earlier release wrote the text of the directory and nothing after it,
/// so the last byte of that text is the path's even when it is a line break.
/// A name that does not decode is no name;
/// the snapshot is then kept,
/// never settled on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedGitDir = (text) => text.startsWith('gitdirHex=') ? pathOfHex(text.slice(10).replace(/\n$/, '')) : (text === '' ? undefined : text);
/// ```
pub(super) fn named_git_dir(text: &str) -> Option<PathBuf> {
    let (bytes, plain): (Option<&str>, Option<&str>) = match text.strip_prefix(GITDIR_HEX_PREFIX) {
        Some(hexadecimal) => (Some(hexadecimal.strip_suffix('\n').unwrap_or(hexadecimal)), None),
        None if text.is_empty() => (None, None),
        None => (None, Some(text)),
    };
    let named: PathBuf = path_bytes::path_of_the_two_fields(bytes, plain).ok().flatten()?;
    return has_a_name(named.as_path()).then_some(named);
}

/// Whether a path read from a record names anything at all.
fn has_a_name(path: &Path) -> bool {
    return !path.as_os_str().is_empty();
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
/// a checkout that is gone
///  while its taker has ended cannot come back,
/// so the removal is over however it ended and the records must reach the person;
/// while the taker lives the removal may be under way,
/// and a checkout whose state cannot be read decides nothing.
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
    match attempt_may_be_running(snapshot) {
        Attempts::Ended => {}
        Attempts::Runs => return Removal::Undecided,
        // The taker's own record is past the patience and names no birth this command can read: nothing here is settled or
        //  discarded, and the person reads which record it is and the command that removes it.
        Attempts::CannotBeTold(words) => return Removal::CannotBeTold(words),
    }
    return match checkout_present(git_dir) {
        Some(true) => Removal::Refused,
        Some(false) => Removal::Done,
        None => Removal::Undecided,
    };
}
