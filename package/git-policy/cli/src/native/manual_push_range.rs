//! What:
//!  The commits a push newly publishes, relative to what each destination remote already has.
//! Why:
//!  The manual-push policies scan only what becomes public. A pushed commit is already published when a
//!  remote-tracking ref of the destination, or an authoritative prior value of an update to that remote,
//!  reaches it (`src/policy-engine/manual-push-published.ts`). The rest of the history is the scan's range.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const published = publishedTips({ updates, peeled, tracking });
//! ```

/// The Git runner, the probe's update type, and the object name helpers.
use super::auto_push_single_flight::{PushGit, git_command};
use super::manual_push_probe::PushUpdate;
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// Maps from remote names and peeled object names.
use std::collections::BTreeMap;
/// Owned OS strings for Git arguments and the stdin of batch queries.
use std::ffi::OsString;
/// Pipes for the batch queries' stdin and stdout.
use std::io::Write;
use std::process::Stdio;

/// Remote-tracking ref namespace Git's default fetch refspec writes.
const TRACKING_NAMESPACE: &str = "refs/remotes/";

/// What:
///  Why the range could not be computed.
/// Why:
///   Every variant stops the lifecycle with a failure event; a guessed range would hide commits from the scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushRangeError {
    /// Diagnostic naming the failed query.
    pub message: String,
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
fn range_error(message: String) -> ManualPushRangeError {
    return ManualPushRangeError { message };
}

/// What:
///  Run a Git query with stdin text and return its exact stdout bytes.
/// Why:
///   Batch queries (`cat-file --batch-check`, `rev-list --stdin`) read their requests from stdin,
///  and any Git failure is an error, never an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitBytes({ gitPath, cwd, args, input });
/// ```
pub fn run_git_bytes(
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
    let mut child = command.spawn_retrying().map_err(|error| {
        return range_error(format!("git could not be started: {error}"));
    })?;
    if let (Some(text), Some(mut stdin)) = (input, child.stdin.take()) {
        // A Git exit before reading the whole input surfaces through its status, so the write error is ignored here.
        let _written = stdin.write_all(text.as_bytes());
    }
    let output = child.wait_with_output().map_err(|error| {
        return range_error(format!("git did not finish: {error}"));
    })?;
    if !output.status.success() {
        return Err(range_error(format!(
            "git {} failed: {}",
            arguments
                .iter()
                .map(|argument| {
                    return argument.to_string_lossy().into_owned();
                })
                .collect::<Vec<String>>()
                .join(" "),
            String::from_utf8_lossy(&output.stderr).trim_end()
        )));
    }
    return Ok(output.stdout);
}

/// Split Git line output into non-empty lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').filter((line) => line.length > 0)
/// ```
fn lines_of(bytes: &[u8]) -> Vec<String> {
    return String::from_utf8_lossy(bytes)
        .split('\n')
        .filter(|line| {
            return !line.is_empty();
        })
        .map(str::to_owned)
        .collect();
}

/// What:
///  The commit a prior destination value names, when it exists locally and is a commit.
///  `Option<T>` is "a value or nothing".
/// Why:
///   Only locally present commits can bound the range; an object Git never fetched is unknown, not published.
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
    let input: String = unique
        .iter()
        .map(|oid| {
            return format!("{oid}^{{}}\n");
        })
        .collect();
    let arguments: Vec<OsString> = ["cat-file", "--batch-check=%(objectname) %(objecttype)"]
        .iter()
        .map(OsString::from)
        .collect();
    let output = run_git_bytes(git, &arguments, Some(&input))?;
    let records = lines_of(&output);
    if records.len() != unique.len() {
        return Err(range_error(
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
            return Err(range_error(format!(
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
///  The commits each destination remote is known to have, from its remote-tracking refs.
/// Why:
///   The namespace is matched literally, so a remote name is never read as a pattern.
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
                return Err(range_error(format!(
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
///  The published tips of each remote: its tracking commits and the commits of its prior destination values.
/// Why:
///   Every update to one remote contributes its prior value, so a second branch is bounded by the first's prior state.
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
        let entry = tips.entry(update.remote_name.clone()).or_insert_with(|| {
            return tracking
                .get(&update.remote_name)
                .cloned()
                .unwrap_or_default();
        });
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
///   A deletion publishes nothing; a commit with a known published base contributes its new commits;
///  a commit with no known base contributes its whole history's tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UpdateScope = { kind: 'none' } | { kind: 'commits'; commits: string[] } | { kind: 'final-tree'; tip: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateScope {
    /// Nothing new is published: a deletion or a commit the remote already has.
    None,
    /// The commits not reachable from anything the remote has.
    Commits(Vec<String>),
    /// No published base is known: the pushed tip's whole tree is new.
    FinalTree(String),
}

/// What:
///  The scope of one update, from its local object and the published tips of its remote.
/// Why:
///   `rev-list` names exactly the commits the remote lacks, so a scan never repeats published history.
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
        return Err(range_error(format!(
            "pushed object {local} is not a commit; the manual-push scan of this object type is not ported"
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
