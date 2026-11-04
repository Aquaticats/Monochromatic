//! What:
//!  Publishing a landed commit when the remote branch advanced after the commit landed:
//!       the merge of the remote tip and the local tip is computed in the object store,
//!       committed without a worktree or an index, and pushed with a plain push.
//! Why:
//!  A rejected `push` of a branch that the remote moved cannot succeed again, and an ordinary merge in the active
//!  worktree can be blocked, or made unsafe, by another consumer's uncommitted files.
//!  `git merge-tree --write-tree` and `git commit-tree` read and write only objects, so the active worktree,
//!  its index and its untracked files are never touched while merging, and both histories stay reachable from the
//!  published tip.
//!  Only after the push succeeds is the branch fast-forwarded, and only where nothing can be lost: a dirty worktree
//!  follows too when the move changes none of the files its uncommitted work touches.
//!  An amended or rebased branch is merged like any other: nothing already published is lost, and the replaced
//!  commits stay in the remote history beside the new ones.

/// The coordinator's Git, its command builder, queries and error type.
use crate::auto_push_single_flight::{
    PushAttempt, PushGit, SingleFlightError, git_command, git_output, run_query,
};
/// The deadline-bound runner of network commands and the limits it applies.
use crate::auto_push_dirty_paths::{diff_paths, listed, overlapping_paths, status_paths};
use crate::bounded_push::{STOP_GRACE, push_patience, run_bounded_push};
/// The auto-push's bounded `git push` with the Git LFS upload.
use crate::lfs_bounded_push::run_push;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::{debug, warn};
/// The word-level second pass over the conflicts of the merge tree, and the attribute tree it reads.
use crate::merge_second_pass::{GitReply, GitRunner, Merged, conflicts_of_merge_tree};
use crate::journald_native::{CLIENT_IDENTIFIER, IDENTIFIER};
use crate::merge_second_pass_tree::{AttributeProbe, announce, attribute_tree_variables, report, resolve_tree};
use crate::path_bytes::path_message_text;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
/// Where Git's words go now that the fetch and the fast-forward carry no `--quiet`.
use crate::inner_git_words::{record_streams, record_text};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// Prefix of every line this module adds to a push's output, so the terminal filter can keep it.
pub const RECONCILE_LINE_PREFIX: &str = "cli-git: auto-push: ";

/// Start of the line that explains why a rejected push stays unpublished; the generic "run `git push`" note is then
/// wrong advice and is not printed.
pub const RECONCILE_FAILED_PREFIX: &str = "cli-git: auto-push: not published: ";

/// How many times the remote tip is fetched, merged and pushed before the push is reported as failed.
pub const MAX_RECONCILE_ATTEMPTS: u32 = 10;

/// The marker Git prints for a push the remote refused as stale; the word is not translated.
const REJECTED_MARKER: &str = "[rejected]";

/// The marker Git prints for an update that the remote's own rules refused, such as GitHub's protected branch (GH006).
/// A remote that is updated by two pushes at once refuses the slower one this way, with no word about staleness.
const REMOTE_REJECTED_MARKER: &str = "[remote rejected]";

/// Where the plain push goes, as `auto_push_destination` decides it.
pub use crate::auto_push_destination::Destination;
/// The destination of the plain push of a branch, from the settings alone.
use crate::auto_push_destination::read_destination;
/// The record of a fast-forward in flight, its settling, and the identity of the process that made it.
use crate::iso_time::now_iso;
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent, write_merge_intent};
use crate::merge_isolation_recovery::{MergeRecoveryAction, RecoveryPlace, settle_failed_merge};
use crate::random_id::random_uuid;
use crate::transaction_git::GitContext;
use crate::transaction_owner::current_transaction_owner;
/// The words that name a conflict in the report.
use crate::push_retry::CONFLICT_PHRASE;

/// What:
///  Whether a push's output shows a refusal that a moved remote branch can explain: Git's stale `[rejected]`, or the
///  remote's own `[remote rejected]`.
/// Why:
///   Only a refusal of an update that raced another push can be cured by merging; an authentication failure or an offline
///  error must be reported as they are.
///  The message cannot tell a race from a rule that refuses every push, so the caller compares the remote tip with the
///  branch and returns the refusal unchanged when the remote has not moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wasRejectedAsStale = (output: string) => output.includes('[rejected]') || output.includes('[remote rejected]');
/// ```
pub fn was_rejected_as_stale(output: &str) -> bool {
    return output.contains(REJECTED_MARKER) || output.contains(REMOTE_REJECTED_MARKER);
}

/// A failed attempt that explains itself and keeps the push output that came before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { exitCode: 1, output: `${previous}\n${RECONCILE_FAILED_PREFIX}${reason}` }
/// ```
fn unpublished(failed_prefix: &str, previous: &str, exit_code: i64, reason: &str) -> PushAttempt {
    return PushAttempt {
        exit_code: if exit_code == 0 { 1 } else { exit_code },
        output: format!("{previous}\n{failed_prefix}{reason}"),
    };
}

/// Argument list from string literals and owned text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['merge-tree', '--write-tree', ...]
/// ```
fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// What:
///  Whether the first commit is the second or one of its ancestors, or nothing when Git cannot tell.
/// Why:
///   Exit `0` and `1` are the answers; any other exit means an object is missing, and that is not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function isAncestor({ ancestor, descendant }): Promise<boolean | undefined>;
/// ```
fn is_ancestor(
    git: &PushGit,
    ancestor: &str,
    descendant: &str,
) -> Result<Option<bool>, SingleFlightError> {
    let output = run_query(
        git,
        &argv(&["merge-base", "--is-ancestor", ancestor, descendant]),
    )?;
    return Ok(match output.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    });
}

/// What:
///  Fetch the remote branch into a private ref and return the commit it names.
/// Why:
///   `FETCH_HEAD` is shared by every fetch in the repository, so another process could replace it between the fetch
///  and the read; a ref named for this process cannot be replaced, and it is deleted before this returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fetchRemoteTip({ destination }): Promise<string | PushAttempt>;
/// ```
fn fetch_remote_tip(
    git: &PushGit,
    destination: &Destination,
) -> Result<Result<String, PushAttempt>, SingleFlightError> {
    let private_ref = format!("refs/cli-git/auto-push/fetch-{}", std::process::id());
    let refspec = format!("+{}:{private_ref}", destination.merge_ref);
    let arguments = argv(&[
        "fetch",
        "--no-tags",
        &destination.remote,
        &refspec,
    ]);
    // No low-speed abort of the wrapper's own is added (`bounded_push.rs`): the fetch waits while its connection is open.
    let command = git_command(git, &arguments);
    let fetched = run_bounded_push(command, push_patience(&git.environment), STOP_GRACE, git.environment.as_slice())?;
    if fetched.exit_code != 0 {
        return Ok(Err(fetched));
    }
    // The fetch carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
    record_text(&["fetch", "--no-tags", destination.remote.as_str(), refspec.as_str()], fetched.output.as_str());
    let tip = git_output(
        git,
        &argv(&[
            "rev-parse",
            "--verify",
            &format!("{private_ref}^{{commit}}"),
        ]),
    );
    // The private ref is scaffolding; a failed deletion leaves one small ref that the next run overwrites.
    let _deleted = run_query(git, &argv(&["update-ref", "-d", &private_ref]));
    return Ok(Ok(tip?));
}

/// What:
///  The journal identifier of the entries of a merged file: the wrapper's for a push a person typed, the service's for the
///  auto-push, which runs inside the service.
/// Why:
///   An entry belongs next to the others of the program that sent it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identifier = explicit ? 'cli-git' : 'cli-git-service';
/// ```
pub fn journal_identifier(explicit: bool) -> &'static str {
    return if explicit { CLIENT_IDENTIFIER } else { IDENTIFIER };
}

/// The coordinator's Git as a runner for the second pass: its commands, its environment, an optional stdin and index file.
struct PushRunner<'a> {
    /// The coordinator's Git.
    git: &'a PushGit,
}

/// Starting Git the way the coordinator's queries do, with the input a second-pass command needs.
impl GitRunner for PushRunner<'_> {
    fn run(&self, arguments: &[&str], input: Option<&[u8]>, index_file: Option<&Path>) -> Result<GitReply, String> {
        let mut command = git_command(self.git, &argv(arguments));
        if let Some(index) = index_file {
            command.env("GIT_INDEX_FILE", index);
        }
        command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|error| { return format!("git {} could not start: {error}", arguments.join(" ")); })?;
        // The input goes in from a thread of its own, so a command that answers while it reads cannot block on a full pipe.
        let pipe = child.stdin.take();
        let bytes: Vec<u8> = input.map(<[u8]>::to_vec).unwrap_or_default();
        let writer = std::thread::spawn(move || {
            if let Some(mut open) = pipe
                && let Err(error) = open.write_all(bytes.as_slice())
            {
                debug(TAG, &format!("the input of a second-pass command was not taken: {error}"));
            }
        });
        let output = child.wait_with_output().map_err(|error| { return format!("git {} did not finish: {error}", arguments.join(" ")); })?;
        if writer.join().is_err() {
            return Err(format!("the input thread of git {} ended abnormally", arguments.join(" ")));
        }
        return Ok(GitReply { code: output.status.code(), stdout: output.stdout, stderr: output.stderr });
    }
}

/// What the merge of two tips came to.
struct CommittedMerge {
    /// The merge commit.
    oid: String,
    /// The files the word pass merged, listed in the commit message and in the report.
    merged: Vec<Merged>,
}

/// What:
///  Commit the merge of two tips from objects alone and return its OID, or the paths that conflict.
/// Why:
///   `merge-tree --write-tree` exits `1` for a conflicted merge and lists the paths; nothing is written to a worktree.
///   The conflicts get the word-level second pass, and the merge is committed only when every conflicted file resolved;
///  the commit message then lists every merged part, so the history keeps what was combined. The attributes of the local
///  tip apply, because no worktree is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ local, remote, message }): Promise<{ oid, merged } | { conflicts }>;
/// ```
fn commit_merge(
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
    let merged = runner.run(&merge_arguments, None, None).map_err(|reason| {
        return SingleFlightError::Git {
            arguments: format!("merge-tree --write-tree {local} {remote}"),
            code: None,
            stderr: reason,
        };
    })?;
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
                let paths: Vec<String> = conflicts.iter().map(|conflict| { return path_message_text(&conflict.path); }).collect();
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

/// The repository's common Git directory, where the scratch the second pass needs lives.
fn common_dir_of(git: &PushGit) -> Result<PathBuf, SingleFlightError> {
    let answered = git_output(git, &argv(&["rev-parse", "--git-common-dir"]))?;
    return Ok(git.cwd.join(answered));
}

/// What:
///  Whether `commit.gpgsign` asks for signed commits.
/// Why:
///   `git commit-tree` signs only when given `-S` and never reads `commit.gpgsign`, so the merge would be the one
///  unsigned commit of a signed branch, which a remote that requires verified signatures rejects.
///   An unset key is `git config` exit `1`; any other failure is reported like every failed command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function signsCommits(git): Promise<boolean>;
/// ```
fn signs_commits(git: &PushGit) -> Result<bool, SingleFlightError> {
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
///  Whether the status read after a fast-forward differs from the one read before it.
///  `&[u8]` borrows Git's output.
/// Why:
///   The uncommitted work must read exactly as it did; a difference is the only sign that the move touched it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusDiffers(before: Uint8Array, after: Uint8Array): boolean;
/// ```
fn status_differs(before: &[u8], after: &[u8]) -> bool {
    return before != after;
}

/// What:
///  Git's fast-forward to the published merge, with an intent record written first and settled after, so a branch that moved in
///  the instant between Git's checkout and its ref update, or a service that died there, leaves the checkout as it was or put
///  back to the moved head, never half merged.
/// Why:
///   The owner's rule "Auto merging should never fail." (`SPEC.md`, "Merges and commits never fail"): the person's own merge and
///  the reconcile can both move the same branch, and the loser's checkout must not stay beside a branch that does not hold it.
///  The same record and settling serve the person's merge (`merge_isolation_recovery.rs`), and a record whose owner died is
///  settled by the next wrapper command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await fastForwardRecorded({ git, branchRef, local, mergeOid }); // undefined when it moved
/// ```
fn fast_forward_recorded(git: &PushGit, branch_ref: &str, local: &str, merge_oid: &str) -> Result<Result<(), String>, SingleFlightError> {
    let moved_arguments = argv(&["merge", "--ff-only", merge_oid]);
    let git_dir: Option<PathBuf> = {
        let named = run_query(git, &argv(&["rev-parse", "--absolute-git-dir"]))?;
        (named.code == Some(0)).then(|| { return PathBuf::from(String::from_utf8_lossy(&named.stdout).trim()); })
    };
    let recorded: Option<(PathBuf, MergeIntent)> = git_dir.as_deref().and_then(|dir| {
        let owner = current_transaction_owner(random_uuid().ok()?.as_str(), now_iso().as_str()).ok()?;
        let intent: MergeIntent = MergeIntent {
            owner,
            repository_root: git.cwd.to_string_lossy().into_owned(),
            target_ref: Some(branch_ref.to_owned()),
            old_oid: local.to_owned(),
            new_oid: merge_oid.to_owned(),
        };
        let record: PathBuf = write_merge_intent(dir, &intent).ok()?;
        return Some((record, intent));
    });
    let moved = run_query(git, &moved_arguments)?;
    // The fast-forward carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
    if moved.code == Some(0) {
        record_streams(&["merge", "--ff-only", merge_oid], &moved.stdout, &moved.stderr);
    }
    let Some((record, intent)) = recorded else {
        // Without a record the fast-forward runs as it always did, and Git's refusal is the reason.
        return Ok(if moved.code == Some(0) { Ok(()) } else { Err(refusal_of(&moved.stderr)) });
    };
    if moved.code == Some(0) {
        if let Err(error) = remove_merge_intent(record.as_path()) {
            debug(TAG, &format!("the merge record could not be removed: {error}"));
        }
        return Ok(Ok(()));
    }
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext { real_git: git.real_git.clone(), overlay: git.environment.clone(), global_prefix: Vec::new(), common_dir: git_dir.clone() },
        cwd: git.cwd.clone(),
        common_dir: git_dir.unwrap_or_default(),
    };
    return Ok(match settle_failed_merge(&place, record.as_path(), &intent) {
        Ok(MergeRecoveryAction::Completed) => Ok(()),
        Ok(_) => Err(refusal_of(&moved.stderr)),
        Err(error) => Err(format!("{} ({})", refusal_of(&moved.stderr), error.0)),
    });
}

/// The reason line for a fast-forward Git refused: its first line of words.
fn refusal_of(stderr: &[u8]) -> String {
    let detail = String::from_utf8_lossy(stderr);
    return format!("Git refused the fast-forward: {}", detail.lines().next().unwrap_or_default().trim());
}

/// What:
///  Fast-forward the pushed branch and this worktree to the published merge, or the reason that was not done.
///  The success value says whether uncommitted work was left in place.
/// Why:
///   A branch left behind the remote makes the next commit rejected again, and the person at the keyboard would have
///  to merge by hand; the merge contains the branch tip, so the move is a pure fast-forward.
///   It is attempted only where nothing can be lost or surprised: the worktree has the branch checked out at the tip
///  that was pushed, and no uncommitted change (tracked file, staged entry, or either name of a staged rename)
///  lies in a file the move writes.
///   Another session's work in progress therefore never holds the branch behind the remote, while a move that
///  would write over such a file is left for the person.
///   The status read takes no optional lock, and `git merge --ff-only` refuses, without changing anything, a move that
///  would overwrite an untracked file or a change made after the status read.
///   Every refusal is a reason for the report, never a failure: the merge is already published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advanceLocalBranch({ git, branchRef, local, mergeOid }): Promise<{ reason?: string; keptChanges?: boolean }>;
/// ```
fn advance_local_branch(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Result<bool, String>, SingleFlightError> {
    let checked_out = run_query(git, &argv(&["symbolic-ref", "--quiet", "HEAD"]))?;
    if checked_out.code != Some(0) {
        return Ok(Err(String::from("HEAD is detached")));
    }
    if String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Err(String::from(
            "this worktree has another branch checked out",
        )));
    }
    let head = run_query(git, &argv(&["rev-parse", "--verify", "HEAD^{commit}"]))?;
    if String::from_utf8_lossy(&head.stdout).trim() != local {
        return Ok(Err(String::from(
            "the branch moved after the commit landed",
        )));
    }
    let status_arguments = argv(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ]);
    let status = run_query(git, &status_arguments)?;
    if status.code != Some(0) {
        return Ok(Err(String::from("the worktree status could not be read")));
    }
    let dirty = status_paths(&status.stdout);
    let kept: bool = !dirty.is_empty();
    if kept {
        let incoming = run_query(
            git,
            &argv(&["diff", "--name-only", "-z", "--no-renames", local, merge_oid]),
        )?;
        if incoming.code != Some(0) {
            return Ok(Err(String::from(
                "the files the merge changes could not be read",
            )));
        }
        let overlap = overlapping_paths(&dirty, &diff_paths(&incoming.stdout));
        if !overlap.is_empty() {
            return Ok(Err(format!(
                "the worktree has uncommitted changes to files the merge changes: {}",
                listed(&overlap)
            )));
        }
    }
    if let Err(reason) = fast_forward_recorded(git, branch_ref, local, merge_oid)? {
        return Ok(Err(reason));
    }
    if kept {
        // The uncommitted work must read exactly as it did; the move is already done, so a difference is reported.
        let after = run_query(git, &status_arguments)?;
        if status_differs(&status.stdout, &after.stdout) {
            warn(
                TAG,
                "the worktree status differs after the fast-forward; compare it with the status before the commit",
            );
        }
    }
    return Ok(Ok(kept));
}

/// What:
///  Merge the remote tip into the pushed branch's tip and publish the result, a bounded number of times.
/// Why:
///   Another worktree advancing the remote makes the plain push fail for good, while both histories are valid and
///  must both be kept; the merge is published as a new commit above the local tip, so the landed commit stays
///  reachable and nothing is forced.
///   Once the merge is published, the branch and its worktree are fast-forwarded to it when that cannot lose or
///  overwrite anything; otherwise they are left as they are and the report says why.
///   A push destination Git cannot name, a conflict, an exhausted retry budget and every failure of a command are
///  reported, and only the first leaves the original rejection unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reconcileDivergentPush({ git, branchRef, rejected }): Promise<PushAttempt>;
/// ```
pub fn reconcile_divergent_push(
    git: &PushGit,
    branch_ref: &str,
    rejected: PushAttempt,
) -> Result<PushAttempt, SingleFlightError> {
    if !was_rejected_as_stale(&rejected.output) {
        return Ok(rejected);
    }
    let Some(destination) = read_destination(git, branch_ref)? else {
        debug(
            TAG,
            "the push destination cannot be resolved to a remote branch; no reconcile",
        );
        return Ok(rejected);
    };
    let local = git_output(
        git,
        &argv(&["rev-parse", "--verify", &format!("{branch_ref}^{{commit}}")]),
    )?;
    let plan = Reconcile {
        destination,
        local,
        branch_ref: Some(branch_ref),
        label: "auto-push",
        explicit: false,
    };
    let mut push_merge = |target: &Destination, merge_oid: &str| -> Result<PushAttempt, SingleFlightError> {
        let arguments = argv(&[
            "push",
            &target.remote,
            &format!("{merge_oid}:{}", target.merge_ref),
        ]);
        return run_push(git, &arguments);
    };
    return Ok(match integrate_and_push(git, &plan, rejected, &mut push_merge)? {
        Integrated::NotDivergent(attempt) | Integrated::Finished(attempt) => attempt,
    });
}

/// What:
///  What one reconcile run integrates, where, and how it words and limits itself.
///  `Option<&str>` is "the branch, or nothing for a detached HEAD".
/// Why:
///   Auto-push and an explicit push share the loop; they differ in the line prefix they print, in whether the branch
///  follows the published merge, and in how much of a failure they keep quiet about.
///   An explicit push already printed Git's own refusal, so a remote that cannot be fetched, a remote that did not
///  diverge, and a failure that is not a stale update are left exactly as the push reported them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reconcile = { destination: Destination; local: string; branchRef?: string; label: string; explicit: boolean };
/// ```
pub struct Reconcile<'a> {
    /// The remote and the ref the rejected push updated.
    pub destination: Destination,
    /// The commit the rejected push sent.
    pub local: String,
    /// The branch that follows the published merge when it can, or nothing for a detached HEAD.
    pub branch_ref: Option<&'a str>,
    /// The word after `cli-git:` in every line the run adds.
    pub label: &'a str,
    /// Whether the rejected push was typed by a person and has already shown its own output.
    pub explicit: bool,
}

/// How a reconcile run ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Integrated = { kind: 'not-divergent'; attempt } | { kind: 'finished'; attempt };
/// ```
pub enum Integrated {
    /// The remote did not diverge, or the run could not tell; the rejected attempt stands as it was.
    NotDivergent(PushAttempt),
    /// The run published the merge, or says why it did not.
    Finished(PushAttempt),
}

/// What:
///  Merge the remote tip into the pushed commit and publish the result, a bounded number of times.
/// Why:
///   Another worktree advancing the remote makes the plain push fail for good, while both histories are valid and
///  must both be kept; the merge is published as a new commit above the local tip, so the landed commit stays
///  reachable and nothing is forced.
///   `push_merge` publishes one merge, and is told the destination and the merge commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function integrateAndPush({ git, plan, rejected, pushMerge }): Promise<Integrated>;
/// ```
pub fn integrate_and_push(
    git: &PushGit,
    plan: &Reconcile<'_>,
    rejected: PushAttempt,
    push_merge: &mut dyn FnMut(&Destination, &str) -> Result<PushAttempt, SingleFlightError>,
) -> Result<Integrated, SingleFlightError> {
    let destination = plan.destination.clone();
    let local = plan.local.clone();
    let label = plan.label;
    let line_prefix = format!("cli-git: {label}: ");
    let failed_prefix = format!("{line_prefix}not published: ");
    let short = destination
        .merge_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&destination.merge_ref)
        .to_owned();
    let message = format!(
        "Merge {}/{short} into {short}\n\nCreated by cli-git {label} because {}/{short} advanced after the commit landed.",
        destination.remote, destination.remote
    );
    let mut previous = rejected.output.clone();
    // The remote tip a merge was pushed against, with the refusal it got: a remote that still shows that tip did not
    // move, so the refusal was a rule and not a race.
    let mut refused_against: Option<(String, PushAttempt)> = None;
    for attempt in 1..=MAX_RECONCILE_ATTEMPTS {
        let remote_tip = match fetch_remote_tip(git, &destination)? {
            Ok(tip) => tip,
            Err(failed) => {
                if plan.explicit && refused_against.is_none() {
                    debug(
                        TAG,
                        "the remote could not be fetched after an explicit push failed; its own report stands",
                    );
                    return Ok(Integrated::NotDivergent(rejected));
                }
                return Ok(Integrated::Finished(unpublished(
                    &failed_prefix,
                    &format!("{previous}\n{}", failed.output),
                    failed.exit_code,
                    &format!(
                        "{}/{short} has commits this branch lacks, and fetching them failed; nothing was pushed and the worktree was not touched.",
                        destination.remote
                    ),
                )));
            }
        };
        if let Some((tip, refusal)) = refused_against.take()
            && tip == remote_tip
        {
            debug(
                TAG,
                "the remote did not move after it refused the merge; the refusal is final",
            );
            return Ok(Integrated::Finished(refusal));
        }
        match (
            is_ancestor(git, &remote_tip, &local)?,
            is_ancestor(git, &local, &remote_tip)?,
        ) {
            (Some(true), _) => {
                debug(
                    TAG,
                    "the remote tip is behind the branch; the rejection was not a divergence",
                );
                return Ok(Integrated::NotDivergent(rejected));
            }
            (_, Some(true)) if plan.explicit => {
                debug(
                    TAG,
                    "the remote already contains the pushed commit; the explicit push failed for another reason",
                );
                return Ok(Integrated::NotDivergent(rejected));
            }
            (_, Some(true)) => {
                return Ok(Integrated::Finished(PushAttempt {
                    exit_code: 0,
                    output: format!(
                        "{line_prefix}{}/{short} already contains {local}; nothing to push.",
                        destination.remote
                    ),
                }));
            }
            (Some(false), Some(false)) => {}
            _ => {
                if plan.explicit {
                    return Ok(Integrated::NotDivergent(rejected));
                }
                return Ok(Integrated::Finished(unpublished(
                    &failed_prefix,
                    &previous,
                    rejected.exit_code,
                    "Git could not relate the branch to the remote tip; nothing was pushed and the worktree was not touched.",
                )));
            }
        }
        let committed = match commit_merge(git, &local, &remote_tip, &message)? {
            Ok(found) => found,
            Err(paths) => {
                return Ok(Integrated::Finished(unpublished(
                    &failed_prefix,
                    &previous,
                    rejected.exit_code,
                    &format!(
                        "merging {}/{short} into {short}{CONFLICT_PHRASE}{}. Nothing was pushed and the worktree was not touched; \
                         merge {}/{short} in a separate worktree, resolve the conflicts and push.",
                        destination.remote,
                        paths.join(", "),
                        destination.remote
                    ),
                )));
            }
        };
        let merge_oid = committed.oid.clone();
        debug(
            TAG,
            &format!(
                "attempt {attempt}: pushing the merge {merge_oid} of {local} and {remote_tip}"
            ),
        );
        let pushed = push_merge(&destination, &merge_oid)?;
        if pushed.exit_code == 0 {
            let branch = plan
                .branch_ref
                .map(|reference| {
                    return reference.strip_prefix("refs/heads/").unwrap_or(reference);
                })
                .unwrap_or("HEAD");
            let advanced = match plan.branch_ref {
                Some(branch_ref) => advance_local_branch(git, branch_ref, &local, &merge_oid)?,
                None => Err(String::from("HEAD is detached")),
            };
            let followed = match advanced {
                Ok(false) => format!("{branch} and the worktree were fast-forwarded to it."),
                Ok(true) => format!(
                    "{branch} and the worktree were fast-forwarded to it; your uncommitted changes were left as they are."
                ),
                Err(reason) => {
                    debug(TAG, &format!("{branch} was not fast-forwarded: {reason}"));
                    format!(
                        "{branch} itself and the worktree are unchanged ({reason}); update {branch} with `git merge --ff-only {}/{short}` \
                         when the worktree allows.",
                        destination.remote
                    )
                }
            };
            let merged_parts: String = announce(&line_prefix, (journal_identifier(plan.explicit), label), &committed.merged)
                .iter()
                .map(|line| { return format!("{line}\n"); })
                .collect();
            return Ok(Integrated::Finished(PushAttempt {
                exit_code: 0,
                output: format!(
                    "{}\n{merged_parts}{line_prefix}{}/{short} had new commits, so the merge {merge_oid} of {branch} and {}/{short} \
                     was published. {followed}",
                    pushed.output, destination.remote, destination.remote
                )
                .trim_start_matches('\n')
                .to_owned(),
            }));
        }
        if !plan.explicit && !was_rejected_as_stale(&pushed.output) {
            return Ok(Integrated::Finished(pushed));
        }
        previous = pushed.output.clone();
        refused_against = Some((remote_tip, pushed));
    }
    return Ok(Integrated::Finished(unpublished(
        &failed_prefix,
        &previous,
        rejected.exit_code,
        &format!(
            "{}/{short} advanced again during each of {MAX_RECONCILE_ATTEMPTS} attempts, so nothing was pushed and the worktree was not touched. \
             Push again when the remote is quiet.",
            destination.remote
        ),
    )));
}

/// Reconcile behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_reconcile_tests.rs"]
mod tests;
