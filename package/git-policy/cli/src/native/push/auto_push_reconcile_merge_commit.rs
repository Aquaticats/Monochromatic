//! What:
//!  The merge commit of a reconcile:
//! the merge of the remote tip and the local tip computed in the object store (with the
//!  word-level second pass for conflicts) and committed without a worktree or an index.
//! Why:
//!  The worktree and its index stay untouched while merging,
//! so another consumer's uncommitted files cannot block or be
//!  hurt by it.
//! The loop that calls this is in `auto_push_reconcile.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const committed = await commitMerge({ git, attributed, local, remote }); // { oid, merged } | conflicting paths
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, git_command, git_output, run_query,
};
use crate::diagnostic_log::debug;
use crate::merge_second_pass::{GitReply, GitRunner, Merged, conflicts_of_merge_tree};
use crate::merge_second_pass_tree::{AttributeProbe, attribute_tree_variables, report, resolve_tree};
use crate::path_bytes::path_message_text;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
/// The coordinator's Git as a runner for the second pass:
/// its commands,
/// its environment,
/// an optional stdin and index file.
pub(crate) struct PushRunner<'a> {
    /// The coordinator's Git.
    pub(crate) git: &'a PushGit,
}

/// Starting Git the way the coordinator's queries do,
/// with the input a second-pass command needs.
impl GitRunner for PushRunner<'_> {
    fn run(&self, arguments: &[&str], input: Option<&[u8]>, index_file: Option<&Path>) -> Result<GitReply, String> {
        let mut command = git_command(self.git, &argv(arguments));
        if let Some(index) = index_file {
            command.env("GIT_INDEX_FILE", index);
        }
        command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(started) => started,
            Err(error) => return Err(format!("git {} could not start: {error}", arguments.join(" "))),
        };
        // The input goes in from a thread of its own, so a command that answers while it reads cannot block on a full pipe.
        let pipe = child.stdin.take();
        let bytes: Vec<u8> = input.map(<[u8]>::to_vec).unwrap_or_default();
        let writer = match crate::scoped_start::spawn_named("cli-git-second-pass-input", move || {
            if let Some(mut open) = pipe
                && let Err(error) = open.write_all(bytes.as_slice())
            {
                debug(TAG, &format!("the input of a second-pass command was not taken: {error}"));
            }
        }) {
            Ok(started) => started,
            // Writing the input here could block on a full pipe while Git waits for its output to be read; the command is stopped instead.
            Err(error) => {
                let _stopped = child.kill();
                let _reaped = child.wait();
                return Err(format!("no thread could be started to give git {} its input: {error}", arguments.join(" ")));
            }
        };
        let output = match child.wait_with_output() {
            Ok(finished) => finished,
            Err(error) => return Err(format!("git {} did not finish: {error}", arguments.join(" "))),
        };
        if writer.join().is_err() {
            return Err(format!("the input thread of git {} ended abnormally", arguments.join(" ")));
        }
        return Ok(GitReply { code: output.status.code(), stdout: output.stdout, stderr: output.stderr });
    }
}

/// What the merge of two tips came to.
pub(crate) struct CommittedMerge {
    /// The merge commit.
    pub(crate) oid: String,
    /// The files the word pass merged,
    /// listed in the commit message and in the report.
    pub(crate) merged: Vec<Merged>,
}

/// What:
///  Commit the merge of two tips from objects alone and return its OID,
/// or the paths that conflict.
/// Why:
///   `merge-tree --write-tree` exits `1` for a conflicted merge and lists the paths;
/// nothing is written to a worktree.
///   The conflicts get the word-level second pass,
/// and the merge is committed only when every conflicted file resolved;
///  the commit message then lists every merged part,
/// so the history keeps what was combined.
/// The attributes of the local
///  tip apply,
/// because no worktree is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ local, remote, message }): Promise<{ oid, merged } | { conflicts }>;
/// ```
pub(crate) fn commit_merge(
    git: &PushGit,
    local: &str,
    remote: &str,
    message: &str,
) -> Result<Result<CommittedMerge, Vec<String>>, SingleFlightError> {
    let probe = AttributeProbe { directory: git.cwd.as_path(), global_prefix: &[], variables: git.environment.as_slice(), runner: &PushRunner { git } };
    let attributed = PushGit {
        real_git: git.real_git.clone(),
        cwd: git.cwd.clone(),
        environment: git.environment.iter().cloned().chain(attribute_tree_variables(local, &probe)).collect(),
        lfs: git.lfs,
    };
    let runner = PushRunner { git: &attributed };
    let merge_arguments = [
        "merge-tree",
        "--write-tree",
        "--no-messages",
        "-z",
        "--allow-unrelated-histories",
        local,
        remote,
    ];
    let merged = match runner.run(&merge_arguments, None, None) {
        Ok(reply) => reply,
        Err(reason) => {
            return Err(SingleFlightError::Git {
                arguments: format!("merge-tree --write-tree {local} {remote}"),
                code: None,
                stderr: reason,
            });
        }
    };
    let (mut tree, conflicts) = conflicts_of_merge_tree(&merged.stdout);
    let mut second_pass_files: Vec<Merged> = Vec::new();
    if merged.code == Some(1) {
        let common = common_dir_of(git)?;
        // The second pass reads paths from the top of the worktree, whatever directory the commit landed in.
        let top = PathBuf::from(git_output(git, &argv(&["rev-parse", "--show-toplevel"]))?);
        let from_top = PushGit {
            real_git: attributed.real_git.clone(),
            cwd: top,
            environment: attributed.environment.clone(),
            lfs: attributed.lfs,
        };
        match resolve_tree(&PushRunner { git: &from_top }, common.as_path(), &tree, &conflicts) {
            Some(resolved) => {
                tree = resolved.tree;
                second_pass_files = resolved.merged;
            }
            None => {
                let mut paths: Vec<String> = Vec::new();
                for conflict in &conflicts {
                    paths.push(path_message_text(&conflict.path));
                }
                return Ok(Err(paths));
            }
        }
    } else if merged.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("merge-tree --write-tree {local} {remote}"),
            code: merged.code,
            stderr: String::from_utf8_lossy(&merged.stderr).into_owned(),
        });
    }
    let mut full_message = message.to_owned();
    if !second_pass_files.is_empty() {
        full_message.push_str("\n\n");
        full_message.push_str(report("", &second_pass_files).join("\n").as_str());
    }
    let mut commit_tree = vec!["commit-tree"];
    if signs_commits(git)? {
        commit_tree.push("-S");
    }
    commit_tree.extend(["-p", local, "-p", remote, "-m", &full_message, &tree]);
    let oid = git_output(git, &argv(&commit_tree))?;
    return Ok(Ok(CommittedMerge { oid, merged: second_pass_files }));
}

/// The repository's common Git directory,
/// where the scratch the second pass needs lives.
pub(crate) fn common_dir_of(git: &PushGit) -> Result<PathBuf, SingleFlightError> {
    let answered = git_output(git, &argv(&["rev-parse", "--git-common-dir"]))?;
    return Ok(git.cwd.join(answered));
}

/// What:
///  Whether `commit.gpgsign` asks for signed commits.
/// Why:
///   `git commit-tree` signs only when given `-S` and never reads `commit.gpgsign`,
/// so the merge would be the one
///  unsigned commit of a signed branch,
/// which a remote that requires verified signatures rejects.
///   An unset key is `git config` exit `1`;
/// any other failure is reported like every failed command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function signsCommits(git): Promise<boolean>;
/// ```
pub(crate) fn signs_commits(git: &PushGit) -> Result<bool, SingleFlightError> {
    let asked = run_query(
        git,
        &argv(&["config", "--type=bool", "--get", "commit.gpgsign"]),
    )?;
    if asked.code == Some(1) {
        return Ok(false);
    }
    if asked.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("config --type=bool --get commit.gpgsign"),
            code: asked.code,
            stderr: String::from_utf8_lossy(&asked.stderr).into_owned(),
        });
    }
    return Ok(asked.stdout.trim_ascii() == b"true");
}
