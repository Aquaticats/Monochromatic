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
///  and the notice then says the tries ran without an answer it can quote.
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
///  only after the words are where a command shows them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = readFile(pathOf(gitDir, id), 'utf8').trim() ?? undefined;
/// ```
pub fn recall(git_dir: &Path, id: &str) -> Option<String> {
    let text: String = std::fs::read_to_string(path_of(git_dir, id)).ok()?;
    let reason: String = text.trim_end_matches('\n').to_owned();
    // A file that holds nothing says nothing, and the notice then words the tries as ones that gave no answer at all.
    return (!reason.is_empty()).then_some(reason);
}

/// What:
///  Take the kept reason away.
/// Why:
///   A record that settles is read by nothing again,
///  so its reason would be a file the person never sees and nothing removes.
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
