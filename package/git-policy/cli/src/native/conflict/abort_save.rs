//! What:
//!  What an abort or a skip copies before it runs,
//!  and what it leaves for the comparison made immediately before Git starts
//!  (`abort_save_again.rs`).
//! Why:
//!  The owner's rule is "save,
//!  then allow",
//!  and the copy has to be of what Git is about to discard:
//!  the worktree files the listing named,
//!  and the content that stands only in the index,
//!  which no worktree file holds.
//!  A file can also change after it was copied and before Git starts,
//!  and that edit is discarded with no copy,
//!  so each saved source is compared once more at the last moment the wrapper controls,
//!  and whatever changed is copied again:
//!  a bounded number of rounds,
//!  so a file another program keeps writing cannot hold the command for good,
//!  after which the command is refused and the file is named.
//!  What remains after that comparison is a window no program outside Git can close,
//!  and `SPEC.md` and the notice say so in plain words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const saved = saveBeforeTheCommand({ root, commonDir, git, paths, stagedPaths, milliseconds });
//! planTheLastChance(saved.plan); // the comparison itself is `abort_save_again.ts`
//! ```

use crate::abort_save_again::{SAVE_AGAIN_ROUNDS, SaveAgainPlan};
use crate::conflict_side_source::{FailureKind, SaveFailure, WhenTheSourceChanges};
use crate::conflict_sides::{SAVED_DIRECTORY, SaveAttempt, SavedCopy, SavedSource, fresh_saved_directory, save_into_watching};
use crate::forwarding::git_command;
use crate::hooks_off::StartOwner;
use crate::private_storage::{create_private_directory_all, sync_directory};
use crate::transaction_git::{GitContext, input_file};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The name of the folder the index's own content is copied into,
///  beside the worktree copies.
/// Why:
///   A person looking for the content of a staged file finds it at the path the file has,
///  and the folder says which stage it came from,
///  because a worktree copy of a path and the index's copy cannot share a name.
pub const STAGED_FOLDER: &str = "index-stage-0";

/// The folder a first save puts a source that changed while it was copied into.
const FIRST_SAVE_AGAIN_FOLDER: &str = "again";

/// What:
///  What a command saved before it runs, and what the last comparison needs.
/// Why:
///   The copies go into the notice and the audit record,
///  and the sources the wrapper must compare again travel with the command
///  until the instant before Git starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SavedBeforeCommand = { copies: SavedCopy[]; plan?: SaveAgainPlan };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedBeforeCommand {
    /// Every copy this command's saves made,
    ///  in the order they were made.
    pub copies: Vec<SavedCopy>,
    /// The folder they are in,
    ///  which holds the worktree copies, the index's copies in its own subfolder,
    ///  and any copy a source that changed under the copy earned.
    pub directory: PathBuf,
    /// The comparison the wrapper owes immediately before Git starts,
    ///  or nothing when it copied no worktree file.
    pub plan: Option<SaveAgainPlan>,
}

/// What:
///  Save the worktree files and the content the index holds, then say what the last comparison owes.
///  `&[Vec<u8>]` borrows the paths the listing named and the ones no worktree file holds.
/// Why:
///   The order is the owner's: everything the command would discard is on disk before it runs.
///  A source that changes while it is copied is copied again,
///  within the same bound the last comparison uses,
///  and one that never holds still is a refusal that names it.
///  The content of a path the worktree lost while the index kept it is copied from the index itself,
///  into a folder that names the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = saveBeforeTheCommand({ root, commonDir, git, paths, stagedPaths, milliseconds });
/// ```
pub fn save_before_the_command(
    root: &Path,
    common_dir: &Path,
    git: &GitContext,
    paths: &[Vec<u8>],
    staged_paths: &[Vec<u8>],
    milliseconds: i64,
) -> Result<SavedBeforeCommand, SaveFailure> {
    return save_before_the_command_watching(root, common_dir, git, paths, staged_paths, milliseconds, &mut nothing_between);
}

/// What:
///  A pause that does nothing between the chunks of a copy.
/// Why:
///   A shipped save has nothing to do between chunks,
///  and the reading above names it rather than a closure with an empty body at each call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingBetween = (): void => {};
/// ```
fn nothing_between() {}

/// What:
///  `save_before_the_command`,
///  running `between` after each chunk of each copy.
/// Why:
///   The same seam the source module offers its controls,
///  so a control can make a source grow while it is being copied and see the save copy it again
///  instead of refusing the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = saveBeforeTheCommandWatching({ root, commonDir, git, paths, stagedPaths, milliseconds, between });
/// ```
pub fn save_before_the_command_watching(
    root: &Path,
    common_dir: &Path,
    git: &GitContext,
    paths: &[Vec<u8>],
    staged_paths: &[Vec<u8>],
    milliseconds: i64,
    between: &mut dyn FnMut(),
) -> Result<SavedBeforeCommand, SaveFailure> {
    let directory: PathBuf = match fresh_saved_directory(common_dir, milliseconds) {
        Ok(made) => made,
        Err(error) => return Err(SaveFailure::io(common_dir.join(SAVED_DIRECTORY).display().to_string(), error.to_string())),
    };
    let mut copies: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    let mut sources: Vec<SavedSource> = Vec::<SavedSource>::new();
    let mut remaining: Vec<Vec<u8>> = paths.to_vec();
    let mut round: u8 = 0;
    while !remaining.is_empty() {
        let into: PathBuf = if round == 0 {
            directory.clone()
        } else {
            directory.join(format!("{FIRST_SAVE_AGAIN_FOLDER}-{round}"))
        };
        let attempt: SaveAttempt = save_into_watching(root, into.as_path(), remaining.as_slice(), between, WhenTheSourceChanges::Remove)?;
        copies.extend(attempt.copies.iter().cloned());
        if let Some(failure) = attempt.changed.first().cloned() {
            // The source that changed has neither a copy nor a stamp; it is asked for again until the bound,
            //  and every path the round had not reached yet is asked for again too.
            if round >= SAVE_AGAIN_ROUNDS {
                return Err(changed_source_failure(&failure, directory.as_path()));
            }
            round += 1;
            remaining = unsaved_paths(remaining.as_slice(), attempt.sources.as_slice());
            continue;
        }
        sources = attempt.sources;
        remaining = Vec::<Vec<u8>>::new();
    }
    let staged: Vec<SavedCopy> = save_staged_content(git, common_dir, directory.as_path(), staged_paths)?;
    copies.extend(staged);
    if sources.is_empty() {
        return Ok(SavedBeforeCommand { copies, directory, plan: None });
    }
    return Ok(SavedBeforeCommand {
        copies,
        plan: Some(SaveAgainPlan { root: root.to_path_buf(), directory: directory.clone(), sources }),
        directory,
    });
}

/// What:
///  The refusal of a source that changed through every round of the first save.
/// Why:
///   The save step's own reason says the source changed while it was copied;
///  the person also needs to know that nothing was changed and where the copies are.
fn changed_source_failure(failure: &SaveFailure, directory: &Path) -> SaveFailure {
    return SaveFailure {
        subject: failure.subject.clone(),
        reason: format!(
            "{}; it changed through {SAVE_AGAIN_ROUNDS} rounds of copying, so nothing was changed and the copies that were made stand in {}",
            failure.reason,
            directory.display()
        ),
        kind: FailureKind::Changed,
    };
}

/// What:
///  The paths one round did not save, in their own order:
///  the source that changed under it and every path after that one.
/// Why:
///   The round stops reading when a source changes,
///  so the paths after it have no copy yet and must be asked for again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remaining = unsavedPaths({ paths, savedSources });
/// ```
fn unsaved_paths(paths: &[Vec<u8>], saved: &[SavedSource]) -> Vec<Vec<u8>> {
    let mut remaining: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in paths {
        let copied: bool = saved.iter().any(|source| { return source.path == *path && source.stamp.is_some(); });
        if !copied {
            remaining.push(path.clone());
        }
    }
    return remaining;
}

/// What:
///  Copy the content the index holds at each path below `<directory>/index-stage-0`.
/// Why:
///   An abort resets the index as well as the worktree,
///  so content that stands only there would be discarded,
///  and no worktree file holds it to copy.
///  Git's own `checkout-index` writes it,
///  from the index the abort is about to reset,
///  so the copy is exactly what the index held rather than a second reading of it,
///  and a path Git writes nothing for is a refusal that names the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = saveStagedContent({ git, commonDir, directory, paths });
/// ```
pub fn save_staged_content(git: &GitContext, common_dir: &Path, directory: &Path, paths: &[Vec<u8>]) -> Result<Vec<SavedCopy>, SaveFailure> {
    if paths.is_empty() {
        return Ok(Vec::<SavedCopy>::new());
    }
    let staged_root: PathBuf = directory.join(STAGED_FOLDER);
    if let Err(error) = create_private_directory_all(staged_root.as_path()) {
        return Err(SaveFailure::io(staged_root.display().to_string(), error.to_string()));
    }
    let mut input: Vec<u8> = Vec::<u8>::new();
    for path in paths {
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    let mut arguments: Vec<OsString> = git.global_prefix.clone();
    for token in ["--no-optional-locks", "checkout-index", "--stdin", "-z"] {
        arguments.push(OsString::from(token));
    }
    let mut prefix: OsString = OsString::from("--prefix=");
    prefix.push(staged_root.as_os_str());
    prefix.push("/");
    arguments.push(prefix);
    let mut command: Command = git_command(git.real_git.as_path(), arguments.as_slice(), git.overlay.as_slice(), StartOwner::Wrapper);
    let handle: std::fs::File = match input_file(input.as_slice(), Some(common_dir)) {
        Ok(file) => file,
        Err(error) => return Err(SaveFailure::io(common_dir.display().to_string(), error.to_string())),
    };
    command.stdin(Stdio::from(handle));
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());
    let output: std::process::Output = match command.output() {
        Ok(finished) => finished,
        Err(error) => return Err(SaveFailure::io(git.real_git.display().to_string(), error.to_string())),
    };
    if !output.status.success() {
        let detail: String = String::from_utf8_lossy(output.stderr.as_slice()).lines().next().unwrap_or_default().trim().to_string();
        return Err(SaveFailure::io(String::from_utf8_lossy(paths[0].as_slice()).into_owned(), detail));
    }
    let mut copies: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    for path in paths {
        let copy: PathBuf = staged_root.join(OsString::from_vec(path.clone()));
        let placed: std::fs::File = match std::fs::File::open(copy.as_path()) {
            Ok(opened) => opened,
            Err(error) => {
                return Err(SaveFailure::io(
                    String::from_utf8_lossy(path.as_slice()).into_owned(),
                    format!("Git wrote no copy of the content the index holds ({error})"),
                ));
            }
        };
        // The copy is durable before the command that resets the index runs.
        if let Err(error) = placed.sync_all() {
            return Err(SaveFailure::io(copy.display().to_string(), error.to_string()));
        }
        copies.push(SavedCopy {
            path: format!("{} ({STAGED_FOLDER})", String::from_utf8_lossy(path.as_slice())),
            saved_to: Some(copy),
        });
    }
    if let Err(error) = sync_directory(staged_root.as_path()) {
        return Err(SaveFailure::io(staged_root.display().to_string(), error.to_string()));
    }
    return Ok(copies);
}

/// The saves and the staged content.
#[cfg(test)]
#[path = "abort_save_tests.rs"]
mod tests;
