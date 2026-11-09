//! What:
//!  The retry of a commit the service sent back because a path it names changed on the branch since its base:
//! the change
//!  the first preparation captured is merged onto the branch as it is now,
//! and the worktree is not read again.
//! Why:
//!  A commit lands the bytes that were there when the person ran the command (`doc/decision/cli-git-concurrent-commits.md`,
//!  "Serial landing";
//! `SPEC.md`:
//! "Cli-git never re-prepares a commit from current worktree bytes").
//! Preparing again from
//!  the worktree landed bytes written after the command,
//! and bytes another command meant.
//! The merge base is the
//!  preparation base,
//! and a path the other commits changed that the captured change already contains is kept as captured
//!  (the same subsumption rule the earlier landing path applies,
//! `replay_subsumption.rs`).
//! The first merge is one Git
//!  run;
//! the subsumption runs only for the paths that conflicted,
//! so a retry that merges cleanly costs three runs here:
//!  `write-tree` for the captured tree,
//! `merge-tree`,
//! and `read-tree` of the merged tree into the private index.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const captured = await captureTree(readyCommit);              // after the service sent the commit back
//! const merged = await mergeOntoBranch(captured, currentTip);   // { tree } | { conflict }
//! ```

pub use crate::service_prepare_ready_shapes::{LandedFirst, ReplayConflict};
use crate::service_replay_path_and_line_text::{byte_of_character, is_not_empty_line, latin1_of_path};
use crate::private_index_request::indexed_request;
use crate::commit_transaction::{TransactionResult, TransactionStop, failed};
use crate::path_bytes::{os_from_path_bytes, path_message_text};
use crate::policy_events::PolicyEvent;
use crate::merge_second_pass::{Conflict, ContextRunner, Merged as MergedFile};
use crate::child_environment::environment_with_overlay;
use crate::journald_native::CLIENT_IDENTIFIER;
use crate::merge_second_pass_tree::{AttributeProbe, announce, attribute_tree_variables, resolve_tree};
use crate::replay_capture_order::CaptureOrderDecision;
use crate::replay_merge::{ReplayMerge, merge_replay_tree};
use crate::replay_shared_paths::{SharedPath, list_shared_paths};
use crate::replay_subsumption::{SubsumptionBase, subsume_landed_changes};
use crate::service_messages::NOTHING_CHANGED;
use crate::service_prepare_ready_shapes::Ready;
use crate::service_protocol::LandedCapture;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The transform that names the finding a retry ends with,
/// as for every other concurrent-commit finding.
const CORE_ID: &str = "concurrent-commit";

/// What the first preparation captured,
/// kept for the retry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captured {
    /// The commit the change was prepared against,
    /// which is the merge base of every retry.
    pub base: String,
    /// The tree of that base with the captured change applied.
    pub tree: String,
    /// When the worktree's bytes were captured (`CommitRequest::captured`);
    /// zero when unknown.
    /// A retry keeps the first
    ///  capture's stamp,
    /// because the merged bytes are not a new capture.
    pub captured: u64,
    /// The commits the service landed since,
    /// with the stamps of their captures,
    /// as its last answer told.
    pub landed: Vec<LandedCapture>,
}

/// What:
///  The stamp of a capture finished now:
/// nanoseconds of `CLOCK_MONOTONIC`,
/// zero where the platform has none.
/// Why:
///   That clock is the host's,
/// one for every process,
/// and never steps back,
/// so a capture made later by another wrapper
///  process gets a larger stamp (`service_captures.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.hrtime.bigint()
/// ```
pub fn capture_stamp() -> u64 {
    #[cfg(unix)]
    {
        let mut now = libc::timespec { tv_sec: 0, tv_nsec: 0 };
        // SAFETY: `clock_gettime` writes one `timespec` through a valid pointer.
        let read: i32 = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &raw mut now) };
        if read != 0 {
            return 0;
        }
        let seconds: u64 = u64::try_from(now.tv_sec).unwrap_or(0);
        let nanoseconds: u64 = u64::try_from(now.tv_nsec).unwrap_or(0);
        return seconds.saturating_mul(1_000_000_000).saturating_add(nanoseconds).max(1);
    }
    #[cfg(not(unix))]
    {
        return 0;
    }
}

/// A retry:
/// the change the first preparation captured,
/// and which retry this is.
pub struct Retry<'a> {
    /// What the first preparation captured.
    pub captured: &'a Captured,
    /// Which retry this is,
    /// from 1,
    /// which names the retry's scratch files.
    pub attempt: u64,
}

/// What merging the captured change onto the branch came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Merged {
    /// The change applies;
    /// the merged tree exists in the repository.
    Tree(String),
    /// The change applies after the word-level second pass merged every file Git's line merge left in conflict;
    /// the tree
    ///  exists in the repository,
    /// and the files are listed to the person and in the journal.
    Combined {
        /// The merged tree.
        tree: String,
        /// The files the word pass merged,
        /// with their parts.
        files: Vec<MergedFile>,
    },
    /// The change overlaps what landed.
    Conflict(Vec<Vec<u8>>),
}

/// What the merge needs to know about where it runs.
pub struct MergeSite<'a> {
    /// How to run Git.
    pub context: &'a GitContext,
    /// The directory the commit was run from.
    pub cwd: &'a Path,
    /// The common Git directory,
    /// which holds the objects and which the object-only steps run against.
    pub common_dir: &'a Path,
    /// The object store receiving what the steps write.
    pub object_directory: &'a Path,
    /// The commit's private directory,
    /// where the subsumption keeps its scratch index.
    pub scratch: &'a Path,
}

/// What:
///  What a retry replays,
/// from the preparation that was sent back:
/// its tree on its base,
/// its capture stamp,
/// and the commits
///  the service says landed since.
/// Why:
///   Both the commit the service builds and the commit Git builds are replayed from the tree of their private index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryOf(context, cwd, ready, landed)
/// ```
pub fn retry_of(context: &GitContext, cwd: &Path, ready: &Ready, landed: Vec<LandedCapture>) -> Result<Captured, TransactionStop> {
    let tree: String = captured_tree(context, cwd, ready.inputs.commit_index.as_path(), ready.inputs.capture.layout.object_directory.as_path())?;
    return Ok(Captured { base: ready.base.clone(), tree, captured: ready.captured_at, landed });
}

/// What:
///  The tree the private index holds,
/// which is the captured change on its base.
/// Why:
///   The retry merges this tree,
/// so it is written once,
/// when the service sends the commit back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await capturedTree(context, cwd, commitIndex, objectDirectory)
/// ```
pub fn captured_tree(context: &GitContext, cwd: &Path, commit_index: &Path, object_directory: &Path) -> Result<String, TransactionStop> {
    let written = run_git_checked(context, &indexed_request(cwd, &["write-tree"], Some(commit_index), Some(object_directory)))?;
    return Ok(String::from_utf8_lossy(written.stdout.as_slice()).trim().to_string());
}

/// What:
///  A path as the person reads it in quotes.
/// Why:
///  `paths_text` lists the paths in conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quotedPath = (path: Uint8Array) => `\`${pathMessageText(path)}\``;
/// ```
fn quoted_path(path: &Vec<u8>) -> String {
    return format!("`{}`", path_message_text(path.as_slice()));
}

/// What:
///  Merges the captured change onto `tip` with `base` as the merge base.
/// Why:
///  `merge_captured` merges twice at most:
/// with the base of the preparation and with the base the compared commits settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeAt = (site, captured, tip, base) => mergeReplayTree(site, base, tip, captured.tree)
/// ```
fn merge_at(site: &MergeSite<'_>, captured: &Captured, tip: &str, base: &str) -> Result<ReplayMerge, TransactionStop> {
    return match merge_replay_tree(site.context, site.common_dir, site.cwd, base, tip, captured.tree.as_str()) {
        Ok(merged) => Ok(merged),
        Err(error) => Err(failed(format!("merging the commit onto the branch failed: {error:?}"))),
    };
}

/// What:
///  Merges the captured change onto `tip` with the preparation base as the merge base.
/// Why:
///   Clean merges,
/// the common case,
/// are one Git run.
/// A conflict is looked at again for the paths whose captured bytes
///  already contain what landed (the person's worktree held the other commit's edit when this one was run),
/// which the
///  merge alone reports as a conflict on adjacent edits;
/// only paths still in conflict after that are a conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mergeCaptured(site, captured, tip) // { tree } | { conflict: paths }
/// ```
pub fn merge_captured(site: &MergeSite<'_>, captured: &Captured, tip: &str, attempt: u64) -> Result<Merged, TransactionStop> {
    let (conflicting, first_tree, first_conflicts) = match merge_at(site, captured, tip, captured.base.as_str())? {
        ReplayMerge::Merged { tree_oid } => return Ok(Merged::Tree(tree_oid)),
        ReplayMerge::Conflict { paths, tree_oid, conflicts } => (paths, tree_oid, conflicts),
    };
    let names: Vec<String> = conflicting.iter().map(latin1_of_path).collect();
    let listed: Vec<SharedPath> = match list_shared_paths(site.context, site.common_dir, site.cwd, captured.base.as_str(), tip, captured.tree.as_str()) {
        Ok(found) => found,
        Err(error) => return Err(failed(format!("listing the paths the branch changed failed: {error:?}"))),
    };
    // Only the paths still in conflict are compared with what landed.
    let mut shared: Vec<SharedPath> = Vec::new();
    for entry in listed {
        if names.contains(&entry.path) {
            shared.push(entry);
        }
    }
    let mut order: BTreeMap<String, CaptureOrderDecision> = BTreeMap::new();
    for entry in &shared {
        let decision: CaptureOrderDecision = capture_order(site, captured, tip, entry.path.as_str())?;
        order.insert(entry.path.clone(), decision);
    }
    let compared = subsume_landed_changes(
        site.context,
        site.common_dir,
        site.object_directory,
        site.cwd,
        site.scratch,
        attempt,
        captured.base.as_str(),
        shared.as_slice(),
        &order,
    );
    let settled: SubsumptionBase = match compared {
        Ok(base) => base,
        Err(error) => return Err(failed(format!("comparing the commit with what landed failed: {error:?}"))),
    };
    if settled.subsumed_paths.is_empty() && settled.kept_landed_paths.is_empty() {
        return Ok(word_pass(site, tip, (first_tree.as_str(), first_conflicts.as_slice()), conflicting));
    }
    return match merge_at(site, captured, tip, settled.merge_base.as_str())? {
        ReplayMerge::Merged { tree_oid } => Ok(Merged::Tree(tree_oid)),
        ReplayMerge::Conflict { paths, tree_oid, conflicts } => Ok(word_pass(site, tip, (tree_oid.as_str(), conflicts.as_slice()), paths)),
    };
}

/// What:
///  The event that tells the person which files the word pass merged for a replayed commit,
/// or nothing when none were.
/// Why:
///   The replayed commit carries the person's own message,
/// which is not changed;
/// the parts are told next to the commit's other
///  events,
/// and the journal gets one entry per file at the same time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordMergeNotice(files) // { type: 'word-merged', lines } | undefined
/// ```
pub fn word_merge_notice(files: &[MergedFile]) -> Option<PolicyEvent> {
    if files.is_empty() {
        return None;
    }
    return Some(PolicyEvent::WordMerged { lines: announce("", (CLIENT_IDENTIFIER, "commit-replay"), files) });
}

/// What:
///  The last resort for a merge that still conflicts:
/// the word-level second pass over its conflicted files.
/// Why:
///   Git's line merge and the subsumption rules have both had their say;
/// what is left is conflicts of adjacent lines and
///  of different words,
/// which the owner decided (2026-11-03) the wrapper's own merges re-merge by word,
/// all or none.
/// The
///  attributes of the landed tip apply,
/// because the merge reads no worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordPass(site, tip, { tree, conflicts }, paths) // { combined: { tree, files } } | { conflict: paths }
/// ```
/// What:
///  The top of the worktree Git printed,
/// with its line break removed and nothing else.
/// Why:
///   A directory may end in a space,
/// and trimming the line Git printed takes that space with it,
/// so every command of the second pass would
///  run in a directory that does not exist and the commit would be refused as an uncombinable change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const top = stdout.toString().replace(/\r?\n$/, '');
/// ```
fn toplevel_of(stdout: &[u8]) -> PathBuf {
    let text: String = String::from_utf8_lossy(stdout).into_owned();
    // Exactly one terminator goes, so a directory whose name ends in a line break keeps it.
    let without_terminator: &str = match text.strip_suffix("\r\n") {
        Some(rest) => rest,
        None => text.strip_suffix('\n').unwrap_or(text.as_str()),
    };
    return PathBuf::from(without_terminator);
}

fn word_pass(site: &MergeSite<'_>, tip: &str, merge: (&str, &[Conflict]), paths: Vec<Vec<u8>>) -> Merged {
    let (tree, conflicts): (&str, &[Conflict]) = merge;
    // The commit may have been run from a subdirectory; the second pass reads paths from the top of the worktree.
    let mut top_request: GitRequest = GitRequest::new(site.cwd, &["rev-parse", "--show-toplevel"]);
    top_request.without_prefix = true;
    let top: PathBuf = match run_git_checked(site.context, &top_request) {
        Ok(found) => toplevel_of(found.stdout.as_slice()),
        Err(_) => return Merged::Conflict(paths),
    };
    let process_environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(process_environment.as_slice(), site.context.overlay.as_slice());
    let probing: ContextRunner<'_> = ContextRunner { context: site.context, cwd: top.as_path(), without_prefix: true, object_directory: None, environment: Vec::new() };
    let probe: AttributeProbe<'_> = AttributeProbe { directory: top.as_path(), global_prefix: &[], variables: variables.as_slice(), runner: &probing };
    let runner: ContextRunner<'_> = ContextRunner {
        context: site.context,
        cwd: top.as_path(),
        without_prefix: true,
        object_directory: Some(site.object_directory),
        environment: attribute_tree_variables(tip, &probe),
    };
    return match resolve_tree(&runner, site.common_dir, tree, conflicts, &probe) {
        Some(resolved) => Merged::Combined { tree: resolved.tree, files: resolved.merged },
        None => Merged::Conflict(paths),
    };
}

/// What:
///  Which of this commit's capture and the landed commits' captures of one path is the later.
/// Why:
///   The service remembers the stamps of the commits it landed for this worktree.
/// When every commit that changed the path
///  since the base is remembered with a stamp,
/// and all are earlier than this capture,
/// this capture holds their edits and
///  its bytes land;
/// when all are later,
/// theirs hold this commit's edit and the landed bytes stay.
/// Anything else (a commit
///  the service does not know,
/// a stamp that is missing,
/// one earlier and one later) is not decided by order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await captureOrder(site, captured, tip, path) // 'prepared' | 'landed' | 'unordered'
/// ```
fn capture_order(site: &MergeSite<'_>, captured: &Captured, tip: &str, path: &str) -> Result<CaptureOrderDecision, TransactionStop> {
    if captured.captured == 0 {
        return Ok(CaptureOrderDecision::Unordered);
    }
    let bytes: Vec<u8> = path.chars().filter_map(byte_of_character).collect();
    let exclusion: String = format!("^{}", captured.base);
    let mut request: GitRequest = GitRequest::new(site.cwd, &["log", "--format=%H", "--no-color", tip, exclusion.as_str(), "--"]);
    let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
    pathspec.extend_from_slice(bytes.as_slice());
    request.arguments.push(os_from_path_bytes(pathspec.as_slice()));
    request.environment = vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("0"))];
    let listed = run_git_checked(site.context, &request)?;
    let text: String = String::from_utf8_lossy(listed.stdout.as_slice()).into_owned();
    // The capture stamp of each commit that changed the path, nothing for a commit this service did not land.
    let mut stamps: Vec<Option<u64>> = Vec::new();
    for commit in text.lines() {
        if commit.is_empty() {
            continue;
        }
        let mut stamp: Option<u64> = None;
        for known in captured.landed.iter() {
            if known.commit == commit {
                stamp = Some(known.captured);
                break;
            }
        }
        stamps.push(stamp);
    }
    if stamps.is_empty() || stamps.iter().any(Option::is_none) {
        return Ok(CaptureOrderDecision::Unordered);
    }
    let known: Vec<u64> = stamps.into_iter().flatten().collect();
    // Every commit was captured before this one, or every commit after it; any mix is unordered.
    let mut all_before: bool = true;
    let mut all_after: bool = true;
    for stamp in &known {
        if *stamp >= captured.captured {
            all_before = false;
        }
        if *stamp <= captured.captured {
            all_after = false;
        }
    }
    if all_before {
        return Ok(CaptureOrderDecision::Prepared);
    }
    if all_after {
        return Ok(CaptureOrderDecision::Landed);
    }
    return Ok(CaptureOrderDecision::Unordered);
}

/// What:
///  The first commit after `base`,
/// up to `tip`,
/// that changed one of `paths`,
/// with its abbreviated name and subject.
/// Why:
///   The conflict names the commit that landed first,
/// in the words the person sees in their own log.
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
    let Some(first) = text.lines().find(is_not_empty_line) else { return Ok(None) };
    let (short, subject) = first.split_once('\0').unwrap_or((first, ""));
    return Ok(Some(LandedFirst { short: short.to_string(), subject: subject.to_string() }));
}

/// The paths as the person reads them:
/// one in quotes,
/// several as a list.
fn paths_text(paths: &[Vec<u8>]) -> String {
    let quoted: Vec<String> = paths.iter().map(quoted_path).collect();
    return match quoted.as_slice() {
        [only] => format!("the file {only}"),
        many => format!("these files: {}", many.join(", ")),
    };
}

/// What:
///  The words for a conflict:
/// what overlapped,
/// which commit landed first,
/// where the message is saved,
/// how to commit again.
/// Why:
///   Plain words,
/// every input named and every way forward given,
/// nothing about how the commit is landed.
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
