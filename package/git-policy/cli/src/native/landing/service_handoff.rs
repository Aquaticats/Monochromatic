//! What:
//!  The wrapper's side of after-commit work:
//! leave the push and upkeep records for a commit the service landed and wake
//!  the service.
//! Why:
//!  The commit returns as soon as it is saved.
//! The push (which can take as long as the network does) and the upkeep run
//!  in the service afterwards;
//! a push that fails is reported by the person's next wrapper command (`push_failure_notice.rs`)
//!  (`doc/planning/cli-git-resident-service.md`,
//! "After-commit work").
//! The wrapper starts no program and waits for no
//!  one:
//! it writes whole files and sends one message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueue(workDir, push); await enqueue(workDir, maintenance); await wake(commonDir);
//! ```

use crate::blocked_commit_lines::named_reason;
use crate::held_push_commit_on_the_remote::{OnTheRemote, RemoteRecord};
use crate::held_push_holder_and_verdict::{Holding, Verdict, holding_in, pid_of, proven_gone};
use crate::held_push_release_wakes_the_service::WakeTheService;
use crate::iso_time::now_iso;
use crate::service_client::{ClientConfig, wake};
use crate::service_messages::ClientFailure;
use crate::service_work::work_directory;
use crate::service_review_key::{ReviewKey, key_from_environment};
use crate::standards_review_input::has_shadow_log;
use crate::work_item::{WorkItem, WorkKind, enqueue, enqueue_all, state_directory};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What the wrapper knows about a commit the service landed.
#[derive(Clone, Debug)]
pub struct ServiceHandoff {
    /// How to reach the service.
    pub config: ClientConfig,
    /// The worktree root.
    pub repository: PathBuf,
    /// The worktree's own Git directory,
    /// which holds the work records and names the service instance.
    pub git_dir: PathBuf,
    /// The branch the commit went to.
    pub reference: String,
    /// The branch value the commit was made on.
    pub parent: String,
}

/// The kinds of work a landed commit leaves for the service.
pub const HANDED_OVER: [WorkKind; 1] = [WorkKind::Maintenance];

/// What:
///  The value of an environment variable,
/// nothing when it is unset or not text.
/// Why:
///  `hand_over` reads the review key from the process environment through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const environmentVariable = (name: string) => process.env[name];
/// ```
fn environment_variable(name: &str) -> Option<String> {
    return std::env::var(name).ok();
}

/// What:
///  Writes the work records for a landed commit and wakes the service.
/// Why:
///   The records are written before the message is sent,
/// so a service that wakes always finds them;
/// records that were
///  written but not announced stay pending and are run by the next wake for the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOver(handoff, landed)
/// ```
pub fn hand_over(handoff: &ServiceHandoff, landed: &str, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let key: Option<ReviewKey> = key_from_environment(&environment_variable);
    return hand_over_with_key(handoff, landed, key.as_ref(), notices);
}

/// What:
///  `hand_over` with the standards-review credential given.
/// The records written to disk never hold it;
/// the wake message
///  carries it to the service,
/// which keeps it in memory for the review.
/// Why:
///   A systemd user service does not inherit the wrapper's environment,
/// and the credential must not be stored in a work
///  item,
/// a unit file or the manager's environment.
/// The wrapper therefore sends it over the user-only socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverWithKey(handoff, landed, key)
/// ```
pub fn hand_over_with_key(handoff: &ServiceHandoff, landed: &str, review_key: Option<&ReviewKey>, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let work_dir: PathBuf = work_directory(handoff.git_dir.as_path());
    // A review is left only where the repository keeps a shadow log; the file is the repository's own switch.
    let review: &[WorkKind] = if has_shadow_log(handoff.repository.as_path()) { &[WorkKind::Review] } else { &[] };
    // One round for the commit's records: each state directory the queue asks about is listed once, however many kinds the
    //  commit leaves.
    let mut items: Vec<WorkItem> = Vec::new();
    for kind in HANDED_OVER.iter().chain(review.iter()).copied() {
        items.push(WorkItem {
            kind,
            repository: handoff.repository.clone(),
            git_dir: handoff.git_dir.clone(),
            branch_ref: handoff.reference.clone(),
            oid: landed.to_string(),
            parent: handoff.parent.clone(),
            created: now_iso(),
            tries: 0,
        });
    }
    enqueue_all(work_dir.as_path(), items.as_slice()).map_err(ClientFailure::WorkNotHandedOver)?;
    return wake(&handoff.config, handoff.git_dir.as_path(), review_key, notices).map_err(not_handed_over);
}

/// What:
///  Promotes the push record the service held for a commit it landed,
/// once the post-commit gate approves the backup.
/// Why:
///   The record must outlive a wrapper killed between the answer and the gate,
/// and it must not run before the gate:
///  the service writes no record of its own,
/// so nothing pushes a commit whose gate blocked it,
/// and a record the gate
///  then allows is the one the hand-off wakes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await leaveThePush(handoff, landed)
/// ```
///
/// # Errors
///
/// The work queue's own words when the record cannot be written.
pub fn promote_the_push(handoff: &ServiceHandoff, landed: &str) -> Result<(), ClientFailure> {
    let item: WorkItem = push_item(handoff, landed);
    let work_dir: PathBuf = work_directory(handoff.git_dir.as_path());
    // A record an earlier release held is named by the kind and the commit alone, and it is this push's.
    for name in [item.id(), item.id_before_the_branch()] {
        // The pass is written into the record before it runs: a command killed in this gap leaves a record whose own words say the
        //  checks passed, and a later command releases it from them (`held_push_holder_and_verdict.rs`). The record then names this
        //  command, which is the retry's own when a retry is what finishes the commit.
        if crate::work_item::record_pass(work_dir.as_path(), name.as_str()) {
            crate::service_fault::reach(crate::service_fault::AFTER_THE_VERDICT);
        }
        if crate::work_item::promote(work_dir.as_path(), name.as_str()) {
            return Ok(());
        }
    }
    // Nothing was held for this commit: the record is written here instead, so the push is not lost.
    return match enqueue(work_dir.as_path(), &item) {
        Ok(_enqueued) => Ok(()),
        Err(reason) => Err(ClientFailure::WorkNotHandedOver(reason)),
    };
}

/// What:
///  Takes the push record back when the post-commit gate blocks the backup,
/// and keeps what the gate said.
/// Why:
///   A blocked gate keeps the commit local,
/// and a record left behind would push it at the next wake.
/// A record another round already
///  claimed is not reached here:
/// the gate is what says whether the push may run.
///   The verdict keeps the gate's words,
/// because the queue pushes the branch and not the commit:
/// the next push of
///  that branch must stop before this commit and say why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetThePush(handoff, landed, reason) // unlink the pending record, keep the verdict; words when the verdict could not be kept
/// ```
/// The words a command says when the verdict could not be kept,
/// or nothing when it was.
pub fn forget_the_push(handoff: &ServiceHandoff, landed: &str, reason: &str) -> Option<String> {
    let work_dir: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let item: WorkItem = push_item(handoff, landed);
    // The refusal is written into the record before it is settled: a command killed in this gap leaves a record whose own words say
    //  the gate refused this commit, and a later command settles it into the same verdict instead of leaving it held
    //  (`held_push_holder_and_verdict.rs`).
    for name in [item.id(), item.id_before_the_branch()] {
        if crate::work_item::record_refusal(work_dir.as_path(), name.as_str(), reason) {
            crate::service_fault::reach(crate::service_fault::AFTER_THE_VERDICT);
        }
    }
    // The verdict is durable: a later recovery of the same batch must not hold the push again, and the next push of the branch
    //  reads the same verdict to know where it must stop. A verdict that could not be written says so, and the held records then
    //  stay: a record with a verdict keeps the commit local, and a record without one does too (`work_item_holding.rs`).
    let durable: bool = crate::work_item::block(work_dir.as_path(), &item, reason);
    // A record an earlier release named by the kind and the commit alone is this push's too, so the gate forgets it as well:
    //  left in `pending`, the next wake would push a commit the gate kept local, and left in `held`, every command would name a push
    //  nothing may make. The held record is kept while the verdict could not be, because it is then the only copy of the refusal.
    for id in [item.id(), item.id_before_the_branch()] {
        let _removed: std::io::Result<()> = std::fs::remove_file(state_directory(work_dir.as_path(), crate::work_item::PENDING).join(id.as_str()));
        if durable {
            crate::work_item::forget_held(work_dir.as_path(), id.as_str());
        }
    }
    if durable {
        return None;
    }
    return Some(format!(
        "the commit {landed} stays local, because its post-commit checks refused it ({reason}), but the verdict that stops this branch's automatic push could not be written to {}. The held record keeps the refusal, so no push this repository makes sends the commit, and every later command tries to keep that verdict again; a push you make of the branch still sends it.",
        work_directory(handoff.git_dir.as_path()).join(crate::work_item_holding::BLOCKED).display()
    ));
}

/// The push record of one landed commit of a worktree.
fn push_item(handoff: &ServiceHandoff, landed: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: handoff.repository.clone(),
        git_dir: handoff.git_dir.clone(),
        branch_ref: handoff.reference.clone(),
        oid: landed.to_string(),
        parent: handoff.parent.clone(),
        created: now_iso(),
        tries: 0,
    };
}

/// What:
///  Writes the push record for a branch tip that a command other than a commit moved,
/// before anything asks whether a service can be reached.
/// Why:
///   A merge,
/// a cherry-pick,
/// a revert,
/// an applied patch or a pull's merge adds commits that the next commit would otherwise
///  be the first to push.
/// Only the push is left:
/// the review and the upkeep belong to a commit.
///   The record is a directory of this worktree's own Git directory,
/// so writing it needs no service,
/// no socket and no session bus,
/// and a context with no route to the service still leaves the push on disk for the next wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePushRecord({ repository, gitDir, reference, tip, parent })
/// ```
pub fn write_push_record(repository: &std::path::Path, git_dir: &std::path::Path, reference: &str, tip: &str, parent: &str) -> Result<(), ClientFailure> {
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repository.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: reference.to_string(),
        oid: tip.to_string(),
        parent: parent.to_string(),
        created: now_iso(),
        tries: 0,
    };
    enqueue(work_directory(git_dir).as_path(), &item).map_err(ClientFailure::WorkNotHandedOver)?;
    return Ok(());
}

/// What:
///  `write_push_record` for a handoff,
/// and then the wake of the service.
/// Why:
///   The caller that already knows how to reach the service leaves both in one call;
/// a caller that may find no route writes the record with `write_push_record` and wakes only when it has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverPush(handoff, tip)
/// ```
pub fn hand_over_push(handoff: &ServiceHandoff, tip: &str, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    write_push_record(handoff.repository.as_path(), handoff.git_dir.as_path(), handoff.reference.as_str(), tip, handoff.parent.as_str())?;
    return wake(&handoff.config, handoff.git_dir.as_path(), None, notices).map_err(not_handed_over);
}

/// What:
///  The failure for work that could not be handed to the commit service after the commit landed.
/// Why:
///   The commit landed,
/// so the words of the wake's own failure are false when they reach the person:
/// `NoCommonVersion` says
///  "this commit was not made ... Nothing in the repository changed",
/// and the caller renders whatever the hand-off returned
///  (`commit_command.rs`).
/// The audit of service restarts and version skew (2026-11-09) printed exactly that after a commit that
///  was on the branch:
/// the service was killed after the branch moved,
/// the unit named another release,
/// and the hand-off met it.
/// The words that are true are the ones for work that was not handed over,
/// and they name the reason and the remedy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handed = (failure) => ({ kind: 'work-not-handed-over', reason: reasonOf(failure) });
/// ```
fn not_handed_over(failure: ClientFailure) -> ClientFailure {
    let reason: &str = match &failure {
        ClientFailure::NoActivation(_) => "this account cannot start the commit service on demand",
        ClientFailure::ServiceSilent => "the commit service stopped answering",
        ClientFailure::DidNotStart => "the commit service did not start",
        ClientFailure::NoCommonVersion => "the commit service that is running was started from a newer release of this git wrapper",
        ClientFailure::OtherRelease => "a commit service started from another release of this git wrapper is still running",
        ClientFailure::ProgramMissing(_) | ClientFailure::CouldNotStart(_) | ClientFailure::UnitInTheWay(_) | ClientFailure::Unavailable => "the commit service could not be reached",
        _ => "the commit service could not be told about it",
    };
    return ClientFailure::WorkNotHandedOver(String::from(reason));
}

/// Handoff controls stay out of the release executable.
#[cfg(test)]
#[path = "service_handoff_tests.rs"]
mod tests;

/// What:
///  The one sentence the worktree says about a held push,
/// and what still holds it.
/// Why:
///   What is known is that the record is held,
/// that the commit's checks have not cleared it,
/// and what a
///  later look found about the command that would have decided them;
/// the sentence says exactly that and names what sends the
///  commit,
/// so a hold is never silent and never waits for something nobody will do.
///  The sentence never claims a commit stays local when nothing stops a push:
/// a record with no verdict holds no branch back (`push_held_back_by_a_blocked_commit.rs`),
/// so the next automatic push of the branch carries the commit,
/// and a record that names no command or whose command ended can only say that and what a person's own push does.
/// A ref that lags the remote cannot make this line false,
/// because the line claims no publication state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldPushLine(branchRef, commit, holding, gone)
/// ```
pub fn held_push_line(reference: &str, commit: &str, holding: &Holding, gone: bool) -> String {
    // A refusal the record holds and no verdict answers is the one state in which this worktree has the gate's words and no reader of
    //  them beside the record: the commit must never be published here, and the line says why nothing has been and what happens next.
    //  It says nothing about the command's liveness, because the refusal is the same whether that command ended or not.
    if holding.verdict == Verdict::Blocked {
        return a_refusal_that_could_not_be_kept(reference, commit, holding.reason.as_str());
    }
    // The command still runs: the wait is its own, the automatic push stops before the commit, and it ends with that command's answer.
    if !holding.holder.is_empty() && !gone {
        let waited: String = format!(
            "the commit {commit} is held: the command that landed it (process {}) has not finished its checks, and the push goes out when it does",
            pid_text(holding.holder.as_str())
        );
        return match reference.is_empty() {
            true => format!("{waited}, and a push you make yourself sends it"),
            false => format!("{waited}, and a push you make of {reference} sends it"),
        };
    }
    // A record that names no command, and one whose command ended, holds nothing back: no push stops before the commit, so the next
    //  push of the branch carries it, and the line says that instead of claiming the commit stays local.
    let carried: String = match reference.is_empty() {
        true => String::from(
            "the next automatic push of this repository carries the commit if it is not published yet, and a push you make yourself sends it the same way",
        ),
        false => format!(
            "the next automatic push of {reference} carries the commit if it is not published yet, and a push you make of {reference} sends it the same way"
        ),
    };
    return match holding.holder.is_empty() {
        // A record that names no command is one of an earlier release, or one whose hold was written without an answer.
        true => format!("the commit {commit} is held: no command has said whether its checks passed, so this record pushes nothing; {carried}"),
        // The command that would have decided is proven gone, so nothing will ever write a verdict for this commit.
        false => format!(
            "the commit {commit} is held: the command that landed it (process {}) ended before its checks ran, so this record pushes nothing; {carried}",
            pid_text(holding.holder.as_str())
        ),
    };
}

/// What:
///  The one sentence the worktree says about a commit whose refusal the queue could not keep as a verdict beside it,
/// and what still holds it.
/// Why:
///   The refusal is in the held record,
/// and the `blocked` verdict every push reads is not,
///  so the record stays where it is and the line says what holds the commit and what still sends it.
/// A push the person makes of their own branch is forwarded unchanged,
/// which is the one thing that does send such a commit
///  (`work_item_holding.rs`,
/// `block`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aRefusalThatCouldNotBeKept(branchRef, commit, reason)
/// ```
pub fn a_refusal_that_could_not_be_kept(reference: &str, commit: &str, reason: &str) -> String {
    let by_the_person: &str = if reference.is_empty() { "a push you make yourself still sends it" } else { "a push you make of it still sends it" };
    return format!(
        "the commit {commit} is held: the checks refused it ({}), and the verdict that stops this branch's push could not be written, so nothing this \
         repository pushes sends it, and {by_the_person}",
        named_reason(reason)
    );
}

/// How a sentence names the command a holder text names,
/// or says that its number could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pidText = (holder) => String(pidOf(holder) ?? 'of unknown number');
/// ```
fn pid_text(holder: &str) -> String {
    return match pid_of(holder) {
        Some(pid) => pid.to_string(),
        None => String::from("of unknown number"),
    };
}

/// What:
///  The one sentence the worktree says about a push that is queued and has not been made.
/// Why:
///   What is known is that the record is queued and that no service has taken it,
/// and that the service makes it
///  without another commit;
/// the sentence claims exactly that and nothing about why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queuedPushLine(branchRef, commit)
/// ```
pub fn queued_push_line(reference: &str, commit: &str) -> String {
    if reference.is_empty() {
        return format!("the commit {commit} is queued for its push: the push has not been made yet, and the commit service makes it the next time it looks at this repository, which needs no further commit");
    }
    return format!(
        "the commit {commit} is queued for its push from {reference}: the push has not been made yet, and the commit service makes it the next time it looks at this repository, which needs no further commit"
    );
}

/// What:
///  The lines a command says about the pushes this worktree holds,
/// read from the held records themselves.
/// Why:
///   The record is the only place the state exists,
/// so the line cannot disagree with it:
/// a kill between
///  writing the record and writing a second file cannot lose the line,
/// a release cannot leave a stale
///  one,
/// and two commands cannot interleave into either.
/// A release,
/// a block or a claim removes or moves the record,
///  and the line goes with it.
///   Two proofs settle a record here rather than leaving it named.
/// Git's own record of the branch's remote
///  (`record`) settles a commit a later push of the branch carried,
/// so the line and `git status` agree;
/// a question
///  that answers nothing leaves the line standing.
/// And a record whose own words say what the command that landed the commit
///  decided settles from them once that command is proven gone:
/// a pass becomes a push the service is woken for,
/// a refusal becomes the
///  verdict every later push of the branch stops before,
/// and a record with no verdict stays held with the line saying what holds it
///  (`held_push_holder_and_verdict.rs`).
/// While the command still runs,
/// the wait is its own,
/// and this look leaves the record to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldPushNotices(gitDir, record, wake) // one line per held record Git does not already hold, and a settlement for each whose command is gone
/// ```
pub fn held_push_notices(git_dir: &Path, record: &dyn RemoteRecord, wake: &dyn WakeTheService) -> Vec<String> {
    let work_dir: PathBuf = work_directory(git_dir);
    let held: PathBuf = state_directory(work_dir.as_path(), crate::work_item::HELD);
    let Ok(listing) = std::fs::read_dir(held.as_path()) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = Vec::new();
    // One wake for every record this command released, sent once at the end: a released push runs the next time the service looks,
    //  and a command that says the push is queued wakes it rather than waiting for the next commit.
    let mut released: bool = false;
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(commit) = crate::work_item_ids_and_names::object_of_a_record_name(name.as_str()) else { continue };
        // A record that cannot be read still names its commit through its own file name, and its branch is unknown: the line is
        //  said with the branch the record would have carried when it can be read, and without a branch otherwise.
        let text: Option<String> = std::fs::read_to_string(entry.path()).ok();
        let item: Option<WorkItem> = text.clone().and_then(work_item_of);
        let branch: String = item.as_ref().map_or_else(String::new, branch_of);
        if !branch.is_empty() && record.on_the_remote(branch.as_str(), commit) == OnTheRemote::There {
            // Git holds the commit in the branch's remote, so the push this record would make has been made by another push of the
            //  branch. Nothing waits on it: it is settled where a finished push goes.
            settle_held(work_dir.as_path(), name.as_str());
            continue;
        }
        let holding: Holding = text.as_deref().map_or_else(Holding::undecided, holding_in);
        // Only a proven-gone command settles a record: while the command that landed the commit lives, the wait is its own, and a
        //  record it is about to promote or forget is not this command's to decide (`held_push_holder_and_verdict.rs`).
        let gone: bool = proven_gone(holding.holder.as_str());
        if gone {
            match holding.verdict {
                // The checks passed and nothing is left to run the record: it becomes runnable, and this command says so.
                Verdict::Passed => {
                    if crate::work_item::promote(work_dir.as_path(), name.as_str()) {
                        released = true;
                        lines.push(queued_push_line(branch.as_str(), commit));
                        continue;
                    }
                }
                // The checks refused the commit and nothing is left to write the verdict: it is settled exactly as the command that
                //  refused it would have settled it, so the commit stays local and the next push of the branch stops before it. The
                //  record is given up only once that verdict is kept. When it cannot be, the record is the only copy of the gate's words
                //  and a refusal it states is one of the verdicts that stop a push, so it is left where it is and the line says what holds
                //  the commit and what still sends it.
                Verdict::Blocked => {
                    if let Some(found) = item.as_ref() {
                        if crate::work_item::block(work_dir.as_path(), found, holding.reason.as_str()) {
                            crate::work_item::forget_held(work_dir.as_path(), name.as_str());
                            continue;
                        }
                        // The verdict could not be written, so this record is the only statement that the commit stays local: it is
                        //  left where it is and the line below says what holds it.
                        lines.push(a_refusal_that_could_not_be_kept(branch.as_str(), commit, holding.reason.as_str()));
                        continue;
                    }
                }
                // No verdict was ever written: the commit stays local, and the line says what holds it.
                Verdict::Undecided => {}
            }
        }
        lines.push(held_push_line(branch.as_str(), commit, &holding, gone));
    }
    if released {
        wake.wake(git_dir);
    }
    return lines;
}

/// What:
///  Moves a held record whose commit Git holds in the branch's remote to `done`,
/// where the queue keeps a push that was made.
/// Why:
///   `done` is the state a settlement already has,
/// so the record stays known to the queue (a repeat landing adds
///  nothing) and stops being a thing no round may run.
/// A move that fails leaves the record held,
/// which costs a line that is
///  still true of the record and loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settleHeld = (workDir, id) => { mkdirp(join(workDir, 'done')); rename(join(workDir, 'held', id), join(workDir, 'done', id)); };
/// ```
fn settle_held(work_dir: &Path, name: &str) {
    let done: PathBuf = state_directory(work_dir, crate::work_item::DONE);
    if crate::private_storage::create_private_directory_all(done.as_path()).is_err() {
        return;
    }
    let _settled: std::io::Result<()> = std::fs::rename(state_directory(work_dir, crate::work_item::HELD).join(name), done.join(name));
}

/// The branch one held record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const branchOf = (item) => item.branchRef;
/// ```
fn branch_of(item: &WorkItem) -> String {
    return item.branch_ref.clone();
}

/// The item one held record's text holds,
/// or nothing when it cannot be read.
fn work_item_of(text: String) -> Option<WorkItem> {
    return WorkItem::decode(text.as_str()).ok();
}

/// What:
///  How long a push record must have waited in `pending` before a command names it as queued.
/// Why:
///   A record a service has not taken yet is a queued push,
/// and the state is the same whether the wrapper was killed between promoting the record and waking the service
/// or the
///  service went away before it looked;
/// the bound only keeps the line from naming a push that is merely in flight.
/// Measured on this host at 2026-11-09T16:40-05:00 (`measure-claim-stamp.ts` and `measure-landing-to-claim.ts`
/// in `${HOME}/temp/agent/pi-runs/push-delivery-fix/repro`,
/// debug build,
/// local bare remote):
/// the time from a landing to the claim of its push record was between 15 ms and 3.9 s with the service idle
///  for 8 s between commits,
/// and between 14 ms and 10.7 s when 12 commits were made back to back.
/// The bound is above every in-flight window measured,
/// so a record it names has outlasted any service that was merely busy.
/// Exempt from a citation under `LQM` (class:
/// notice pacing):
/// it times a line the person reads and nothing waits on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits".
pub const PUSH_QUEUED_AFTER: std::time::Duration = std::time::Duration::from_secs(15);

/// What:
///  The one line a command says about the pushes this worktree has queued and no service has taken,
/// and how many more wait the same way.
/// Why:
///   A push record in `pending` is a push that has not been made and that the service takes without any further commit.
/// A wrapper
///  killed between promoting the record and waking the service leaves one,
/// and a service that went away before it
///  looked leaves one;
/// the command that would have said "the push is queued" died,
/// so the record is the only place
///  the statement can come from,
/// and every later command reads it from there.
///   One push is named and the rest are counted,
/// because the person acts on the oldest and the list of a
///  container's unserved commits has no end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queuedPushNotices(gitDir, now) // the oldest pending push, then a count
/// ```
pub fn queued_push_notices(git_dir: &Path, now: std::time::SystemTime) -> Vec<String> {
    let work_dir: PathBuf = work_directory(git_dir);
    let mut waiting: Vec<(String, std::time::SystemTime)> = Vec::new();
    for (name, queued) in crate::work_item::entries_oldest_first(work_dir.as_path(), crate::work_item::PENDING) {
        if !name.starts_with(format!("{}-", WorkKind::Push.name()).as_str()) {
            continue;
        }
        if now.duration_since(queued).unwrap_or(std::time::Duration::ZERO) < PUSH_QUEUED_AFTER {
            continue;
        }
        waiting.push((name, queued));
    }
    let Some((oldest, _)) = waiting.first() else { return Vec::new() };
    // The record names its own commit and branch; one that cannot be read still names its commit through its file name, and its
    //  branch is unknown.
    let path: PathBuf = state_directory(work_dir.as_path(), crate::work_item::PENDING).join(oldest.as_str());
    let record: Option<WorkItem> = std::fs::read_to_string(path).ok().and_then(work_item_of);
    let (commit, reference): (String, String) = match record {
        Some(item) => (item.oid, item.branch_ref),
        None => match crate::work_item_ids_and_names::object_of_a_record_name(oldest.as_str()) {
            Some(commit) => (commit.to_string(), String::new()),
            None => return Vec::new(),
        },
    };
    let mut lines: Vec<String> = vec![queued_push_line(reference.as_str(), commit.as_str())];
    let more: usize = waiting.len() - 1;
    if more > 0 {
        let (subject, possessive): (&str, &str) = if more == 1 { ("commit is", "its") } else { ("commits are", "their") };
        lines.push(format!("{more} more {subject} queued for {possessive} push the same way"));
    }
    return lines;
}
