//! What:
//! The worktrees of the repository,
//! as `git worktree list --porcelain` reports them,
//! and the lock file of each.
//! Why:
//! The cleanup of unnamed versions reads each worktree's lock,
//! and the cleanup of old copies reads each worktree's link directories;
//! both ask git the same question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // worktrees.ts
//! export function listedWorktrees(runner: Runner, root: string): { directory: string; prunable: boolean }[]  // throws
//! export function worktreeLockFiles(runner: Runner, root: string): string[]  // throws
//! ```

/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Instant` from `std::time`.
use std::time::Instant;

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest, Runner};

/// One worktree `git worktree list --porcelain` reports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedWorktree {
    /// The worktree's directory as git prints it.
    pub directory: PathBuf,
    /// Whether git marks it `prunable` (its directory is gone and `git worktree prune` would drop it).
    pub prunable: bool,
}

/// What:
/// Every worktree of the repository at `root`,
/// as `git worktree list --porcelain` reports them.
/// Why:
/// The cleanup of unnamed versions reads each worktree's lock,
/// and the cleanup of old copies reads each worktree's link directories;
/// both ask git the same question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function listedWorktrees(runner: Runner, root: string): { directory: string; prunable: boolean }[]  // throws
/// ```
pub fn listed_worktrees(runner: &dyn Runner, root: &Path, deadline: Option<Instant>) -> Result<Vec<ListedWorktree>, ToolchainError> {
    let mut request: RunRequest = RunRequest::new("git", &["-C", &root.to_string_lossy(), "worktree", "list", "--porcelain"]);
    // What: a caller with a work limit hands it here, so a Git that never ends is ended with the pass.
    // Why: this program is the one the cleanup starts, and it held an install for good without a bound.
    if let Some(moment) = deadline {
        request = request.with_deadline(moment);
    }
    let outcome: RunOutcome = runner.run(&request);
    if !outcome.succeeded() {
        return Err(ToolchainError::failure(format!(
            "cannot list the repository's worktrees, so the locks and link directories of the other checkouts cannot be found: {}",
            outcome.failure_text()
        )));
    }
    let mut listed: Vec<ListedWorktree> = Vec::new();
    for block in outcome.stdout.split("\n\n") {
        let mut path: Option<&str> = None;
        let mut prunable: bool = false;
        for line in block.lines() {
            if let Some(found) = line.strip_prefix("worktree ") {
                path = Some(found);
            } else if line == "prunable" || line.starts_with("prunable ") {
                prunable = true;
            }
        }
        if let Some(directory) = path {
            listed.push(ListedWorktree { directory: PathBuf::from(directory), prunable });
        }
    }
    return Ok(listed);
}

/// The worktree lock files `git worktree list --porcelain` reports for the repository at `root`.
/// A worktree whose directory is gone is skipped only when git marks it `prunable`;
/// otherwise (an unmounted disk,
/// say) the command refuses.
pub fn worktree_lock_files(runner: &dyn Runner, root: &Path, deadline: Option<Instant>) -> Result<Vec<PathBuf>, ToolchainError> {
    let listed: Vec<ListedWorktree> = listed_worktrees(runner, root, deadline)?;
    let mut files: Vec<PathBuf> = Vec::new();
    for worktree in listed {
        if !worktree.directory.is_dir() {
            if worktree.prunable {
                continue;
            }
            return Err(ToolchainError::failure(format!(
                "the worktree {} is listed by git but its directory is missing and not marked prunable, \
                 so its lock cannot be read; nothing was deleted. Mount it, run `git worktree prune` if it is gone, \
                 or name its lock with --lock",
                worktree.directory.display()
            )));
        }
        files.push(worktree.directory.join("toolchain.lock.json"));
    }
    return Ok(files);
}
