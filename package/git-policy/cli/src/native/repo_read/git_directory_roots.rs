//! What:
//!  The Git directories a command works in,
//! found without starting Git:
//! the Git directory and the common Git directory.
//! Why:
//!  The sweep of a stopped child's locks and the clearing of a killed Git's reference locks both need to know where a
//!  repository keeps its lock files.
//! Git is not asked,
//! because both run when Git has just failed or is about to be
//!  started,
//! and reading a few small files costs nothing.
//!  The two files are read as plain ones (`crate::plain_file`),
//! and the `commondir` rule is the one reader of
//!  `foundation/common_directory_of_a_git_directory.rs`:
//! the copy that stood here read the file with a call that waits on a named pipe.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const roots = rootsOfCommand(command); // [gitDir] or [gitDir, commonDir]; [] when no repository is found
//! ```

/// The one reader of a Git directory's `commondir` file,
/// which opens it as a plain file so a named pipe there cannot hold the sweep.
use crate::common_directory_of_a_git_directory::common_directory_of;
/// The reader that cannot wait, and the bound a `.git` pointer file is read to.
use crate::plain_file::read_plain_file;
use crate::worktree_convert_inspection::GIT_FILE_LIMIT;
/// Owned OS strings of a command's environment.
use std::ffi::OsStr;
/// Paths of the repository.
use std::path::{
    Path,
    PathBuf,
};
/// The command whose repository is wanted.
use std::process::Command;

/// Where the Git directory of a working directory is:
/// `.git` as a directory,
/// a `gitdir:` file,
/// or a bare repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitDirectoryOf(start: string): string | undefined;
/// ```
pub fn git_directory_of(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        let dot_git: PathBuf = directory.join(".git");
        if let Ok(metadata) = std::fs::symlink_metadata(dot_git.as_path()) {
            if metadata.is_dir() {
                return Some(dot_git);
            }
            if metadata.is_file() {
                // The pointer is read as a plain file, so a name swapped for a pipe after the look above is refused at once
                //  rather than waited on (`git_directory_roots_tests`).
                let bytes: Vec<u8> = read_plain_file(dot_git.as_path(), GIT_FILE_LIMIT).ok()?;
                let text: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
                let named: &str = text.lines().next()?.strip_prefix("gitdir:")?.trim();
                return Some(directory.join(named));
            }
        }
        if directory.join("HEAD").is_file() && directory.join("objects").is_dir() {
            return Some(directory.to_path_buf());
        }
    }
    return None;
}

/// The Git directory and the common Git directory (when it differs) of a working directory,
/// or of `GIT_DIR` when one is
///  named;
/// nothing when no repository is found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsFor(working: string, named?: string): string[];
/// ```
pub fn roots_for(
    working: &Path,
    named: Option<PathBuf>,
) -> Vec<PathBuf> {
    let git_dir: Option<PathBuf> = match named {
        Some(explicit) => Some(explicit),
        None => git_directory_of(working),
    };
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(directory) = git_dir {
        // The one reader of `commondir` answers with the directory itself when the file names none, so both roots come from
        //  the same rule every other caller of this crate uses.
        let common: PathBuf = common_directory_of(directory.as_path());
        roots.push(directory.clone());
        if common != directory {
            roots.push(common);
        }
    }
    return roots;
}

/// The roots of a command that has not started yet:
/// its repository comes from `GIT_DIR` in its environment,
/// or from its
///  working directory (the process's own when the command sets none).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsOfCommand(command: Command): string[];
/// ```
pub fn roots_of_command(command: &Command) -> Vec<PathBuf> {
    let working: PathBuf = match command.get_current_dir() {
        Some(directory) => directory.to_path_buf(),
        None => std::env::current_dir().unwrap_or_default(),
    };
    // The last `GIT_DIR` the command sets wins; a removal of it (no value) names nothing.
    let mut git_dir_value: Option<Option<&OsStr>> = None;
    for (name, value) in command.get_envs() {
        if name == OsStr::new("GIT_DIR") {
            git_dir_value = Some(value);
        }
    }
    let mut named: Option<PathBuf> = None;
    if let Some(Some(path)) = git_dir_value {
        named = Some(working.join(path));
    }
    return roots_for(working.as_path(), named);
}

/// Controls for the two files these roots are read from stay out of the release executable.
#[cfg(test)]
#[path = "git_directory_roots_tests.rs"]
mod tests;
