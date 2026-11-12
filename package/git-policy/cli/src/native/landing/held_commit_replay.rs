//! What:
//!  The change of a held commit put onto the branch as it is now,
//!  when that branch moved while the lock stood:
//!  the tree `git merge-tree` writes,
//!  and the commit made again on the branch's tip.
//! Why:
//!  The owner's rule for a commit that lands on a branch another session moved is the same rule here
//!  (`doc/decision/cli-git-concurrent-commits.md`,
//!  "Serial landing"),
//!  so this is the part of that rule that needs no client:
//!  the change of each prepared commit merges onto the tip from the base it was prepared on,
//!  and the commit is written again with the same authorship,
//!  committer and message,
//!  its one parent the tip as it is now.
//!  Nothing here runs a policy,
//!  a hook or the word-level second pass:
//!  the command that made the commit had ended,
//!  so a path that conflicts leaves the commit held and is named
//!  (`held_commit_words::conflicts`).
//!  A commit made again would have to be signed again by a program that may ask the person for a key,
//!  which is a wait no command may start on its own,
//!  so a signed commit is left held too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const replay = await replayOnto(held, current); // { kind: 'replayed', commit } | { kind: 'conflict', paths } | { kind: 'cannot', because }
//! ```

use crate::child_environment::environment_with_overlay;
use crate::held_commit_record::Held;
use crate::merge_second_pass::conflicts_of_merge_tree;
use crate::merge_second_pass::ContextRunner;
use crate::merge_second_pass_tree::{AttributeProbe, attribute_tree_variables_on_the_runtime};
use crate::service_amend_replayed::{BuildSite, rebuilt_onto, stored_commit_fields};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::ffi::OsString;
use std::path::Path;

/// `git merge-tree` exit status reporting conflicts;
/// a tree ID still prints.
const MERGE_CONFLICT_EXIT: i32 = 1;

/// What a Git that could not be started is said to be,
/// in one sentence the callers share.
fn git_did_not_start(error: std::io::Error) -> String {
    return format!("Git could not be started ({error})");
}

/// The exit status `git rev-parse --verify --quiet` gives for a name that is no object.
const NAME_IS_NO_OBJECT: i32 = 1;

/// What became of the change of a held commit.
#[derive(Debug, Eq, PartialEq)]
pub enum Replay {
    /// The change was merged onto the tip and written as a commit whose one parent is that tip.
    Replayed {
        /// The commits made again,
        /// each parented on the one before,
        /// the last of them the tip.
        commits: Vec<String>,
    },
    /// The change does not combine with the tip on these paths,
    /// in Git's path byte order.
    Conflict {
        /// The paths that conflict.
        paths: Vec<Vec<u8>>,
    },
    /// The change cannot be carried onto the tip at all,
    /// for the reason given.
    Cannot {
        /// Why nothing was written.
        because: String,
    },
}

/// What:
///  The branch as it is now,
///  its commit name,
///  empty when Git says for certain that the reference is not there.
/// Why:
///   A held commit's branch can move and can be deleted while the lock stands,
///  and only Git's own answer about the name decides which happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// revParseQuiet(reference) // '' when the ref is missing, undefined when Git gave no answer
/// ```
fn current_value(git: &GitContext, held: &Held) -> Option<String> {
    let request: GitRequest = GitRequest::new(held.repository.as_path(), held.reference.value_question().as_slice());
    let output: GitOutput = run_git(git, &request).ok()?;
    if output.succeeded() {
        return Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
    }
    return if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() { Some(String::new()) } else { None };
}

/// The parents of a commit,
/// as the object store names them.
fn parents_of(git: &GitContext, repository: &Path, commit: &str) -> Option<Vec<String>> {
    let request: GitRequest = GitRequest::new(repository, &["rev-list", "--parents", "--max-count=1", commit]);
    let output: GitOutput = run_git(git, &request).ok()?;
    if !output.succeeded() {
        return None;
    }
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let mut words = text.split_whitespace();
    if words.next() != Some(commit) {
        return None;
    }
    return Some(words.map(str::to_string).collect());
}

/// What:
///  The tree of the prepared commit's change merged onto `head` from `base`.
/// Why:
///  `git merge-tree --write-tree` is the one primitive that merges from objects alone,
///  and its exit status says whether the two sides met cleanly.
///  The attributes of the landed tip decide the merge drivers,
///  so a `union` rule merges as it does for a commit that lost a landing race.
///  The merged tree is written into the repository's own object store and is reachable from the commit
///  made from it at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const merged = await mergeTree({ base, head, prepared }); // { tree } | { paths } | { because }
/// ```
fn merged_tree(git: &GitContext, held: &Held, base: &str, head: &str, prepared: &str) -> Result<Result<String, Vec<Vec<u8>>>, String> {
    let merge_base: String = format!("--merge-base={base}");
    let mut request: GitRequest = GitRequest::new(held.repository.as_path(), &["merge-tree", "--write-tree", "-z", merge_base.as_str(), head, prepared]);
    // No worktree is read, so the landed tip's `.gitattributes` is the one the merge applies.
    let process_environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(process_environment.as_slice(), git.overlay.as_slice());
    // The probe keeps none of the caller's options, so the repository the record names is carried here: a Git directory beside
    //  the work tree is found only when it is named, and a plain layout needs no argument at all
    //  (`repository_selection.rs`, `of_paths`).
    let repository: Vec<OsString> = crate::repository_selection::of_paths(held.git_dir.as_path(), held.repository.as_path());
    let probing: ContextRunner = ContextRunner { context: git, cwd: held.repository.as_path(), without_prefix: true, repository, object_directory: None, environment: Vec::new() };
    let probe: AttributeProbe = AttributeProbe { directory: held.repository.as_path(), global_prefix: &[], variables: variables.as_slice(), runner: &probing };
    request.environment.extend(attribute_tree_variables_on_the_runtime(head, &probe));
    let output: GitOutput = run_git(git, &request).map_err(git_did_not_start)?;
    let (tree, conflicts) = conflicts_of_merge_tree(output.stdout.as_slice());
    return match output.code {
        Some(0) => Ok(Ok(tree)),
        Some(MERGE_CONFLICT_EXIT) => {
            let mut paths: Vec<Vec<u8>> = Vec::new();
            for conflict in &conflicts {
                paths.push(conflict.path.clone());
            }
            paths.sort();
            paths.dedup();
            Ok(Err(paths))
        }
        other => Err(format!("Git could not merge {prepared} onto {head} (exit {other:?}): {}", String::from_utf8_lossy(output.stderr.as_slice()).trim())),
    };
}

/// What:
///  The raw commit object `commit`,
///  as Git stored it.
/// Why:
///   The commit made again keeps every header line the prepared one carries,
///  and the object's bytes are the only record of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git('cat-file', 'commit', commit).stdout
/// ```
fn raw_commit(git: &GitContext, repository: &Path, commit: &str) -> Option<Vec<u8>> {
    let request: GitRequest = GitRequest::new(repository, &["cat-file", "commit", commit]);
    let output: GitOutput = run_git(git, &request).ok()?;
    return if output.succeeded() { Some(output.stdout) } else { None };
}

/// What:
///  The change of every commit of the held batch onto the branch as it is now.
/// Why:
///   A batch is landed as one chain,
///  so its commits are carried over one at a time,
///  each merged from the commit it was prepared on and parented on the one before,
///  and the branch moves once to the last of them
///  (`held_commit_settle.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let head = current; let base = held.old;
/// for (const prepared of held.commits) { head = await carryOne(prepared, base, head); base = prepared }
/// ```
pub fn onto(git: &GitContext, held: &Held, current: &str, common_dir: &Path) -> Replay {
    let Ok(scratch) = crate::scratch_location::prepare_repository_scratch(common_dir) else {
        return Replay::Cannot { because: String::from("the repository's scratch directory cannot be written") };
    };
    let Ok(message_folder) = tempfile::tempdir_in(scratch.as_path()) else {
        return Replay::Cannot { because: String::from("a temporary directory cannot be made in the repository's scratch directory") };
    };
    let site: BuildSite<'_> = BuildSite { context: git, cwd: held.repository.as_path(), scratch: message_folder.path(), signing: None };
    let mut head: String = current.to_string();
    let mut base: String = held.old.clone();
    let mut written: Vec<String> = Vec::new();
    for prepared in &held.commits {
        let Some(parents) = parents_of(git, held.repository.as_path(), prepared.as_str()) else {
            return Replay::Cannot { because: format!("the commit {prepared} cannot be read") };
        };
        match parents.len() {
            1 => {}
            0 => return Replay::Cannot { because: format!("the commit {prepared} has no parent, so the branch it was made on was created while the lock stood") },
            many => {
                return Replay::Cannot { because: format!("the commit {prepared} has {many} parents, and a commit that concludes a merge cannot be carried onto another head") };
            }
        }
        let merged: String = match merged_tree(git, held, base.as_str(), head.as_str(), prepared.as_str()) {
            Err(because) => return Replay::Cannot { because },
            Ok(Ok(merged)) => merged,
            Ok(Err(paths)) => return Replay::Conflict { paths },
        };
        let Some(raw) = raw_commit(git, held.repository.as_path(), prepared.as_str()) else {
            return Replay::Cannot { because: format!("the commit {prepared} cannot be read") };
        };
        let Some(fields) = stored_commit_fields(raw.as_slice()) else {
            return Replay::Cannot { because: format!("the commit {prepared} cannot be read as a commit") };
        };
        if fields.is_signed() {
            return Replay::Cannot { because: format!("the commit {prepared} is signed, and a commit made again would have to be signed again by a program that may ask for a key") };
        }
        match rebuilt_onto(&fields, &site, merged.as_str(), head.as_str()) {
            Ok(rebuilt) => {
                written.push(rebuilt.clone());
                head = rebuilt;
                base = prepared.clone();
            }
            Err(_stop) => return Replay::Cannot { because: format!("the commit made from {prepared} could not be written") },
        }
    }
    return Replay::Replayed { commits: written };
}

/// The branch value the held commit expects,
/// for the caller that compares it.
pub(crate) fn value_now(git: &GitContext, held: &Held) -> Option<String> {
    return current_value(git, held);
}
