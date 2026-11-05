//! What:
//!  Settle every file and folder this program keeps inside a repository's Git directories:
//!  each one an earlier release wrote under its earlier name is renamed in place to the current name,
//!  and a reader that must not move anything finds the one that exists.
//! Why:
//!  The owner renamed the folders and files the wrapper keeps (`doc/planning/git-policy-cli-program-rename.md`), and the
//!  code that reads and writes them spells only the current name. The move runs once, when the wrapper first learns which
//!  Git directories a command works in, so no other file repeats the rule, and a name that was never needed is never
//!  touched. A log that rotates keeps its numbered older copies (`<log>.1`), and they move with it, because the audit log
//!  copies name saved files still on disk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! settleKeptNames(gitDir, commonDir); // renames cli-git/ to monochromatic-git-policy-cli/ when only the earlier one exists
//! ```

/// The move of one name,
///  and the finder of the one that exists.
use crate::earlier_name_moved_to_current_in_place::{existing_kept_name, kept_name_for_use};
/// The rename that never replaces.
use crate::rename_that_never_replaces::rename_noreplace;
/// The pairs of names.
use crate::wrapper_names_old_and_new::{
    COMMIT_TIMING_LOG, DATA_FOLDER, DISCARD_AUDIT_LOG, MERGE_INTENT_FOLDER, PUSH_TRIGGER_LOG, RenamedName, WORKTREE_ADD_TIMING_LOG,
    WORKTREE_COPY_RECORD_FOLDER,
};
/// Paths.
use std::path::{Path, PathBuf};

/// The names kept in the common Git directory that rotate into numbered older copies.
const ROTATING_LOGS_IN_COMMON_DIRECTORY: [RenamedName; 3] = [DISCARD_AUDIT_LOG, COMMIT_TIMING_LOG, WORKTREE_ADD_TIMING_LOG];

/// The names kept in the common Git directory that do not rotate.
const OTHER_NAMES_IN_COMMON_DIRECTORY: [RenamedName; 3] = [DATA_FOLDER, PUSH_TRIGGER_LOG, WORKTREE_COPY_RECORD_FOLDER];

/// The names kept in a worktree's own Git directory.
const NAMES_IN_WORKTREE_DIRECTORY: [RenamedName; 2] = [DATA_FOLDER, MERGE_INTENT_FOLDER];

/// Whether one byte of a name is an ASCII digit;
///  a named function so `all` can take it.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
///  Move the numbered older copies of a rotated log,
///  `<earlier>.<number>` to `<current>.<number>`,
///  each by a rename that never replaces.
///  `std::fs::read_dir` lists a folder.
/// Why:
///   The audit log's older copies hold lines that name saved files still on disk,
///  so they are never left behind under a name nothing reads.
///  A copy whose new name exists stays where it is,
///  and the base-name notice has already told the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of await readdir(parent)) if (/^cli-git-audit\.jsonl\.\d+$/.test(entry)) await renameNoReplace(...);
/// ```
fn move_numbered_copies(parent: &Path, name: RenamedName) {
    let Ok(listing) = std::fs::read_dir(parent) else {
        return;
    };
    let wanted_start: String = format!("{}.", name.earlier);
    for entry in listing.flatten() {
        let file_name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(number) = file_name.strip_prefix(wanted_start.as_str()) else {
            continue;
        };
        if number.is_empty() || !number.bytes().all(is_digit_byte) {
            continue;
        }
        let target: PathBuf = parent.join(format!("{}.{number}", name.current));
        let _moved = rename_noreplace(entry.path().as_path(), target.as_path());
    }
}

/// What:
///  Settle the names of one repository's Git directories.
///  `&Path` borrows a path.
/// Why:
///   Called once per process where the wrapper learns the directories;
///  a directory that does not exist (a bare or absent layout) costs two failed looks per name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleKeptNames(gitDir: string, commonDir: string): void;
/// ```
pub fn settle_kept_names(git_dir: &Path, common_dir: &Path) {
    settle_worktree_directory_names(git_dir);
    settle_common_directory_names(common_dir);
}

/// Settle the names kept in a worktree's own Git directory:
///  for a command that learned only that directory.
pub fn settle_worktree_directory_names(git_dir: &Path) {
    for name in NAMES_IN_WORKTREE_DIRECTORY {
        let _placed = kept_name_for_use(git_dir, name);
    }
}

/// Settle the names kept in the common Git directory:
///  for a command that learned only that directory,
///  such as the health report.
pub fn settle_common_directory_names(common_dir: &Path) {
    for name in OTHER_NAMES_IN_COMMON_DIRECTORY {
        let _placed = kept_name_for_use(common_dir, name);
    }
    for name in ROTATING_LOGS_IN_COMMON_DIRECTORY {
        let _placed = kept_name_for_use(common_dir, name);
        move_numbered_copies(common_dir, name);
    }
}

/// What:
///  A path below a Git directory that is spelled with the current data folder in front
///  (`monochromatic-git-policy-cli/work`),
///  for a reader:
///  below the earlier folder when only that exists.
///  `&str` is borrowed text compiled into the executable.
/// Why:
///   The readers of notices,
///  work records and snapshots visit Git directories that were not settled by this process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathForReading(directory: string, relative: string): string;
/// ```
pub fn path_for_reading(directory: &Path, relative: &str) -> PathBuf {
    let prefix: String = format!("{}/", DATA_FOLDER.current);
    if let Some(rest) = relative.strip_prefix(prefix.as_str()) {
        return data_folder_for_reading(directory).join(rest);
    }
    return directory.join(relative);
}

/// What:
///  The folder that holds a Git directory's saved copies,
///  notices and work records,
///  for a reader that visits a directory it does not own:
///  the current folder,
///  or the earlier one when only that exists.
/// Why:
///   Another worktree's folder may be in use by an older program at this moment,
///  so only the program that owns the directory moves it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dataFolderForReading(gitDir: string): string;
/// ```
pub fn data_folder_for_reading(directory: &Path) -> PathBuf {
    return existing_kept_name(directory, DATA_FOLDER);
}

#[cfg(test)]
#[path = "kept_names_in_the_git_directories_tests.rs"]
mod tests;
