//! What:
//!  The real index a landing installs, computed against the real index as it is at landing,
//!  never against the invocation-time copy.
//! Why:
//!  A landing that copied the invocation-time index would erase another process's staging
//!  and stage a revert of content that already landed. Index mode takes the exact private index only
//!  when nothing was restaged; otherwise it merges the landed entries path by path, and explicit-path
//!  mode resets only the committed paths and the guarded paths still holding their invocation entry
//!  (`src/policy-engine/commit-landing-index.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await computeLandingPostIndex({ gitPath, cwd, mode, preLandingIndexPath, capturedIndexPath, landedIndexPath, postIndexPath, ... });
//! ```

/// Index copy with the timestamps Git reads.
use super::commit_index::copy_index_with_timestamps;
/// Snapshot comparison: exact bytes, refusing symbolic links.
use super::recovery_files::files_equal;
/// Warning text to the diagnostic log.
use super::diagnostic_log::warn;
use super::landing_invariant::warn_when_entries_moved;
/// Exact path bytes as command-line arguments, and as text for messages.
use super::path_bytes::{os_from_path_bytes, path_message_text};
/// Commit selection mode of the transaction.
use super::transaction_journal::TransactionMode;
/// Running real Git, checked and unchecked.
use super::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git, run_git_checked};
/// Native `HashMap` of stage records and the ordered path list.
use std::collections::HashMap;
/// `OsString` is owned operating-system text; `Path`/`PathBuf` are filesystem paths.
use std::ffi::OsString;
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "computeLandingPostIndex";

/// What:
///  The inputs of one post-index computation.
/// Why:
///   The index artifacts and the landed tree travel together; a struct keeps the call readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { mode, preLandingIndexPath, capturedIndexPath, landedIndexPath, postIndexPath, landedTreeOid, baseRevision, committedPaths, guardedPaths, exactPrivateIndex }
/// ```
pub struct PostIndexInput<'input> {
    /// Commit selection mode.
    pub mode: TransactionMode,
    /// Exact copy of the real index taken at landing.
    pub pre_landing_index: &'input Path,
    /// Invocation-time index copy.
    pub captured_index: &'input Path,
    /// Private index the landed commit was made from.
    pub landed_index: &'input Path,
    /// Post-index written by this computation.
    pub post_index: &'input Path,
    /// Tree the landing installs.
    pub landed_tree: &'input str,
    /// Preparation base commit, or the empty tree when unborn.
    pub base_revision: &'input str,
    /// Paths an explicit-path commit carries, including policy-added paths, as the exact bytes Git reported.
    pub committed_paths: &'input [Vec<u8>],
    /// Paths a commit hook changed, as the exact bytes Git reported.
    pub guarded_paths: &'input [Vec<u8>],
    /// Whether `landed_index` is exactly the index the commit was prepared from.
    pub exact_private_index: bool,
}

/// What:
///  Runs real Git with the literal-pathspec override that keeps `:(top,literal)` paths concrete.
/// Why:
///   The caller may disable pathspec magic, and the landing names exact paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// environment: { GIT_LITERAL_PATHSPECS: '0' }
/// ```
fn literal_pathspecs() -> Vec<(OsString, OsString)> {
    return vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("0"))];
}

/// What:
///  The `:(top,literal)` pathspec of one exact path, as the operating system takes it.
/// Why:
///   The landing names exact paths, and a name that is not UTF-8 must reach Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `:(top,literal)${path}`
/// ```
fn literal_pathspec(path: &[u8]) -> OsString {
    let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
    pathspec.extend_from_slice(path);
    return os_from_path_bytes(pathspec.as_slice());
}

/// What:
///  Splits NUL-terminated records and drops empty ones, keeping the bytes of each.
/// Why:
///   Every `-z` listing ends each record with NUL, and a path in one is bytes, never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bytes.split(0).filter(nonempty)
/// ```
fn records(bytes: &[u8]) -> Vec<Vec<u8>> {
    return bytes
        .split(|byte| { return *byte == 0; })
        .filter(|record| { return !record.is_empty(); })
        .map(<[u8]>::to_vec)
        .collect();
}

/// What:
///  Lists the stage records of concrete paths in one index, grouped by path.
/// Why:
///   Comparing the stages of a path decides whether anyone restaged it since invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listStages({ gitPath, cwd, indexPath, paths });
/// ```
fn list_stages(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
    paths: &[Vec<u8>],
) -> Result<HashMap<Vec<u8>, Vec<Vec<u8>>>, GitFailure> {
    let mut grouped: HashMap<Vec<u8>, Vec<Vec<u8>>> = HashMap::new();
    if paths.is_empty() {
        return Ok(grouped);
    }
    let mut arguments: Vec<OsString> = ["ls-files", "--stage", "-z", "--full-name", "--"]
        .iter()
        .map(OsString::from)
        .collect();
    arguments.extend(paths.iter().map(|path| { return literal_pathspec(path.as_slice()); }));
    let mut request = GitRequest::new(cwd, &arguments);
    request.index = Some(index.to_path_buf());
    request.environment = literal_pathspecs();
    let output: GitOutput = run_git_checked(context, &request)?;
    for record in records(&output.stdout) {
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            return Err(GitFailure(format!("ls-files record has no path: {}", path_message_text(record.as_slice()))));
        };
        grouped.entry(record[tab + 1..].to_vec()).or_default().push(record.clone());
    }
    return Ok(grouped);
}

/// What:
///  Lists the paths two trees differ in, without rename pairing.
/// Why:
///   The landed tree's changes are the paths the landing introduces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git diff-tree -r --no-renames --name-only -z from to
/// ```
fn changed_paths(context: &GitContext, cwd: &Path, from: &str, to: &str) -> Result<Vec<Vec<u8>>, GitFailure> {
    let request = GitRequest::new(cwd, &["diff-tree", "-r", "--no-renames", "--name-only", "-z", from, to]);
    let output: GitOutput = run_git_checked(context, &request)?;
    return Ok(records(&output.stdout));
}

/// What:
///  Lists paths the landed tree changes relative to what was staged at invocation.
/// Why:
///   When the invocation-time index cannot form a tree, nothing is added by this rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changedSinceCaptured({ gitPath, cwd, capturedIndexPath, landedTreeOid });
/// ```
fn changed_since_captured(
    context: &GitContext,
    cwd: &Path,
    captured: &Path,
    landed_tree: &str,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    let mut request = GitRequest::new(cwd, &["write-tree"]);
    request.index = Some(captured.to_path_buf());
    let output: GitOutput = run_git(context, &request)
        .map_err(|error| { return GitFailure(format!("git write-tree could not start: {error}")); })?;
    if !output.succeeded() {
        return Ok(Vec::new());
    }
    let tree: String = String::from_utf8_lossy(&output.stdout).trim().to_string();
    return changed_paths(context, cwd, tree.as_str(), landed_tree);
}

/// What:
///  Merges the landed entries of every changed path that nobody restaged since invocation.
/// Why:
///   Other staging stays as it is; only paths still holding their invocation entry take the landed one,
///  and a path the commit deleted is removed from the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ indexPath: postIndexPath, args: ['update-index', '-z', '--index-info'], input });
/// ```
fn merge_landed_entries(
    context: &GitContext,
    cwd: &Path,
    artifacts: &Artifacts,
    changed: &[Vec<u8>],
) -> Result<(), GitFailure> {
    let captured = list_stages(context, cwd, artifacts.captured, changed)?;
    let current = list_stages(context, cwd, artifacts.post, changed)?;
    let landed = list_stages(context, cwd, artifacts.landed, changed)?;
    let untouched: Vec<&Vec<u8>> = changed
        .iter()
        .filter(|path| {
            return captured.get(path.as_slice()).map(Vec::as_slice).unwrap_or(&[])
                == current.get(path.as_slice()).map(Vec::as_slice).unwrap_or(&[]);
        })
        .collect();
    if untouched.is_empty() {
        return Ok(());
    }
    let zero: String = "0".repeat(artifacts.landed_tree.len());
    let mut body: Vec<u8> = Vec::new();
    for path in untouched {
        match landed.get(path.as_slice()) {
            Some(entries) => {
                for entry in entries {
                    body.extend_from_slice(entry.as_slice());
                    body.push(0);
                }
            }
            None => {
                body.extend_from_slice(format!("0 {zero}\t").as_bytes());
                body.extend_from_slice(path.as_slice());
                body.push(0);
            }
        }
    }
    let mut request = GitRequest::new(cwd, &["update-index", "-z", "--index-info"]);
    request.index = Some(artifacts.post.to_path_buf());
    request.input = Some(body);
    run_git_checked(context, &request)?;
    return Ok(());
}

/// What:
///  The four index files and the landed tree of one computation.
/// Why:
///   The helpers take the same paths together; grouping keeps their signatures short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { capturedIndexPath, postIndexPath, landedIndexPath, landedTreeOid }
/// ```
struct Artifacts<'artifacts> {
    /// Invocation-time index copy.
    captured: &'artifacts Path,
    /// Post-index being written.
    post: &'artifacts Path,
    /// Private index the commit was made from.
    landed: &'artifacts Path,
    /// Tree the landing installs.
    landed_tree: &'artifacts str,
}

/// What:
///  Resets guarded paths to the landed tree while they still hold their invocation entry.
/// Why:
///   A path restaged after invocation keeps the staging the other process made, and the warning names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resetGuardedPaths({ gitPath, cwd, capturedIndexPath, postIndexPath, landedTreeOid, guardedPaths });
/// ```
fn reset_guarded_paths(
    context: &GitContext,
    cwd: &Path,
    artifacts: &Artifacts,
    guarded: &[Vec<u8>],
) -> Result<(), GitFailure> {
    if guarded.is_empty() {
        return Ok(());
    }
    let captured = list_stages(context, cwd, artifacts.captured, guarded)?;
    let current = list_stages(context, cwd, artifacts.post, guarded)?;
    let mut untouched: Vec<&Vec<u8>> = Vec::new();
    for path in guarded {
        let before: &[Vec<u8>] = captured.get(path.as_slice()).map(Vec::as_slice).unwrap_or(&[]);
        let now: &[Vec<u8>] = current.get(path.as_slice()).map(Vec::as_slice).unwrap_or(&[]);
        if before == now {
            untouched.push(path);
        } else {
            let shown: String = path_message_text(path.as_slice());
            warn(
                TAG,
                &format!(
                    "A commit hook changed {shown}, but it was staged again while the commit ran; the committed bytes are in HEAD and your staged entry was kept. Compare them with git diff --cached -- {shown}."
                ),
            );
        }
    }
    if untouched.is_empty() {
        return Ok(());
    }
    let mut arguments: Vec<OsString> = vec!["reset".into(), "--quiet".into(), artifacts.landed_tree.into(), "--".into()];
    arguments.extend(untouched.iter().map(|path| { return literal_pathspec(path.as_slice()); }));
    let mut request = GitRequest::new(cwd, &arguments);
    request.index = Some(artifacts.post.to_path_buf());
    request.environment = literal_pathspecs();
    run_git_checked(context, &request)?;
    return Ok(());
}

/// What:
///  Computes the post-index a landing installs.
/// Why:
///   The landing's real index must reflect this commit and every other process's staging at the moment of landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await computeLandingPostIndex({ gitPath, cwd, mode, ... });
/// ```
pub fn compute_landing_post_index(
    context: &GitContext,
    cwd: &Path,
    input: &PostIndexInput,
) -> Result<(), GitFailure> {
    let artifacts = Artifacts {
        captured: input.captured_index,
        post: input.post_index,
        landed: input.landed_index,
        landed_tree: input.landed_tree,
    };
    if input.mode == TransactionMode::ExplicitPath {
        copy(input.pre_landing_index, input.post_index)?;
        if !input.committed_paths.is_empty() {
            let mut arguments: Vec<OsString> = vec!["reset".into(), "--quiet".into(), input.landed_tree.into(), "--".into()];
            arguments.extend(input.committed_paths.iter().map(|path| { return literal_pathspec(path.as_slice()); }));
            let mut request = GitRequest::new(cwd, &arguments);
            request.index = Some(input.post_index.to_path_buf());
            request.environment = literal_pathspecs();
            run_git_checked(context, &request)?;
        }
        let outside: Vec<Vec<u8>> = input
            .guarded_paths
            .iter()
            .filter(|path| { return !input.committed_paths.contains(path); })
            .cloned()
            .collect();
        reset_guarded_paths(context, cwd, &artifacts, &outside)?;
        // Every entry outside the committed and guarded paths must read as it did before the landing.
        let mut allowed: Vec<Vec<u8>> = input.committed_paths.to_vec();
        allowed.extend(input.guarded_paths.iter().cloned());
        warn_when_entries_moved(context, cwd, input.pre_landing_index, input.post_index, &allowed);
        return Ok(());
    }
    if input.exact_private_index && files_equal(input.pre_landing_index, input.captured_index)
        .map_err(|error| { return GitFailure(error.0); })?
    {
        copy(input.landed_index, input.post_index)?;
        return Ok(());
    }
    warn(TAG, "real index changed since invocation; merging landed entries per path");
    copy(input.pre_landing_index, input.post_index)?;
    let mut changed: Vec<Vec<u8>> = changed_paths(context, cwd, input.base_revision, input.landed_tree)?;
    for path in changed_since_captured(context, cwd, input.captured_index, input.landed_tree)? {
        if !changed.contains(&path) {
            changed.push(path);
        }
    }
    return merge_landed_entries(context, cwd, &artifacts, &changed);
}

/// What:
///  Copies one index file with the timestamps Git reads.
/// Why:
///   The post-index keeps the stat data Git uses to skip re-hashing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyIndexFile({ sourcePath, destinationPath });
/// ```
fn copy(source: &Path, destination: &Path) -> Result<(), GitFailure> {
    return copy_index_with_timestamps(source, destination)
        .map_err(|error| { return GitFailure(format!("copying {} failed: {error}", source.display())); });
}

/// Post-index controls stay out of the production code path.
#[cfg(test)]
#[path = "landing_index_tests.rs"]
mod tests;
