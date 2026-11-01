//! What:
//!  The resident service's side of the worktree copy: it runs the copy requests of a repository one at a time in the
//!  order they were written, settles what an earlier service left half done, and tells the clients that wait.
//! Why:
//!  One process runs every copy, so no copy has to be kept apart from another by a lock, a claim, or a proof that some
//!  process is dead: a request on disk is work to do, a journal on disk is work a killed service left, and the
//!  registration's identity says whether the worktree is still the one the request was written for
//!  (`doc/worktree-copy-claim.md`). A client that waits blocks on its connection; a client that is killed leaves the
//!  copy to the service.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const job of jobs) { await drainCopies(job.commonDir, (event) => reply(job, event)); }
//! ```

use super::json_record::quote;
use super::service_fault::reach;
use super::service_work::{notice_directory, write_notice};
use super::transaction_git::GitContext;
use super::worktree_copy_error::{WorktreeCopyError, raw_failure};
use super::worktree_copy_journal::{
    Role, assert_private_path, create_journal_for, remove_pending_journal, remove_stage,
};
use super::worktree_copy_recovery::{
    Finished, Registration, Step, finish_journal, recover_worktree_copy_transactions, registration_of, report_recovered,
};
use super::worktree_copy_publish::reconcile_intents;
use super::worktree_copy_registry::{WorktreeCopyObservation, admin_identity, copied_for, is_locked, registered_roots_of};
use super::worktree_copy_request::{
    CopyRequest, DoneKind, MAX_TRIES, Terminal, adopt_temporaries, finish_request, give_up, given_up_directory,
    read_requests, remove_request, terminal_of, write_request,
};
use super::worktree_copy_stage::stage_ignored_snapshot;
use super::worktree_copy_transaction::Outcome;
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::time::{SystemTime, UNIX_EPOCH};

/// What one request came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CopyOutcome {
    /// The copy ran; the count is the number of entries it created.
    Copied(usize),
    /// The registration was already marked copied, so nothing ran.
    AlreadyCopied,
    /// The worktree was removed, or replaced by another registration, so nothing was created.
    Removed,
    /// The copy did not run yet and stays on disk without a try counted: its worktree is locked by another command, or the
    ///  worktree it copies from is not filled yet. The text says why.
    Waiting(String),
    /// The copy failed; the text says why. The request stays for a later run unless it was given up on.
    Failed(String),
}

/// What a drain tells its caller as it goes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CopyEvent {
    /// A copy begins.
    Started {
        /// The request's identity.
        id: String,
        /// The worktree it fills.
        destination: PathBuf,
    },
    /// A line for the person about something the drain settled.
    Notice(String),
    /// A request ended.
    Done {
        /// The request's identity.
        id: String,
        /// How it ended.
        outcome: CopyOutcome,
    },
}

/// The points of one copy at which a test can stop the service, as a kill would: before staging, with the journal
///  written, with the files installed, and with the registration marked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrainStep {
    /// Nothing is staged yet.
    Staging,
    /// The stage is complete and journaled; nothing is installed.
    Staged,
    /// The entries are installed; the registration is not marked.
    Installed,
    /// The registration is marked; the journal and stage still exist.
    Marked,
}

impl DrainStep {
    /// The service fault point named after the step, which `service_fault::reach` can end the process at.
    pub fn fault_point(self) -> &'static str {
        return match self {
            DrainStep::Staging => "copy-staging",
            DrainStep::Staged => "copy-staged",
            DrainStep::Installed => "copy-installed",
            DrainStep::Marked => "copy-marked",
        };
    }
}

/// How a drain ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Drained {
    /// Every pending request was run.
    Complete,
    /// A checkpoint function asked to stop, so the state stands as a kill at that point leaves it.
    Interrupted,
}

/// The text of a failure.
fn text_of(error: &WorktreeCopyError) -> String {
    return error.0.clone();
}

/// What:
///  Orders requests so that a worktree that is another request's source is filled first.
/// Why:
///   A worktree that another command copied from while it was still being filled would pass on a partial snapshot; the
///  request that fills it runs first, and the one that copies from it sees the result. Requests otherwise run in the
///  order they were written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// orderRequests(requests)
/// ```
pub fn order_requests(requests: Vec<CopyRequest>) -> Vec<CopyRequest> {
    let mut remaining: Vec<CopyRequest> = requests;
    let mut ordered: Vec<CopyRequest> = Vec::new();
    while !remaining.is_empty() {
        // The first request whose source is not the destination of another request still to run; with a cycle, the first.
        let next: usize = remaining
            .iter()
            .position(|candidate| {
                return !remaining.iter().any(|other| {
                    return other.id != candidate.id && other.destination_root == candidate.source_root;
                });
            })
            .unwrap_or(0);
        ordered.push(remaining.remove(next));
    }
    return ordered;
}

/// Removes the stage container a run of this request left, if any.
fn remove_leftover_stage(request: &CopyRequest) -> Result<(), WorktreeCopyError> {
    let container: PathBuf = request.stage_container();
    // `match` on the lookup: absent is fine, present must be a private stage before it is removed.
    match std::fs::symlink_metadata(container.as_path()) {
        Ok(_) => {
            assert_private_path(container.as_path(), Role::PrivateStage)?;
            return remove_stage(container.as_path());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// The text of a copy that was given up on, from its record.
pub fn given_up_text(request: &CopyRequest, common_dir: &Path) -> String {
    return format!(
        "cli-git: the ignored-state copy into {} was tried {} times and is given up on; the request is kept in {}.",
        quote(request.destination_root.to_string_lossy().as_ref()),
        request.tries.max(MAX_TRIES),
        quote(given_up_directory(common_dir).to_string_lossy().as_ref())
    );
}

/// Records a failed run: the request is tried again later, or given up on after `MAX_TRIES`.
fn record_failure(common_dir: &Path, request: &CopyRequest, failure: &WorktreeCopyError) -> CopyOutcome {
    let tried: CopyRequest = CopyRequest { tries: request.tries + 1, ..request.clone() };
    let giving_up: bool = tried.tries >= MAX_TRIES;
    let kept: Result<(), WorktreeCopyError> =
        if giving_up { give_up(common_dir, &tried) } else { write_request(common_dir, &tried) };
    if let Err(error) = kept {
        return CopyOutcome::Failed(format!("{} The request could not be kept: {}", failure.0, error.0));
    }
    if giving_up {
        return CopyOutcome::Failed(format!("{} {}", failure.0, given_up_text(&tried, common_dir)));
    }
    return CopyOutcome::Failed(failure.0.clone());
}

/// Whether the request's registration is gone, or is no longer the directory the wrapper observed.
fn is_replaced(common_dir: &Path, request: &CopyRequest) -> Result<bool, WorktreeCopyError> {
    let admin_root: PathBuf = common_dir.join("worktrees");
    let observed: bool = admin_identity(admin_root.as_path(), request.admin_id.as_str())?.as_deref() == Some(request.identity.as_str());
    if !observed {
        return Ok(true);
    }
    return Ok(registration_of(
        common_dir,
        request.destination_root.as_path(),
        request.admin_id.as_str(),
        request.registration.as_str(),
    )? == Registration::Unregistered);
}

/// What:
///  Runs one request, or stops where a checkpoint function says.
/// Why:
///   The registration is checked by identity before anything is staged and again after the stage is journaled, which
///  takes minutes; the files are installed only into the registration the request was written for, and the marker is
///  durable before the journal goes. A worktree that Git still has locked is another command's to finish creating, so
///  the request waits for it unless its creator asked for the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await runRequest({ context, commonDir, request, events, checkpoint });
/// ```
fn run_request(
    context: &GitContext,
    common_dir: &Path,
    request: &CopyRequest,
    events: &mut dyn FnMut(CopyEvent),
    checkpoint: &mut dyn FnMut(&CopyRequest, DrainStep) -> bool,
) -> Result<Option<CopyOutcome>, WorktreeCopyError> {
    let admin_root: PathBuf = common_dir.join("worktrees");
    if copied_for(admin_root.as_path(), request.admin_id.as_str(), request.registration.as_str())? {
        remove_leftover_stage(request)?;
        finish_request(common_dir, request, DoneKind::Already)?;
        return Ok(Some(CopyOutcome::AlreadyCopied));
    }
    let removed_before_staging: bool = is_replaced(common_dir, request)?;
    if removed_before_staging {
        remove_leftover_stage(request)?;
        finish_request(common_dir, request, DoneKind::Removed)?;
        return Ok(Some(CopyOutcome::Removed));
    }
    let still_locked: bool = !request.allow_locked && is_locked(admin_root.as_path(), request.admin_id.as_str())?;
    if still_locked {
        return Ok(Some(CopyOutcome::Waiting(format!(
            "cli-git: the ignored-state copy into {} waits: Git still has that worktree locked, which means another command is creating it. It runs when the lock is gone.",
            quote(request.destination_root.to_string_lossy().as_ref())
        ))));
    }
    events(CopyEvent::Started { id: request.id.clone(), destination: request.destination_root.clone() });
    // A run of this request that was cut off left its stage under this name; it is removed before the stage is made again.
    remove_leftover_stage(request)?;
    if !checkpoint(request, DrainStep::Staging) {
        return Ok(None);
    }
    let observation: WorktreeCopyObservation = WorktreeCopyObservation {
        admin_root: admin_root.clone(),
        before_admin_ids: BTreeSet::new(),
        common_dir: common_dir.to_path_buf(),
        effective_cwd: request.source_root.clone(),
        source_root: Some(request.source_root.clone()),
        selection: request.selection.clone(),
    };
    let registered_roots: Vec<PathBuf> = registered_roots_of(context, &observation)?;
    let snapshot = stage_ignored_snapshot(
        context,
        request.source_root.as_path(),
        request.stage_container().as_path(),
        registered_roots.as_slice(),
        &request.selection,
    )?;
    let pending = match create_journal_for(
        common_dir,
        request.destination_root.as_path(),
        &snapshot,
        Some((request.admin_id.as_str(), request.registration.as_str())),
    ) {
        Ok(pending) => pending,
        Err(error) => {
            remove_leftover_stage(request)?;
            return Err(error);
        }
    };
    if !checkpoint(request, DrainStep::Staged) {
        return Ok(None);
    }
    let removed_while_staging: bool = is_replaced(common_dir, request)?;
    if removed_while_staging {
        remove_pending_journal(&pending)?;
        finish_request(common_dir, request, DoneKind::Removed)?;
        return Ok(Some(CopyOutcome::Removed));
    }
    let staged = snapshot.clone();
    let finished: Finished = finish_journal(common_dir, pending, &|_intended| return Ok(staged.clone()), &mut |step| {
        return checkpoint(request, match step {
            Step::Installed => DrainStep::Installed,
            Step::Marked => DrainStep::Marked,
        });
    })?;
    return match finished {
        Finished::Interrupted => Ok(None),
        Finished::Done(Outcome::Installed(copied)) => {
            finish_request(common_dir, request, DoneKind::Copied(copied))?;
            Ok(Some(CopyOutcome::Copied(copied)))
        }
        Finished::Done(Outcome::Ended(failure)) => Ok(Some(record_failure(common_dir, request, &failure))),
    };
}

/// The destinations whose copy was given up on and whose registration is not marked copied: worktrees that stay unfilled.
fn unfilled_given_up(common_dir: &Path) -> HashSet<PathBuf> {
    let mut unfilled: HashSet<PathBuf> = HashSet::new();
    let admin_root: PathBuf = common_dir.join("worktrees");
    let Ok(listing) = std::fs::read_dir(given_up_directory(common_dir)) else { return unfilled };
    for listed in listing.flatten() {
        let Ok(text) = std::fs::read_to_string(listed.path()) else { continue };
        let Ok(request) = CopyRequest::decode(text.as_str()) else { continue };
        let filled: bool =
            copied_for(admin_root.as_path(), request.admin_id.as_str(), request.registration.as_str()).unwrap_or(false);
        if !filled {
            unfilled.insert(request.destination_root);
        }
    }
    return unfilled;
}

/// The milliseconds since the epoch, for the age of an intent.
fn now_milliseconds() -> u128 {
    return SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| return elapsed.as_millis());
}

/// What:
///  Runs everything pending for one repository: settles the journals an earlier service left, finds the worktrees a killed
///  wrapper created, then runs every request in order, and tells the caller what happens.
/// Why:
///   The service is the only process that runs copies, so anything on disk is work to do. The requests are read again
///  after each run, so one a wrapper published meanwhile is ordered with the rest; a request whose source worktree has
///  not been filled (its own copy failed, waits, or was given up on) waits for it instead of copying an empty directory.
///  A request that failed stays for the next run, and one that failed `MAX_TRIES` times is set aside with a notice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainCopies(context, commonDir, events, checkpoint)
/// ```
pub fn drain_copies(
    context: &GitContext,
    common_dir: &Path,
    events: &mut dyn FnMut(CopyEvent),
    checkpoint: &mut dyn FnMut(&CopyRequest, DrainStep) -> bool,
) -> Result<Drained, String> {
    let report = recover_worktree_copy_transactions(common_dir).map_err(|error| return text_of(&error))?;
    let mut lines: Vec<u8> = Vec::new();
    report_recovered(&report, &mut lines).map_err(|error| return text_of(&error))?;
    for line in String::from_utf8_lossy(lines.as_slice()).lines() {
        events(CopyEvent::Notice(line.to_string()));
    }
    let mut handled: HashSet<String> = HashSet::new();
    let mut unfinished: HashSet<PathBuf> = HashSet::new();
    loop {
        let _adopted = adopt_temporaries(common_dir);
        match reconcile_intents(common_dir, now_milliseconds()) {
            Ok(notices) => {
                for line in notices {
                    events(CopyEvent::Notice(line));
                }
            }
            Err(failure) => events(CopyEvent::Notice(failure.0)),
        }
        let (requests, refused) = read_requests(common_dir).map_err(|error| return text_of(&error))?;
        for line in refused {
            events(CopyEvent::Notice(line));
        }
        let waiting: Vec<CopyRequest> = requests.into_iter().filter(|request| return !handled.contains(&request.id)).collect();
        let Some(request) = order_requests(waiting).into_iter().next() else { break };
        handled.insert(request.id.clone());
        // A request whose record of being given up on is already durable only lost its removal to a kill.
        if matches!(terminal_of(common_dir, request.id.as_str()), Ok(Terminal::GivenUp(_))) {
            remove_request(common_dir, &request).map_err(|error| return text_of(&error))?;
            continue;
        }
        let outcome: CopyOutcome = if unfinished.contains(&request.source_root) {
            CopyOutcome::Waiting(format!(
                "cli-git: the ignored-state copy into {} waits: it copies from {}, which has not been filled yet. It runs after that copy does.",
                quote(request.destination_root.to_string_lossy().as_ref()),
                quote(request.source_root.to_string_lossy().as_ref())
            ))
        } else if unfilled_given_up(common_dir).contains(&request.source_root) {
            let ended: CopyRequest = CopyRequest { tries: MAX_TRIES, ..request.clone() };
            let reason: String = format!(
                "cli-git: the ignored-state copy into {} copies from {}, whose own copy was given up on and which is still not filled.",
                quote(request.destination_root.to_string_lossy().as_ref()),
                quote(request.source_root.to_string_lossy().as_ref())
            );
            match give_up(common_dir, &ended) {
                Ok(()) => CopyOutcome::Failed(format!("{reason} {}", given_up_text(&ended, common_dir))),
                Err(error) => CopyOutcome::Failed(format!("{reason} The request could not be set aside: {}", error.0)),
            }
        } else {
            let ended: Result<Option<CopyOutcome>, WorktreeCopyError> =
                run_request(context, common_dir, &request, events, checkpoint);
            match ended {
                Ok(Some(outcome)) => outcome,
                Ok(None) => return Ok(Drained::Interrupted),
                Err(failure) => record_failure(common_dir, &request, &failure),
            }
        };
        if matches!(outcome, CopyOutcome::Failed(_) | CopyOutcome::Waiting(_)) {
            unfinished.insert(request.destination_root.clone());
        }
        events(CopyEvent::Done { id: request.id.clone(), outcome });
    }
    return Ok(Drained::Complete);
}

//region The wire: one JSON object per line, as the rest of the service's conversation

/// What a copy client reads from the service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CopyReply {
    /// A copy begins.
    Started {
        /// The request's identity.
        id: String,
        /// The worktree it fills.
        destination: String,
    },
    /// A line for the person.
    Notice(String),
    /// A request ended.
    Done {
        /// The request's identity.
        id: String,
        /// How it ended.
        outcome: CopyOutcome,
    },
    /// Everything this connection asked about has been answered.
    Finished,
}

fn object(entries: Vec<(&str, Value)>) -> String {
    let mut map: Map<String, Value> = Map::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    return Value::Object(map).to_string();
}

/// The line for a reply.
pub fn encode_copy_reply(reply: &CopyReply) -> String {
    return match reply {
        CopyReply::Started { id, destination } => object(vec![
            ("type", Value::from("copy-started")),
            ("id", Value::from(id.as_str())),
            ("destination", Value::from(destination.as_str())),
        ]),
        CopyReply::Notice(text) => object(vec![("type", Value::from("copy-notice")), ("text", Value::from(text.as_str()))]),
        CopyReply::Finished => object(vec![("type", Value::from("copy-finished"))]),
        CopyReply::Done { id, outcome } => {
            let mut entries: Vec<(&str, Value)> = vec![("type", Value::from("copy-done")), ("id", Value::from(id.as_str()))];
            match outcome {
                CopyOutcome::Copied(count) => {
                    entries.push(("outcome", Value::from("copied")));
                    entries.push(("entries", Value::from(*count)));
                }
                CopyOutcome::AlreadyCopied => entries.push(("outcome", Value::from("already"))),
                CopyOutcome::Removed => entries.push(("outcome", Value::from("removed"))),
                CopyOutcome::Waiting(text) => {
                    entries.push(("outcome", Value::from("waiting")));
                    entries.push(("text", Value::from(text.as_str())));
                }
                CopyOutcome::Failed(text) => {
                    entries.push(("outcome", Value::from("failed")));
                    entries.push(("text", Value::from(text.as_str())));
                }
            }
            object(entries)
        }
    };
}

/// Reads one reply line, strictly.
pub fn decode_copy_reply(line: &str) -> Result<CopyReply, String> {
    let value: Value = serde_json::from_str(line).map_err(|error| return error.to_string())?;
    let text = |name: &str| -> Result<String, String> {
        return value.get(name).and_then(Value::as_str).map(String::from).ok_or_else(|| return format!("field {name} is missing"));
    };
    return match text("type")?.as_str() {
        "copy-started" => Ok(CopyReply::Started { id: text("id")?, destination: text("destination")? }),
        "copy-notice" => Ok(CopyReply::Notice(text("text")?)),
        "copy-finished" => Ok(CopyReply::Finished),
        "copy-done" => {
            let outcome: CopyOutcome = match text("outcome")?.as_str() {
                "copied" => CopyOutcome::Copied(
                    value
                        .get("entries")
                        .and_then(Value::as_u64)
                        .and_then(|count| return usize::try_from(count).ok())
                        .ok_or("entries is missing")?,
                ),
                "already" => CopyOutcome::AlreadyCopied,
                "removed" => CopyOutcome::Removed,
                "waiting" => CopyOutcome::Waiting(text("text")?),
                "failed" => CopyOutcome::Failed(text("text")?),
                other => return Err(format!("unknown outcome {other}")),
            };
            Ok(CopyReply::Done { id: text("id")?, outcome })
        }
        other => Err(format!("unknown reply {other}")),
    };
}

//endregion

//region The service's copy loop

/// One client's request to run copies: which repository, which requests it waits for, and where to answer.
pub struct CopyJob {
    /// The repository's common Git directory.
    pub common_dir: PathBuf,
    /// The requests the client waits for; empty when the client only woke the service and waits for nothing.
    pub ids: Vec<String>,
    /// The connection to answer on.
    pub stream: UnixStream,
}

/// What runs the copies of a repository.
pub trait CopyRunner: Sync {
    /// Runs everything pending for the repository and reports each event.
    fn drain(&self, common_dir: &Path, events: &mut dyn FnMut(CopyEvent)) -> Result<(), String>;
}

/// The runner that stages, installs and marks with real Git.
pub struct GitCopyRunner {
    /// The context every Git child of the service runs in.
    pub context: GitContext,
}

impl CopyRunner for GitCopyRunner {
    fn drain(&self, common_dir: &Path, events: &mut dyn FnMut(CopyEvent)) -> Result<(), String> {
        // The fault points end the process the way a kill does, for the tests that kill the service at each durable point.
        return drain_copies(&self.context, common_dir, events, &mut |_request, step| {
            reach(step.fault_point());
            return true;
        })
        .map(|_drained| {});
    }
}

/// Sends one reply; false when the client is gone.
fn send(stream: &UnixStream, reply: &CopyReply) -> bool {
    let mut line: String = encode_copy_reply(reply);
    line.push('\n');
    return (&*stream).write_all(line.as_bytes()).is_ok();
}

/// The key of the notice that carries the failure of one request until its client has shown it.
pub fn copy_notice_key(id: &str) -> String {
    return format!("copy-{id}");
}

/// What:
///  Withdraws the notice of one request's failure.
/// Why:
///   The service leaves a notice before it tells a waiting client about a failure, because a successful write to a socket
///  does not prove the client read it. The client withdraws the notice once it has shown the failure itself, so a client
///  that was killed first leaves the notice for the next commit, and one that showed it leaves none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(join(noticeDirectory(commonDir), `${copyNoticeKey(id)}.txt`), { force: true })
/// ```
pub fn withdraw_notice(common_dir: &Path, id: &str) {
    let _removed = std::fs::remove_file(notice_directory(common_dir).join(format!("{}.txt", copy_notice_key(id))));
}

/// How a request ended, from the disk alone, for a client the service did not tell while it ran.
fn outcome_on_disk(common_dir: &Path, id: &str) -> CopyOutcome {
    return match terminal_of(common_dir, id) {
        Ok(Terminal::Done(DoneKind::Copied(count))) => CopyOutcome::Copied(count),
        Ok(Terminal::Done(DoneKind::Already)) => CopyOutcome::AlreadyCopied,
        Ok(Terminal::Done(DoneKind::Removed)) => CopyOutcome::Removed,
        Ok(Terminal::GivenUp(request)) => CopyOutcome::Failed(given_up_text(&request, common_dir)),
        Ok(Terminal::Pending) => CopyOutcome::Waiting(format!(
            "cli-git: the ignored-state copy request {id} has not been run yet; it runs the next time the commit service is asked."
        )),
        Ok(Terminal::Unknown) => CopyOutcome::Failed(format!(
            "cli-git: the commit service has no record of how the ignored-state copy request {id} ended, so the files may not have been copied."
        )),
        Err(error) => CopyOutcome::Failed(error.0),
    };
}

/// One client's conversation: which requests it still waits for, and whether it can still be told.
struct Conversation<'a> {
    job: &'a CopyJob,
    wanted: HashSet<String>,
    answered: HashSet<String>,
    reachable: bool,
}

impl Conversation<'_> {
    fn tell(&mut self, reply: &CopyReply) {
        if self.reachable {
            self.reachable = send(&self.job.stream, reply);
        }
    }

    /// Tells the client how one of its requests ended, and that everything it asked about is answered after the last.
    fn done(&mut self, id: &str, outcome: &CopyOutcome) {
        if !self.wanted.contains(id) || !self.answered.insert(id.to_string()) {
            return;
        }
        self.tell(&CopyReply::Done { id: id.to_string(), outcome: outcome.clone() });
        if self.answered.len() == self.wanted.len() {
            self.tell(&CopyReply::Finished);
        }
    }
}

/// What:
///  Serves one client: runs what is pending unless every request it named has an answer already, and answers for the
///  requests the client named as soon as each one ends.
/// Why:
///   A client that connected after another one's run finished the same requests still gets the outcomes, from the run's
///  memory or from the disk's record, and a request nobody knows is a failure, never a success. A failure is written as a
///  notice before the client is told, and the client withdraws it after showing the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await serveJob(runner, job, outcomes)
/// ```
fn serve_job(runner: &dyn CopyRunner, job: &CopyJob, outcomes: &mut HashMap<String, CopyOutcome>) {
    let mut conversation: Conversation = Conversation {
        job,
        wanted: job.ids.iter().cloned().collect(),
        answered: HashSet::new(),
        reachable: true,
    };
    let common_dir: &Path = job.common_dir.as_path();
    let must_run: bool = job.ids.is_empty()
        || job.ids.iter().any(|id| {
            return !outcomes.contains_key(id) && matches!(terminal_of(common_dir, id.as_str()), Ok(Terminal::Pending));
        });
    if must_run {
        let drained: Result<(), String> = runner.drain(common_dir, &mut |event| match event {
            CopyEvent::Started { id, destination } => {
                if conversation.wanted.contains(&id) {
                    conversation.tell(&CopyReply::Started { id, destination: destination.display().to_string() });
                }
            }
            CopyEvent::Notice(text) => {
                if !job.ids.is_empty() {
                    conversation.tell(&CopyReply::Notice(text));
                }
            }
            CopyEvent::Done { id, outcome } => {
                outcomes.insert(id.clone(), outcome.clone());
                if let CopyOutcome::Failed(text) = &outcome {
                    let _noticed = write_notice(common_dir, copy_notice_key(id.as_str()).as_str(), text.as_str());
                }
                conversation.done(id.as_str(), &outcome);
            }
        });
        if let Err(text) = drained {
            let unanswered: Vec<String> = job.ids.iter().filter(|id| return !conversation.answered.contains(*id)).cloned().collect();
            for id in unanswered {
                let outcome: CopyOutcome = CopyOutcome::Failed(text.clone());
                let _noticed = write_notice(common_dir, copy_notice_key(id.as_str()).as_str(), text.as_str());
                conversation.done(id.as_str(), &outcome);
            }
            if job.ids.is_empty() {
                let _noticed = write_notice(common_dir, "copy-recovery", text.as_str());
            }
        }
    }
    for id in &job.ids {
        if conversation.answered.contains(id) {
            continue;
        }
        let outcome: CopyOutcome = outcomes.get(id).cloned().unwrap_or_else(|| return outcome_on_disk(common_dir, id.as_str()));
        if let CopyOutcome::Failed(text) = &outcome {
            let _noticed = write_notice(common_dir, copy_notice_key(id.as_str()).as_str(), text.as_str());
        }
        conversation.done(id.as_str(), &outcome);
    }
}

/// What:
///  The loop of the thread that runs copies: takes every job that is waiting, serves them in arrival order, and goes back
///  to waiting.
/// Why:
///   One thread runs every copy, so two copies never overlap, whatever the number of clients; `busy` keeps the service
///  from exiting for idleness while a copy runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const jobs of batches(receiver)) for (const job of jobs) await serveJob(runner, job, outcomes);
/// ```
pub fn copy_loop(receiver: &Receiver<CopyJob>, runner: &dyn CopyRunner, busy: &AtomicUsize) {
    // What the requests that ended came to, kept for clients that ask after another client's run finished them. Small
    // records, bounded by the cap, and forgotten with the service.
    let mut outcomes: HashMap<String, CopyOutcome> = HashMap::new();
    while let Ok(first) = receiver.recv() {
        busy.fetch_add(1, Ordering::SeqCst);
        let jobs: Vec<CopyJob> = std::iter::once(first).chain(std::iter::from_fn(|| return receiver.try_recv().ok())).collect();
        for job in &jobs {
            serve_job(runner, job, &mut outcomes);
        }
        if outcomes.len() > REMEMBERED_OUTCOMES {
            outcomes.clear();
        }
        busy.fetch_sub(1, Ordering::SeqCst);
    }
}

/// How many outcomes the copy loop remembers before it forgets them all.
const REMEMBERED_OUTCOMES: usize = 4096;

//endregion

/// Copy-service controls stay out of the release executable.
#[cfg(test)]
#[path = "service_copy_tests.rs"]
pub(crate) mod tests;
