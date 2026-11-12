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
    PushGit, SingleFlightError, git_command, git_output_awaiting, run_query_awaiting,
};
/// The synchronous faces of the two single questions,
/// which only the control that compares the two faces still asks.
#[cfg(test)]
use crate::auto_push_single_flight::{git_output, run_query};
use crate::bounded_push::{MAX_PUSH_OUTPUT, STOP_GRACE};
use crate::bounded_query::{QueryOutput, query_window, run_bounded_git_awaiting};
use crate::merge_second_pass::{GitReply, GitRunner, LARGEST_STAGE_BYTES, Merged, conflicts_of_merge_tree};
use crate::merge_second_pass_tree::{AttributeProbe, attribute_tree_variables, report, resolve_tree};
use crate::path_bytes::path_message_text;
use crate::push_deadline_and_patience_settings::merge_ceiling;
use std::path::{Path, PathBuf};
use crate::auto_push_reconcile_basics_and_limits::argv;
/// The coordinator's Git as a runner for the second pass:
/// its commands,
/// its environment,
/// an optional stdin and index file.
pub(crate) struct PushRunner<'a> {
    /// The coordinator's Git.
    pub(crate) git: &'a PushGit,
}

/// The bytes one `git cat-file --batch` header may take:
/// the object name,
/// its kind,
/// the size and the line feed.
const CAT_FILE_HEADER_BYTES: u64 = 128;

/// The most bytes the content of one `merge-file` answer of the second pass may take:
/// the three stages it is given,
/// and the text Git settled around its regions.
/// Why:
///   A stage the pass takes is at most `LARGEST_STAGE_BYTES`,
/// and its answer holds the text Git settled plus the three sides of every region;
/// each line of a stage lands in the answer once,
/// either in the settled text or in a region's side,
/// so three stages hold the content and the fourth is the room a line can shift between them.
const MERGE_ANSWER_CONTENT_BYTES: u64 = 4 * LARGEST_STAGE_BYTES;

/// The most bytes the marker lines of one `merge-file` answer of the second pass may take:
/// eight stages,
/// which is about four hundred thousand regions.
/// Why:
///   Each region adds four marker lines,
/// which with the pass's labels are fifty,
/// fifty,
/// fifty-one and eight bytes for `ours-` and a 36-character UUID;
/// the number of regions is bounded by the lines of the stages and never by a margin of their bytes,
/// so a triple of near-limit stages with a conflict every other line holds a hundred and three thousand regions
/// and prints a 37.9 MB answer although its content is under four stages
/// (measured 2026-11-11 with Git 2.56.0:
/// stages of 7,605,777 bytes each,
/// 103,000 regions,
/// 37,871,557 bytes of answer;
/// cross-vendor review of round 198,
/// finding 2).
/// An answer is held whole and then cut,
/// so the answer of a merge at this ceiling costs its own bytes twice,
/// about 190 MB,
/// plus the merged file and its parts,
/// under the 352 MB the pass's own word merge of three 8 MiB texts already costs
/// (`merge_second_pass.rs`).
/// This ceiling is a memory backstop and not the bound of what eligible stages can print:
/// marker bytes grow with the lines of the stages and not with their bytes,
/// so a stage filled with two-byte lines holds more regions,
/// and more marker lines,
/// than any ceiling can take,
/// and such an answer is refused and the path stays as Git left it,
/// for the person to resolve.
/// A merge this ceiling newly admits costs the merge its own time in Git before the pass reads anything:
/// `merge-tree` took 192 seconds on the fixture above in one run under a 3-CPU scope on a host at load 66 to 75,
/// so the 300-second ceiling of the merge step is what ends the slowest of them.
const MERGE_ANSWER_MARKER_BYTES: u64 = 8 * LARGEST_STAGE_BYTES;

/// The most bytes one `merge-file` of the second pass may print:
/// its content and its marker lines.
const MERGE_FILE_ANSWER_BYTES: u64 = MERGE_ANSWER_CONTENT_BYTES + MERGE_ANSWER_MARKER_BYTES;

/// What:
///  The largest answer one command of the second pass may print.
/// Why:
///   `cat-file --batch` prints one blob per name it was given and a header for each,
/// and no stage the pass merges is larger than its own limit,
/// so the
///  answer for the names it asks for is bounded and a larger one would be refused after being read:
/// the ceiling stops the command
///  instead of holding a blob the pass would not take.
/// `merge-file --stdout` prints the merge of three stages,
/// so its answer is larger than each of them
/// and the push report's cap would refuse a merge the pass resolves.
/// Every other command of the pass prints a line or nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cap = (args, input) => args[0] === 'cat-file' ? Math.max(MAX_PUSH_OUTPUT, names * (LARGEST_STAGE_BYTES + 128)) : args[0] === 'merge-file' ? MERGE_FILE_ANSWER_BYTES : MAX_PUSH_OUTPUT;
/// ```
fn answer_cap(arguments: &[&str], input: Option<&[u8]>) -> u64 {
    match arguments.first() {
        // The names `cat-file` asks for are counted from its input; every other command gets a fixed ceiling.
        Some(&"cat-file") => {
            let mut asked: u64 = 0;
            for byte in input.unwrap_or_default() {
                if *byte == b'\n' {
                    asked += 1;
                }
            }
            return asked.saturating_mul(LARGEST_STAGE_BYTES.saturating_add(CAT_FILE_HEADER_BYTES)).max(MAX_PUSH_OUTPUT);
        }
        Some(&"merge-file") => return MERGE_FILE_ANSWER_BYTES,
        _ => return MAX_PUSH_OUTPUT,
    }
}

/// What:
///  Run a command of the second pass through the same bounded wait as a query,
/// with its input on the standard input,
/// an answer ceiling of one stage per name it asks for,
/// and a total ceiling on the step.
/// Why:
///   These children are not a person's:
/// the reconcile runs them in the object store of a repository that another program may have wedged,
///  and a `merge-tree` or a merge driver that never returns holds the push worker,
/// its claim and the drain of that repository for good.
/// The meter,
/// the stop of the service and `STOP_GRACE` are the ones the push and its queries use,
/// so a stuck command ends and is named like a stopped query;
///  a driver that keeps working without returning is ended by the total ceiling alone,
/// because it shows progress in every sample the
///  stall window looks at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, stdout, stderr } = await runBoundedGitAwaiting({ command, arguments, windowMs, ceilingMs, graceMs, environment, input, cap });
/// ```
impl GitRunner for PushRunner<'_> {
    fn run<'a>(
        &'a self,
        arguments: &'a [&'a str],
        input: Option<&'a [u8]>,
        index_file: Option<&'a Path>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GitReply, String>> + Send + 'a>> {
        return Box::pin(async move {
            let mut command = git_command(self.git, &argv(arguments));
            if let Some(index) = index_file {
                command.env("GIT_INDEX_FILE", index);
            }
            let shown: String = arguments.join(" ");
            let cap: u64 = answer_cap(arguments, input);
            let ceiling: Option<std::time::Duration> = Some(merge_ceiling(self.git.environment.as_slice()));
            let output: QueryOutput = match run_bounded_git_awaiting(
                command,
                shown.as_str(),
                query_window(self.git.environment.as_slice()),
                ceiling,
                STOP_GRACE,
                self.git.environment.as_slice(),
                input.map(<[u8]>::to_vec),
                cap,
            )
            .await
            {
                Ok(output) => output,
                Err(refusal) => return Err(could_not_run(shown.as_str(), refusal)),
            };
            return Ok(GitReply { code: output.code, stdout: output.stdout, stderr: output.stderr });
        });
    }
}

/// The words of a command that could not be run at all,
/// naming the command and the reason.
fn could_not_run(shown: &str, refusal: SingleFlightError) -> String {
    return format!("git {shown} could not be run: {refusal}");
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
/// Why a control still calls it:
///   The loop merges and commits through the awaited face (`commit_merge_awaiting`),
/// and the control that compares the two faces asks both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ local, remote, message }): Promise<{ oid, merged } | { conflicts }>;
/// ```
#[cfg(test)]
pub(crate) fn commit_merge(
    git: &PushGit,
    local: &str,
    remote: &str,
    message: &str,
) -> Result<Result<CommittedMerge, Vec<String>>, SingleFlightError> {
    return match crate::lazy_runtime::run_on_runtime(commit_merge_awaiting(git, local, remote, message)) {
        Ok(found) => found,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(refusal.to_string()))),
    };
}

/// What:
///  The same merge commit,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's reconcile merges and commits as a task,
/// and every read it makes here enters the runtime on a synchronous face and is refused from there.
/// The second pass over the conflicts is a task's work too
/// (`merge_second_pass_git_runner.rs`),
/// so this body awaits it whole.
/// The attributes,
/// the message,
/// the parents and the signing decision are the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const committed = await commitMergeOnTheRuntime({ local, remote, message });
/// ```
pub(crate) async fn commit_merge_awaiting(
    git: &PushGit,
    local: &str,
    remote: &str,
    message: &str,
) -> Result<Result<CommittedMerge, Vec<String>>, SingleFlightError> {
    let probe = AttributeProbe { directory: git.cwd.as_path(), global_prefix: &[], variables: git.environment.as_slice(), runner: &PushRunner { git } };
    let attributed = PushGit {
        real_git: git.real_git.clone(),
        cwd: git.cwd.clone(),
        environment: git.environment.iter().cloned().chain(attribute_tree_variables(local, &probe).await).collect(),
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
    let merged = match runner.run(&merge_arguments, None, None).await {
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
        let common = common_dir_of_awaiting(git).await?;
        // The second pass reads paths from the top of the worktree, whatever directory the commit landed in.
        let top = PathBuf::from(git_output_awaiting(git, &argv(&["rev-parse", "--show-toplevel"])).await?);
        let from_top = PushGit {
            real_git: attributed.real_git.clone(),
            cwd: top,
            environment: attributed.environment.clone(),
            lfs: attributed.lfs,
        };
        match resolve_tree(&PushRunner { git: &from_top }, common.as_path(), &tree, &conflicts, &probe).await {
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
    let mut commit_tree: Vec<String> = vec![String::from("commit-tree")];
    if signs_commits_awaiting(git).await? {
        commit_tree.push(String::from("-S"));
    }
    for word in ["-p", local, "-p", remote, "-F", "-", tree.as_str()] {
        commit_tree.push(String::from(word));
    }
    // What: the message goes on the standard input, not in `-m`,
    // and it ends with a line feed the way `-m` made it end.
    // Why: the kernel bounds one argument at about 128 KiB, and a merge of many parts lists one line per part
    // (`report`), so a merge whose every region resolved failed its commit with "Argument list too long" instead of
    // publishing (cross-vendor review of round 198, finding 2). `-F -` reads the same bytes, the runner feeds them,
    // and the failure names the command without them; `-m` completes the message's last line (`strbuf_complete_line`)
    // and `-F` does not, so the line feed is added here to keep the same commit message.
    if !full_message.ends_with('\n') {
        full_message.push('\n');
    }
    let shown: Vec<&str> = commit_tree.iter().map(String::as_str).collect();
    let description: String = format!("commit-tree -p {local} -p {remote} -F - {tree}");
    let created: GitReply = match runner.run(shown.as_slice(), Some(full_message.as_bytes()), None).await {
        Ok(reply) => reply,
        Err(reason) => {
            return Err(SingleFlightError::Git {
                arguments: description,
                code: None,
                stderr: reason,
            });
        }
    };
    if created.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: description,
            code: created.code,
            stderr: String::from_utf8_lossy(&created.stderr).into_owned(),
        });
    }
    let oid: String = String::from_utf8_lossy(&created.stdout).trim().to_owned();
    return Ok(Ok(CommittedMerge { oid, merged: second_pass_files }));
}

/// The repository's common Git directory,
/// where the scratch the second pass needs lives.
/// Why a control still calls it:
///   The merge commit awaits this read,
/// and the control that compares the two faces asks both.
#[cfg(test)]
pub(crate) fn common_dir_of(git: &PushGit) -> Result<PathBuf, SingleFlightError> {
    let answered = git_output(git, &argv(&["rev-parse", "--git-common-dir"]))?;
    return Ok(git.cwd.join(answered));
}

/// What:
///  The same common directory,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's reconcile reads where the repository's common directory is as a task,
/// and the synchronous query enters the runtime and is refused from there.
/// The join with the working directory is the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.join(git.cwd, await gitOutput(['rev-parse', '--git-common-dir']));
/// ```
pub async fn common_dir_of_awaiting(git: &PushGit) -> Result<PathBuf, SingleFlightError> {
    let answered = git_output_awaiting(git, &argv(&["rev-parse", "--git-common-dir"])).await?;
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
#[cfg(test)]
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

/// What:
///  The same question,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's reconcile asks whether the merge it publishes must carry a signature as a task,
/// and the synchronous query enters the runtime and is refused from there.
/// The exit rule and the reading of the value are the synchronous face's,
/// so a merge is signed or not for the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await signsCommitsOnTheRuntime(git): Promise<boolean>;
/// ```
pub async fn signs_commits_awaiting(git: &PushGit) -> Result<bool, SingleFlightError> {
    let asked = run_query_awaiting(
        git,
        &argv(&["config", "--type=bool", "--get", "commit.gpgsign"]),
    )
    .await?;
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

/// The ceilings of the second pass's commands,
/// and the answer they cover.
#[cfg(test)]
#[path = "auto_push_reconcile_merge_commit_tests.rs"]
mod tests;
