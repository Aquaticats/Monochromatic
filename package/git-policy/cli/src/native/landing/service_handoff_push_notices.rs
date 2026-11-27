//! What:
//!  What a command says to the person about the pushes this worktree holds and the pushes it queued:
//! the one sentence per held record,
//! the one sentence for a refusal the queue could not keep beside it,
//! the id of the command a record names,
//! the look over the held and the waiting records that settles what Git already holds,
//! and the line for the pushes nobody has taken.
//! Why:
//!  The record is the only place these states exist,
//! so each line is read from the records themselves and can never disagree with them,
//! while the hand-off that writes those records (`service_handoff.rs`) sits elsewhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! heldPushNotices(gitDir, record, wake); queuedPushNotices(gitDir, now, record, wake);
//! ```

use crate::blocked_commit_lines::named_reason;
use crate::held_push_commit_on_the_remote::{
    OnTheRemote,
    RemoteRecord,
};
use crate::held_push_holder_and_verdict::{
    Holding,
    Verdict,
    holding_in,
    pid_of,
    proven_gone,
};
use crate::held_push_release_wakes_the_service::WakeTheService;
use crate::service_handoff_queued_push_words::queued_push_line;
use crate::service_work::work_directory;
use crate::work_item::{
    WorkItem,
    WorkKind,
    state_directory,
};
use std::path::{
    Path,
    PathBuf,
};

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
pub fn held_push_line(
    reference: &str,
    commit: &str,
    holding: &Holding,
    gone: bool,
) -> String {
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
        true => format!(
            "the commit {commit} is held: no command has said whether its checks passed, so this record pushes nothing; {carried}"
        ),
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
pub fn a_refusal_that_could_not_be_kept(
    reference: &str,
    commit: &str,
    reason: &str,
) -> String {
    let by_the_person: &str = if reference.is_empty() {
        "a push you make yourself still sends it"
    } else {
        "a push you make of it still sends it"
    };
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
pub fn held_push_notices(
    git_dir: &Path,
    record: &dyn RemoteRecord,
    wake: &dyn WakeTheService,
) -> Vec<String> {
    let work_dir: PathBuf = work_directory(git_dir);
    let held: PathBuf = state_directory(work_dir.as_path(), crate::work_item::HELD);
    let Ok(listing) = std::fs::read_dir(held.as_path()) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = Vec::new();
    // One wake for every record this command released, sent once at the end: a released push runs the next time the service looks,
    //  and a command that says the push is queued wakes it rather than waiting for the next commit.
    let mut released: bool = false;
    // The push records this command settled or removed here, by the id the service holds their values against.
    let mut forgotten: Vec<String> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(commit) = crate::work_item_ids_and_names::object_of_a_record_name(name.as_str())
        else {
            continue;
        };
        // A record that cannot be read still names its commit through its own file name, and its branch is unknown: the line is
        //  said with the branch the record would have carried when it can be read, and without a branch otherwise.
        //  A named pipe at its name answers as unreadable at once (`crate::plain_file::read_plain_text`) instead of waiting on.
        let text: Option<String> = crate::plain_file::read_plain_text(&entry.path()).ok();
        let item: Option<WorkItem> = text.clone().and_then(work_item_of);
        let branch: String = item.as_ref().map_or_else(String::new, branch_of);
        if !branch.is_empty() && record.on_the_remote(branch.as_str(), commit) == OnTheRemote::There
        {
            // Git holds the commit in the branch's remote, so the push this record would make has been made by another push of the
            //  branch. Nothing waits on it: it is settled where a finished push goes.
            settle_held(work_dir.as_path(), name.as_str());
            forgotten.push(name.clone());
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
                        // The record became runnable in this command, so it has not waited at all.
                        lines.push(queued_push_line(
                            branch.as_str(),
                            commit,
                            std::time::Duration::ZERO,
                        ));
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
                        if crate::work_item::block(
                            work_dir.as_path(),
                            found,
                            holding.reason.as_str(),
                        ) {
                            crate::work_item::forget_held(work_dir.as_path(), name.as_str());
                            forgotten.push(name.clone());
                            continue;
                        }
                        // The verdict could not be written, so this record is the only statement that the commit stays local: it is
                        //  left where it is and the line below says what holds it.
                        lines.push(a_refusal_that_could_not_be_kept(
                            branch.as_str(),
                            commit,
                            holding.reason.as_str(),
                        ));
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
    // The service learns nothing else about a record this command settled: the values it was given are given up now.
    wake.forget_programs(git_dir, forgotten.as_slice());
    return lines;
}

/// What:
///  Moves a push record whose commit Git holds in the branch's remote to `done`,
/// from the state it waits in.
/// Why:
///   `done` is the state a settlement already has,
/// so the record stays known to the queue (a repeat landing adds
///  nothing) and stops being a thing no round may run.
/// A move that fails leaves the record where it is,
/// which costs a line that
///  is still true of the record and loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settle = (workDir, state, id) => { mkdirp(join(workDir, 'done')); rename(join(workDir, state, id), join(workDir, 'done', id)); };
/// ```
fn settle_where_a_push_that_was_made_is_kept(
    work_dir: &Path,
    state: &str,
    name: &str,
) {
    let done: PathBuf = state_directory(work_dir, crate::work_item::DONE);
    if crate::private_storage::create_private_directory_all(done.as_path()).is_err() {
        return;
    }
    let _settled: std::io::Result<()> =
        std::fs::rename(state_directory(work_dir, state).join(name), done.join(name));
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
fn settle_held(
    work_dir: &Path,
    name: &str,
) {
    settle_where_a_push_that_was_made_is_kept(work_dir, crate::work_item::HELD, name);
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
///   A push record in `pending` is a push that has not been made and that a running service takes without any further
/// commit
/// (`LOOK_AGAIN_AFTER` in `service_claim_watch.rs`).
/// A wrapper killed between promoting the record and waking the service
///  leaves one,
/// and a service that went away before it looked leaves one;
/// the command that would have said "the push is
///  queued" died,
/// so the record is the only place the statement can come from,
/// and every later command reads it from there.
///   A record whose commit Git already holds in the branch's upstream ref is a push that was made,
/// by the service or by
///  the person's own `git push`,
/// and nothing waits on it:
/// the same proof that settles a held record settles it
///  (`held_push_commit_on_the_remote.rs`),
/// so the line and `git status` agree instead of the line outliving the push.
/// The
///  records that still wait wake the service,
/// as a released held record does,
/// so the command that says a push is queued is also
///  the command that has the service look at it now.
///   One push is named and the rest are counted,
/// because the person acts on the oldest and the list of a
///  container's unserved commits has no end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queuedPushNotices(gitDir, now, record, wake) // settle what Git already holds; the oldest waiting push, then a count; wake
/// ```
pub fn queued_push_notices(
    git_dir: &Path,
    now: std::time::SystemTime,
    record: &dyn RemoteRecord,
    wake: &dyn WakeTheService,
) -> Vec<String> {
    let work_dir: PathBuf = work_directory(git_dir);
    let mut waiting: Vec<(std::time::SystemTime, String, String)> = Vec::new();
    // The pending push records this command settles here, by the id the service holds their values against.
    let mut forgotten: Vec<String> = Vec::new();
    for (name, queued) in
        crate::work_item::entries_oldest_first(work_dir.as_path(), crate::work_item::PENDING)
    {
        if !name.starts_with(format!("{}-", WorkKind::Push.name()).as_str()) {
            continue;
        }
        if now
            .duration_since(queued)
            .unwrap_or(std::time::Duration::ZERO)
            < PUSH_QUEUED_AFTER
        {
            continue;
        }
        // The record names its own commit and branch; one that cannot be read still names its commit through its file name, and
        //  a record with no branch has no upstream to ask about.
        let path: PathBuf =
            state_directory(work_dir.as_path(), crate::work_item::PENDING).join(name.as_str());
        let (commit, reference): (String, String) = match crate::plain_file::read_plain_text(&path)
            .ok()
            .and_then(work_item_of)
        {
            Some(item) => (item.oid, item.branch_ref),
            None => match crate::work_item_ids_and_names::object_of_a_record_name(name.as_str()) {
                Some(commit) => (commit.to_string(), String::new()),
                None => continue,
            },
        };
        if !reference.is_empty()
            && record.on_the_remote(reference.as_str(), commit.as_str()) == OnTheRemote::There
        {
            settle_where_a_push_that_was_made_is_kept(
                work_dir.as_path(),
                crate::work_item::PENDING,
                name.as_str(),
            );
            forgotten.push(name.clone());
            continue;
        }
        waiting.push((queued, commit, reference));
    }
    // A record this command settled keeps no value: nothing runs it any more, and the service learns nothing else about it.
    wake.forget_programs(git_dir, forgotten.as_slice());
    let Some((queued, oldest_commit, oldest_reference)) = waiting.first() else {
        return Vec::new();
    };
    let waited: std::time::Duration = now
        .duration_since(*queued)
        .unwrap_or(std::time::Duration::ZERO);
    let mut lines: Vec<String> = vec![queued_push_line(
        oldest_reference.as_str(),
        oldest_commit.as_str(),
        waited,
    )];
    let more: usize = waiting.len() - 1;
    if more > 0 {
        let (subject, possessive): (&str, &str) = if more == 1 {
            ("commit is", "its")
        } else {
            ("commits are", "their")
        };
        lines.push(format!(
            "{more} more {subject} queued for {possessive} push the same way"
        ));
    }
    // The service is told now, so the record that waits is taken without waiting for the next commit either.
    wake.wake(git_dir);
    return lines;
}
