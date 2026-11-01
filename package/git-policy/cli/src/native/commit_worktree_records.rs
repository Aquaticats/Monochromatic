//! What:
//!  The worktree completion records a landed commit owes the worktree:
//!  selected final-newline corrections, and the paths a `pre-commit` hook re-staged.
//! Why:
//!  The commit keeps what the private index holds, so the real worktree still holds the bytes the user wrote.
//!  After the landing, each recorded file takes the committed bytes only while it still holds its earlier bytes
//!  (`src/policy-engine/commit-transaction-selected-worktree.ts` and `commit-hook-changes.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const selected = await selectedWorktreeRecords({ gitPath, cwd, repositoryRoot, indexPath, initialCandidates, newlinePaths });
//! const changes = await describeHookChanges({ gitPath, cwd, objectDirectory, fromTree, toTree });
//! ```

use super::candidate_object::CandidateMode;
use super::diagnostic_log::debug;
use super::direct_fix_install::InstallChange;
use super::path_bytes::os_from_path_bytes;
use super::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
use super::transaction_journal::AddedPath;
use super::worktree_completion::{WorktreeFileState, inspect_worktree_file};
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "commitWorktreeRecords";

/// The fields of one raw diff record: source mode, destination mode, source object, destination object, status.
const RAW_FIELD_COUNT: usize = 5;

/// What:
///  What a `pre-commit` hook staged beyond the tree it saw.
/// Why:
///   The landing guards these paths against a concurrent staging, and the worktree takes the committed bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HookChanges = { paths: readonly Uint8Array[]; worktreeRecords: readonly AddedPathRecord[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HookChanges {
    /// Every path whose entry differs between the tree the hook saw and the tree it left.
    pub paths: Vec<Vec<u8>>,
    /// Modified ordinary files whose worktree copy receives the committed bytes while it holds the pre-hook bytes.
    pub worktree_records: Vec<AddedPath>,
}

impl HookChanges {
    /// What:
    ///  Join the changes of successive preparations and revalidations.
    /// Why:
    ///   A replay starts again from the prepared commit, so the union of paths and every completion stay.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// combineHookChanges({ earlier, later });
    /// ```
    pub fn combine(earlier: &HookChanges, later: &HookChanges) -> HookChanges {
        let mut paths: Vec<Vec<u8>> = earlier.paths.clone();
        for path in &later.paths {
            if !paths.contains(path) {
                paths.push(path.clone());
            }
        }
        let mut worktree_records: Vec<AddedPath> = earlier.worktree_records.clone();
        worktree_records.extend(later.worktree_records.iter().cloned());
        return HookChanges {
            paths,
            worktree_records,
        };
    }
}

/// What:
///  Parse NUL-terminated `diff-tree -r -z --raw --no-renames --no-abbrev` output into the records of ordinary files.
/// Why:
///   A malformed record is a Git failure, not a silent skip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRawChanges(output);
/// ```
fn parse_hook_changes(output: &[u8]) -> Result<HookChanges, GitFailure> {
    let fields: Vec<&[u8]> = output.split(|byte| { return *byte == 0; }).collect();
    let mut changes: HookChanges = HookChanges::default();
    let mut index: usize = 0;
    while index < fields.len() {
        // The metadata is ASCII; a path is bytes and is never decoded.
        let field: String = String::from_utf8_lossy(fields[index]).into_owned();
        if field.is_empty() {
            index += 1;
            continue;
        }
        let Some(path) = fields.get(index + 1) else {
            return Err(GitFailure(format!("git diff-tree returned a malformed record: {field:?}")));
        };
        let metadata: Vec<&str> = field.trim_start_matches(':').split(' ').collect();
        if metadata.len() != RAW_FIELD_COUNT {
            return Err(GitFailure(format!("git diff-tree returned a malformed record: {field:?}")));
        }
        let (from_mode, to_mode, from_oid, to_oid) = (metadata[0], metadata[1], metadata[2], metadata[3]);
        changes.paths.push(path.to_vec());
        if from_mode == to_mode && (to_mode == "100644" || to_mode == "100755") {
            changes.worktree_records.push(AddedPath {
                path: path.to_vec(),
                git_mode: String::from(to_mode),
                original_oid: String::from(from_oid),
                intended_oid: String::from(to_oid),
            });
        }
        index += 2;
    }
    return Ok(changes);
}

/// What:
///  Describe what a hook changed between the tree it saw and the tree it left.
/// Why:
///   The commit keeps the hook's staged changes, as native Git does; landing reconciles those paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await describeHookChanges({ gitPath, cwd, objectDirectory, fromTree, toTree });
/// ```
pub fn describe_hook_changes(
    context: &GitContext,
    cwd: &Path,
    object_directory: &Path,
    from_tree: &str,
    to_tree: &str,
) -> Result<HookChanges, GitFailure> {
    if from_tree == to_tree {
        return Ok(HookChanges::default());
    }
    let mut request: GitRequest = GitRequest::new(
        cwd,
        &["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", from_tree, to_tree],
    );
    request.without_prefix = true;
    request.object_directory = Some(object_directory.to_path_buf());
    let output: GitOutput = run_git_checked(context, &request)?;
    let changes: HookChanges = parse_hook_changes(output.stdout.as_slice())?;
    debug(
        TAG,
        format!("pre-commit changed {} staged paths; the commit keeps them as native Git does", changes.paths.len()).as_str(),
    );
    return Ok(changes);
}

/// What:
///  The Git mode text a candidate mode carries, when it is an ordinary file.
/// Why:
///   Only regular and executable files take a corrected worktree copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// candidate.mode === 'executable' ? '100755' : '100644'
/// ```
fn ordinary_git_mode(mode: CandidateMode) -> Option<&'static str> {
    return match mode {
        CandidateMode::Regular => Some("100644"),
        CandidateMode::Executable => Some("100755"),
        _ => None,
    };
}

/// What:
///  The blob hash of some bytes, without writing an object.
/// Why:
///   The record names the original staged blob, which only its bytes determine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitBlobId(bytes)
/// ```
fn blob_id(context: &GitContext, cwd: &Path, bytes: &[u8]) -> Result<String, GitFailure> {
    let mut request: GitRequest = GitRequest::new(cwd, &["hash-object", "--stdin"]);
    request.without_prefix = true;
    request.input = Some(bytes.to_vec());
    let output: GitOutput = run_git_checked(context, &request)?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  The settled mode and object of one path in the private index, or nothing when it has no stage-zero entry.
/// Why:
///   A correction counts only while the settled entry still has the ordinary mode of the original.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await loadIndexEntries({ gitPath, cwd, indexPath, paths })).get(path)
/// ```
pub fn settled_entry(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
    path: &[u8],
) -> Result<Option<(String, String)>, GitFailure> {
    let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
    pathspec.extend_from_slice(path);
    let mut request: GitRequest = GitRequest::new(cwd, &["ls-files", "--stage", "-z", "--"]);
    request.arguments.push(os_from_path_bytes(pathspec.as_slice()));
    request.without_prefix = true;
    request.index = Some(index.to_path_buf());
    let output: GitOutput = run_git_checked(context, &request)?;
    for record in output.stdout.split(|byte| { return *byte == 0; }) {
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            continue;
        };
        let metadata: String = String::from_utf8_lossy(&record[..tab]).into_owned();
        let fields: Vec<&str> = metadata.split(' ').collect();
        if fields.len() == 3 && fields[2] == "0" {
            return Ok(Some((String::from(fields[0]), String::from(fields[1]))));
        }
    }
    return Ok(None);
}

/// What:
///  The records for selected files that final-newline corrected, only where the worktree copy still equals the
///  original staged bytes.
/// Why:
///   A partially staged edit cannot be replaced by a commit correction; the installer re-checks before replacing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await selectedWorktreeRecords({ gitPath, cwd, repositoryRoot, indexPath, initialCandidates, newlinePaths });
/// ```
pub fn selected_worktree_records(
    context: &GitContext,
    cwd: &Path,
    repository_root: &Path,
    index: &Path,
    changes: &[InstallChange],
) -> Result<Vec<AddedPath>, GitFailure> {
    let mut records: Vec<AddedPath> = Vec::new();
    for change in changes {
        let Some(git_mode) = ordinary_git_mode(change.mode) else {
            continue;
        };
        let original_oid: String = blob_id(context, cwd, &change.original)?;
        let Some((entry_mode, entry_oid)) = settled_entry(context, cwd, index, change.path.as_slice())? else {
            continue;
        };
        if entry_mode != git_mode || entry_oid == original_oid {
            continue;
        }
        let destination = repository_root.join(path_from_bytes(change.path.as_slice()));
        match inspect_worktree_file(destination.as_path(), git_mode, &change.original, &change.replacement) {
            Ok(WorktreeFileState::Original(_)) => {}
            // A missing copy is reported as a conflict by the inspection, so it is skipped here with the other conflicts.
            Ok(_) => continue,
            Err(error) => {
                return Err(GitFailure(format!(
                    "reading {} failed: {error}",
                    destination.display()
                )));
            }
        }
        records.push(AddedPath {
            path: change.path.clone(),
            git_mode: String::from(git_mode),
            original_oid,
            intended_oid: entry_oid,
        });
    }
    return Ok(records);
}

/// What:
///  A repository-relative path from its bytes.
/// Why:
///   The worktree path is joined from the exact bytes Git reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(repositoryRoot, path)
/// ```
fn path_from_bytes(bytes: &[u8]) -> std::path::PathBuf {
    return std::path::PathBuf::from(os_from_path_bytes(bytes));
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_worktree_records_tests.rs"]
mod tests;
