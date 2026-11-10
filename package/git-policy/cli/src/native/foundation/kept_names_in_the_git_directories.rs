//! What:
//!  Settle every file and folder this program keeps inside a repository's Git directories:
//!  each one an earlier release wrote under its earlier name is renamed in place to the current name,
//!  and a reader that must not move anything finds the one that exists.
//! Why:
//!  The owner renamed the folders and files the wrapper keeps (`doc/planning/git-policy-cli-program-rename.md`),
//! and the
//!  code that reads and writes them spells only the current name.
//! The move runs once,
//! when the wrapper first learns which
//!  Git directories a command works in,
//! so no other file repeats the rule,
//! and a name that was never needed is never
//!  touched.
//! A log keeps its numbered older copies (`<log>.1`),
//! and they move with it,
//! because an audit line can name a
//!  saved copy still on disk.
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
/// The fold of a name that exists under both spellings,
/// and the surroundings its look reads.
use crate::earlier_names_folded_into_current::{FoldSurroundings, KeptKind, KeptName, fold_and_tell, fold_kept_names_and_tell};
/// The steps a fold reports through,
/// and the ones a command that stops for nothing uses.
use crate::earlier_log_lines_added_to_current_log::{FoldSteps, NeverStopped};
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

/// The names to fold in a worktree's own Git directory when both spellings exist.
const WORKTREE_DIRECTORY_FOLDS: [KeptName; 2] = [
    KeptName { name: DATA_FOLDER, kind: KeptKind::Folder },
    KeptName { name: MERGE_INTENT_FOLDER, kind: KeptKind::Folder },
];

/// The names to fold in the common Git directory when both spellings exist.
const COMMON_DIRECTORY_FOLDS: [KeptName; 6] = [
    KeptName { name: DATA_FOLDER, kind: KeptKind::Folder },
    KeptName { name: WORKTREE_COPY_RECORD_FOLDER, kind: KeptKind::Folder },
    KeptName { name: PUSH_TRIGGER_LOG, kind: KeptKind::Log },
    KeptName { name: DISCARD_AUDIT_LOG, kind: KeptKind::Log },
    KeptName { name: COMMIT_TIMING_LOG, kind: KeptKind::Log },
    KeptName { name: WORKTREE_ADD_TIMING_LOG, kind: KeptKind::Log },
];

/// The names to fold in a Git directory that is both the worktree's own and the common one (the main worktree).
const MAIN_DIRECTORY_FOLDS: [KeptName; 7] = [
    KeptName { name: DATA_FOLDER, kind: KeptKind::Folder },
    KeptName { name: MERGE_INTENT_FOLDER, kind: KeptKind::Folder },
    KeptName { name: WORKTREE_COPY_RECORD_FOLDER, kind: KeptKind::Folder },
    KeptName { name: PUSH_TRIGGER_LOG, kind: KeptKind::Log },
    KeptName { name: DISCARD_AUDIT_LOG, kind: KeptKind::Log },
    KeptName { name: COMMIT_TIMING_LOG, kind: KeptKind::Log },
    KeptName { name: WORKTREE_ADD_TIMING_LOG, kind: KeptKind::Log },
];

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
    settle_kept_names_within(git_dir, common_dir, &FoldSurroundings::of_this_process(), &mut NeverStopped);
}

/// What:
///  `settle_kept_names` with the surroundings and steps given.
/// Why:
///   The fold's look reads a process table and a state folder,
/// and a control must give its own so its result does not depend on the machine's services.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settleKeptNamesWithin(gitDir, commonDir, { procRoot: fixtureProc, stateDir: null }, steps);
/// ```
pub fn settle_kept_names_within(git_dir: &Path, common_dir: &Path, surroundings: &FoldSurroundings, steps: &mut dyn FoldSteps) {
    rename_worktree_directory_names(git_dir);
    rename_common_directory_names(common_dir);
    // The main worktree's directory is both, and its looks at the earlier program are made once.
    if git_dir == common_dir {
        fold_kept_names_and_tell(common_dir, &MAIN_DIRECTORY_FOLDS, surroundings, steps);
    } else {
        fold_kept_names_and_tell(git_dir, &WORKTREE_DIRECTORY_FOLDS, surroundings, steps);
        fold_kept_names_and_tell(common_dir, &COMMON_DIRECTORY_FOLDS, surroundings, steps);
    }
}

/// Move the names kept in a worktree's own Git directory that exist only under the earlier spelling.
fn rename_worktree_directory_names(git_dir: &Path) {
    for name in NAMES_IN_WORKTREE_DIRECTORY {
        let _placed = kept_name_for_use(git_dir, name);
    }
}

/// Move the names kept in the common Git directory that exist only under the earlier spelling.
fn rename_common_directory_names(common_dir: &Path) {
    for name in OTHER_NAMES_IN_COMMON_DIRECTORY {
        let _placed = kept_name_for_use(common_dir, name);
    }
    for name in ROTATING_LOGS_IN_COMMON_DIRECTORY {
        let _placed = kept_name_for_use(common_dir, name);
        move_numbered_copies(common_dir, name);
    }
}

/// Settle the names kept in a worktree's own Git directory:
///  for a command that learned only that directory.
pub fn settle_worktree_directory_names(git_dir: &Path) {
    rename_worktree_directory_names(git_dir);
    fold_and_tell(git_dir, &WORKTREE_DIRECTORY_FOLDS);
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

/// What:
///  A path below a Git directory,
///  spelled relatively and with the data folder in front,
///  for a reader that opens it by
///  walking below the Git directory without following a link on the way:
///  under the current folder,
///  or under the
///  earlier one when only that exists.
///  `String` is owned text.
/// Why:
///   `path_for_reading` gives the same answer as an absolute path;
///  a reader that walks the components from the Git directory's own descriptor needs
///  the relative spelling,
///  and must never move the folder it is reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePathForReading(directory: string, relative: string): string;
/// ```
/// What:
///  Every relative path below a Git directory a reader must visit for `relative`,
/// one for each data folder that exists,
///  the current name first.
///  `Vec<String>` is owned relative paths.
/// Why:
///   A repository can hold both data-folder names at once,
/// and the earlier one can hold saved copies while the current one
///  holds none;
/// a reader that moves nothing must look at both,
/// and one that looked only at the name it found first hid
///  the other folder's data from the person who asked.
///  When neither folder exists the current spelling is answered once,
/// because a folder that is absent is not a
///  failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePathsForReading(directory: string, relative: string): string[];
/// ```
pub fn relative_paths_for_reading(directory: &Path, relative: &str) -> Vec<String> {
    let prefix: String = format!("{}/", DATA_FOLDER.current);
    let Some(rest) = relative.strip_prefix(prefix.as_str()) else {
        return vec![relative.to_string()];
    };
    let mut paths: Vec<String> = Vec::new();
    for name in [DATA_FOLDER.current, DATA_FOLDER.earlier] {
        if directory.join(name).exists() {
            paths.push(format!("{name}/{rest}"));
        }
    }
    if paths.is_empty() {
        paths.push(format!("{prefix}{rest}"));
    }
    return paths;
}

/// What:
///  The relative path below a Git directory a reader must use for `relative`,
/// under the current data
///  folder,
/// or under the earlier one when only that exists.
/// Why:
///   A reader that visits one folder and never both is answered here;
/// a reader that must not miss a folder uses
///  `relative_paths_for_reading`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePathForReading(directory: string, relative: string): string;
/// ```
pub fn relative_path_for_reading(directory: &Path, relative: &str) -> String {
    let prefix: String = format!("{}/", DATA_FOLDER.current);
    let Some(rest) = relative.strip_prefix(prefix.as_str()) else {
        return relative.to_string();
    };
    let folder: PathBuf = data_folder_for_reading(directory);
    let Some(name) = folder.file_name().and_then(std::ffi::OsStr::to_str) else {
        return relative.to_string();
    };
    return format!("{name}/{rest}");
}

#[cfg(test)]
#[path = "kept_names_in_the_git_directories_tests.rs"]
mod tests;
