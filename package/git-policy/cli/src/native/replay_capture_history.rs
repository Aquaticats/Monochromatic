//! What:
//!  The capture-order inputs of one replay:
//!       the first-parent commits landed since the preparation base with their changed paths,
//!       and the decisions for the shared paths read from `captured.json` and the landed-capture records.
//! Why:
//!  Two Git processes list the history whatever its length; each landed commit's record says which worktree
//!       captured which path and when, which is what the decision compares.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await decideSharedPaths({ gitPath, shadowPath, gitDir, transactionDirectory, base, current, paths });
//! ```

/// The capture-order decision and its parsers.
use super::replay_capture_order::{CaptureOrderDecision, CaptureView, LandedChange, capture_view, decide_capture_order, parse_commit_changes};
/// The captured and landed record readers.
use super::capture_records::{landed_record_directory, parse_landed_capture_record, read_captured_record};
use super::capture_store::read_store_file;
use super::replay_shadow::run_shadow_checked;
use super::transaction_git::{GitContext, GitFailure};
/// Decisions and records keyed by path or commit.
use std::collections::BTreeMap;
/// `Path` names the shadow, the git directory and the transaction.
use std::path::Path;

/// What:
///  A capture-order failure:
///       a Git command failed, or a record or listing could not be read or parsed.
/// Why:
///   Replay reports each as `transaction-failed`; a malformed record is never treated as "unordered".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CaptureOrderRecordError(message);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureHistoryError {
    /// A Git command failed.
    Git(GitFailure),
    /// A record file could not be read, or the listing did not parse.
    Record(String),
}

/// What:
///  Lists the commits landed since the preparation base on the first-parent line with the paths each changed.
/// Why:
///   `rev-list --first-parent` gives the line; one `diff-tree --stdin` gives every commit's changes,
///       so the process count does not depend on the history length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listLandedChanges({ gitPath, shadowPath, base, current });
/// ```
pub fn list_landed_changes(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    base: Option<&str>,
    current: &str,
) -> Result<Vec<LandedChange>, CaptureHistoryError> {
    let exclusion: String = base.map(|oid| { return format!("^{oid}"); }).unwrap_or_default();
    // `mut` lets the exclusion be included only for a committed base.
    let mut arguments: Vec<&str> = vec!["rev-list", "--first-parent", current];
    if base.is_some() {
        arguments.push(exclusion.as_str());
    }
    arguments.push("--");
    let listed = run_shadow_checked(context, shadow, cwd, &arguments, None, None, &[]).map_err(CaptureHistoryError::Git)?;
    if listed.stdout.is_empty() {
        return Ok(Vec::new());
    }
    let changes = run_shadow_checked(
        context,
        shadow,
        cwd,
        &[
            "diff-tree",
            "--stdin",
            "--root",
            "-r",
            "-z",
            "--raw",
            "--no-renames",
            "--no-abbrev",
            "-m",
            "--first-parent",
        ],
        Some(listed.stdout.as_slice()),
        None,
        &[],
    )
    .map_err(CaptureHistoryError::Git)?;
    let text: String = changes.stdout.iter().map(|byte| { return char::from(*byte); }).collect();
    return parse_commit_changes(text.as_str()).map_err(|error| { return CaptureHistoryError::Record(error.0); });
}

/// What:
///  Reads the landed-capture record of each listed commit that has one, as capture views keyed by commit.
/// Why:
///   A commit without a record leaves the path unordered, so a missing record is skipped, not an error;
///       a malformed record fails the replay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readLandedCaptures({ gitDir, commits });
/// ```
fn read_landed_captures(git_dir: &Path, commits: &[LandedChange]) -> Result<BTreeMap<String, CaptureView>, CaptureHistoryError> {
    let directory = landed_record_directory(git_dir);
    // `mut` lets the views be collected one commit at a time.
    let mut views: BTreeMap<String, CaptureView> = BTreeMap::new();
    for change in commits {
        let name: String = format!("{}.json", change.commit);
        let text: Option<String> = read_store_file(directory.join(&name).as_path())
            .map_err(|error| { return CaptureHistoryError::Record(error.0); })?;
        let Some(found) = text else {
            continue;
        };
        let record = parse_landed_capture_record(found.as_str(), &format!("landed/{name}"))
            .map_err(|error| { return CaptureHistoryError::Record(error.0); })?;
        views.insert(
            change.commit.clone(),
            capture_view(record.worktree_id.as_str(), record.sequence, &record.worktree_paths),
        );
    }
    return Ok(views);
}

/// What:
///  Decides every shared path of one replay: the transaction's `captured.json` and the landed records
///       of the first-parent history since the base.
/// Why:
///   When the transaction's capture names none of the shared paths, or the transaction has no capture,
///       every path is unordered and no history process runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await decideSharedPaths({ gitPath, shadowPath, gitDir, transactionDirectory, base, current, paths });
/// ```
pub fn decide_shared_paths(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    git_dir: &Path,
    transaction_directory: &Path,
    base: Option<&str>,
    current: &str,
    paths: &[String],
) -> Result<BTreeMap<String, CaptureOrderDecision>, CaptureHistoryError> {
    let captured = match read_captured_record(transaction_directory).map_err(|error| { return CaptureHistoryError::Record(error.0); })? {
        Some(found) => found,
        None => return Ok(BTreeMap::new()),
    };
    let own: CaptureView = capture_view(captured.worktree_id.as_str(), captured.sequence, &captured.worktree_paths);
    let candidates: Vec<&String> = paths.iter().filter(|path| { return own.paths.contains(path.as_str()); }).collect();
    if candidates.is_empty() {
        return Ok(BTreeMap::new());
    }
    let history: Vec<LandedChange> = list_landed_changes(context, shadow, cwd, base, current)?;
    let landed: BTreeMap<String, CaptureView> = read_landed_captures(git_dir, &history)?;
    return Ok(candidates
        .into_iter()
        .map(|path| {
            let decision: CaptureOrderDecision = decide_capture_order(path.as_str(), &own, &history, &landed);
            return (path.to_string(), decision);
        })
        .collect());
}

#[cfg(test)]
#[path = "replay_capture_history_tests.rs"]
mod tests;
