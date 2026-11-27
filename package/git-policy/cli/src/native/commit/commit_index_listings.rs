//! What:
//!  What the private index says:
//!  the intended tree written from it,
//!  the paths with unmerged stages,
//!  and the paths it changes against the recorded base.
//! Why:
//!  The tree is the exact content the commit will carry,
//!  a conflicted path cannot be committed by automatic fixes so preparation refuses it,
//!  and the changed paths are read against the base at invocation,
//!  so a landing that moved `HEAD` since then changes nothing here.
//!  Each listing is the records of a `-z` listing split on the NUL byte Git ends one with,
//!  because a pathname is a byte string and not text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = await writePrivateTree({ gitPath, cwd, indexPath });
//! const unmerged = await listUnmergedIndexPaths({ gitPath, cwd, indexPath });
//! ```

/// The one Git start of a private-index step.
use crate::commit_index_private_git::private_git;
/// One required Git start,
/// its request builder and the failure text.
use crate::private_index_request::indexed_request;
/// The transaction Git wrapper:
///  the context a step runs in,
///  the failure it reports,
///  the output it prints and the start that requires success.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
    run_git_checked,
};
/// `Path` is a borrowed filesystem path.
use std::path::Path;
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
///  Whether one byte is the tab that separates the stage columns from the path in `git ls-files -u`.
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
///  Writes the intended tree from the private index into the shadow object store.
/// Why:
///   The tree is the exact content the commit will carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePrivateTree({ workspace, gitPath, cwd });
/// ```
pub fn write_private_tree(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
    object_directory: &Path,
) -> Result<String, GitFailure> {
    let output: GitOutput = private_git(
        context,
        cwd,
        commit_index,
        object_directory,
        &[String::from("write-tree")],
    )?;
    let text: &str = match std::str::from_utf8(output.stdout.as_slice()) {
        Ok(decoded) => decoded,
        Err(_) => return Err(GitFailure(String::from("Git returned non-UTF-8 output."))),
    };
    let oid: &str = text.trim();
    if oid.is_empty() {
        return Err(GitFailure(String::from(
            "Git returned empty private tree identity.",
        )));
    }
    return Ok(String::from(oid));
}

/// What:
///  Lists the repository paths with unmerged stages,
/// once each,
/// in first-seen order.
/// Why:
///   A conflicted path cannot be committed by automatic fixes,
/// so preparation refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listUnmergedIndexPaths({ gitPath, cwd, indexPath });
/// ```
pub fn list_unmerged_index_paths(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    // `--full-name`: the names are handed to `:(top,literal)<path>` specs and to the person, and a commit run
    // from a subdirectory would otherwise be answered with names read from that directory.
    let output: GitOutput = run_git_checked(
        context,
        &indexed_request(
            context,
            cwd,
            &["ls-files", "--unmerged", "-z", "--full-name"],
            Some(commit_index),
            None,
        ),
    )?;
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for record in output.stdout.split(is_nul_byte) {
        let Some(tab) = record.iter().position(is_tab_byte) else {
            continue;
        };
        let path: Vec<u8> = record[tab + 1..].to_vec();
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    return Ok(paths);
}

/// What:
///  Lists the paths the private index changes against the recorded base.
/// Why:
///   The base is the recorded one,
/// so a landing that moved `HEAD` since invocation changes nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listChangedIndexPaths({ gitPath, cwd, indexPath, baseRevision, objectDirectory });
/// ```
pub fn list_changed_index_paths(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
    base_revision: &str,
    object_directory: &Path,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    let output: GitOutput = run_git_checked(
        context,
        &indexed_request(
            context,
            cwd,
            &["diff", "--cached", "--name-only", "-z", base_revision],
            Some(commit_index),
            Some(object_directory),
        ),
    )?;
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for path in output.stdout.split(is_nul_byte) {
        if !path.is_empty() {
            paths.push(path.to_vec());
        }
    }
    return Ok(paths);
}
