//! What:
//!  The retry of a commit the service sent back because a path it names changed on the branch since its base: the change
//!  the first preparation captured is merged onto the branch as it is now, and the worktree is not read again.
//! Why:
//!  A commit lands the bytes that were there when the person ran the command (`doc/decision/cli-git-concurrent-commits.md`,
//!  "Serial landing"; `SPEC.md`: "Cli-git never re-prepares a commit from current worktree bytes"). Preparing again from
//!  the worktree landed bytes written after the command, and bytes another command meant. The merge base is the
//!  preparation base, and a path the other commits changed that the captured change already contains is kept as captured
//!  (the same subsumption rule the earlier landing path applies, `replay_subsumption.rs`). The first merge is one Git
//!  run; the subsumption runs only for the paths that conflicted, so a retry that merges cleanly costs three runs here:
//!  `write-tree` for the captured tree, `merge-tree`, and `read-tree` of the merged tree into the private index.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const captured = await captureTree(readyCommit);              // after the service sent the commit back
//! const merged = await mergeOntoBranch(captured, currentTip);   // { tree } | { conflict }
//! ```

use super::capture_order::indexed_request;
use super::commit_transaction::{TransactionResult, TransactionStop, failed};
use super::path_bytes::{os_from_path_bytes, path_message_text};
use super::policy_events::PolicyEvent;
use super::replay_merge::{ReplayMerge, merge_replay_tree};
use super::replay_shared_paths::{SharedPath, list_shared_paths};
use super::replay_subsumption::{SubsumptionBase, subsume_landed_changes};
use super::service_messages::NOTHING_CHANGED;
use super::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The transform that names the finding a retry ends with, as for every other concurrent-commit finding.
const CORE_ID: &str = "concurrent-commit";

/// What the first preparation captured, kept for the retry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captured {
    /// The commit the change was prepared against, which is the merge base of every retry.
    pub base: String,
    /// The tree of that base with the captured change applied.
    pub tree: String,
}

/// A retry: the change the first preparation captured, and which retry this is.
pub struct Retry<'a> {
    /// What the first preparation captured.
    pub captured: &'a Captured,
    /// Which retry this is, from 1, which names the retry's scratch files.
    pub attempt: u64,
}

/// The first commit on the branch since the preparation base that changed one of the conflicting paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedFirst {
    /// Its abbreviated name, as Git abbreviates it in this repository.
    pub short: String,
    /// Its subject.
    pub subject: String,
}

/// A retry that could not combine the captured change with the branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayConflict {
    /// The conflicting paths in Git's byte order, as the exact bytes Git reported.
    pub paths: Vec<Vec<u8>>,
    /// The commit that landed first on them, when it could be found.
    pub landed: Option<LandedFirst>,
    /// The Git directory of the worktree, where a saved message is kept.
    pub git_dir: PathBuf,
}

/// What merging the captured change onto the branch came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Merged {
    /// The change applies; the merged tree exists in the repository.
    Tree(String),
    /// The change overlaps what landed.
    Conflict(Vec<Vec<u8>>),
}

/// What the merge needs to know about where it runs.
pub struct MergeSite<'a> {
    /// How to run Git.
    pub context: &'a GitContext,
    /// The directory the commit was run from.
    pub cwd: &'a Path,
    /// The common Git directory, which holds the objects and which the object-only steps run against.
    pub common_dir: &'a Path,
    /// The object store receiving what the steps write.
    pub object_directory: &'a Path,
    /// The commit's private directory, where the subsumption keeps its scratch index.
    pub scratch: &'a Path,
}

/// What:
///  The tree the private index holds, which is the captured change on its base.
/// Why:
///   The retry merges this tree, so it is written once, when the service sends the commit back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await capturedTree(context, cwd, commitIndex, objectDirectory)
/// ```
pub fn captured_tree(context: &GitContext, cwd: &Path, commit_index: &Path, object_directory: &Path) -> Result<String, TransactionStop> {
    let written = run_git_checked(context, &indexed_request(cwd, &["write-tree"], Some(commit_index), Some(object_directory)))?;
    return Ok(String::from_utf8_lossy(written.stdout.as_slice()).trim().to_string());
}

/// The path as the Latin-1 text the shared-path listing names it by, so every byte survives the comparison.
fn latin1(bytes: &[u8]) -> String {
    return bytes.iter().map(|byte| { return char::from(*byte); }).collect();
}

/// What:
///  Merges the captured change onto `tip` with the preparation base as the merge base.
/// Why:
///   Clean merges, the common case, are one Git run. A conflict is looked at again for the paths whose captured bytes
///  already contain what landed (the person's worktree held the other commit's edit when this one was run), which the
///  merge alone reports as a conflict on adjacent edits; only paths still in conflict after that are a conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mergeCaptured(site, captured, tip) // { tree } | { conflict: paths }
/// ```
pub fn merge_captured(site: &MergeSite<'_>, captured: &Captured, tip: &str, attempt: u64) -> Result<Merged, TransactionStop> {
    let merge = |base: &str| -> Result<ReplayMerge, TransactionStop> {
        return merge_replay_tree(site.context, site.common_dir, site.cwd, base, tip, captured.tree.as_str())
            .map_err(|error| { return failed(format!("merging the commit onto the branch failed: {error:?}")); });
    };
    let conflicting: Vec<Vec<u8>> = match merge(captured.base.as_str())? {
        ReplayMerge::Merged { tree_oid } => return Ok(Merged::Tree(tree_oid)),
        ReplayMerge::Conflict { paths } => paths,
    };
    let names: Vec<String> = conflicting.iter().map(|path| { return latin1(path.as_slice()); }).collect();
    let shared: Vec<SharedPath> = list_shared_paths(site.context, site.common_dir, site.cwd, captured.base.as_str(), tip, captured.tree.as_str())
        .map_err(|error| { return failed(format!("listing the paths the branch changed failed: {error:?}")); })?
        .into_iter()
        .filter(|entry| { return names.contains(&entry.path); })
        .collect();
    let settled: SubsumptionBase = subsume_landed_changes(
        site.context,
        site.common_dir,
        site.object_directory,
        site.cwd,
        site.scratch,
        attempt,
        captured.base.as_str(),
        shared.as_slice(),
        &BTreeMap::new(),
    )
    .map_err(|error| { return failed(format!("comparing the commit with what landed failed: {error:?}")); })?;
    if settled.subsumed_paths.is_empty() {
        return Ok(Merged::Conflict(conflicting));
    }
    return match merge(settled.merge_base.as_str())? {
        ReplayMerge::Merged { tree_oid } => Ok(Merged::Tree(tree_oid)),
        ReplayMerge::Conflict { paths } => Ok(Merged::Conflict(paths)),
    };
}

/// What:
///  The first commit after `base`, up to `tip`, that changed one of `paths`, with its abbreviated name and subject.
/// Why:
///   The conflict names the commit that landed first, in the words the person sees in their own log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await firstLanded(context, cwd, base, tip, paths) // { short, subject } | undefined
/// ```
pub fn first_landed(context: &GitContext, cwd: &Path, base: &str, tip: &str, paths: &[Vec<u8>]) -> Result<Option<LandedFirst>, TransactionStop> {
    let exclusion: String = format!("^{base}");
    let mut request: GitRequest = GitRequest::new(
        cwd,
        &["log", "--reverse", "--format=%h%x00%s", "--no-color", "--no-show-signature", tip, exclusion.as_str(), "--"],
    );
    for path in paths {
        let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
        pathspec.extend_from_slice(path.as_slice());
        request.arguments.push(os_from_path_bytes(pathspec.as_slice()));
    }
    request.environment = vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("0"))];
    let listed = run_git_checked(context, &request)?;
    let text: String = String::from_utf8_lossy(listed.stdout.as_slice()).into_owned();
    let Some(first) = text.lines().find(|line| { return !line.is_empty(); }) else { return Ok(None) };
    let (short, subject) = first.split_once('\0').unwrap_or((first, ""));
    return Ok(Some(LandedFirst { short: short.to_string(), subject: subject.to_string() }));
}

/// The paths as the person reads them: one in quotes, several as a list.
fn paths_text(paths: &[Vec<u8>]) -> String {
    let quoted: Vec<String> = paths.iter().map(|path| { return format!("`{}`", path_message_text(path.as_slice())); }).collect();
    return match quoted.as_slice() {
        [only] => format!("the file {only}"),
        many => format!("these files: {}", many.join(", ")),
    };
}

/// What:
///  The words for a conflict: what overlapped, which commit landed first, where the message is saved, how to commit again.
/// Why:
///   Plain words, every input named and every way forward given, nothing about how the commit is landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// conflictText(conflict, savedMessage)
/// ```
pub fn conflict_text(conflict: &ReplayConflict, saved: Option<&Path>) -> String {
    let landed: String = match &conflict.landed {
        Some(first) => format!("the commit {} (\"{}\")", first.short, first.subject),
        None => String::from("another commit"),
    };
    let look: String = match &conflict.landed {
        Some(first) => format!("Look at what that commit changed with `git show {}`, bring your", first.short),
        None => String::from("Look at what the branch changed, bring your"),
    };
    let again: String = match saved {
        Some(file) => format!(
            "Your message is saved in {}. {look} edit up to date with it, and run the commit again with `git commit -F {}` and the same paths.",
            file.display(),
            file.display()
        ),
        None => format!("{look} edit up to date with it, and run the commit again."),
    };
    return format!(
        "This commit was not made. While it was being prepared, {landed} changed {}, which this commit also changes, and the two \
         changes cannot be combined. {again} {NOTHING_CHANGED}",
        paths_text(conflict.paths.as_slice())
    );
}

/// The finding a conflict ends the command with.
pub fn conflict_result(conflict: &ReplayConflict, saved: Option<&Path>) -> TransactionResult {
    return TransactionResult {
        events: vec![PolicyEvent::CoreFinding { core_id: CORE_ID, code: "replay-conflict", message: conflict_text(conflict, saved) }],
        exit_code: 1,
        landed_oid: None,
        uses_lfs: false,
    };
}

/// The ending of a retry whose merged result is what the branch already holds.
pub fn already_committed_result(branch: &str) -> TransactionResult {
    return TransactionResult {
        events: vec![PolicyEvent::CoreFinding {
            core_id: CORE_ID,
            code: "already-committed",
            message: format!(
                "This commit was not made. While it was being prepared, other commits put everything it changes on the branch {branch}, \
                 so nothing is left to commit. Run `git log` to see them. {NOTHING_CHANGED}"
            ),
        }],
        exit_code: 1,
        landed_oid: None,
        uses_lfs: false,
    };
}

/// Replay controls stay out of the release executable.
#[cfg(test)]
#[path = "service_replay_tests.rs"]
mod tests;
