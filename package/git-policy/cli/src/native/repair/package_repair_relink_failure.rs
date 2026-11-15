//! What:
//!  Why a relink's last try failed,
//!  kept beside the work records until the record settles or the person is told.
//! Why:
//!  A record whose tries run out is given up on by the recovery,
//!  not by the round that ran the tool,
//!  so the step that words the notice holds the record and never the answer the tool gave.
//!  A count of tries would then be the only word about a worktree whose package links still name the checkout
//!  it was copied from,
//!  and a count names nothing a person can change.
//!  The reason is one line in a file of its own,
//!  named after the record,
//!  written when the round asks for another try and read (and taken away) by the notice,
//!  so nothing has to be parsed to show it and a record that settles leaves nothing behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! remember(gitDir, id, reason); const reason = recall(gitDir, id); forget(gitDir, id);
//! ```

use crate::plain_file::{SMALL_FILE_LIMIT, read_plain_text_at};
use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced};
use std::path::{Path, PathBuf};

/// Directory under the worktree's own Git directory that holds one reason per relink record.
pub const FAILURE_DIRECTORY: &str = "monochromatic-git-policy-cli/relink-failures";

/// What:
///  The file one record's reason lives in.
/// Why:
///   The notice and the round that keeps the reason both name it,
///  and a record's id is already a name no shell or filesystem reads as a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = (gitDir, id) => join(gitDir, 'monochromatic-git-policy-cli/relink-failures', id);
/// ```
fn path_of(git_dir: &Path, id: &str) -> PathBuf {
    return git_dir.join(FAILURE_DIRECTORY).join(id);
}

/// What:
///  Keep `reason` as what the last try of `id` failed with.
/// Why:
///  The same record may be tried several times,
///  and the person is owed the answer the last try gave,
///  so a later reason replaces an earlier one.
///  A reason that cannot be kept does not stop the retry the answer asked for:
///  the record is still the service's work,
///  and the notice then says no answer was kept and points at the service's journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(join(gitDir, DIR)); replaceFile(pathOf(gitDir, id), `${reason}\n`);
/// ```
pub fn remember(git_dir: &Path, id: &str, reason: &str) -> Result<(), String> {
    let directory: PathBuf = git_dir.join(FAILURE_DIRECTORY);
    create_private_directory_all(directory.as_path()).map_err(|error| format!("the directory {} could not be made ({error})", directory.display()))?;
    let path: PathBuf = path_of(git_dir, id);
    // A record's second try replaces the reason its first try left, and a name that holds a record already is not a
    //  reason to keep the older one: the person is owed the answer the last try gave.
    return replace_private_file_unsynced(path.as_path(), format!("{reason}\n").as_bytes()).map_err(|error| format!("the reason could not be written to {} ({error})", path.display()));
}

/// What:
///  The reason the last try of `id` failed with,
///  or nothing when no try failed and no reason was kept.
/// Why:
///   A notice is worded once and read once,
///  so the reason is read here and the record's own file is taken away by `forget`
///  only after the record is where nothing reads it again.
///  The file is read the way a record beside it is: only a plain file is opened, without following a link,
///  and only so many bytes of it are read,
///  so a named pipe at that name holds neither a command nor the service,
///  and no file decides how much memory a command reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = readPlainTextAt(pathOf(gitDir, id), SMALL_FILE_LIMIT).trim() ?? undefined;
/// ```
pub fn recall(git_dir: &Path, id: &str) -> Option<String> {
    let text: String = read_plain_text_at(path_of(git_dir, id).as_path(), SMALL_FILE_LIMIT).ok()?;
    let reason: String = text.trim_end_matches('\n').to_owned();
    // A file that holds nothing says nothing, and the notice then words the account as one with no answer to quote.
    return (!reason.is_empty()).then_some(reason);
}

/// What:
///  Take the kept reason away.
/// Why:
///   A record that is in `failed` is read by nothing again,
///  so its reason would be a file the person never sees and nothing removes.
///  It is called only once the record is where it will stay, because a pass killed between the words and that move
///  finds the reason still here and says the same thing again,
///  which is what the recovery promises a kill in that window.
///  A file that is already gone is nothing to report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unlink(pathOf(gitDir, id)) // ignore ENOENT
/// ```
pub fn forget(git_dir: &Path, id: &str) {
    let path: PathBuf = path_of(git_dir, id);
    let _removed = std::fs::remove_file(path.as_path());
}

/// Failure controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_failure_tests.rs"]
mod tests;
