//! What:
//!  The Git directories a command works in, found without starting Git: the Git directory and the common Git directory.
//! Why:
//!  The sweep of a stopped child's locks and the clearing of a killed Git's reference locks both need to know where a
//!  repository keeps its lock files. Git is not asked, because both run when Git has just failed or is about to be
//!  started, and reading a few small files costs nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const roots = rootsOfCommand(command); // [gitDir] or [gitDir, commonDir]; [] when no repository is found
//! ```

/// Owned OS strings of a command's environment.
use std::ffi::OsStr;
/// Paths of the repository.
use std::path::{Path, PathBuf};
/// The command whose repository is wanted.
use std::process::Command;

/// The common Git directory a linked worktree's Git directory names in its `commondir` file, or the directory itself.
///  The few lines are repeated here because the module that reads it belongs to the service run, which the client path
///  may not name (`standards_review_boundary_tests.rs`); a path that cannot be resolved is used as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync(join(gitDir, 'commondir')) ? realpath(resolve(gitDir, readFile(...).trimEnd())) : gitDir
/// ```
pub fn common_dir_for(git_dir: &Path) -> PathBuf {
    let Ok(text) = std::fs::read_to_string(git_dir.join("commondir")) else {
        return git_dir.to_path_buf();
    };
    let named: &str = text.trim_end_matches(['\r', '\n']);
    if named.is_empty() {
        return git_dir.to_path_buf();
    }
    let joined: PathBuf = git_dir.join(named);
    return std::fs::canonicalize(&joined).unwrap_or(joined);
}

/// Where the Git directory of a working directory is: `.git` as a directory, a `gitdir:` file, or a bare repository.
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
                let text: String = std::fs::read_to_string(dot_git.as_path()).ok()?;
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

/// The Git directory and the common Git directory (when it differs) of a working directory, or of `GIT_DIR` when one is
///  named; nothing when no repository is found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsFor(working: string, named?: string): string[];
/// ```
pub fn roots_for(working: &Path, named: Option<PathBuf>) -> Vec<PathBuf> {
    let git_dir: Option<PathBuf> = match named {
        Some(explicit) => Some(explicit),
        None => git_directory_of(working),
    };
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(directory) = git_dir {
        let common: PathBuf = common_dir_for(directory.as_path());
        roots.push(directory.clone());
        if common != directory {
            roots.push(common);
        }
    }
    return roots;
}

/// The roots of a command that has not started yet: its repository comes from `GIT_DIR` in its environment, or from its
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
