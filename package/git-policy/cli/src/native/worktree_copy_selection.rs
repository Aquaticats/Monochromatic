//! What:
//!  Which source paths the linked-worktree copy selects:
//!  the ignored roots Git reports under the standard exclusion stack,
//!  the ignored special entries (sockets, FIFOs, devices) Git's listing omits,
//!  and the validation every repository path passes before it touches the filesystem.
//! Why:
//!  Git decides what is ignored; the copy only adds the entries Git's index-oriented listing
//!  cannot report, and it never trusts a path that could leave the worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const roots = await readIgnoredRoots({ sourceRoot, gitPath, excludedRoots });
//! ```

use super::transaction_git::{GitContext, GitRequest, run_git};
use super::worktree_copy_error::{WorktreeCopyError, raw_failure};
use crate::json_record::quote;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// What:
///  The Git directory marker that `ls-files --directory` appends to ignored directories.
/// Why:
///   The copy works on the directory itself, so the marker is removed before validation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DIRECTORY_SUFFIX = '/';
/// ```
const DIRECTORY_SUFFIX: char = '/';

/// What:
///  The private stage prefix the copy reserves; no selected path may contain a component that starts with it.
/// Why:
///   A selected path that is a private stage would copy the copy into itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const STAGE_PREFIX = '.cli-git-worktree-copy-';
/// ```
pub const STAGE_PREFIX: &str = ".cli-git-worktree-copy-";

/// What:
///  Whether a repository path equals or descends from another repository path.
/// Why:
///   Comparison is by whole components, so `cache2` is not inside `cache`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (candidate === parent) || candidate.startsWith(`${parent}/`)
/// ```
pub fn repository_path_within(candidate: &str, parent: &str) -> bool {
    // `||` keeps the exact-equality case and the component-prefix case in one expression.
    return candidate == parent || candidate.starts_with(format!("{parent}/").as_str());
}

/// What:
///  Validates one slash-separated path that Git reported before it is joined to a filesystem root.
/// Why:
///   An empty, absolute, `.` or `..` component could select a path outside the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertSafeRepositoryPath(repositoryPath: string): void;
/// ```
pub fn assert_safe_repository_path(repository_path: &str) -> Result<(), WorktreeCopyError> {
    let unsafe_path: bool = repository_path.is_empty()
        || repository_path.starts_with('/')
        || Path::new(repository_path).is_absolute()
        || repository_path
            .split('/')
            .any(|component| return component.is_empty() || component == "." || component == "..");
    if unsafe_path {
        return Err(WorktreeCopyError(format!(
            "cli-git: Git returned unsafe ignored path {}.",
            quote(repository_path)
        )));
    }
    return Ok(());
}

/// What:
///  The native path of one validated repository path under a filesystem root.
/// Why:
///   Every filesystem access of the copy goes through this join, so the validation cannot be skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(root, ...repositoryPath.split('/'));
/// ```
pub fn filesystem_path(root: &Path, repository_path: &str) -> Result<PathBuf, WorktreeCopyError> {
    assert_safe_repository_path(repository_path)?;
    let mut path: PathBuf = root.to_path_buf();
    for component in repository_path.split('/') {
        path.push(component);
    }
    return Ok(path);
}

/// What:
///  Sorts strings the way JavaScript's default `toSorted` does: by UTF-16 code units.
/// Why:
///   Manifest and journal order must match the incumbent's, which differs from byte order
///  only for supplementary-plane characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...values].toSorted()
/// ```
pub fn js_order(left: &str, right: &str) -> Ordering {
    return left.encode_utf16().cmp(right.encode_utf16());
}

/// What:
///  Keeps the outermost of several ignored roots, dropping every root under another one.
/// Why:
///   Copying a parent already copies its descendants, so a second copy would duplicate work and entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collapseIgnoredRoots(paths: readonly string[]): readonly string[];
/// ```
fn collapse_ignored_roots(paths: &[String]) -> Vec<String> {
    let mut candidates: Vec<String> = paths.to_vec();
    candidates.sort_by(|left, right| return js_order(left, right));
    candidates.dedup();
    let mut roots: Vec<String> = Vec::new();
    for candidate in candidates {
        let owned: bool = roots
            .iter()
            .any(|root| return repository_path_within(candidate.as_str(), root.as_str()));
        if !owned {
            roots.push(candidate);
        }
    }
    return roots;
}

/// What:
///  Whether Git's standard exclusion stack ignores one special entry.
/// Why:
///   `check-ignore --no-index` answers the same question for a path Git's listing never names;
///  exit status 1 is "not ignored" and every other failure is fatal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runMetadataGit({ args: ['-C', sourceRoot, 'check-ignore', '--quiet', '--no-index', '--', repositoryPath] })
/// ```
fn is_ignored_special_path(
    context: &GitContext,
    source_root: &Path,
    repository_path: &str,
) -> Result<bool, WorktreeCopyError> {
    let request: GitRequest = GitRequest {
        cwd: source_root.to_path_buf(),
        arguments: vec![
            "check-ignore".into(),
            "--quiet".into(),
            "--no-index".into(),
            "--".into(),
            repository_path.into(),
        ],
        without_prefix: true,
        ..GitRequest::default()
    };
    let output = run_git(context, &request).map_err(|error| return raw_failure(&error))?;
    // `match` on the exit code: 0 is ignored, 1 is not ignored, anything else is a failure.
    match output.code {
        Some(0) => return Ok(true),
        Some(1) => return Ok(false),
        _ => {
            return Err(raw_failure(&String::from_utf8_lossy(
                output.stderr.as_slice(),
            )));
        }
    }
}

/// What:
///  Whether a native path lies at or under one of the excluded absolute directories.
/// Why:
///   Walks skip registered worktrees and the private stage, so they never copy themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isFilesystemPathWithin({ candidate, parent })
/// ```
fn is_excluded(candidate: &Path, excluded_roots: &[PathBuf]) -> bool {
    return excluded_roots
        .iter()
        .any(|root| return candidate.starts_with(root.as_path()));
}

/// What:
///  One pending directory of the special-entry walk, with its repository path.
/// Why:
///   The walk is an explicit stack so depth cannot exhaust the call stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pending: string[] = ['.'];
/// ```
fn special_entries(
    context: &GitContext,
    source_root: &Path,
    selected_roots: &[String],
    excluded_roots: &[PathBuf],
) -> Result<Vec<String>, WorktreeCopyError> {
    let mut pending: Vec<String> = vec![String::from(".")];
    let mut special_paths: Vec<String> = Vec::new();
    while let Some(current) = pending.pop() {
        let directory_path: PathBuf = if current == "." {
            source_root.to_path_buf()
        } else {
            filesystem_path(source_root, current.as_str())?
        };
        let listing = std::fs::read_dir(directory_path.as_path())
            .map_err(|error| return raw_failure(&error))?;
        for listed in listing {
            let entry: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
            let name: String = entry.file_name().to_string_lossy().into_owned();
            let repository_path: String = if current == "." {
                name
            } else {
                format!("{current}/{name}")
            };
            if repository_path == ".git" {
                continue;
            }
            let source_path: PathBuf = filesystem_path(source_root, repository_path.as_str())?;
            if is_excluded(source_path.as_path(), excluded_roots) {
                continue;
            }
            let selected_ancestor: bool = selected_roots
                .iter()
                .any(|root| return repository_path_within(repository_path.as_str(), root.as_str()));
            if selected_ancestor {
                continue;
            }
            let file_type: std::fs::FileType = entry
                .file_type()
                .map_err(|error| return raw_failure(&error))?;
            // A directory is walked; a file or symbolic link is already selected by Git's listing;
            // anything else is a special entry that needs Git's own ignore answer.
            if file_type.is_dir() {
                pending.push(repository_path);
            } else if !file_type.is_file() && !file_type.is_symlink() {
                special_paths.push(repository_path);
            }
        }
    }
    let mut ignored: Vec<String> = Vec::new();
    for repository_path in special_paths {
        if is_ignored_special_path(context, source_root, repository_path.as_str())? {
            ignored.push(repository_path);
        }
    }
    return Ok(ignored);
}

/// What:
///  Every existing source path ignored by Git's standard exclusion stack,
///  as collapsed repository paths.
/// Why:
///   This is the one selection the copy stages, so the same validation and collapse apply everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readIgnoredRoots({ sourceRoot, gitPath, excludedRoots }): Promise<readonly string[]>;
/// ```
pub fn read_ignored_roots(
    context: &GitContext,
    source_root: &Path,
    excluded_roots: &[PathBuf],
) -> Result<Vec<String>, WorktreeCopyError> {
    let request: GitRequest = GitRequest {
        cwd: source_root.to_path_buf(),
        arguments: vec![
            "ls-files".into(),
            "--others".into(),
            "--ignored".into(),
            "--exclude-standard".into(),
            "--directory".into(),
            "-z".into(),
        ],
        without_prefix: true,
        ..GitRequest::default()
    };
    let output = run_git(context, &request).map_err(|error| return raw_failure(&error))?;
    if output.code != Some(0) {
        return Err(raw_failure(&String::from_utf8_lossy(
            output.stderr.as_slice(),
        )));
    }
    let mut paths: Vec<String> = Vec::new();
    for field in output.stdout.split(|byte| return *byte == 0) {
        if field.is_empty() {
            continue;
        }
        let text: String = String::from_utf8_lossy(field).into_owned();
        // `strip_suffix` removes the directory marker only when present, as the incumbent does.
        let path: String = match text.strip_suffix(DIRECTORY_SUFFIX) {
            Some(stripped) => String::from(stripped),
            None => text,
        };
        assert_safe_repository_path(path.as_str())?;
        paths.push(path);
    }
    let selected: Vec<String> = paths.clone();
    let special: Vec<String> =
        special_entries(context, source_root, selected.as_slice(), excluded_roots)?;
    let mut all: Vec<String> = paths;
    all.extend(special);
    return Ok(collapse_ignored_roots(all.as_slice()));
}

/// What:
///  Which of a source's ignored paths one copy takes:
///  every ignored root, or only the named repository paths.
/// Why:
///   A linked or bare source hands over all of its ignored state, as the incumbent does;
///  a main checkout holds the repository's heavy ignored trees, so it hands over only what the caller names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopySelection = { kind: 'ignored' } | { kind: 'paths'; paths: readonly string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CopySelection {
    /// Every path Git's standard exclusion stack ignores, plus the ignored special entries.
    Ignored,
    /// Only these repository paths, and only the ones Git ignores and the source has.
    Paths(Vec<String>),
}

/// What:
///  The named repository paths that exist in the source and that Git's standard exclusion stack ignores.
/// Why:
///   A configured file such as the forbidden-strings rules is copied by name, so the source is never enumerated;
///  a name Git does not list exactly (absent, tracked, not ignored, or a directory) selects nothing,
///  which leaves the destination without it instead of inventing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readNamedIgnoredRoots({ sourceRoot, paths, gitPath }): Promise<readonly string[]>;
/// ```
pub fn read_named_ignored_roots(
    context: &GitContext,
    source_root: &Path,
    paths: &[String],
) -> Result<Vec<String>, WorktreeCopyError> {
    let mut selected: Vec<String> = Vec::new();
    for path in paths {
        assert_safe_repository_path(path.as_str())?;
        let request: GitRequest = GitRequest {
            cwd: source_root.to_path_buf(),
            arguments: vec![
                "ls-files".into(),
                "--others".into(),
                "--ignored".into(),
                "--exclude-standard".into(),
                "-z".into(),
                "--".into(),
                // `top` anchors the name at the worktree root and `literal` turns off globbing and magic.
                format!(":(top,literal){path}").into(),
            ],
            without_prefix: true,
            ..GitRequest::default()
        };
        let output = run_git(context, &request).map_err(|error| return raw_failure(&error))?;
        if output.code != Some(0) {
            return Err(raw_failure(&String::from_utf8_lossy(
                output.stderr.as_slice(),
            )));
        }
        // Git lists a directory's files rather than the directory, so only an exact name is a file Git ignores.
        let listed: bool = output
            .stdout
            .split(|byte| return *byte == 0)
            .any(|field| return field == path.as_bytes());
        if listed {
            selected.push(path.clone());
        }
    }
    return Ok(collapse_ignored_roots(selected.as_slice()));
}

/// What:
///  The collapsed repository paths one copy takes from a source under the chosen selection.
/// Why:
///   Staging asks one question whichever source it copies from, so the selection mode is decided in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const roots = selection.kind === 'ignored' ? await readIgnoredRoots(...) : await readNamedIgnoredRoots(...);
/// ```
pub fn select_roots(
    context: &GitContext,
    source_root: &Path,
    excluded_roots: &[PathBuf],
    selection: &CopySelection,
) -> Result<Vec<String>, WorktreeCopyError> {
    // `match` on the selection: the full listing for linked and bare sources, the named lookup otherwise.
    match selection {
        CopySelection::Ignored => return read_ignored_roots(context, source_root, excluded_roots),
        CopySelection::Paths(paths) => {
            return read_named_ignored_roots(context, source_root, paths.as_slice());
        }
    }
}

/// Selection controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_selection_tests.rs"]
mod tests;
