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
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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
    return current_value_answered(run_git(git, &current_value_request(held)));
}

/// `current_value` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await currentValueAwaiting(git, held) // '' when the ref is missing, undefined when Git gave no answer
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the settlement face waits for the finish awaited faces (doc/planning/git-policy-cli-full-tokio.md)"))]
async fn current_value_awaiting(git: &GitContext, held: &Held) -> Option<String> {
    return current_value_answered(run_git_awaiting(git, &current_value_request(held)).await);
}

/// What:
///  The question `current_value` asks Git:
///  the commit the held batch's reference names now.
/// Why:
///  Both faces ask the one question,
/// so neither reads another reference or another repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: held.repository, arguments: ['rev-parse', '--verify', '--quiet', held.reference] };
/// ```
fn current_value_request(held: &Held) -> GitRequest {
    return GitRequest::new(held.repository.as_path(), &["rev-parse", "--verify", "--quiet", held.reference.as_str()]);
}

/// What:
///  What Git's answer to `current_value_request` names:
///  the commit,
///  an empty name for a reference that is not there,
///  and nothing when Git gave no answer.
/// Why:
///  Both faces read the answer through this one function,
/// so a reference that is gone and a question nobody answered
///  stay two different answers for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentValueAnswered(asked: GitOutput | Error): string | undefined;
/// ```
fn current_value_answered(asked: std::io::Result<GitOutput>) -> Option<String> {
    let output: GitOutput = asked.ok()?;
    if output.succeeded() {
        return Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
    }
    return if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() { Some(String::new()) } else { None };
}

/// The parents of a commit,
/// as the object store names them.
fn parents_of(git: &GitContext, repository: &Path, commit: &str) -> Option<Vec<String>> {
    return parents_answered(run_git(git, &parents_request(repository, commit)), commit);
}

/// `parents_of` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await parentsOfAwaiting(git, repository, commit)
/// ```
async fn parents_of_awaiting(git: &GitContext, repository: &Path, commit: &str) -> Option<Vec<String>> {
    return parents_answered(run_git_awaiting(git, &parents_request(repository, commit)).await, commit);
}

/// What:
///  The question `parents_of` asks Git:
///  the parents of one commit.
/// Why:
///  Both faces ask the one question,
/// so neither counts another commit's parents.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['rev-list', '--parents', '--max-count=1', commit] };
/// ```
fn parents_request(repository: &Path, commit: &str) -> GitRequest {
    return GitRequest::new(repository, &["rev-list", "--parents", "--max-count=1", commit]);
}

/// What:
///  The parents Git's answer to `parents_request` names,
/// and nothing when the answer is not about `commit`.
/// Why:
///  Both faces read the answer through this one function,
/// so a commit that cannot be read and an answer about another
///  commit are the same nothing for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parentsAnswered(asked: GitOutput | Error, commit: string): string[] | undefined;
/// ```
fn parents_answered(asked: std::io::Result<GitOutput>, commit: &str) -> Option<Vec<String>> {
    let output: GitOutput = asked.ok()?;
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
    let mut request: GitRequest = merge_tree_request(held, base, head, prepared);
    // No worktree is read, so the landed tip's `.gitattributes` is the one the merge applies.
    let process_environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(process_environment.as_slice(), git.overlay.as_slice());
    // The probe keeps none of the caller's options, so the repository the record names is carried here: a Git directory beside
    //  the work tree is found only when it is named, and a plain layout needs no argument at all
    //  (`repository_selection.rs`, `of_paths`).
    let repository: Vec<OsString> = crate::repository_selection::of_paths(held.git_dir.as_path(), held.repository.as_path());
    let probing: ContextRunner<'_> = ContextRunner { context: git, cwd: held.repository.as_path(), without_prefix: true, repository, object_directory: None, environment: Vec::new() };
    let probe: AttributeProbe<'_> = AttributeProbe { directory: held.repository.as_path(), global_prefix: &[], variables: variables.as_slice(), runner: &probing };
    request.environment.extend(attribute_tree_variables_on_the_runtime(head, &probe));
    let output: GitOutput = run_git(git, &request).map_err(git_did_not_start)?;
    return merged_answered(output, head, prepared);
}

/// What:
///  The question `merged_tree` asks Git:
///  the prepared commit's change merged onto `head` from `base`,
/// as one written tree.
/// Why:
///  Both faces ask the one question,
/// so neither merges from another base or onto another head.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: held.repository, arguments: ['merge-tree', '--write-tree', '-z', `--merge-base=${base}`, head, prepared] };
/// ```
fn merge_tree_request(held: &Held, base: &str, head: &str, prepared: &str) -> GitRequest {
    let merge_base: String = format!("--merge-base={base}");
    return GitRequest::new(held.repository.as_path(), &["merge-tree", "--write-tree", "-z", merge_base.as_str(), head, prepared]);
}

/// What:
///  What Git's answer to `merge_tree_request` came to:
///  a tree,
///  the paths that conflict,
///  or the reason nothing was written.
/// Why:
///  Both faces read the answer through this one function,
/// so a clean merge,
/// a conflict and a Git that refused are
///  the same three answers for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergedAnswered(output: GitOutput, head, prepared): Result<string | path[], string>;
/// ```
fn merged_answered(output: GitOutput, head: &str, prepared: &str) -> Result<Result<String, Vec<Vec<u8>>>, String> {
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
///  `merged_tree` with its attribute question awaited and its Git child awaited on the runtime.
/// Why:
///   A task of the runtime may neither enter it to wait for a child nor wait for a question the merge
/// itself awaits,
///  and the merge is the step a held commit is carried onto its branch by.
/// The attributes,
/// the base and head it merges from and
///  the refusals are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mergedTreeAwaiting(git, held, base, head, prepared)
/// ```
async fn merged_tree_awaiting(git: &GitContext, held: &Held, base: &str, head: &str, prepared: &str) -> Result<Result<String, Vec<Vec<u8>>>, String> {
    let mut request: GitRequest = merge_tree_request(held, base, head, prepared);
    // No worktree is read, so the landed tip's `.gitattributes` is the one the merge applies, and the question is the one the
    //  synchronous face asks, awaited instead of entered.
    let process_environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(process_environment.as_slice(), git.overlay.as_slice());
    let repository: Vec<OsString> = crate::repository_selection::of_paths(held.git_dir.as_path(), held.repository.as_path());
    let probing: ContextRunner<'_> = ContextRunner { context: git, cwd: held.repository.as_path(), without_prefix: true, repository, object_directory: None, environment: Vec::new() };
    let probe: AttributeProbe<'_> = AttributeProbe { directory: held.repository.as_path(), global_prefix: &[], variables: variables.as_slice(), runner: &probing };
    request.environment.extend(crate::merge_second_pass_tree::attribute_tree_variables(head, &probe).await);
    let output: GitOutput = run_git_awaiting(git, &request).await.map_err(git_did_not_start)?;
    return merged_answered(output, head, prepared);
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
    return raw_commit_answered(run_git(git, &raw_commit_request(repository, commit)));
}

/// `raw_commit` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rawCommitAwaiting(git, repository, commit)
/// ```
async fn raw_commit_awaiting(git: &GitContext, repository: &Path, commit: &str) -> Option<Vec<u8>> {
    return raw_commit_answered(run_git_awaiting(git, &raw_commit_request(repository, commit)).await);
}

/// What:
///  The question `raw_commit` asks Git:
///  the commit object `commit`,
///  as Git stored it.
/// Why:
///  Both faces ask the one question,
/// so the headers a commit made again keeps are the same bytes for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['cat-file', 'commit', commit] };
/// ```
fn raw_commit_request(repository: &Path, commit: &str) -> GitRequest {
    return GitRequest::new(repository, &["cat-file", "commit", commit]);
}

/// What:
///  The object Git's answer to `raw_commit_request` holds,
/// and nothing when Git refused it.
/// Why:
///  Both faces read the object through this one function,
/// so a commit that cannot be read is the same nothing for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rawCommitAnswered(asked: GitOutput | Error): Uint8Array | undefined;
/// ```
fn raw_commit_answered(asked: std::io::Result<GitOutput>) -> Option<Vec<u8>> {
    let output: GitOutput = asked.ok()?;
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

/// What:
///  The place one replay writes its message in,
///  owned so that a job of the blocking pool can take it.
/// Why:
///   The scratch directory of the repository and a directory inside it are storage,
/// and both are made before the
///  first commit is carried over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScratchForAReplay = { commonDir: string };
/// ```
struct ScratchForAReplay {
    /// The repository's common Git directory,
    /// which holds the scratch directory the replay works in.
    common_dir: PathBuf,
}

/// What:
///  The directory one replay writes its messages in,
/// or the words that say why it cannot be made.
/// Why:
///   The scratch directory of the repository and the temporary directory inside it are made once per replay,
/// and a
///  replay that cannot make them writes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replayScratch({ commonDir }) { try { return mkdtemp(join(prepareRepositoryScratch(commonDir))) } catch { return 'the repository\'s scratch directory cannot be written' } }
/// ```
fn replay_scratch(work: ScratchForAReplay) -> Result<tempfile::TempDir, String> {
    let Ok(scratch) = crate::scratch_location::prepare_repository_scratch(work.common_dir.as_path()) else {
        return Err(String::from("the repository's scratch directory cannot be written"));
    };
    return match tempfile::tempdir_in(scratch.as_path()) {
        Ok(folder) => Ok(folder),
        Err(_) => Err(String::from("a temporary directory cannot be made in the repository's scratch directory")),
    };
}

/// What:
///  `onto` with its storage steps jobs of the blocking pool and every Git child awaited on the runtime.
/// Why:
///   A task of the runtime may neither wait on a filesystem nor enter the runtime to wait for a child,
/// and a
///  settlement that is a task carries a held batch onto its branch through this face.
/// The order of the commits,
/// the base each one
///  merges from,
/// the parent each one takes and every sentence of a refusal are the synchronous face's,
/// pinned by
///  `the_awaited_replay_carries_the_batch_what_the_synchronous_one_does`.
/// The scratch directory keeps no ceiling:
/// a directory
///  abandoned half-made is one nothing reads,
/// and the job of the pool holds its place until it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let head = current; let base = held.old;
/// for (const prepared of held.commits) { head = await carryOneAwaiting(prepared, base, head); base = prepared }
/// ```
pub async fn onto_awaiting(git: &GitContext, held: &Held, current: &str, common_dir: &Path) -> Replay {
    let work: ScratchForAReplay = ScratchForAReplay { common_dir: common_dir.to_path_buf() };
    let made: Result<tempfile::TempDir, String> = match run_blocking_job_within_the_budget(move || { return replay_scratch(work); }).await {
        Ok(made) => made,
        // The place was refused, so nothing was made and nothing was written.
        Err(error) => Err(error.to_string()),
    };
    let Ok(message_folder) = made else {
        return Replay::Cannot { because: String::from("the repository's scratch directory cannot be written") };
    };
    let site: BuildSite<'_> = BuildSite { context: git, cwd: held.repository.as_path(), scratch: message_folder.path(), signing: None };
    let mut head: String = current.to_string();
    let mut base: String = held.old.clone();
    let mut written: Vec<String> = Vec::new();
    for prepared in &held.commits {
        let Some(parents) = parents_of_awaiting(git, held.repository.as_path(), prepared.as_str()).await else {
            return Replay::Cannot { because: format!("the commit {prepared} cannot be read") };
        };
        match parents.len() {
            1 => {}
            0 => return Replay::Cannot { because: format!("the commit {prepared} has no parent, so the branch it was made on was created while the lock stood") },
            many => {
                return Replay::Cannot { because: format!("the commit {prepared} has {many} parents, and a commit that concludes a merge cannot be carried onto another head") };
            }
        }
        let merged: String = match merged_tree_awaiting(git, held, base.as_str(), head.as_str(), prepared.as_str()).await {
            Err(because) => return Replay::Cannot { because },
            Ok(Ok(merged)) => merged,
            Ok(Err(paths)) => return Replay::Conflict { paths },
        };
        let Some(raw) = raw_commit_awaiting(git, held.repository.as_path(), prepared.as_str()).await else {
            return Replay::Cannot { because: format!("the commit {prepared} cannot be read") };
        };
        let Some(fields) = stored_commit_fields(raw.as_slice()) else {
            return Replay::Cannot { because: format!("the commit {prepared} cannot be read as a commit") };
        };
        if fields.is_signed() {
            return Replay::Cannot { because: format!("the commit {prepared} is signed, and a commit made again would have to be signed again by a program that may ask for a key") };
        }
        match crate::service_amend_replayed::rebuilt_onto_awaiting(&fields, &site, merged.as_str(), head.as_str()).await {
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

/// `value_now` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await valueNowAwaiting(git, held)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the settlement face waits for the finish awaited faces (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn value_now_awaiting(git: &GitContext, held: &Held) -> Option<String> {
    return current_value_awaiting(git, held).await;
}

/// Controls for the awaited faces of this replay stay out of the release executable.
#[cfg(test)]
#[path = "held_commit_replay_awaiting_tests.rs"]
mod awaiting_tests;
