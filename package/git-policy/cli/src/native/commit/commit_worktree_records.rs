//! What:
//!  The worktree completion records a landed commit owes the worktree:
//! selected final-newline corrections.
//! Why:
//!  The commit keeps what the private index holds,
//! so the real worktree still holds the bytes the user wrote.
//!  After the landing,
//! each recorded file takes the committed bytes only while it still holds its earlier bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const selected = await selectedWorktreeRecords({ gitPath, cwd, repositoryRoot, indexPath, initialCandidates, newlinePaths });
//! ```

use crate::candidate_object::CandidateMode;
use crate::direct_fix_install::InstallChange;
use crate::path_bytes::os_from_path_bytes;
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
use crate::transaction_journal::AddedPath;
use crate::worktree_completion::{WorktreeFileState, inspect_worktree_file};
use std::path::Path;

/// What:
///  The Git mode text a candidate mode carries,
/// when it is an ordinary file.
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
///  The blob hash of some bytes,
/// without writing an object.
/// Why:
///   The record names the original staged blob,
/// which only its bytes determine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitBlobId(bytes)
/// ```
fn blob_id(context: &GitContext, cwd: &Path, bytes: &[u8]) -> Result<String, GitFailure> {
    let mut request: GitRequest = GitRequest::new(cwd, &["hash-object", "--stdin"]);
    request.without_prefix = true;
    request.repository_selection = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    request.input = Some(bytes.to_vec());
    let output: GitOutput = run_git_checked(context, &request)?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  Whether one byte is the NUL byte that ends each record of a `-z` listing.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether one byte is the tab that separates the stage columns from the path in `git ls-files -s`.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTabByte = (byte: number) => byte === '\t'.charCodeAt(0);
/// ```
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  The settled mode and object of one path in the private index,
/// or nothing when it has no stage-zero entry.
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
    // The pathspec is magic, and this start skips the caller's prefix, so every mode the caller set is
    // removed from the child's environment: under literal mode the name would match nothing and a landed
    // correction would never reach the worktree copy, and two glob modes at once make Git refuse the listing
    // and fail the commit.
    // `--full-name` answers with repository-relative names, which is how `path` is written: from a
    // nested working directory `ls-files` would otherwise report `a.txt` for the `sub/a.txt` asked about.
    let mut request: GitRequest = GitRequest::new(cwd, &["ls-files", "--stage", "-z", "--full-name", "--"]).own_pathspecs();
    request.arguments.push(os_from_path_bytes(pathspec.as_slice()));
    request.without_prefix = true;
    // The listing runs in the caller's repository, which a Git directory beside its work tree only names.
    request.repository_selection = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    request.index = Some(index.to_path_buf());
    let output: GitOutput = run_git_checked(context, &request)?;
    for record in output.stdout.split(is_nul_byte) {
        let Some(tab) = record.iter().position(is_tab_byte) else {
            continue;
        };
        // This asks about one path, so a record naming another is not its settled entry: under
        // `GIT_ICASE_PATHSPECS=1` Git lists every entry whose name differs only in case, and the first one is
        // not necessarily the path asked about.
        if record.get(tab + 1..) != Some(path) {
            continue;
        }
        let metadata: String = String::from_utf8_lossy(&record[..tab]).into_owned();
        let fields: Vec<&str> = metadata.split(' ').collect();
        if fields.len() == 3 && fields[2] == "0" {
            return Ok(Some((String::from(fields[0]), String::from(fields[1]))));
        }
    }
    return Ok(None);
}

/// What:
///  The records for selected files that final-newline corrected,
/// only where the worktree copy still equals the
///  original staged bytes.
/// Why:
///   A partially staged edit cannot be replaced by a commit correction;
/// the installer re-checks before replacing.
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
