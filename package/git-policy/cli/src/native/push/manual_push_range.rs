//! What:
//!  The commits a push newly publishes,
//! relative to what each destination remote already has.
//! Why:
//!  The manual-push policies scan only what becomes public.
//! A pushed commit is already published when a
//!  remote-tracking ref of the destination,
//! or an authoritative prior value of an update to that remote,
//!  reaches it (`src/policy-engine/manual-push-published.ts`).
//! The rest of the history is the scan's range.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const published = publishedTips({ updates, peeled, tracking });
//! ```

/// The Git runner,
/// the probe's update type,
/// and the object name helpers.
use crate::auto_push_single_flight::{PushGit, git_command};
use crate::manual_push_cause::ManualPushCause;
use crate::manual_push_probe::PushUpdate;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::RetryStart;
/// Maps from remote names and peeled object names.
use std::collections::BTreeMap;
/// Owned OS strings for Git arguments and the stdin of batch queries.
use std::ffi::OsString;
/// Pipes for the batch queries' stdin and stdout.
use std::io::Write;
/// The child a query starts,
/// handed to the thread that waits for it.
use std::process::Child;
use std::process::Stdio;
/// How long a range query may run.
use std::time::Duration;
/// The descriptor that names the query for good.
use crate::pinned_member_signalled_through_a_pidfd::{Delivered, PinnedProcess, pin_of_our_own_child};

/// Remote-tracking ref namespace Git's default fetch refspec writes.
const TRACKING_NAMESPACE: &str = "refs/remotes/";

/// What:
///  Why the range could not be computed.
/// Why:
///   Every variant stops the lifecycle with a failure event;
/// a guessed range would hide commits from the scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushRangeError {
    /// Diagnostic naming the failed query.
    pub message: String,
    /// What the person can do about it,
    /// which the hint is chosen by.
    pub cause: ManualPushCause,
}

impl std::fmt::Display for ManualPushRangeError {
    /// Writes the diagnostic text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

impl std::error::Error for ManualPushRangeError {}

/// Build a range error from text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ManualPushProbeError(message)
/// ```
fn range_error(cause: ManualPushCause, message: String) -> ManualPushRangeError {
    return ManualPushRangeError { message, cause };
}

/// The longest one of these read-only queries runs before it is ended:
/// the ceiling of a quick Git step of the wrapper
/// (`src/native/service/wait_ceilings.rs`),
/// because each of them reads the object store or the refs and answers at once.
/// A query that is slower than that is reported rather than waited for longer.
const RANGE_QUERY_CEILING: Duration = crate::wait_ceilings::PLUMBING_STEP_CEILING;

/// What:
///  Run a Git query with stdin text and return its exact stdout bytes.
/// Why:
///   Batch queries (`cat-file --batch-check`,
/// `rev-list --stdin`) read their requests from stdin,
///  and any Git failure is an error,
/// never an empty answer.
///  The request list goes in from a thread of its own while the answers are read on another,
/// because Git reads
///  and answers in one loop:
/// a list larger than a pipe was written before anything was read,
/// so a full reply pipe
///  stopped Git reading the list and the write waited for it for good
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 9).
///  The wait carries `RANGE_QUERY_CEILING`,
/// so a Git that never answers is reported instead of waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await withDeadline(runWithInputFromAnotherThread(command, input), ceiling);
/// ```
pub fn run_git_bytes(
    git: &PushGit,
    arguments: &[OsString],
    input: Option<&str>,
) -> Result<Vec<u8>, ManualPushRangeError> {
    return run_git_bytes_starting_with(git, arguments, input);
}

/// What:
///  The body of `run_git_bytes`,
/// where a control's refused thread start is honoured by the shared starter.
/// Why:
///  Both of the query's threads are started before Git is,
/// so the refusal has to reach the code that starts them
///  and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runGitBytesWithinItsThreads(git, args, input): Buffer;
/// ```
fn run_git_bytes_starting_with(
    git: &PushGit,
    arguments: &[OsString],
    input: Option<&str>,
) -> Result<Vec<u8>, ManualPushRangeError> {
    let mut command = git_command(git, arguments);
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Both threads are started before Git is, so a refused start costs the query its start and never leaves Git, killed and
    //  unreaped, behind a command that met a shortage of processes.
    let (stdin_hand_over, stdin_taken) = std::sync::mpsc::channel::<std::process::ChildStdin>();
    // The input is written on this thread while `wait_with_output` reads both replies on the caller's: neither can hold the
    //  other up, which is what a request list larger than a pipe did before.
    let writing: Option<std::thread::JoinHandle<()>> = match input {
        Some(requests) => {
            let text: String = requests.to_owned();
            match crate::thread_started_before_its_child::start_named_thread("cli-git-range-input", Box::new(move || {
                // Nothing is handed over when the start below fails, and then there is no request list to write.
                let Ok(mut stdin) = stdin_taken.recv() else {
                    return;
                };
                // A Git exit before reading the whole input surfaces through its status, so the write error is ignored here.
                let _written = stdin.write_all(text.as_bytes());
            })) {
                Ok(written) => Some(written),
                Err(error) => return Err(range_error(ManualPushCause::GitNotStarted, format!(
                    "git was not started, because no thread could be given its request list: {error}. Hint: check the process limit \
                     (`ulimit -u`) and the control group's `pids.max`, free some, and run the command again"
                ))),
            }
        }
        None => None,
    };
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::Output>>();
    let (hand_over, taken) = std::sync::mpsc::channel::<Child>();
    if let Err(error) = crate::thread_started_before_its_child::start_named_thread("cli-git-range-answer", Box::new(move || {
        // Nothing is handed over when the start below fails, and then there is no child to wait for.
        let Ok(child) = taken.recv() else {
            return;
        };
        // This thread owns the child until it ends; a caller that stopped waiting took the failure.
        let _abandoned = sender.send(child.wait_with_output());
    })) {
        return Err(range_error(ManualPushCause::GitNotStarted, format!(
            "git was not started, because no thread could wait for it: {error}. Hint: check the process limit (`ulimit -u`) and the \
             control group's `pids.max`, free some, and run the command again"
        )));
    }
    let mut child: Child = match command.spawn_retrying() {
        Ok(started) => started,
        Err(error) => return Err(range_error(ManualPushCause::GitNotStarted, format!("git could not be started: {error}"))),
    };
    // The descriptor is taken while the number certainly names this process's own unreaped child, so the signal a ceiling sends
    //  later reaches that child and never a number that was reused after the answering thread reaped it.
    let reach: Option<PinnedProcess> = pin_of_our_own_child(child.id());
    if let Some(stdin) = child.stdin.take() {
        let _handed_over = stdin_hand_over.send(stdin);
    }
    let _handed_over = hand_over.send(child);
    let answered: std::io::Result<std::process::Output> = match receiver.recv_timeout(RANGE_QUERY_CEILING) {
        Ok(output) => output,
        Err(_timed_out) => {
            let ended: bool = end_the_query(&reach);
            let shown: String = words_of(arguments);
            let ending: &str = if ended {
                "and was ended"
            } else {
                "and nothing was signalled, because no descriptor naming it could be taken before it was reaped, so check with `ps` \
                 whether it is still running"
            };
            return Err(range_error(ManualPushCause::GitSilent, format!(
                "git {shown} did not answer within {} seconds, the longest one of these queries waits, {ending}. Hint: check with `ps` \
                 what that Git waits for, and run the command again.",
                RANGE_QUERY_CEILING.as_secs()
            )));
        }
    };
    // What: the thread that writes the request list is not joined.
    // Why:  a program the query started can hold the read end of its input without reading it, so the write stays in
    //       `write_all` past the answer and past `RANGE_QUERY_CEILING`. What ends it is the last read end of that input
    //       closing, whose next write fails with `EPIPE`, or this process exiting, which ends every thread of it.
    let _input_thread_left_to_end = writing;
    let output = match answered {
        Ok(finished) => finished,
        Err(error) => return Err(range_error(ManualPushCause::GitSilent, format!("git did not finish: {error}"))),
    };
    if !output.status.success() {
        let mut shown: Vec<String> = Vec::new();
        for argument in arguments {
            shown.push(argument.to_string_lossy().into_owned());
        }
        return Err(range_error(ManualPushCause::ContentMissing, format!(
            "git {} failed: {}",
            shown.join(" "),
            String::from_utf8_lossy(&output.stderr).trim_end()
        )));
    }
    return Ok(output.stdout);
}

/// What:
///  End a query past its ceiling with `SIGKILL`,
/// through the descriptor taken while the number certainly named it.
/// Why:
///  A number is a safe address only while its process is this process's own unreaped child,
/// and the answering thread reaps the
///  query as soon as it ends.
/// The descriptor reaches that one process or fails,
/// and never a successor;
///  a host that gave
/// none proves nothing about what the number names by now,
/// so nothing is signalled and the sentence says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endTheQuery(reach: Pin | undefined): boolean;
/// ```
fn end_the_query(reach: &Option<PinnedProcess>) -> bool {
    return match reach {
        Some(pinned) => pinned.signal(libc::SIGKILL) == Delivered::Sent,
        None => false,
    };
}

/// One argument as the words a message shows it with.
fn word_of(argument: &OsString) -> String {
    return argument.to_string_lossy().into_owned();
}

/// The words of an argument list,
/// for a message that names the command.
fn words_of(arguments: &[OsString]) -> String {
    return arguments.iter().map(word_of).collect::<Vec<String>>().join(" ");
}

/// Split Git line output into non-empty lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').filter((line) => line.length > 0)
/// ```
fn lines_of(bytes: &[u8]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(bytes).split('\n') {
        if !line.is_empty() {
            lines.push(line.to_owned());
        }
    }
    return lines;
}

/// What:
///  The commit a prior destination value names,
/// when it exists locally and is a commit.
///  `Option<T>` is "a value or nothing".
/// Why:
///   Only locally present commits can bound the range;
/// an object Git never fetched is unknown,
/// not published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// peeled.get(oid)?.type === 'commit' ? [oid] : []
/// ```
pub fn peel_commits(
    git: &PushGit,
    oids: &[String],
) -> Result<BTreeMap<String, String>, ManualPushRangeError> {
    let mut unique: Vec<String> = oids.to_vec();
    unique.sort();
    unique.dedup();
    if unique.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut input: String = String::new();
    for oid in &unique {
        input.push_str(format!("{oid}^{{}}\n").as_str());
    }
    let arguments: Vec<OsString> = ["cat-file", "--batch-check=%(objectname) %(objecttype)"]
        .iter()
        .map(OsString::from)
        .collect();
    let output = run_git_bytes(git, &arguments, Some(&input))?;
    let records = lines_of(&output);
    if records.len() != unique.len() {
        return Err(range_error(
            ManualPushCause::UnexpectedOutput,
            "Git object batch-check returned an unexpected record count.".to_owned(),
        ));
    }
    let mut commits: BTreeMap<String, String> = BTreeMap::new();
    for (oid, record) in unique.iter().zip(records.iter()) {
        if record == &format!("{oid}^{{}} missing") {
            continue;
        }
        let fields: Vec<&str> = record.split(' ').collect();
        if fields.len() != 2 {
            return Err(range_error(ManualPushCause::UnexpectedOutput, format!(
                "Git object batch-check returned malformed record: {record}"
            )));
        }
        if fields[1] == "commit" {
            commits.insert(oid.clone(), fields[0].to_owned());
        }
    }
    return Ok(commits);
}

/// What:
///  The commits each destination remote is known to have,
/// from its remote-tracking refs.
/// Why:
///   The namespace is matched literally,
/// so a remote name is never read as a pattern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function trackingCommits({ gitPath, cwd, remoteNames }): Promise<ReadonlyMap<string, string[]>>;
/// ```
pub fn tracking_commits(
    git: &PushGit,
    remote_names: &[String],
) -> Result<BTreeMap<String, Vec<String>>, ManualPushRangeError> {
    if remote_names.is_empty() {
        return Ok(BTreeMap::new());
    }
    let arguments: Vec<OsString> = [
        "for-each-ref",
        "--format=%(objectname) %(objecttype) %(refname)",
        TRACKING_NAMESPACE,
    ]
    .iter()
    .map(OsString::from)
    .collect();
    let refs = lines_of(&run_git_bytes(git, &arguments, None)?);
    let mut by_remote: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for remote_name in remote_names {
        let prefix = format!("{TRACKING_NAMESPACE}{remote_name}/");
        let mut commits: Vec<String> = Vec::new();
        for record in &refs {
            let fields: Vec<&str> = record.split(' ').collect();
            if fields.len() != 3 {
                return Err(range_error(ManualPushCause::UnexpectedOutput, format!(
                    "Git for-each-ref returned malformed record: {record}"
                )));
            }
            if fields[1] == "commit" && fields[2].starts_with(&prefix) {
                commits.push(fields[0].to_owned());
            }
        }
        by_remote.insert(remote_name.clone(), commits);
    }
    return Ok(by_remote);
}

/// What:
///  The published tips of each remote:
/// its tracking commits and the commits of its prior destination values.
/// Why:
///   Every update to one remote contributes its prior value,
/// so a second branch is bounded by the first's prior state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function publishedTips({ updates, peeled, tracking }): ReadonlyMap<string, readonly GitObjectId[]>;
/// ```
pub fn published_tips(
    updates: &[PushUpdate],
    prior_commits: &BTreeMap<String, String>,
    tracking: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, Vec<String>> {
    let mut tips: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for update in updates {
        let entry = match tips.entry(update.remote_name.clone()) {
            std::collections::btree_map::Entry::Occupied(present) => present.into_mut(),
            std::collections::btree_map::Entry::Vacant(absent) => absent.insert(tracking.get(&update.remote_name).cloned().unwrap_or_default()),
        };
        if let Some(prior) = update.remote_oid.as_ref()
            && let Some(commit) = prior_commits.get(prior)
            && !entry.contains(commit)
        {
            entry.push(commit.clone());
        }
    }
    return tips;
}

/// What:
///  The scope one pushed update contributes to the scan.
/// Why:
///   A deletion publishes nothing;
/// a commit with a known published base contributes its new commits;
///  a commit with no known base contributes its whole history's tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UpdateScope = { kind: 'none' } | { kind: 'commits'; commits: string[] } | { kind: 'final-tree'; tip: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateScope {
    /// Nothing new is published:
    /// a deletion or a commit the remote already has.
    None,
    /// The commits not reachable from anything the remote has.
    Commits(Vec<String>),
    /// No published base is known:
    /// the pushed tip's whole tree is new.
    FinalTree(String),
}

/// What:
///  The scope of one update,
/// from its local object and the published tips of its remote.
/// Why:
///   `rev-list` names exactly the commits the remote lacks,
/// so a scan never repeats published history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scopeOf(update, publishedTips): Promise<UpdateScope>;
/// ```
pub fn update_scope(
    git: &PushGit,
    update: &PushUpdate,
    published: &[String],
) -> Result<UpdateScope, ManualPushRangeError> {
    let Some(local) = update.local_oid.as_ref() else {
        return Ok(UpdateScope::None);
    };
    let commit_check = peel_commits(git, std::slice::from_ref(local))?;
    if !commit_check.contains_key(local) {
        return Err(range_error(ManualPushCause::UnsupportedObject, format!(
            "pushed object {local} is not a commit, and the manual-push policies read commits only"
        )));
    }
    if published.is_empty() {
        return Ok(UpdateScope::FinalTree(local.clone()));
    }
    let mut arguments: Vec<OsString> = vec![
        OsString::from("rev-list"),
        OsString::from(local),
        OsString::from("--not"),
    ];
    arguments.extend(published.iter().map(OsString::from));
    let commits = lines_of(&run_git_bytes(git, &arguments, None)?);
    return Ok(UpdateScope::Commits(commits));
}

/// The range functions are controlled in the test module.
#[cfg(test)]
#[path = "manual_push_range_tests.rs"]
mod tests;
