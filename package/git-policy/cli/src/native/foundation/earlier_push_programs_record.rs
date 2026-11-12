//! What:
//!  The record a wrapper of one unreleased branch wrote beside a worktree's Git directory,
//!  and the staging files of its own writes,
//!  removed by the first command that meets them.
//! Why:
//!  That branch wrote `<git-dir>/monochromatic-git-policy-cli/push-programs.json` at the start of every commit,
//! with the person's `PATH`,
//! `GIT_EXEC_PATH`,
//! `GIT_SSH`,
//! `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` copied into it,
//! and the last two of those are whole command lines that can hold a password.
//! The values are now carried in the memory of the commit service and written nowhere
//! (`service_push_programs_held_in_memory`),
//! so nothing writes such a file again,
//! and one left in a worktree is a credential of the person's on a disk for as long as nobody removes it.
//! The branch was never released,
//! so no release has to read the record:
//! every command that finds it removes it and says so.
//!  A write of that release staged its bytes in `.<name>.<random>.partial` beside the name and published them by one rename,
//! so a command killed in the microseconds between the two left the whole record in a file nobody would look at again;
//! those are removed by the same rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removeTheRecordAnEarlierBranchWrote(gitDir); // unlink <gitDir>/monochromatic-git-policy-cli/push-programs.json and .push-programs.json.*.partial
//! ```

use crate::diagnostic_log::debug;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "earlierPushProgramsRecord";

/// The folder under a worktree's own Git directory that held that record,
/// which is the folder the work records still use.
pub const FOLDER: &str = "monochromatic-git-policy-cli";

/// The record's file name.
pub const FILE: &str = "push-programs.json";

/// What:
///  Removes that record, and every staging file one of its writes left, from the Git directory `git_dir`.
/// Why:
///   The file holds command lines that can carry a credential,
/// and no release writes or reads it any more,
/// so meeting it is the one chance to take it off the disk.
/// A command removes it once and says which paths went,
/// so a person who wants to know what was removed finds it in the wrapper's own log.
/// Nothing else in the folder is touched:
/// the work records and the notices beside it are this release's,
/// and a directory of that name is not the file that branch wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = [...stagingFiles(gitDir), recordPath(gitDir)].filter((path) => unlinkIfAFile(path)); if (removed.length) log('removed', removed);
/// ```
pub fn remove_what_an_earlier_branch_wrote(git_dir: &Path) {
    let folder: PathBuf = git_dir.join(FOLDER);
    let mut removed: Vec<PathBuf> = staging_files_of(folder.as_path());
    let record: PathBuf = folder.join(FILE);
    if unlink_a_file(record.as_path()) {
        removed.push(record);
    }
    if removed.is_empty() {
        return;
    }
    let shown: Vec<String> = removed.iter().map(|path| { return path.display().to_string(); }).collect();
    // The paths name no value of the record: what was in it is gone, and its own words could hold a password.
    debug(
        TAG,
        format!(
            "removed {} a wrapper of an unreleased branch wrote: {}. Those values now travel with the request to the commit service that runs the push, and are never written",
            if shown.len() == 1 { "a record" } else { "records" },
            shown.join(", ")
        )
        .as_str(),
    );
}

/// What:
///  The staging files one of that record's writes left in `folder`:
///  the hidden, uniquely named `.push-programs.json.<random>.partial` files.
/// Why:
///   A staging file holds the whole record,
/// so a write killed before its rename left a credential in a name nothing reads.
/// A folder that cannot be read is a folder with nothing to remove,
/// which is the answer a Git directory without that folder gives too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdirSync(folder).filter((name) => name.startsWith(`.${FILE}.`) && name.endsWith('.partial')).map((name) => join(folder, name));
/// ```
fn staging_files_of(folder: &Path) -> Vec<PathBuf> {
    let prefix: String = format!(".{FILE}.");
    let mut found: Vec<PathBuf> = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else { return found };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(prefix.as_str()) || !name.ends_with(".partial") {
            continue;
        }
        let path: PathBuf = entry.path();
        if unlink_a_file(path.as_path()) {
            found.push(path);
        }
    }
    return found;
}

/// What:
///  Removes `path` when it is a file or a link,
///  and answers whether it did.
/// Why:
///   `remove_file` takes a link itself and never what it names,
/// so a link planted in that folder cannot make this command delete anything else.
/// A directory of that name is not what that branch wrote,
/// and a path that is already gone is the ordinary second command in a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAFile = lstatSync(path, { throwIfNoEntry: false })?.isFile() || isSymbolicLink(); return isAFile && unlinkSync(path) === undefined;
/// ```
fn unlink_a_file(path: &Path) -> bool {
    let Some(metadata) = std::fs::symlink_metadata(path).ok() else { return false };
    if !metadata.is_file() && !metadata.is_symlink() {
        return false;
    }
    return std::fs::remove_file(path).is_ok();
}

/// The controls of the removal are in the sidecar.
#[cfg(test)]
#[path = "earlier_push_programs_record_tests.rs"]
mod tests;
