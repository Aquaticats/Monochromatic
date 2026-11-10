//! What:
//!  Which source paths the linked-worktree copy selects:
//!  the ignored roots Git reports under the standard exclusion stack,
//!  the ignored special entries (sockets,
//! FIFOs,
//! devices) Git's listing omits,
//!  and the validation every repository path passes before it touches the filesystem.
//! Why:
//!  Git decides what is ignored;
//! the copy only adds the entries Git's index-oriented listing
//!  cannot report,
//! and it never trusts a path that could leave the worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const roots = await readIgnoredRoots({ sourceRoot, gitPath, excludedRoots });
//! ```

use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::path_bytes::{os_from_path_bytes, path_bytes_of};
use crate::worktree_copy_prune::{Prune, names_a_private_stage};
use crate::worktree_copy_vanished_entry::went_away;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use std::path::{Path, PathBuf};

/// What:
///  The Git directory marker that `ls-files --directory` appends to ignored directories.
/// Why:
///   The copy works on the directory itself,
/// so the marker is removed before validation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DIRECTORY_SUFFIX = '/';
/// ```
const DIRECTORY_SUFFIX: u8 = b'/';

/// What:
///  The private stage prefix the copy reserves;
/// no selected path may contain a component that starts with it.
/// Why:
///   A selected path that is a private stage would copy the copy into itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const STAGE_PREFIX = '.cli-git-worktree-copy-';
/// ```
pub const STAGE_PREFIX: &str = ".cli-git-worktree-copy-";

/// What:
/// The bytes of one repository path,
/// their join,
/// split and order,
/// and the validation every one passes.
pub use crate::worktree_copy_repository_path::{
    assert_safe_repository_path, by_js_order, by_repository_path_order, filesystem_path, is_separator_byte,
    join_repository_path, js_order, parent_of_repository_path, repository_path_order, repository_path_within,
    shown_repository_path, within_any_root,
};


/// What:
///  Keeps the outermost of several ignored roots,
/// dropping every root under another one.
/// Why:
///   Copying a parent already copies its descendants,
/// so a second copy would duplicate work and entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collapseIgnoredRoots(paths: readonly string[]): readonly string[];
/// ```
fn collapse_ignored_roots(paths: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut candidates: Vec<Vec<u8>> = paths.to_vec();
    candidates.sort_by(by_repository_path_order);
    candidates.dedup();
    let mut roots: Vec<Vec<u8>> = Vec::new();
    for candidate in candidates {
        let owned: bool = within_any_root(candidate.as_slice(), roots.as_slice());
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
/// await runMetadataGit({ args: ['-C', sourceRoot, 'check-ignore', '--no-index', '--', repositoryPath] })
/// ```
fn is_ignored_special_path(
    context: &GitContext,
    source_root: &Path,
    repository_path: &[u8],
) -> Result<bool, WorktreeCopyError> {
    let request: GitRequest = GitRequest {
        cwd: source_root.to_path_buf(),
        arguments: vec![
            "check-ignore".into(),
            "--no-index".into(),
            "--".into(),
            os_from_path_bytes(repository_path),
        ],
        without_prefix: true,
        ..GitRequest::default()
    };
    let output = run_git(context, &request).map_err(raw_failure_of)?;
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
///  Whether a byte is NUL,
/// the end of each name in Git's `-z` output.
/// Why:
///   Git's listings are split on it,
/// and a path may hold any other byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether a NUL-separated listing holds `wanted` as one of its names.
/// Why:
///   Git lists a directory's files rather than the directory,
/// so only an exact name is a file Git ignores.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing.split('\0').includes(wanted)
/// ```
fn lists_exactly(listing: &[u8], wanted: &[u8]) -> bool {
    for field in listing.split(is_nul_byte) {
        if field == wanted {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a native path lies at or under one of the excluded absolute directories.
/// Why:
///   Walks skip registered worktrees and the private stage,
/// so they never copy themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isFilesystemPathWithin({ candidate, parent })
/// ```
fn is_excluded(candidate: &Path, excluded_roots: &[PathBuf]) -> bool {
    for root in excluded_roots {
        if candidate.starts_with(root.as_path()) {
            return true;
        }
    }
    return false;
}

/// What:
///  One pending directory of the special-entry walk,
/// with its repository path.
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
    selected_roots: &[Vec<u8>],
    excluded_roots: &[PathBuf],
) -> Result<Vec<Vec<u8>>, WorktreeCopyError> {
    let mut pending: Vec<Vec<u8>> = vec![Vec::new()];
    let mut special_paths: Vec<Vec<u8>> = Vec::new();
    while let Some(current) = pending.pop() {
        let directory_path: PathBuf = if current.is_empty() {
            source_root.to_path_buf()
        } else {
            filesystem_path(source_root, current.as_slice())?
        };
        // A directory the source removed while this walk ran holds nothing to select, and that is no failure: this walk
        //  visits every directory of the source, so the churn of a live checkout reaches it.
        let listing = match std::fs::read_dir(directory_path.as_path()) {
            Ok(listing) => listing,
            Err(error) if went_away(&error) => continue,
            Err(error) => return Err(raw_failure(&error)),
        };
        for listed in listing {
            let entry: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
            let name: Vec<u8> = path_bytes_of(Path::new(entry.file_name().as_os_str())).to_vec();
            let repository_path: Vec<u8> = if current.is_empty() {
                name
            } else {
                join_repository_path(current.as_slice(), name.as_slice())
            };
            if repository_path == b".git" || names_a_private_stage(repository_path.as_slice()) {
                continue;
            }
            let source_path: PathBuf = filesystem_path(source_root, repository_path.as_slice())?;
            if is_excluded(source_path.as_path(), excluded_roots) {
                continue;
            }
            let selected_ancestor: bool = within_any_root(repository_path.as_slice(), selected_roots);
            if selected_ancestor {
                continue;
            }
            let file_type: std::fs::FileType = match entry.file_type() {
                Ok(file_type) => file_type,
                // The entry went away between the listing and this question, so there is nothing of it to select.
                Err(error) if went_away(&error) => continue,
                Err(error) => return Err(raw_failure(&error)),
            };
            // A directory is walked; a file or symbolic link is already selected by Git's listing;
            // anything else is a special entry that needs Git's own ignore answer.
            if file_type.is_dir() {
                pending.push(repository_path);
            } else if !file_type.is_file() && !file_type.is_symlink() {
                special_paths.push(repository_path);
            }
        }
    }
    let mut ignored: Vec<Vec<u8>> = Vec::new();
    for repository_path in special_paths {
        if is_ignored_special_path(context, source_root, repository_path.as_slice())? {
            ignored.push(repository_path);
        }
    }
    return Ok(ignored);
}

/// What:
///  Every existing source path ignored by Git's standard exclusion stack,
///  as collapsed repository paths.
/// Why:
///   This is the one selection the copy stages,
/// so the same validation and collapse apply everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readIgnoredRoots({ sourceRoot, gitPath, excludedRoots }): Promise<readonly string[]>;
/// ```
pub fn read_ignored_roots(
    context: &GitContext,
    source_root: &Path,
    excluded_roots: &[PathBuf],
) -> Result<Vec<Vec<u8>>, WorktreeCopyError> {
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
    let output = run_git(context, &request).map_err(raw_failure_of)?;
    if output.code != Some(0) {
        return Err(raw_failure(&String::from_utf8_lossy(
            output.stderr.as_slice(),
        )));
    }
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for field in output.stdout.split(is_nul_byte) {
        if field.is_empty() {
            continue;
        }
        // The bytes Git listed are the name; the directory marker is the one byte a selected root drops, and a name
        // that is not text keeps every other byte it has.
        let path: Vec<u8> = match field.last() {
            Some(marker) if *marker == DIRECTORY_SUFFIX => field[..field.len() - 1].to_vec(),
            _ => field.to_vec(),
        };
        assert_safe_repository_path(path.as_slice())?;
        paths.push(path);
    }
    let selected: Vec<Vec<u8>> = paths.clone();
    let special: Vec<Vec<u8>> =
        special_entries(context, source_root, selected.as_slice(), excluded_roots)?;
    let mut all: Vec<Vec<u8>> = paths;
    all.extend(special);
    return Ok(collapse_ignored_roots(all.as_slice()));
}

/// What:
///  Which of a source's ignored paths one copy takes:
///  every ignored root,
/// every ignored root minus what a main checkout leaves out,
/// or only the named repository paths.
/// Why:
///   A linked or bare source hands over all of its ignored state,
/// as the incumbent does;
///  a main checkout hands over the same minus its caches and every other checkout,
/// so a worktree added from it works at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopySelection = { kind: 'ignored' } | { kind: 'main-checkout' } | { kind: 'paths'; paths: readonly string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CopySelection {
    /// Every path Git's standard exclusion stack ignores,
    /// plus the ignored special entries.
    Ignored,
    /// The same listing,
    /// walked under the main checkout's leave-out rule (`worktree_copy_prune.rs`).
    MainCheckout,
    /// Only these repository paths,
    /// and only the ones Git ignores and the source has.
    Paths(Vec<Vec<u8>>),
}

/// What:
///  The named repository paths that exist in the source and that Git's standard exclusion stack ignores.
/// Why:
///   A configured file such as the forbidden-strings rules is copied by name,
/// so the source is never enumerated;
///  a name Git does not list exactly (absent,
/// tracked,
/// not ignored,
/// or a directory) selects nothing,
///  which leaves the destination without it instead of inventing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readNamedIgnoredRoots({ sourceRoot, paths, gitPath }): Promise<readonly string[]>;
/// ```
pub fn read_named_ignored_roots(
    context: &GitContext,
    source_root: &Path,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, WorktreeCopyError> {
    let mut selected: Vec<Vec<u8>> = Vec::new();
    for path in paths {
        assert_safe_repository_path(path.as_slice())?;
        let request: GitRequest = GitRequest {
            cwd: source_root.to_path_buf(),
            arguments: vec![
                // The named spec below is pathspec magic the wrapper writes, so every pathspec mode the caller set
                // is removed from this start: under literal mode the name would match nothing and a configured
                // ignored file would silently not be copied, and two glob modes at once make Git refuse the listing.
                "ls-files".into(),
                "--others".into(),
                "--ignored".into(),
                "--exclude-standard".into(),
                "-z".into(),
                "--".into(),
                // `top` anchors the name at the worktree root and `literal` turns off globbing and magic; the name's own
                // bytes follow the magic, so a name that is not text reaches Git unchanged.
                named_pathspec(path.as_slice()),
            ],
            without_prefix: true,
            ..GitRequest::default()
        }
        .own_pathspecs();
        let output = run_git(context, &request).map_err(raw_failure_of)?;
        if output.code != Some(0) {
            return Err(raw_failure(&String::from_utf8_lossy(
                output.stderr.as_slice(),
            )));
        }
        // Git lists a directory's files rather than the directory, so only an exact name is a file Git ignores.
        let listed: bool = lists_exactly(output.stdout.as_slice(), path.as_slice());
        if listed {
            selected.push(path.clone());
        }
    }
    return Ok(collapse_ignored_roots(selected.as_slice()));
}

/// What:
///  The pathspec that names one repository path exactly,
/// anchored at the worktree root with globbing and magic off.
/// Why:
///   The name is bytes,
/// so it is appended to the magic's bytes and the argument carries the name as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.concat([Buffer.from(':(top,literal)'), path])
/// ```
fn named_pathspec(path: &[u8]) -> std::ffi::OsString {
    let mut spec: Vec<u8> = b":(top,literal)".to_vec();
    spec.extend_from_slice(path);
    return os_from_path_bytes(spec.as_slice());
}

/// What:
///  The collapsed repository paths one copy takes from a source under the chosen selection.
/// Why:
///   Staging asks one question whichever source it copies from,
/// so the selection mode is decided in one place.
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
) -> Result<Vec<Vec<u8>>, WorktreeCopyError> {
    // `match` on the selection: the full listing for linked and bare sources, the named lookup otherwise.
    match selection {
        CopySelection::Ignored | CopySelection::MainCheckout => {
            return read_ignored_roots(context, source_root, excluded_roots);
        }
        CopySelection::Paths(paths) => {
            return read_named_ignored_roots(context, source_root, paths.as_slice());
        }
    }
}

impl CopySelection {
    /// What:
    ///  The leave-out rule the selection's walk applies.
    /// Why:
    ///   The stage and the final source check must walk with the same rule,
    /// so both ask the selection.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// selection.kind === 'main-checkout' ? Prune.mainCheckout() : Prune.never
    /// ```
    pub fn prune(&self) -> Prune {
        return match self {
            CopySelection::MainCheckout => Prune::main_checkout(),
            CopySelection::Ignored | CopySelection::Paths(_) => Prune::Never,
        };
    }
}

/// Selection controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_selection_tests.rs"]
mod tests;
