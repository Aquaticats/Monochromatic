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
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_sha::{sha1, sha256};
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
use crate::transaction_journal::AddedPath;
use crate::worktree_completion::{WorktreeFileState, inspect_worktree_file};
use std::collections::BTreeMap;
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
/// which only its bytes determine,
/// and a commit that corrected thousands of files asked Git for each hash:
/// the hash is the framed bytes under the repository's object format,
/// so it is computed here in one pass and starts nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oid = hash(`blob ${bytes.length}\0`, bytes); // sha1 or sha256
/// ```
fn blob_id(bytes: &[u8], format: ObjectFormat) -> String {
    let mut framed: Vec<u8> = format!("blob {}\0", bytes.len()).into_bytes();
    framed.extend_from_slice(bytes);
    let digest: Vec<u8> = match format {
        ObjectFormat::Sha1 => sha1(framed.as_slice()).to_vec(),
        ObjectFormat::Sha256 => sha256(framed.as_slice()).to_vec(),
    };
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(digest.as_slice());
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
///  The settled mode and object of each path in the private index,
/// in the order the listing names them,
/// with a path that has no stage-zero entry left out.
/// Why:
///   A correction counts only while the settled entry still has the ordinary mode of the original,
/// and one listing answers every corrected path of a pass,
/// so a commit that corrected thousands of files starts one Git, not one for each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = await loadIndexEntries({ indexPath, paths }); // one ls-files --stage
/// ```
pub fn settled_entries(context: &GitContext, cwd: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, (String, String)>, GitFailure> {
    /// The most paths one listing names: a thousand paths of a quarter kilobyte each stay far under the
    /// argument limit of one start, and a pass of many corrections pays one start per thousand instead of one per path.
    const PATHS_PER_LISTING: usize = 1_024;
    let mut settled: BTreeMap<Vec<u8>, (String, String)> = BTreeMap::new();
    for chunk in paths.chunks(PATHS_PER_LISTING) {
        let mut request: GitRequest = GitRequest::new(cwd, &["ls-files", "--stage", "-z", "--full-name", "--"]).own_pathspecs();
        for path in chunk {
            let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
            pathspec.extend_from_slice(path.as_slice());
            request.arguments.push(os_from_path_bytes(pathspec.as_slice()));
        }
        request.without_prefix = true;
        // The listing runs in the caller's repository, which a Git directory beside its work tree only names.
        request.repository_selection = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
        request.index = Some(index.to_path_buf());
        let output: GitOutput = run_git_checked(context, &request)?;
        for record in output.stdout.split(is_nul_byte) {
            let Some(tab) = record.iter().position(is_tab_byte) else {
                continue;
            };
            let metadata: String = String::from_utf8_lossy(&record[..tab]).into_owned();
            let fields: Vec<&str> = metadata.split(' ').collect();
            if fields.len() == 3 && fields[2] == "0" {
                settled.insert(record[tab + 1..].to_vec(), (String::from(fields[0]), String::from(fields[1])));
            }
        }
    }
    return Ok(settled);
}

/// What:
///  The settled entry of one path,
/// read with one Git start.
/// Why:
///   The record of one correction is written from one path;
/// a control that asks about one path uses this,
/// and the pass itself reads every path in one listing (`settled_entries`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await loadIndexEntries({ indexPath, paths: [path] })).get(path)
/// ```
pub fn settled_entry(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
    path: &[u8],
) -> Result<Option<(String, String)>, GitFailure> {
    let mut settled = settled_entries(context, cwd, index, &[path.to_vec()])?;
    return Ok(settled.remove(path));
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
    format: ObjectFormat,
    changes: &[InstallChange],
) -> Result<Vec<AddedPath>, GitFailure> {
    let wanted: Vec<Vec<u8>> = changes.iter().filter(|change| return ordinary_git_mode(change.mode).is_some()).map(|change| return change.path.clone()).collect();
    let settled: BTreeMap<Vec<u8>, (String, String)> = settled_entries(context, cwd, index, wanted.as_slice())?;
    let mut records: Vec<AddedPath> = Vec::new();
    for change in changes {
        let Some(git_mode) = ordinary_git_mode(change.mode) else {
            continue;
        };
        let original_oid: String = blob_id(change.original.as_ref(), format);
        let Some((entry_mode, entry_oid)) = settled.get(&change.path) else {
            continue;
        };
        if entry_mode.as_str() != git_mode || entry_oid.as_str() == original_oid.as_str() {
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
            intended_oid: entry_oid.clone(),
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
