//! What:
//!  The merge and rebuild steps of one replay:
//!       the replay merge base, the tree from `git merge-tree --write-tree`, the first landed commit that touched
//!       a conflicting path, and the replayed commit written from the prepared one.
//! Why:
//!  Replay runs outside both landing locks in the shadow repository.
//!       An unsigned prepared commit is rebuilt by rewriting its raw object, which keeps every byte;
//!       a signed one is re-signed with `git commit-tree -S`, because no Git primitive re-signs a raw object.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await mergeReplayTree({ gitPath, shadowPath, mergeBase, current, prepared, worktreeRoot });
//! await writeReplayedCommit({ gitPath, shadowPath, directory, attempt, commit, treeOid, parentOid, signingKey, globalArgs });
//! ```

/// Raw commit parsing, rewriting, identities, and dropped headers.
use super::path_bytes::os_from_path_bytes;
use super::replay_object::{RawCommit, dropped_replay_headers, identity_environment, rewrite_commit_object};
/// Shadow runs and the raw runner for a merge whose exit status matters.
use super::replay_shadow::run_shadow_checked;
use super::private_storage::write_private_file;
use super::shadow_git::shadow_request;
use super::transaction_git::{GitContext, GitFailure, GitRequest, run_git, run_git_checked};
/// Names and environment entries.
use std::ffi::OsString;
/// Filesystem paths.
use std::path::{Path, PathBuf};

/// `git merge-tree` exit status reporting conflicts; a tree ID still prints.
const MERGE_CONFLICT_EXIT: i32 = 1;

/// What:
///  The outcome of merging the prepared change onto the current target.
/// Why:
///   A clean merge yields a tree to rebuild a commit on; a conflict yields the paths that name the winning commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReplayMerge = { kind: 'merged'; treeOid } | { kind: 'conflict'; paths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayMerge {
    /// The change applies cleanly; the merged tree is in the shadow store.
    Merged {
        /// Merged tree.
        tree_oid: String,
    },
    /// The change conflicts with the target.
    Conflict {
        /// Conflicting paths in Git path byte order, each once, as the exact bytes Git reported.
        paths: Vec<Vec<u8>>,
    },
}

/// What:
///  A merge or rebuild failure:
///       a Git command failed, or the commit could not be rebuilt.
/// Why:
///   Replay reports each as `transaction-failed`, never as a conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(message);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayMergeError {
    /// A Git command failed, or `merge-tree` exited above `1`.
    Git(GitFailure),
    /// The prepared commit could not be rebuilt.
    Rebuild(String),
}

/// What:
///  The replay merge base:
///       the preparation base, or for an unborn branch a root commit of the empty tree written into the shadow.
/// Why:
///   Git 2.40 accepts only a commit as `--merge-base`, so an unborn base needs a commit over the empty tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await replayMergeBase({ gitPath, shadowPath, base, emptyTreeOid });
/// ```
pub fn replay_merge_base(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    base: Option<&str>,
    empty_tree_oid: &str,
) -> Result<String, ReplayMergeError> {
    if let Some(commit) = base {
        return Ok(String::from(commit));
    }
    let message: String = format!(
        "tree {empty_tree_oid}\nauthor cli-git <cli-git@localhost> 0 +0000\ncommitter cli-git <cli-git@localhost> 0 +0000\n\ncli-git replay base for an unborn branch\n"
    );
    let written = run_shadow_checked(
        context,
        shadow,
        cwd,
        &["hash-object", "-t", "commit", "-w", "--stdin"],
        Some(message.as_bytes()),
        None,
        &[],
    )
    .map_err(ReplayMergeError::Git)?;
    return Ok(String::from_utf8_lossy(&written.stdout).trim().to_string());
}

/// What:
///  Merges the prepared change onto the current target in the shadow:
///       the tree when clean, or the conflicted paths in byte order.
/// Why:
///   `git merge-tree --write-tree` gives the answer with exit `1` for a conflict and a tree ID in both cases;
///       any other non-zero exit is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mergeReplayTree({ gitPath, shadowPath, mergeBase, current, prepared, worktreeRoot });
/// ```
pub fn merge_replay_tree(
    context: &GitContext,
    shadow: &Path,
    worktree_root: &Path,
    merge_base: &str,
    current: &str,
    prepared: &str,
) -> Result<ReplayMerge, ReplayMergeError> {
    let merge_base_argument: String = format!("--merge-base={merge_base}");
    let request: GitRequest = shadow_request(
        shadow,
        worktree_root,
        &[
            "merge-tree",
            "--write-tree",
            "--name-only",
            "-z",
            merge_base_argument.as_str(),
            current,
            prepared,
        ],
    );
    let output = run_git(context, &request).map_err(|error| {
        return ReplayMergeError::Git(GitFailure(format!("git merge-tree could not start: {error}")));
    })?;
    let code: Option<i32> = output.code;
    if code != Some(0) && code != Some(MERGE_CONFLICT_EXIT) {
        return Err(ReplayMergeError::Git(GitFailure(format!(
            "git merge-tree could not replay {prepared} onto {current} (exit {code:?}): {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))));
    }
    // The first field is the tree; on a conflict the conflicted paths follow up to an empty record.
    // The paths stay bytes: a name that is not UTF-8 is reported with its exact bytes.
    let mut fields = output.stdout.split(|byte| { return *byte == 0; });
    let tree_oid: String = String::from_utf8_lossy(fields.next().unwrap_or_default()).into_owned();
    if code == Some(0) {
        return Ok(ReplayMerge::Merged { tree_oid });
    }
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for field in fields {
        if field.is_empty() {
            break;
        }
        if !paths.iter().any(|known| { return known.as_slice() == field; }) {
            paths.push(field.to_vec());
        }
    }
    paths.sort();
    return Ok(ReplayMerge::Conflict { paths });
}

/// What:
///  The first commit after the preparation base, in the landed history, that touches one of the paths;
///       the current target when none does.
/// Why:
///   A conflict names the commit it lost to, so the integrator can show what changed under the prepared commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await firstCommitTouching({ gitPath, cwd, base, current, paths });
/// ```
pub fn first_commit_touching(
    context: &GitContext,
    cwd: &Path,
    base: Option<&str>,
    current: &str,
    paths: &[Vec<u8>],
) -> Result<String, ReplayMergeError> {
    let exclusion: String = base.map(|oid| { return format!("^{oid}"); }).unwrap_or_default();
    let pathspecs: Vec<OsString> = paths
        .iter()
        .map(|path| {
            let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
            pathspec.extend_from_slice(path.as_slice());
            return os_from_path_bytes(pathspec.as_slice());
        })
        .collect();
    // `mut` lets the arguments be assembled in order.
    let mut arguments: Vec<&str> = vec!["rev-list", "--reverse", current];
    if base.is_some() {
        arguments.push(exclusion.as_str());
    }
    arguments.push("--");
    let mut request: GitRequest = GitRequest::new(cwd, &arguments);
    request.arguments.extend(pathspecs);
    request.environment = vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("0"))];
    let listed = run_git_checked(context, &request).map_err(ReplayMergeError::Git)?;
    // The listing holds object names, which are ASCII.
    let text: String = String::from_utf8_lossy(&listed.stdout).into_owned();
    return Ok(text
        .lines()
        .find(|line| { return !line.is_empty(); })
        .map(String::from)
        .unwrap_or_else(|| { return String::from(current); }));
}

/// What:
///  The replayed commit written to the shadow store, and the custom headers a signed rebuild dropped.
/// Why:
///   The integrator reports `commit-replayed` with the OID and `replay-headers-dropped` with the names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReplayedCommit = { oid: string; droppedHeaders: readonly string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayedCommit {
    /// Replayed commit.
    pub oid: String,
    /// Custom headers a signed rebuild dropped, empty for an unsigned rewrite.
    pub dropped_headers: Vec<String>,
}

/// What:
///  Writes the prepared commit onto a new tree and parent in the shadow store.
/// Why:
///   An unsigned commit keeps every byte except `tree` and `parent`; a signed one is re-signed under the same identities and dates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeReplayedCommit({ gitPath, shadowPath, directory, attempt, commit, treeOid, parentOid, signingKey, globalArgs });
/// ```
pub fn write_replayed_commit(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    directory: &Path,
    attempt: u64,
    commit: &RawCommit,
    tree_oid: &str,
    parent_oid: &str,
    signing_key: Option<&str>,
    global_arguments: &[String],
) -> Result<ReplayedCommit, ReplayMergeError> {
    if !commit.signed {
        let rewritten: Vec<u8> = rewrite_commit_object(commit, tree_oid, parent_oid);
        let written = run_shadow_checked(
            context,
            shadow,
            cwd,
            &["hash-object", "-t", "commit", "-w", "--stdin"],
            Some(&rewritten),
            None,
            &[],
        )
        .map_err(ReplayMergeError::Git)?;
        return Ok(ReplayedCommit {
            oid: String::from_utf8_lossy(&written.stdout).trim().to_string(),
            dropped_headers: Vec::new(),
        });
    }
    let message_path: PathBuf = directory.join(format!("replay-message-{attempt}"));
    write_private_file(message_path.as_path(), &commit.message)
        .map_err(|error| { return ReplayMergeError::Rebuild(format!("message file {}: {error}", message_path.display())); })?;
    let encoding: String = commit.encoding.clone().unwrap_or_else(|| { return String::from("utf8"); });
    let signed_key: String = format!("-S{}", signing_key.unwrap_or(""));
    let encoding_option: String = format!("i18n.commitEncoding={encoding}");
    let message_text: String = message_path.to_string_lossy().into_owned();
    // `mut` lets the caller's global options come first, as the incumbent passes them.
    let mut arguments: Vec<&str> = global_arguments.iter().map(String::as_str).collect();
    arguments.extend([
        "-c",
        encoding_option.as_str(),
        "commit-tree",
        tree_oid,
        "-p",
        parent_oid,
        signed_key.as_str(),
        "-F",
        message_text.as_str(),
    ]);
    let mut environment: Vec<(OsString, OsString)> = Vec::new();
    for role in ["author", "committer"] {
        let identity = identity_environment(commit, role).map_err(|error| { return ReplayMergeError::Rebuild(error.0); })?;
        for (name, value) in identity {
            environment.push((OsString::from(name), OsString::from(value)));
        }
    }
    let signed = run_shadow_checked(context, shadow, cwd, &arguments, None, None, &environment)
        .map_err(ReplayMergeError::Git)?;
    return Ok(ReplayedCommit {
        oid: String::from_utf8_lossy(&signed.stdout).trim().to_string(),
        dropped_headers: dropped_replay_headers(commit),
    });
}

#[cfg(test)]
#[path = "replay_merge_tests.rs"]
mod tests;
