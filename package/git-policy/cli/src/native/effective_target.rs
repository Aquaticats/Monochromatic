//! What:
//!  Classify the worktree an invocation targets for worktree-enforcing policies.
//! Why:
//!  Linked-worktree safeguards discipline a person's repositories,
//!  not the
//!      disposable clones a tool keeps in its own cache.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // classifyEffectiveTarget(identity, allowedDirs) === 'linked-worktree'
//! ```

/// Import getenv-style lookup and the resolved repository identity.
use super::child_environment::environment_value;
use super::worktree_identity::WorktreeIdentity;
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Cache directories come from environment values that need not be UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.env values, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// What:
///  The four targets worktree policies distinguish.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   A bare repository or no repository has no worktree to protect;
///  an
///       allowlisted tool cache is exempt;
///  main versus linked decides enforcement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EffectiveTarget = 'outside-worktree' | 'main-worktree' | 'linked-worktree' | 'allowlisted';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveTarget {
    /// No worktree:
    ///  outside any repository,
    ///  or a bare repository.
    OutsideWorktree,
    /// The repository's main worktree.
    MainWorktree,
    /// A linked worktree.
    LinkedWorktree,
    /// A repository under a tool-cache directory exempt from enforcement.
    Allowlisted,
}

/// What:
///  Resolve uv's cache directory the way uv does:
///  `UV_CACHE_DIR`,
///  then
///       `XDG_CACHE_HOME/uv`,
///  then `HOME/.cache/uv`;
///  an empty value counts as unset.
///       `Option<&Path>` is "a borrowed home directory or nothing";
///       `Option<PathBuf>` is "an owned path or nothing".
/// Why:
///   uv runs destructive Git inside throwaway clones it owns there.
///  The home
///       directory is injected so tests never read the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uvCacheDir(environment: [string, string][], home?: string): string | undefined;
/// ```
pub fn uv_cache_dir(environment: &[(OsString, OsString)], home: Option<&Path>) -> Option<PathBuf> {
    // `if let Some(x) = ... && cond` runs only when the variable is set and the second
    // condition also holds, like `x !== undefined && x !== ""`.
    if let Some(explicit) = environment_value(environment, "UV_CACHE_DIR")
        && !explicit.is_empty()
    {
        // `Some(...)` is the "present" variant.
        return Some(PathBuf::from(explicit));
    }
    if let Some(cache_home) = environment_value(environment, "XDG_CACHE_HOME")
        && !cache_home.is_empty()
    {
        // `.join` appends one path segment and returns an owned path.
        return Some(PathBuf::from(cache_home).join("uv"));
    }
    // What: `let Some(x) = ... else { return None; };` unwraps the home or exits.
    // Why:  Without any hint and without a home directory there is no cache to exempt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (home === undefined) return undefined;
    // ```
    let Some(home_directory) = home else {
        // `None` is the "absent" variant.
        return None;
    };
    return Some(home_directory.join(".cache").join("uv"));
}

/// What:
///  Resolve mise's data directory the way mise does:
///  `MISE_DATA_DIR`,
///  then
///       `XDG_DATA_HOME/mise`,
///  then `HOME/.local/share/mise`;
///  an empty value counts as unset.
///       `Option<&Path>` is "a borrowed home directory or nothing";
///       `Option<PathBuf>` is "an owned path or nothing".
/// Why:
///   mise keeps each tool plugin as a Git clone under `plugins/` there and updates it with a
///       forced checkout, which the main-worktree rule would reject.
///  The home directory is injected so
///       tests never read the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function miseDataDir(environment: [string, string][], home?: string): string | undefined;
/// ```
pub fn mise_data_dir(environment: &[(OsString, OsString)], home: Option<&Path>) -> Option<PathBuf> {
    // `if let Some(x) = ... && cond` runs only when the variable is set and the second
    // condition also holds, like `x !== undefined && x !== ""`.
    if let Some(explicit) = environment_value(environment, "MISE_DATA_DIR")
        && !explicit.is_empty()
    {
        // `Some(...)` is the "present" variant.
        return Some(PathBuf::from(explicit));
    }
    if let Some(data_home) = environment_value(environment, "XDG_DATA_HOME")
        && !data_home.is_empty()
    {
        // `.join` appends one path segment and returns an owned path.
        return Some(PathBuf::from(data_home).join("mise"));
    }
    // What: `let Some(x) = ... else { return None; };` unwraps the home or exits.
    // Why:  Without any hint and without a home directory there is no data directory to exempt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (home === undefined) return undefined;
    // ```
    let Some(home_directory) = home else {
        // `None` is the "absent" variant.
        return None;
    };
    return Some(home_directory.join(".local").join("share").join("mise"));
}

/// What:
///  The tool directories exempt from worktree enforcement on this machine.
///       `Vec<PathBuf>` is an owned list of owned paths.
/// Why:
///   The set is a property of installed tooling,
///  not of any repository,
///  so it is
///       compiled in rather than configured.
///  Each entry is a directory whose Git clones a tool
///       creates and discards itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function defaultAllowedWorktreeDirs(environment, home): string[];
/// ```
pub fn default_allowed_worktree_dirs(
    environment: &[(OsString, OsString)],
    home: Option<&Path>,
) -> Vec<PathBuf> {
    // `Vec::<PathBuf>::new()` is an empty owned list; `mut` allows pushing.
    let mut directories: Vec<PathBuf> = Vec::<PathBuf>::new();
    if let Some(uv) = uv_cache_dir(environment, home) {
        directories.push(uv);
    }
    if let Some(mise) = mise_data_dir(environment, home) {
        directories.push(mise);
    }
    return directories;
}

/// What:
///  Report whether the canonical `candidate` lies inside any allowed directory.
/// Why:
///   Each allowed directory is resolved through symbolic links first,
///  so a home
///       reached through a link still matches;
///  a directory that does not exist drops
///       out.
///  `Path::starts_with` compares whole segments,
///  so `/a/b` does not contain
///       `/a/bc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function isAllowedWorktreeDir(candidate: string, allowedDirs: string[]): Promise<boolean>;
/// ```
pub fn is_allowed_worktree_dir(candidate: &Path, allowed_dirs: &[PathBuf]) -> bool {
    // `for directory in allowed_dirs` borrows each entry in turn.
    for directory in allowed_dirs {
        // What: `if let Ok(resolved) = ... && cond` runs only when the directory could be
        //       resolved and the candidate lies inside it.
        // Why:  A tool that is not installed has no cache directory to compare with.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let resolved; try { resolved = await realpath(directory); } catch { continue; }
        // ```
        if let Ok(resolved) = std::fs::canonicalize(directory)
            && candidate.starts_with(resolved.as_path())
        {
            return true;
        }
    }
    return false;
}

/// What:
///  Map a repository identity to the target worktree policies act on.
/// Why:
///   Bare and absent repositories have no worktree;
///  a repository whose Git
///       directory sits under an allowed tool cache is exempt before main/linked matters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyEffectiveTarget(identity: WorktreeIdentity, allowedDirs: string[]): EffectiveTarget;
/// ```
pub fn classify_effective_target(
    identity: &WorktreeIdentity,
    allowed_dirs: &[PathBuf],
) -> EffectiveTarget {
    // What: `match` on the borrowed identity; `{ git_dir, .. }` binds one field and
    //       ignores the rest.
    // Why:  Each shape maps to exactly one target, and the compiler checks none is missed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (identity.kind) { case 'outside-worktree': return 'outside-worktree'; ... }
    // ```
    match identity {
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => {
            return EffectiveTarget::OutsideWorktree;
        }
        WorktreeIdentity::MainWorktree { git_dir, .. } => {
            if is_allowed_worktree_dir(git_dir.as_path(), allowed_dirs) {
                return EffectiveTarget::Allowlisted;
            }
            return EffectiveTarget::MainWorktree;
        }
        WorktreeIdentity::LinkedWorktree { git_dir, .. } => {
            if is_allowed_worktree_dir(git_dir.as_path(), allowed_dirs) {
                return EffectiveTarget::Allowlisted;
            }
            return EffectiveTarget::LinkedWorktree;
        }
    }
}

/// Disposable-directory controls stay out of the release executable.
#[cfg(test)]
#[path = "effective_target_tests.rs"]
mod tests;
