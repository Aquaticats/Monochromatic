//! What:
//! The checkouts of one repository:
//! the main checkout and every registered worktree that still exists,
//! found from the files of the Git directory alone (no Git is started).
//! Why:
//! The tools that decide what is still wanted (a toolchain a lock pins,
//! a worktree whose build tree must stay) need every checkout,
//! and starting Git for it would add a process to a tool that must stay cheap.
//! A worktree's `.git` file names its own directory under `<common>/worktrees/`,
//! which holds `commondir`;
//! each registered worktree has `<common>/worktrees/<name>/gitdir` naming its `.git` file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function listCheckouts(start: string): string[]
//! ```

/// Files and paths.
use std::{fs, path::{Path, PathBuf}};

/// What:
/// `pub fn common_git_directory(checkout: &Path) -> Option<PathBuf>` is the Git directory all checkouts of the repository share.
/// Why:
/// The main checkout's `.git` directory holds the list of worktrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function commonGitDirectory(checkout: string): string | undefined
/// ```
pub fn common_git_directory(checkout: &Path) -> Option<PathBuf> {
    let dot_git: PathBuf = checkout.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let text: String = fs::read_to_string(&dot_git).ok()?;
    let own: PathBuf = PathBuf::from(text.trim().strip_prefix("gitdir:")?.trim());
    let own_absolute: PathBuf = if own.is_absolute() { own } else { checkout.join(own) };
    let common_text: String = fs::read_to_string(own_absolute.join("commondir")).ok()?;
    let common: PathBuf = PathBuf::from(common_text.trim());
    let joined: PathBuf = if common.is_absolute() { common } else { own_absolute.join(common) };
    return fs::canonicalize(joined).ok();
}

/// What:
/// `pub fn list_checkouts(start: &Path) -> Vec<PathBuf>` lists the main checkout and every registered worktree that exists.
/// Why:
/// A registered worktree whose directory is gone is not a checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function listCheckouts(start: string): string[]
/// ```
pub fn list_checkouts(start: &Path) -> Vec<PathBuf> {
    let mut checkouts: Vec<PathBuf> = Vec::new();
    let Some(common) = common_git_directory(start) else { return checkouts };
    if let Some(main) = common.parent()
        && main.exists()
    {
        checkouts.push(main.to_path_buf());
    }
    let Ok(entries) = fs::read_dir(common.join("worktrees")) else { return checkouts };
    for entry in entries.flatten() {
        let Ok(text) = fs::read_to_string(entry.path().join("gitdir")) else { continue };
        let dot_git: PathBuf = PathBuf::from(text.trim());
        if let Some(worktree) = dot_git.parent()
            && worktree.exists()
        {
            checkouts.push(worktree.to_path_buf());
        }
    }
    checkouts.sort();
    return checkouts;
}
