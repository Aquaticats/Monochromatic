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

use crate::journald_native::Priority;
use crate::service_fault::reach;
use crate::service_built_checks::{BuiltCommit, adds_only_present_objects, read_commit};
use crate::service_head::{follows_move, still_checked_out};
use crate::service_pruned::batch_content_missing;
use crate::service_journal::{Intent, OPEN_EXTENSION, remove_start, write_intent, write_start};
pub use crate::service_land_finish::{RecoveryReport, recover, recover_instance};
#[cfg(test)]
use crate::service_land_finish::follow_up;
#[cfg(test)]
use crate::service_journal::{Recovery, list_intents};
#[cfg(test)]
use crate::service_index_lock::try_acquire;
use crate::service_land_finish::finish_after_move;
use crate::service_land_start::witness_of;
use crate::service_index_lock::{Announce, IndexLock, WaitNotes, acquire_noting_wait_within};
use crate::service_timing::{LandTiming, wait_line};
use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
use crate::service_land_follow::{classify, storage_failure};
use crate::service_protocol::{CommitRequest, FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// How many times a batch is built again when the branch moved under it before the failure is reported.
pub const LANDING_ATTEMPTS: usize = 3;

/// What the service needs to land: how to run Git and where the journal lives.
pub struct LandContext {
    /// How to run the real Git, with the bounds of `child_class.rs`.
    pub git: GitContext,
    /// Directory of the journal.
    pub journal: PathBuf,
    /// Test seam: called after the batch's objects exist and before the branch moves.
    pub on_objects_written: Option<Box<dyn Fn() + Send + Sync>>,
    /// How long a landing waits for a foreign program's `index.lock` before it is withdrawn (`wait_ceilings.rs`).
    pub lock_ceiling: std::time::Duration,
}

/// What:
///  Keeps the commit built for a batch that was withdrawn on a side ref, and returns the line that tells the person where it is.
/// Why:
///   The owner's answer for a wait that reaches its bound is that nothing lands later by itself and the commit is kept for a retry;
///  the person's staged changes are untouched, so running the commit again builds the same commit, and the side ref lets them
///  recover this one if the worktree changed meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['update-ref', `refs/cli-git/kept/${seq}`, tip]); return `kept at refs/cli-git/kept/${seq}`
/// ```
fn keep_unlanded(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> String {
    let name: String = format!("refs/cli-git/kept/{seq}");
    let made = run(git, repository, &["update-ref", name.as_str(), tip], None);
    if !made.is_ok_and(|output| { return output.succeeded(); }) {
        return String::from("cli-git: the commit built for you could not be kept on a side ref; your staged changes and your worktree are as they were, so run the commit again when the program above is done.\n");
    }
    return format!(
        "cli-git: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
    );
}

/// What became of one request of a batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The commit is on the branch.
    Landed { commit: String },
    /// A named path changed on the branch since the request's base.
    RetryPrepare { base: String },
    /// An amend, whose parents are the base's parents, found the branch somewhere else than at its base.
    BranchMoved,
}

fn run(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    let output: GitOutput = run_git(git, &request).map_err(|error| {
        // A step ended at its ceiling is told apart from every other failure, and its sentence is kept for the answer.
        if crate::child_watch::is_ceiling(&error) {
            CEILING_STEP.with(|step| { *step.borrow_mut() = Some(error.to_string()); });
            return FailureCode::StepTooSlow;
        }
        return FailureCode::Unavailable;
    })?;
    return Ok(output);
}

thread_local! {
    /// The sentence of the last Git step this thread ended at its ceiling, for the line the clients are told.
    static CEILING_STEP: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// The sentence of the step that was ended at its ceiling, once.
fn take_ceiling_step() -> Option<String> {
    return CEILING_STEP.with(|step| { return step.borrow_mut().take(); });
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

/// The stream that makes one commit per request on a scratch branch that starts at `old`, or, for an amend, one commit
///  whose parents are `amended_parents` and whose whole tree is the request's tree.
fn import_stream(scratch: &str, old: &str, requests: &[&CommitRequest], amended_parents: &[String]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        bytes.extend(format!("commit {scratch}\nmark :{}\n", index + 1).as_bytes());
        if let Some(author) = &request.author {
            bytes.extend(format!("author {author}\n").as_bytes());
        }
        bytes.extend(format!("committer {}\ndata {}\n", request.committer, request.message.len()).as_bytes());
        bytes.extend(request.message.as_bytes());
        bytes.push(b'\n');
        if let Some(tree) = request.tree.as_deref().filter(|_| { return request.amend; }) {
            // An amend replaces the tip: its parents are the base's parents, and its tree is the one the client wrote.
            for (position, parent) in amended_parents.iter().enumerate() {
                bytes.extend(format!("{} {parent}\n", if position == 0 { "from" } else { "merge" }).as_bytes());
            }
            bytes.extend(format!("M 040000 {tree} \"\"\n").as_bytes());
            bytes.push(b'\n');
            continue;
        }
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
    return land_batch_announcing(context, seq, requests, &|_| {});
}

/// `land_batch`, telling the clients of the batch what it waits for when another program holds the index.
pub fn land_batch_announcing(context: &LandContext, seq: u64, requests: &[CommitRequest], announce: &dyn Fn(Announce)) -> Result<Vec<Outcome>, FailureCode> {
    return land_batch_timed(context, seq, requests, announce, &mut LandTiming::default());
}

/// What:
///  `land_batch_announcing`,
/// leaving in `timing` the time each phase of the landing took.
/// Why:
///   The service's log line for each request states the wait for another program's hold on the index,
/// the objects,
/// the journal
///  write,
/// the branch move and the index update;
/// `timing` is filled as the phases end,
/// so a landing that fails part way still
///  reports the phases it reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landBatchTimed(context, seq, requests, announce, timing)
/// ```
///
/// # Errors
///
/// The failure code of the landing,
/// as `land_batch_announcing`.
pub fn land_batch_timed(
    context: &LandContext,
    seq: u64,
    requests: &[CommitRequest],
    announce: &dyn Fn(Announce),
    timing: &mut LandTiming,
) -> Result<Vec<Outcome>, FailureCode> {
    // The landing queues its own syncs and settles them when it ends: nothing is flushed while the person waits for a commit that
    //  is published, and what a landing that publishes nothing wrote is flushed before it returns (`service_durable.rs`). The
    //  index file follows `core.fsync` as native Git's index write does; a configuration that cannot be read is taken to name it.
    let snapshot = requests.first().and_then(|first| { return read_config_snapshot(&context.git, first.repository.as_path()); });
    let hardens_index: bool = snapshot.as_ref().and_then(ConfigSnapshot::hardens_index).unwrap_or(true);
    // Native Git writes a commit's objects as loose objects and flushes none by default; the import of a small batch writes a
    //  temporary pack, flushes it and deletes it, so the flush is dropped unless the repository asks for flushed objects.
    let import_fsync: Option<String> = snapshot.as_ref().and_then(ConfigSnapshot::import_fsync_value);
    let deferral = crate::service_durable::begin(hardens_index);
    let result: Result<Vec<Outcome>, FailureCode> = land_batch_body(context, seq, requests, announce, timing, import_fsync.as_deref());
    // A step ended at its ceiling anywhere else in the landing: the clients are told which, and the commit the batch built, when it
    //  built one, is kept on a side ref. No open intent finishes a batch whose branch has not moved, so nothing lands later.
    if result == Err(FailureCode::StepTooSlow)
        && let Some(step) = take_ceiling_step()
        && let Some(first) = requests.first()
    {
        announce(Announce::Waiting(format!("cli-git: {step}, so this commit was not made.\n")));
        let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
        let scratch: String = format!("refs/worktree/cli-git-service/{seq}");
        if let Ok(tip) = checked(&git, first.repository.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()], None) {
            announce(Announce::Waiting(keep_unlanded(&git, first.repository.as_path(), seq, tip.as_str())));
            let _deleted = run(&git, first.repository.as_path(), &["update-ref", "-d", scratch.as_str()], None);
        }
    }
    let published: bool = result.as_ref().is_ok_and(|outcomes| { return outcomes.iter().any(|outcome| { return matches!(outcome, Outcome::Landed { .. }); }); });
    deferral.settle(published);
    return result;
}

/// The landing itself; `land_batch_timed` adds what is told when a step is ended at its ceiling.
///
/// # Errors
///
/// As `land_batch_timed`.
fn land_batch_body(
    context: &LandContext,
    seq: u64,
    requests: &[CommitRequest],
    announce: &dyn Fn(Announce),
    timing: &mut LandTiming,
    import_fsync: Option<&str>,
) -> Result<Vec<Outcome>, FailureCode> {
    let first: &CommitRequest = requests.first().ok_or(FailureCode::Rejected)?;
    if requests.iter().any(|request| {
        return request.repository != first.repository || request.common_dir != first.common_dir || request.reference != first.reference;
    }) {
        return Err(FailureCode::Rejected);
    }
    // An amend, and a commit built elsewhere, are landed alone: each is only valid while the branch is still at its base,
    //  which a neighbour in the same batch would change.
    let alone: bool = requests.iter().any(CommitRequest::lands_alone);
    if alone && requests.len() != 1 {
        return Err(FailureCode::Rejected);
    }
    let repository: &Path = first.repository.as_path();
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    let scratch: String = format!("refs/worktree/cli-git-service/{seq}");
    let mut outcomes: Vec<Outcome>;
    let mut attempts_left: usize = LANDING_ATTEMPTS;
    // A branch that does not exist yet is prepared against the all-zero name and lands only while it still does not.
    let unborn: bool = is_zero_object(first.base.as_str());
    loop {
        timing.attempts += 1;
        // The commit the branch names, asked in the one spelling the in-process readers answer (`<ref>^{commit}`), so the question
        //  starts no Git; for a branch, which only ever names a commit, the answer is the same as for the bare name.
        let branch_commit: String = format!("{}^{{commit}}", first.reference);
        let old: String = match checked(&git, repository, &["rev-parse", "--verify", "--quiet", branch_commit.as_str()], None) {
            Ok(found) => found,
            Err(_) if unborn => first.base.clone(),
            Err(code) => return Err(code),
        };
        if alone && first.base != old {
            return Ok(vec![Outcome::BranchMoved]);
        }
        // Native Git updates `HEAD`, so a worktree that switched branches while the commit waited fails the commit with nothing
        //  changed. The named branch would move all the same, and the index built for it would replace the other branch's. A
        //  symbolic `HEAD` is not refused by Git's own compare when it is told not to follow it (measured: it replaces the
        //  symbolic reference), so a head that became a branch is found here, before the move. The same question is asked
        //  again once the index lock is held, because the worktree can switch while the lock is awaited.
        if !still_checked_out(&git, repository, first.git_dir.as_path(), first.common_dir.as_path(), first.reference.as_str())? {
            return Ok(vec![Outcome::BranchMoved; requests.len()]);
        }
        // Only an amend uses the parents of the commit it replaces; they are read from the object store, and listed by Git when the
        //  store cannot say (the same rule as for the commit Git built).
        let amended_parents: Vec<String> = if alone && !unborn && first.amend {
            match read_commit(first.common_dir.as_path(), first.base.as_str()) {
                Some(replaced) => replaced.parents,
                None => {
                    let listing: String = checked(&git, repository, &["rev-list", "--parents", "--max-count=1", first.base.as_str()], None)?;
                    listing.split_whitespace().skip(1).map(str::to_string).collect()
                }
            }
        } else {
            Vec::new()
        };
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
            // A member of this batch that an earlier member overlaps would replace that member's bytes in the stream below. It
            //  stays in the batch only when it captured the worktree (a batch has one) later than that member, so its bytes
            //  hold the member's edit; any other request is sent back and merged against the member's landed commit.
            let shadowed: bool = landing.iter().any(|(_, earlier)| {
                let later_capture: bool = earlier.captured != 0 && request.captured > earlier.captured;
                return !later_capture && request.changes.iter().any(|change| { return earlier.changes.iter().any(|member| { return overlaps(change.path.as_str(), member.path.as_str()); }); });
            });
            if !conflicts && !shadowed {
                landing.push((index, request));
            }
        }
        if landing.is_empty() {
            return Ok(outcomes);
        }
        let batch: Vec<&CommitRequest> = landing.iter().map(|(_, request)| { return *request; }).collect();
        // Content that `git gc --prune=now` removed after the preparation is found before anything is imported or moved: the
        //  client prepares the commit again, and says so plainly when that is not enough.
        if batch_content_missing(first.common_dir.as_path(), batch.as_slice()) {
            return Err(FailureCode::Unavailable);
        }
        let objects_began: Instant = Instant::now();
        let (commits, new): (Vec<String>, String) = if let Some(built) = first.commit.as_deref() {
            // A commit Git built elsewhere lands as it is; it must be a commit with the parents this request's rule gives:
            //  an amend has the replaced commit's parents, a commit on a branch that did not exist has none, and any other
            //  commit has the branch tip first (a merge conclusion has the merged commits after it).
            // The parents are read from the object store; a commit it cannot read as a plain commit is listed by Git, whose
            //  answer is the one that counts (a repository with grafts, a shallow history or replacement objects).
            let read: Option<BuiltCommit> = read_commit(first.common_dir.as_path(), built);
            let (named_itself, parents): (bool, Vec<String>) = match &read {
                Some(commit) => (true, commit.parents.clone()),
                None => {
                    let listing: GitOutput = run(&git, repository, &["rev-list", "--parents", "--max-count=1", built], None)?;
                    let text: String = String::from_utf8_lossy(listing.stdout.as_slice()).into_owned();
                    let mut words = text.split_whitespace();
                    let named: bool = listing.succeeded() && words.next() == Some(built);
                    (named, words.map(str::to_string).collect())
                }
            };
            let acceptable: bool = if first.amend {
                parents == amended_parents
            } else if unborn {
                parents.is_empty()
            } else {
                parents.first() == Some(&first.base)
            };
            if !named_itself || !acceptable {
                return Err(FailureCode::Rejected);
            }
            // Git moves a branch to a commit without looking at what the commit holds, so a tree or a blob that a prune took
            //  while the commit object survived would put a damaged commit on the branch.
            reach("before-connectivity");
            // A commit whose one parent is the tip adds the objects that differ from the tip's tree, and the object store shows
            //  they are all there; any other commit, and any object that looks absent, is checked by the `rev-list` Git always ran.
            let proven: bool = read.as_ref().is_some_and(|commit| { return !unborn && adds_only_present_objects(first.common_dir.as_path(), commit, old.as_str()); });
            if !proven {
                let connected: Vec<&str> = if unborn { vec!["rev-list", "--objects", built] } else { vec!["rev-list", "--objects", built, "--not", old.as_str()] };
                if !run(&git, repository, connected.as_slice(), None)?.succeeded() {
                    return Err(FailureCode::Unavailable);
                }
            }
            (vec![built.to_string()], built.to_string())
        } else {
            let marks: PathBuf = context.journal.join(format!("marks-{seq}"));
            let marks_argument: String = format!("--export-marks={}", marks.display());
            // The `core.fsync` of the import is a global option of this one start; it reaches the `unpack-objects` child too, where
            //  it names the same loose-object flush the repository has.
            let fsync_setting: String = import_fsync.map(|value| { return format!("core.fsync={value}"); }).unwrap_or_default();
            let mut import_arguments: Vec<&str> = Vec::new();
            if import_fsync.is_some() {
                import_arguments.extend(["-c", fsync_setting.as_str()]);
            }
            import_arguments.extend(["fast-import", "--done", marks_argument.as_str()]);
            let imported: GitOutput = run(&git, repository, import_arguments.as_slice(), Some(import_stream(scratch.as_str(), old.as_str(), &batch, amended_parents.as_slice())))?;
            if !imported.succeeded() {
                return Err(classify(&imported));
            }
            // The import carries no `--quiet`, so its `--stats` report is kept in the service's log.
            crate::inner_git_words::record_words(&["fast-import", "--done"], &imported);
            let marks_text: String = std::fs::read_to_string(&marks).map_err(|_| { return FailureCode::Unavailable; })?;
            let _marks_removed = std::fs::remove_file(&marks);
            let made: Vec<String> = commits_from_marks(marks_text.as_str(), batch.len()).ok_or(FailureCode::Unavailable)?;
            let last: String = made.last().cloned().ok_or(FailureCode::Unavailable)?;
            (made, last)
        };
        timing.objects += objects_began.elapsed();
        reach("after-objects");
        if let Some(seam) = &context.on_objects_written {
            seam();
        }
        // A commit Git built carries no list of changes: its index entries are the paths that differ from the old branch.
        let derived: bool = first.commit.is_some();
        let paths: Vec<String> = if derived {
            Vec::new()
        } else {
            batch.iter().flat_map(|request| { return request.changes.iter().map(|change| { return change.path.clone(); }); }).collect()
        };
        let intent: Intent = Intent {
            seq,
            repository: first.repository.clone(),
            common_dir: first.common_dir.clone(),
            reference: first.reference.clone(),
            old: old.clone(),
            new: new.clone(),
            ids: batch.iter().map(|request| { return request.id.clone(); }).collect(),
            commits: commits.clone(),
            paths,
            derived,
            cleanup: first.cleanup.clone(),
            index: first.index.clone(),
            git_dir: first.git_dir.clone(),
        };
        let journal_began: Instant = Instant::now();
        let written = write_intent(context.journal.as_path(), &intent);
        timing.journal += journal_began.elapsed();
        written.map_err(|error| { return storage_failure(&error); })?;
        reach("after-intent");
        // Git's own rule: the index lock is held before the branch is moved and given up when the new index is in place, so
        //  a foreign `git` that holds it finishes first and this landing builds on what it left. Everything that can be done
        //  without the index is done before the lock is asked for (the reads, the objects, the intent), so the lock is held
        //  for the branch move and the index build only, and a foreign holder is waited for while the work above is done,
        //  not before it. A lock that is not given up here (a refusal, a failure) is removed when it goes out of scope, and
        //  the index stays as it was.
        let waiting_since: Instant = Instant::now();
        let mut notes: WaitNotes = WaitNotes::default();
        let taken = acquire_noting_wait_within(first.index.as_path(), seq, context.lock_ceiling, announce, &mut notes);
        let waited: std::time::Duration = waiting_since.elapsed();
        timing.lock_wait += waited;
        // A pass over the batch that found the lock free has nothing to add to what an earlier pass found held.
        if notes.looks > 0 || timing.attempts == 1 {
            timing.lock_holder.clone_from(&notes.holder);
            timing.lock_notes = notes.clone();
        }
        // The wait is written down when it ends, not with the answer, so a landing that fails later still leaves it.
        if let Some(line) = wait_line(seq, waited, &notes, taken.is_ok()) {
            // A lock that was refused fails the commit, so the line stands out from a wait that ended in the lock.
            let priority: Priority = if taken.is_ok() { Priority::Info } else { Priority::Error };
            crate::service_timing::log_entry(priority, line.as_str(), &[("CLI_GIT_BATCH", seq.to_string())]);
        }
        let held: IndexLock = match taken {
            Ok(lock) => lock,
            Err(failure) => {
                // Nothing moved: forget the intent, so a restart of the service cannot land the batch later (owner, 2026-11-03: "No,
                //  keep it for a retry"), keep the built commit on a side ref the message names, and forget the scratch branch.
                let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
                if failure.code == FailureCode::IndexInUse {
                    announce(Announce::Waiting(keep_unlanded(&git, repository, seq, new.as_str())));
                }
                let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
                return Err(failure.code);
            }
        };
        let hold_began: Instant = Instant::now();
        // The worktree may have switched while the lock was awaited: the checked-out branch is read again with the lock held and
        //  before the ref moves, and a landing that no longer belongs to this worktree's head stops with nothing changed.
        let still: Result<bool, FailureCode> = still_checked_out(&git, repository, first.git_dir.as_path(), first.common_dir.as_path(), first.reference.as_str());
        if !matches!(still, Ok(true)) {
            let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
            let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
            return match still {
                Ok(_) => Ok(vec![Outcome::BranchMoved; requests.len()]),
                Err(code) => Err(code),
            };
        }
        // The index file now, under the lock, is what a replay of this batch must still find before it resets a path: a replay is
        //  made only over the file the landing saw. It is recorded for a commit Git built as for any other, since the witness
        //  needs no list of paths. A record that cannot be written leaves a batch no replay is made for, and this landing
        //  still updates the index itself while it holds the lock.
        match witness_of(first.index.as_path()) {
            Ok(witness) => {
                if let Err(error) = write_start(context.journal.as_path(), seq, &witness) {
                    crate::service_error!("batch {seq}: the index file the landing saw could not be recorded ({error}); a restart will not replay the index update");
                }
            }
            Err(error) => crate::service_error!("batch {seq}: the index file cannot be looked at ({error}); a restart will not replay the index update"),
        }
        reach("before-ref");
        // A commit built elsewhere made no scratch branch, so there is none to remove with the move.
        let removal: String = if first.commit.is_some() { String::new() } else { format!("delete {scratch}\n") };
        // A detached head is the `HEAD` file itself and is updated as such. The update follows a symbolic `HEAD` as native Git's
        //  own commit does, because a compare that does not follow it replaces a head attached to a branch since the last look
        //  (measured: `option no-deref` with `update HEAD <new> <old>` replaced `ref: refs/heads/B` at the old name with the
        //  detached new one). A head that became a branch at another value fails the compare; at the same value the branch it
        //  names takes the commit, and the head is read again below.
        let transaction: String = format!("start\nupdate {} {new} {old}\n{removal}prepare\ncommit\n", first.reference);
        let ref_began: Instant = Instant::now();
        let ran = run(&git, repository, &["update-ref", "--stdin"], Some(transaction.into_bytes()));
        timing.ref_update += ref_began.elapsed();
        let moved: GitOutput = match ran {
            Ok(output) => output,
            Err(FailureCode::StepTooSlow) => {
                // The update was ended at its ceiling. Git writes a reference by renaming its lock file, so the branch is now at
                //  the request's commit or where it was: the branch is read, and the answer follows what it says.
                let step: String = take_ceiling_step().unwrap_or_else(|| { return String::from("the reference update ran too long and was ended"); });
                let tip: Option<String> = checked(&git, repository, &["rev-parse", "--verify", "--quiet", first.reference.as_str()], None).ok();
                if tip.as_deref() == Some(new.as_str()) {
                    // It renamed before it was ended: the commit is on the branch. The scratch branch the transaction would have
                    //  removed is removed here, and the landing goes on as if the update had returned.
                    let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
                    GitOutput { code: Some(0), stdout: Vec::new(), stderr: Vec::new() }
                } else {
                    // It did nothing: the batch is withdrawn like one that met the lock ceiling. The intent goes, so a restart cannot
                    //  land it; the commit is kept on a side ref; the lock is given up when `held` goes out of scope.
                    let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
                    remove_start(context.journal.as_path(), seq);
                    announce(Announce::Waiting(format!("cli-git: {step}, so this commit was not made.\n")));
                    announce(Announce::Waiting(keep_unlanded(&git, repository, seq, new.as_str())));
                    let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
                    return Err(FailureCode::StepTooSlow);
                }
            }
            Err(code) => return Err(code),
        };
        if !moved.succeeded() {
            // The branch moved under the batch: forget the intent and the scratch branch, and build the batch again.
            let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
            remove_start(context.journal.as_path(), seq);
            let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
            // For an amend, the next round finds the tip away from its base and answers that the branch moved.
            attempts_left -= 1;
            timing.hold += hold_began.elapsed();
            if attempts_left == 0 {
                return Err(FailureCode::BranchKeepsChanging);
            }
            continue;
        }
        reach("after-ref");
        // Git refuses a `symref-verify HEAD` beside an update of the branch `HEAD` names, so a switch after the last check cannot
        //  be caught inside the transaction. The head is read again with the lock held: a worktree that left the branch keeps
        //  its own index, and the commits stay on the branch they were made for. A switch after this look is the state native
        //  Git leaves too (`git commit` then `git symbolic-ref HEAD <other>`): the index describes the commit made.
        let followed: Result<bool, FailureCode> = follows_move(&git, repository, first.git_dir.as_path(), first.common_dir.as_path(), first.reference.as_str(), new.as_str());
        // The branch has moved: the commits exist whatever happens to the index, which recovery finishes.
        let index_update_began: Instant = Instant::now();
        let settled: bool = finish_after_move(&git, &intent, held, context.journal.as_path(), followed);
        timing.index_update += index_update_began.elapsed();
        timing.hold += hold_began.elapsed();
        if settled {
            reach("after-index");
            let _marked = crate::service_journal::mark_done(context.journal.as_path(), seq);
        }
        for (slot, (index, _request)) in landing.iter().enumerate() {
            outcomes[*index] = Outcome::Landed { commit: commits[slot].clone() };
        }
        return Ok(outcomes);
    }
}

/// Landing controls stay out of the release executable.
#[cfg(test)]
#[path = "service_land_tests.rs"]
mod tests;

/// Controls for a replay over the index the landing saw stay out of the release executable.
#[cfg(test)]
#[path = "service_land_witness_tests.rs"]
mod witness_tests;

/// Controls for the worktree's head around a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_head_tests.rs"]
mod head_tests;

/// Controls for the flushes of the Git children of a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_fsync_tests.rs"]
mod fsync_tests;

/// Controls for the checks of a commit Git built stay out of the release executable.
#[cfg(test)]
#[path = "service_land_built_tests.rs"]
mod built_tests;
