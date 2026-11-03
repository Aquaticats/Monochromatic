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

use super::journald_native::Priority;
use super::service_fault::reach;
use super::service_head::still_checked_out;
use super::service_instance::instance_of;
use super::service_pruned::batch_content_missing;
use super::service_journal::{INDEXED_EXTENSION, Intent, Recovery, OPEN_EXTENSION, is_indexed, list_intents, mark_done, mark_indexed, recovery_for, write_intent};
use super::service_index_lock::{Announce, IndexLock, WaitNotes, acquire_noting_wait, remove_stale, try_acquire};
use super::service_timing::{LandTiming, wait_line};
use super::autostash_carry::carry_stages;
use super::service_land_follow::{IndexPaths, build_index, classify, remove_conclusion_state, storage_failure};
use super::service_protocol::{CommitRequest, FailureCode, is_zero_object};
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
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
    let output: GitOutput = run_git(git, &request).map_err(|_| { return FailureCode::Unavailable; })?;
    return Ok(output);
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

/// The index entries a batch's intent names.
fn intent_paths(intent: &Intent) -> IndexPaths<'_> {
    return if intent.derived { IndexPaths::Derived { old: intent.old.as_str() } } else { IndexPaths::Listed(intent.paths.as_slice()) };
}

/// Removes the conclusion state a landed batch leaves, when it has one. Whether it is gone.
fn cleanup_done(git: &GitContext, repository: &Path, intent: &Intent) -> bool {
    return intent.cleanup.as_ref().is_none_or(|cleanup| { return remove_conclusion_state(git, repository, cleanup); });
}

/// Finishes what follows the branch move: the conclusion state is removed, then the index entries are updated. Both are
///  idempotent, and they are recorded apart: the installed index is recorded in the journal while the lock is still held
///  (`<batch>.indexed`), so a restart never replays the index update over what another program staged after the lock was given
///  up, whether or not the conclusion state could be removed. Whether both are done.
fn follow_up(git: &GitContext, repository: &Path, intent: &Intent, lock: IndexLock, journal: &Path) -> bool {
    let cleaned: bool = cleanup_done(git, repository, intent);
    // The new index is built beside the real one and installed by the release of the lock the landing holds.
    let Ok(built) = build_index(git, repository, intent.new.as_str(), intent_paths(intent), &lock, intent.seq) else {
        return false;
    };
    // The files a conflicted autostash apply left unmerged are unmerged in the index the landing installs.
    if let Some(cleanup) = intent.cleanup.as_ref()
        && !carry_stages(git, repository, cleanup, built.as_path())
    {
        let _removed = std::fs::remove_file(&built);
        return false;
    }
    let recorded = || -> std::io::Result<()> { return mark_indexed(journal, intent.seq, intent.new.as_str()); };
    return lock.release_with_record(built.as_path(), &recorded).is_ok() && cleaned;
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
        let old: String = match checked(&git, repository, &["rev-parse", "--verify", "--quiet", first.reference.as_str()], None) {
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
        if !still_checked_out(&git, repository, first.index.as_path(), first.common_dir.as_path(), first.reference.as_str())? {
            return Ok(vec![Outcome::BranchMoved; requests.len()]);
        }
        let amended_parents: Vec<String> = if alone && !unborn {
            let listing: String = checked(&git, repository, &["rev-list", "--parents", "--max-count=1", first.base.as_str()], None)?;
            listing.split_whitespace().skip(1).map(str::to_string).collect()
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
            let listing: GitOutput = run(&git, repository, &["rev-list", "--parents", "--max-count=1", built], None)?;
            let text: String = String::from_utf8_lossy(listing.stdout.as_slice()).into_owned();
            let mut words = text.split_whitespace();
            let named_itself: bool = listing.succeeded() && words.next() == Some(built);
            let parents: Vec<String> = words.map(str::to_string).collect();
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
            let connected: Vec<&str> = if unborn { vec!["rev-list", "--objects", built] } else { vec!["rev-list", "--objects", built, "--not", old.as_str()] };
            if !run(&git, repository, connected.as_slice(), None)?.succeeded() {
                return Err(FailureCode::Unavailable);
            }
            (vec![built.to_string()], built.to_string())
        } else {
            let marks: PathBuf = context.journal.join(format!("marks-{seq}"));
            let marks_argument: String = format!("--export-marks={}", marks.display());
            let imported: GitOutput = run(&git, repository, &["fast-import", "--done", marks_argument.as_str()], Some(import_stream(scratch.as_str(), old.as_str(), &batch, amended_parents.as_slice())))?;
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
        let taken = acquire_noting_wait(first.index.as_path(), seq, announce, &mut notes);
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
                // Nothing moved: forget the intent and the scratch branch, as for a branch that moved under the batch.
                let _intent_removed = std::fs::remove_file(super::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
                let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
                return Err(failure.code);
            }
        };
        let hold_began: Instant = Instant::now();
        // The worktree may have switched while the lock was awaited: the checked-out branch is read again with the lock held and
        //  before the ref moves, and a landing that no longer belongs to this worktree's head stops with nothing changed.
        let still: Result<bool, FailureCode> = still_checked_out(&git, repository, first.index.as_path(), first.common_dir.as_path(), first.reference.as_str());
        if !matches!(still, Ok(true)) {
            let _intent_removed = std::fs::remove_file(super::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
            let _deleted = run(&git, repository, &["update-ref", "-d", scratch.as_str()], None);
            return match still {
                Ok(_) => Ok(vec![Outcome::BranchMoved; requests.len()]),
                Err(code) => Err(code),
            };
        }
        // A commit built elsewhere made no scratch branch, so there is none to remove with the move.
        let removal: String = if first.commit.is_some() { String::new() } else { format!("delete {scratch}\n") };
        // A detached head is the `HEAD` file itself: it is updated without following a symbolic reference, so a head that
        //  became a branch since the commit was prepared fails the compare instead of moving that branch.
        let direct: &str = if first.reference == "HEAD" { "option no-deref\n" } else { "" };
        let transaction: String = format!("start\n{direct}update {} {new} {old}\n{removal}prepare\ncommit\n", first.reference);
        let ref_began: Instant = Instant::now();
        let ran = run(&git, repository, &["update-ref", "--stdin"], Some(transaction.into_bytes()));
        timing.ref_update += ref_began.elapsed();
        let moved: GitOutput = ran?;
        if !moved.succeeded() {
            // The branch moved under the batch: forget the intent and the scratch branch, and build the batch again.
            let _intent_removed = std::fs::remove_file(super::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
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
        // The branch has moved: the commits exist whatever happens to the index, which recovery finishes.
        let sync_began: Instant = Instant::now();
        let synced: bool = follow_up(&git, repository, &intent, held, context.journal.as_path());
        timing.sync += sync_began.elapsed();
        timing.hold += hold_began.elapsed();
        if synced {
            reach("after-index");
            let _marked = mark_done(context.journal.as_path(), seq);
        }
        for (slot, (index, _request)) in landing.iter().enumerate() {
            outcomes[*index] = Outcome::Landed { commit: commits[slot].clone() };
        }
        return Ok(outcomes);
    }
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
///   Runs before the service accepts a connection, and again before each batch (`service_server.rs`), so an index update
///  that gave up while another program held Git's index lock is finished by the next commit and not only by a restart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recover(context)
/// ```
pub fn recover(context: &LandContext) -> Vec<RecoveryReport> {
    return recover_selected(context, &|_| { return true; }, true);
}

/// What:
///  Finishes or drops the open batches of the worktree one instance serves, found in a journal that other worktrees' batches
///  may share.
/// Why:
///   The single service of the releases before one service per worktree kept one journal for every worktree, in the directory
///  above each instance's own, and an instance takes over the batches of its worktree there (found by the Git directory the
///  intent records, which an index outside it, as `GIT_INDEX_FILE` names one, cannot change; an intent that records none is
///  resolved from its repository). A development run may also give several instances one journal. Whatever belongs to
///  another worktree is left to its instance, and a record that cannot be read is not any instance's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInstance(context, instance)
/// ```
pub fn recover_instance(context: &LandContext, instance: &str) -> Vec<RecoveryReport> {
    let ours = |intent: &Intent| -> bool { return instance_of(intent.git_dir.as_path()) == instance; };
    return recover_selected(context, &ours, false);
}

fn recover_selected(context: &LandContext, selected: &dyn Fn(&Intent) -> bool, remove_unreadable: bool) -> Vec<RecoveryReport> {
    let mut reports: Vec<RecoveryReport> = Vec::new();
    for (path, parsed) in list_intents(context.journal.as_path(), OPEN_EXTENSION) {
        let Some(intent) = parsed else {
            if remove_unreadable {
                let _removed = std::fs::remove_file(&path);
            }
            continue;
        };
        if !selected(&intent) {
            continue;
        }
        let git: GitContext = GitContext { common_dir: Some(intent.common_dir.clone()), ..context.git.clone() };
        let current: String = checked(&git, intent.repository.as_path(), &["rev-parse", "--verify", intent.reference.as_str()], None).unwrap_or_default();
        let ancestor: bool = run(&git, intent.repository.as_path(), &["merge-base", "--is-ancestor", intent.new.as_str(), current.as_str()], None)
            .is_ok_and(|output| { return output.succeeded(); });
        let decision: Recovery = recovery_for(&intent, current.as_str(), ancestor);
        match decision {
            Recovery::Drop => {
                remove_stale(intent.index.as_path(), intent.seq);
                let _removed = std::fs::remove_file(&path);
                let _record_removed = std::fs::remove_file(super::service_journal::batch_path(context.journal.as_path(), intent.seq, INDEXED_EXTENSION));
            }
            Recovery::Finish => {
                remove_stale(intent.index.as_path(), intent.seq);
                // A worktree that has since switched away from the batch's branch has an index that describes another branch: the
                //  branch has its commit, so there is no index update to finish and the batch is done.
                if still_checked_out(&git, intent.repository.as_path(), intent.index.as_path(), intent.common_dir.as_path(), intent.reference.as_str()) == Ok(false) {
                    let _marked = mark_done(context.journal.as_path(), intent.seq);
                } else if is_indexed(context.journal.as_path(), intent.seq, intent.new.as_str()) {
                    // The installed index is recorded: it is not replayed, and only the conclusion state may be left.
                    if cleanup_done(&git, intent.repository.as_path(), &intent) {
                        let _marked = mark_done(context.journal.as_path(), intent.seq);
                    }
                } else if let Some(held) = try_acquire(intent.index.as_path(), intent.seq)
                    && follow_up(&git, intent.repository.as_path(), &intent, held, context.journal.as_path())
                {
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
