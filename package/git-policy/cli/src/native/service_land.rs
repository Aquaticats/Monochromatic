//! What:
//!  The service's serialized step: land a batch of prepared commits on a branch, and finish the batches a killed
//!  service left open.
//! Why:
//!  Everything that must happen one commit at a time happens here, in a fixed order, with one durable write: the objects
//!  of the whole batch are written by one `fast-import`, the intent is journaled, the branch moves by one compare-and-swap,
//!  the index entries of the named paths are updated, and the intent is marked done
//!  (`doc/planning/cli-git-resident-service.md`, "The serialized step, group commit, FIFO and backpressure").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await landBatch(context, seq, requests);
//! ```

use super::service_fault::reach;
use super::service_journal::{Intent, Recovery, list_intents, mark_done, recovery_for, write_intent, OPEN_EXTENSION};
use super::service_protocol::{CommitRequest, FailureCode};
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// How many times a batch is built again when the branch moved under it before the failure is reported.
pub const LANDING_ATTEMPTS: usize = 3;

/// How many times the index update is tried while another program holds the index.
const INDEX_ATTEMPTS: usize = 5;

/// The pause between index tries.
const INDEX_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// What the service needs to land: how to run Git and where the journal lives.
pub struct LandContext {
    /// How to run the real Git, with the bounds of `child_class.rs`.
    pub git: GitContext,
    /// Directory of the journal.
    pub journal: PathBuf,
    /// Test seam: called after the batch's objects exist and before the branch moves.
    pub on_objects_written: Option<Box<dyn Fn() + Send + Sync>>,
}

/// What became of one request of a batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The commit is on the branch.
    Landed { commit: String },
    /// A named path changed on the branch since the request's base.
    RetryPrepare { base: String },
}

fn run(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    let output: GitOutput = run_git(git, &request).map_err(|_| { return FailureCode::Unavailable; })?;
    return Ok(output);
}

fn classify(output: &GitOutput) -> FailureCode {
    let text: String = output.error_text().to_lowercase();
    if text.contains("no space left") || text.contains("quota exceeded") {
        return FailureCode::RepositoryFull;
    }
    if text.contains("read-only file system") || text.contains("permission denied") {
        return FailureCode::RepositoryReadOnly;
    }
    if text.contains("index.lock") {
        return FailureCode::IndexInUse;
    }
    return FailureCode::Unavailable;
}

fn checked(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<String, FailureCode> {
    let output: GitOutput = run(git, repository, arguments, input)?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// A path in the quoted form `fast-import` reads, so no byte of a name can end the line or the field.
fn quoted(path: &str) -> String {
    let mut text: String = String::from("\"");
    for character in path.chars() {
        match character {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            other => text.push(other),
        }
    }
    text.push('"');
    return text;
}

/// Whether two repository paths touch the same file or one contains the other.
fn overlaps(path: &str, changed: &str) -> bool {
    return path == changed || path.starts_with(&format!("{changed}/")) || changed.starts_with(&format!("{path}/"));
}

/// The stream that makes one commit per request on a scratch branch that starts at `old`.
fn import_stream(scratch: &str, old: &str, requests: &[&CommitRequest]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        bytes.extend(format!("commit {scratch}\nmark :{}\n", index + 1).as_bytes());
        if let Some(author) = &request.author {
            bytes.extend(format!("author {author}\n").as_bytes());
        }
        bytes.extend(format!("committer {}\ndata {}\n", request.committer, request.message.len()).as_bytes());
        bytes.extend(request.message.as_bytes());
        bytes.push(b'\n');
        if index == 0 {
            bytes.extend(format!("from {old}\n").as_bytes());
        }
        for change in &request.changes {
            match &change.blob {
                Some(blob) => bytes.extend(format!("M {:o} {blob} {}\n", change.mode, quoted(change.path.as_str())).as_bytes()),
                None => bytes.extend(format!("D {}\n", quoted(change.path.as_str())).as_bytes()),
            }
        }
        bytes.push(b'\n');
    }
    bytes.extend(b"done\n");
    return bytes;
}

/// The object names `fast-import` recorded for the marks, in request order.
fn commits_from_marks(text: &str, count: usize) -> Option<Vec<String>> {
    let mut commits: Vec<Option<String>> = vec![None; count];
    for line in text.lines() {
        let mut parts = line.split(' ');
        let (Some(mark), Some(name)) = (parts.next(), parts.next()) else { continue };
        let slot: usize = mark.trim_start_matches(':').parse::<usize>().ok()?;
        if slot >= 1 && slot <= count {
            commits[slot - 1] = Some(name.to_string());
        }
    }
    return commits.into_iter().collect();
}

/// The paths of the repository's index entries a request names, in the form `reset` reads literally.
fn literal(path: &str) -> String {
    return format!(":(top,literal){path}");
}

/// Updates the index entries of `paths` to the commit `new`, trying again while another program holds the index.
fn update_index(git: &GitContext, repository: &Path, new: &str, paths: &[String]) -> Result<(), FailureCode> {
    if paths.is_empty() {
        // `reset` with no path would reset the whole index; an empty commit changes no entry.
        return Ok(());
    }
    let specs: Vec<String> = paths.iter().map(|path| { return literal(path.as_str()); }).collect();
    let mut arguments: Vec<&str> = vec!["reset", "-q", new, "--"];
    arguments.extend(specs.iter().map(String::as_str));
    let mut last: FailureCode = FailureCode::Unavailable;
    for attempt in 0..INDEX_ATTEMPTS {
        match checked(git, repository, &arguments, None) {
            Ok(_) => return Ok(()),
            Err(code) => last = code,
        }
        if attempt + 1 < INDEX_ATTEMPTS {
            std::thread::sleep(INDEX_PAUSE);
        }
    }
    return Err(last);
}

/// What:
///  Lands a batch of requests that share a repository, a common git directory and a branch.
/// Why:
///   Requests whose paths changed on the branch since their base are sent back to be prepared again; the others become
///  one commit each, in order, on the branch, with one journal write and one branch update for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landBatch(context, seq, requests) // one outcome per request, in order
/// ```
pub fn land_batch(context: &LandContext, seq: u64, requests: &[CommitRequest]) -> Result<Vec<Outcome>, FailureCode> {
    let first: &CommitRequest = requests.first().ok_or(FailureCode::Rejected)?;
    if requests.iter().any(|request| {
        return request.repository != first.repository || request.common_dir != first.common_dir || request.reference != first.reference;
    }) {
        return Err(FailureCode::Rejected);
    }
    let repository: &Path = first.repository.as_path();
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    let scratch: String = format!("refs/cli-git-service/{seq}");
    let mut outcomes: Vec<Outcome>;
    let mut attempts_left: usize = LANDING_ATTEMPTS;
    loop {
        let old: String = checked(&git, repository, &["rev-parse", "--verify", first.reference.as_str()], None)?;
        // One tree comparison per distinct base: the paths that changed on the branch since then.
        let mut changed_since: HashMap<&str, BTreeSet<String>> = HashMap::new();
        let mut landing: Vec<(usize, &CommitRequest)> = Vec::new();
        outcomes = vec![Outcome::RetryPrepare { base: old.clone() }; requests.len()];
        for (index, request) in requests.iter().enumerate() {
            if request.base != old && !changed_since.contains_key(request.base.as_str()) {
                let listing: String = checked(&git, repository, &["diff-tree", "-r", "--name-only", "-z", request.base.as_str(), old.as_str()], None)?;
                changed_since.insert(request.base.as_str(), listing.split('\0').filter(|name| { return !name.is_empty(); }).map(str::to_string).collect());
            }
            let conflicts: bool = request.base != old
                && request.changes.iter().any(|change| {
                    return changed_since.get(request.base.as_str()).is_some_and(|names| { return names.iter().any(|name| { return overlaps(change.path.as_str(), name.as_str()); }); });
                });
            if !conflicts {
                landing.push((index, request));
            }
        }
        if landing.is_empty() {
            return Ok(outcomes);
        }
        let batch: Vec<&CommitRequest> = landing.iter().map(|(_, request)| { return *request; }).collect();
        let marks: PathBuf = context.journal.join(format!("marks-{seq}"));
        let marks_argument: String = format!("--export-marks={}", marks.display());
        checked(&git, repository, &["fast-import", "--quiet", "--done", marks_argument.as_str()], Some(import_stream(scratch.as_str(), old.as_str(), &batch)))?;
        let marks_text: String = std::fs::read_to_string(&marks).map_err(|_| { return FailureCode::Unavailable; })?;
        let _marks_removed = std::fs::remove_file(&marks);
        let commits: Vec<String> = commits_from_marks(marks_text.as_str(), batch.len()).ok_or(FailureCode::Unavailable)?;
        let new: String = commits.last().cloned().ok_or(FailureCode::Unavailable)?;
        reach("after-objects");
        if let Some(seam) = &context.on_objects_written {
            seam();
        }
        let paths: Vec<String> = batch.iter().flat_map(|request| { return request.changes.iter().map(|change| { return change.path.clone(); }); }).collect();
        let intent: Intent = Intent {
            seq,
            repository: first.repository.clone(),
            common_dir: first.common_dir.clone(),
            reference: first.reference.clone(),
            old: old.clone(),
            new: new.clone(),
            ids: batch.iter().map(|request| { return request.id.clone(); }).collect(),
            commits: commits.clone(),
            paths: paths.clone(),
        };
        write_intent(context.journal.as_path(), &intent).map_err(|error| { return storage_failure(&error); })?;
        reach("after-intent");
        let transaction: String = format!("start\nupdate {} {new} {old}\ndelete {scratch}\nprepare\ncommit\n", first.reference);
        let moved: GitOutput = run(&git, repository, &["update-ref", "--stdin"], Some(transaction.into_bytes()))?;
        if !moved.succeeded() {
            // The branch moved under the batch: forget the intent and the scratch branch, and build the batch again.
            let _intent_removed = std::fs::remove_file(super::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
            let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
            attempts_left -= 1;
            if attempts_left == 0 {
                return Err(FailureCode::BranchKeepsChanging);
            }
            continue;
        }
        reach("after-ref");
        // The branch has moved: the commits exist whatever happens to the index, which recovery finishes.
        if update_index(&git, repository, new.as_str(), &paths).is_ok() {
            reach("after-index");
            let _marked = mark_done(context.journal.as_path(), seq);
        }
        for (slot, (index, _)) in landing.iter().enumerate() {
            outcomes[*index] = Outcome::Landed { commit: commits[slot].clone() };
        }
        return Ok(outcomes);
    }
}

fn storage_failure(error: &std::io::Error) -> FailureCode {
    return match error.raw_os_error() {
        Some(code) if code == 28 || code == 122 => FailureCode::RepositoryFull,
        Some(code) if code == 30 || code == 13 => FailureCode::RepositoryReadOnly,
        _ => FailureCode::Unavailable,
    };
}

/// One thing a restart did, for the service's log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// The batch.
    pub seq: u64,
    /// What was decided.
    pub decision: Recovery,
}

/// What:
///  Finishes or drops every batch the journal holds open.
/// Why:
///   Runs before the service accepts a connection, and again before each batch for an index update that had to wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recover(context)
/// ```
pub fn recover(context: &LandContext) -> Vec<RecoveryReport> {
    let mut reports: Vec<RecoveryReport> = Vec::new();
    for (path, parsed) in list_intents(context.journal.as_path(), OPEN_EXTENSION) {
        let Some(intent) = parsed else {
            let _removed = std::fs::remove_file(&path);
            continue;
        };
        let git: GitContext = GitContext { common_dir: Some(intent.common_dir.clone()), ..context.git.clone() };
        let current: String = checked(&git, intent.repository.as_path(), &["rev-parse", "--verify", intent.reference.as_str()], None).unwrap_or_default();
        let ancestor: bool = run(&git, intent.repository.as_path(), &["merge-base", "--is-ancestor", intent.new.as_str(), current.as_str()], None)
            .is_ok_and(|output| { return output.succeeded(); });
        let decision: Recovery = recovery_for(&intent, current.as_str(), ancestor);
        match decision {
            Recovery::Drop => {
                let _removed = std::fs::remove_file(&path);
            }
            Recovery::Finish => {
                if update_index(&git, intent.repository.as_path(), intent.new.as_str(), &intent.paths).is_ok() {
                    let _marked = mark_done(context.journal.as_path(), intent.seq);
                }
            }
        }
        reports.push(RecoveryReport { seq: intent.seq, decision });
    }
    return reports;
}

/// Landing controls stay out of the release executable.
#[cfg(test)]
#[path = "service_land_tests.rs"]
mod tests;
