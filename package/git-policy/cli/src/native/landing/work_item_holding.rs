//! What:
//!  The three steps of a work item whose producer has not approved it yet:
//! hold it where nothing drains it,
//! promote it when
//!  the producer approves it,
//! and forget it when the producer refuses it,
//! and the verdict the producer refused it with.
//! Why:
//!   A landed commit's push must be durable before the command that owes it can be killed,
//! and it must not run
//!  before that command's post-commit gate has decided.
//! A held record is both:
//! no round lists the `held` state,
//! so
//!  only an explicit promotion makes the item runnable.
//!   A refusal outlives the command that made it:
//! the queue pushes the branch and not one commit,
//! so the next push of
//!  that branch must still know which commits it may not publish.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await hold(workDir, item); promote(workDir, id); forgetHeld(workDir, id);
//! ```

use crate::held_push_holder_and_verdict::{Holding, Verdict, holding_in, proven_gone, this_command, with_holding};
use crate::private_storage::{
    create_private_directory_all, replace_private_file, replace_private_file_unsynced, sync_directory, write_private_file, write_private_file_unsynced,
};
use crate::work_item::{CLAIMED, DONE, FAILED, HELD, PENDING, RECOVERING, WorkItem, WorkKind, state_directory};
use std::path::{Path, PathBuf};

/// What:
///  Holds one item where nothing drains it,
/// until its producer promotes it,
/// and says which command will decide whether it may run.
/// Why:
///   A landed commit's push must be durable before the command that owes it can be killed,
/// and it must not run
///  before that command's post-commit gate has decided:
/// a held record is both.
/// The holder is what lets a later command tell a wait that has a
///  command behind it from one whose command is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await hold(workDir, item, holder)
/// ```
///
/// # Errors
///
/// The work queue's own words when the record cannot be written.
pub fn hold(work_dir: &Path, item: &WorkItem, holder: &str) -> Result<(), String> {
    // A gate that blocked this commit keeps it local: no later hold, recovery included, may publish it.
    if is_blocked(work_dir, item.id().as_str()) {
        return Ok(());
    }
    if let Err(error) = create_private_directory_all(state_directory(work_dir, HELD).as_path()) {
        return Err(format!("creating the {HELD} directory failed: {error}"));
    }
    let holding: Holding = Holding { holder: holder.to_string(), verdict: Verdict::Undecided, reason: String::new() };
    let item_text: String = item.encode();
    // A record this module cannot rewrite is the item's own words, which hold no holder and so release nothing.
    let record: String = match with_holding(item_text.as_str(), &holding) {
        Some(rewritten) => rewritten,
        None => item_text,
    };
    let path: PathBuf = state_directory(work_dir, HELD).join(item.id());
    return match write_private_file_unsynced(path.as_path(), record.as_bytes()) {
        Ok(()) => Ok(()),
        // The same commit's push is already held; the gate has not decided either way.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("writing the held record failed: {error}")),
    };
}

/// What:
///  Writes what this command decided into the held record of `id`,
/// beside the item the record already states,
/// and answers whether a record was there to write it into.
/// Why:
///   The verdict must be in the record before it is promoted or forgotten:
/// a command killed between its gate
///  and that step leaves a record whose own words say what it decided,
/// so a later command settles it from the verdict rather than
///  from a guess,
/// and a record with no verdict keeps the commit local with the words naming what holds it
///  (`held_push_holder_and_verdict.rs`).
/// The holder written with the verdict is the command that decided,
/// which is the retry's own
///  command when a retry is the one that finishes the commit.
///   A pass is published whole by one rename and flushed by nothing,
/// because a commit's answer waits on no sync of the wrapper's own
/// (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`):
/// a kill keeps the verdict,
/// and a machine crash keeps the record the gate found,
/// which the settlement reads as no verdict,
/// so the commit stays local with the push a person can make named.
///   Flushing a pass alone would protect nothing worth its wait,
/// because `hold` writes the obligation itself into the same record unflushed
///  (`write_private_file_unsynced`):
/// a crash loses the promise to push with the verdict that approved it.
///   A refusal is the one verdict a crash must not lose,
/// because no verdict reads as a commit the branch may publish,
/// so it goes through the flushing writer and `block` keeps the verdict it settles into durably before the record is given up
/// (`held_push_holder_and_verdict.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordVerdict(held/<id>, holder, verdict, reason, durable) // replace the record whole; false when there is none
/// ```
pub fn record_verdict(work_dir: &Path, id: &str, holder: &str, verdict: Verdict, reason: &str) -> bool {
    return write_verdict(work_dir, id, &Holding { holder: holder.to_string(), verdict, reason: reason.to_string() }, false);
}

/// What:
///  `record_verdict` with the record's durability given.
/// Why:
///   Only the refusal waits for the disk:
/// a pass may be lost,
/// because the record it was found in then says the checks never
///  decided and the commit stays local,
/// while a lost refusal is a commit a later push publishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeVerdict(workDir, id, holding, durable) // false when there is no record
/// ```
fn write_verdict(work_dir: &Path, id: &str, holding: &Holding, durable: bool) -> bool {
    let path: PathBuf = state_directory(work_dir, HELD).join(id);
    let Ok(text) = std::fs::read_to_string(path.as_path()) else {
        return false;
    };
    let Some(record) = with_holding(text.as_str(), holding) else {
        return false;
    };
    return match durable {
        true => replace_private_file(path.as_path(), record.as_bytes()).is_ok(),
        false => replace_private_file_unsynced(path.as_path(), record.as_bytes()).is_ok(),
    };
}

/// What:
///  Writes a verdict beside the record this command is about to promote,
/// naming this command as the one that decided.
/// Why:
///   A caller that promotes a record an earlier attempt held takes it over:
/// the record then names the command that
///  finished the commit,
/// which is the one whose absence a later look can prove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const approved = recordVerdict(workDir, id, thisCommand(), 'passed', '');
/// ```
pub fn record_pass(work_dir: &Path, id: &str) -> bool {
    return record_verdict(work_dir, id, this_command().as_str(), Verdict::Passed, "");
}

/// What:
///  Writes a refusal beside the record this command is about to settle,
/// with the gate's own words,
/// flushing the record and its directory before it returns.
/// Why:
///   A refusal recorded before the record is forgotten survives a kill in that gap,
/// and the words it keeps are the ones
///  the next push of the branch must show.
/// A machine crash loses an unflushed rename,
/// and the record it leaves says the checks never decided,
///  which no push reads as a stop:
/// the refused commit would then be published by the next push of its branch,
/// so this one verdict waits for the disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = recordVerdict(workDir, id, thisCommand(), 'blocked', reason) // durable
/// ```
pub fn record_refusal(work_dir: &Path, id: &str, reason: &str) -> bool {
    return write_verdict(work_dir, id, &Holding { holder: this_command(), verdict: Verdict::Blocked, reason: reason.to_string() }, true);
}

/// What:
///  Makes a held item runnable,
/// whole by one rename,
/// and answers whether anything was held.
/// Why:
///   The promotion is the producer's approval,
/// so a record that is not there is no approval and the caller says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promote(workDir, id) // rename held/<id> to pending/<id>
/// ```
pub fn promote(work_dir: &Path, id: &str) -> bool {
    if !create_states(work_dir) {
        return false;
    }
    if std::fs::rename(state_directory(work_dir, HELD).join(id), state_directory(work_dir, PENDING).join(id)).is_err() {
        return false;
    }
    return true;
}

/// The state directory of the blocked verdicts.
pub const BLOCKED: &str = "blocked";

/// What:
///  The commits of one branch whose held push record says its post-commit check has not decided,
///  and whose command still runs.
/// Why:
///   A held record keeps a commit's push out of the queue until its gate answers,
/// and a push of the branch made for another
///  commit sends every commit up to the tip:
/// publishing the tip before that answer would publish a commit the gate may
///  keep local.
///  A record whose command is gone decides nothing ever,
/// so it holds nothing back (`SPEC.md`),
/// and a record that
///  names no command is one of an earlier release,
/// which holds nothing back as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// undecidedCommits(workDir, branchRef) // the commits whose gate runs now
/// ```
pub fn undecided_commits(work_dir: &Path, branch_ref: &str) -> Vec<String> {
    let held: PathBuf = state_directory(work_dir, HELD);
    let Ok(listing) = std::fs::read_dir(held.as_path()) else {
        return Vec::new();
    };
    let mut commits: Vec<String> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(commit) = crate::work_item_ids_and_names::object_of_a_record_name(name.as_str()) else { continue };
        let Ok(text) = std::fs::read_to_string(entry.path()) else { continue };
        // The branch is the record's,
        // and a record this module cannot read names no branch:
        // it is left to the line every command prints for it.
        let Ok(item) = WorkItem::decode(text.as_str()) else { continue };
        if item.branch_ref != branch_ref {
            continue;
        }
        let holding: Holding = holding_in(text.as_str());
        if holding.verdict != Verdict::Undecided || holding.holder.is_empty() || proven_gone(holding.holder.as_str()) {
            continue;
        }
        // An earlier release's name and this release's name can both hold one commit, so the answer is one commit.
        let mut known: bool = false;
        for earlier in &commits {
            known = known || earlier.as_str() == commit;
        }
        if !known {
            commits.push(commit.to_owned());
        }
    }
    return commits;
}

/// What:
///  One commit a post-commit gate kept local,
/// the branch it landed on,
/// and the words of the verdict.
/// Why:
///   The record's name carries the commit and a digest of the branch,
/// and a digest cannot be read back:
/// the automatic push
///  and every command's notice need the branch itself and the reason
/// the gate gave,
/// so both are written beside the
///  verdict.
///   A record of an earlier release holds no reason;
/// `reason` is empty then and the lines that name it fall back to
///  naming only the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BlockedVerdict = { branchRef: string; commit: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockedVerdict {
    /// The branch reference the blocked commit landed on.
    pub branch_ref: String,
    /// The commit the gate kept local.
    pub commit: String,
    /// What the gate said,
    /// as one line,
    /// or empty for a record an earlier release wrote.
    pub reason: String,
}

/// What:
///  Says durably that the gate of this commit kept it local,
/// with the words of the verdict,
/// and gives the held record up,
/// answering whether the verdict is now durable.
/// Why:
///   `forget_the_push` alone leaves no trace of the verdict,
/// so a later recovery of the same batch would hold the push again
///  and publish a commit the gate refused.
///   The verdict keeps the branch and the reason as well,
/// because the automatic push must stop before this commit and every
///  command must say why,
/// and a digest of the branch is not the branch.
///   The held record is given up only after the verdict is there:
/// a record forgotten beside a verdict that was not written leaves neither,
/// `is_blocked` then reads false,
/// and the next hold and push publish the commit the gate refused
///  (a review of `ee545033ad`,
/// finding 2).
/// A verdict that cannot be written leaves the held record standing,
/// which is the state that keeps the commit local,
///  and the failure is said rather than swallowed.
///   The verdict is flushed with its directory before the held record goes,
/// because the record is the only other copy of the refusal:
/// a machine crash that lost an unflushed verdict would leave a refused commit
///  with nothing that holds its branch back,
/// and the next push of the branch would publish it
/// (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// block(workDir, item, reason) // write `blocked/<id>` durably with the branch and the words, then forget the held record; false when it could not be kept
/// ```
pub fn block(work_dir: &Path, item: &WorkItem, reason: &str) -> bool {
    let id: String = item.id();
    let blocked: PathBuf = state_directory(work_dir, BLOCKED);
    if let Err(error) = create_private_directory_all(blocked.as_path()) {
        crate::service_error!(
            "the blocked verdict of the push record {id} cannot be kept, because its directory cannot be made ({error}); the held record stays, so nothing here publishes that commit"
        );
        return false;
    }
    let path: PathBuf = blocked.join(id.as_str());
    let record: String = blocked_record(item.branch_ref.as_str(), item.oid.as_str(), reason);
    match write_private_file(path.as_path(), record.as_bytes()) {
        Ok(()) => {}
        // A verdict another command already wrote for this commit is the same refusal: a record is never replaced, and the verdict stands.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            crate::service_error!(
                "the blocked verdict of the push record {id} could not be written ({error}); the held record stays, so nothing here publishes \
                 that commit"
            );
            return false;
        }
    }
    // The name is the last part of the verdict a crash can lose, so the directory that holds it is flushed too.
    if let Err(error) = sync_directory(blocked.as_path()) {
        crate::service_error!("the blocked verdict of the push record {id} cannot be flushed: {error}");
        return false;
    }
    forget_held(work_dir, id.as_str());
    return true;
}

/// What:
///  The one JSON line of a blocked verdict.
/// Why:
///   The branch and the words hold no character the record's own reader cannot read back,
/// and JSON escapes the ones they
///  might hold,
/// so a branch name with a line feed and a message with a quote both survive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ v: 1, branch, commit, reason }) + '\n'
/// ```
fn blocked_record(branch_ref: &str, commit: &str, reason: &str) -> String {
    let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    fields.insert(String::from("v"), serde_json::Value::from(1));
    for (name, text) in [("branch", branch_ref), ("commit", commit), ("reason", reason)] {
        fields.insert(String::from(name), serde_json::Value::from(text));
    }
    return format!("{}\n", serde_json::Value::Object(fields));
}

/// What:
///  Every verdict this worktree's queue kept,
/// and every refusal a held record still states,
/// in name order,
/// each naming its branch,
/// its commit and the gate's words,
/// and one per commit.
/// Why:
///   The records are the only statement of a verdict,
/// so the automatic push and every command's notice read them here rather than
///  remembering anything.
/// A record that cannot be read is still a verdict on a commit the gate refused,
/// so it is
///  named without its branch and words:
/// silence would let that commit be published.
///  A refusal the held records state is read here too,
/// because the verdict goes into such a record first
///  and that record outliving it is the state a command killed in the gap or a failed write leaves,
/// so a commit the gate refused stays local either way.
/// That reading is not skipped when the `blocked` directory itself cannot be read:
/// a refusal that could not be written is then exactly the state the held records hold,
///  and a push decided from an empty list would publish it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// blockedVerdicts(workDir) // one entry per file in `blocked`, oldest name first, and one per refusal the held records state
/// ```
pub fn blocked_verdicts(work_dir: &Path) -> Vec<BlockedVerdict> {
    let blocked: PathBuf = state_directory(work_dir, BLOCKED);
    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    // The directory may be unusable rather than empty: a storage failure leaves a file in its place, and the refusals the held
    //  records state are read below whatever it answers.
    if let Ok(listing) = std::fs::read_dir(blocked.as_path()) {
        for entry in listing.flatten() {
            let name: String = entry.file_name().to_string_lossy().into_owned();
            entries.push((name, entry.path()));
        }
    }
    entries.sort();
    let mut verdicts: Vec<BlockedVerdict> = Vec::new();
    for (name, path) in entries {
        let commit: String = match crate::work_item_ids_and_names::object_of_a_record_name(name.as_str()) {
            Some(object) => object.to_owned(),
            None => continue,
        };
        // A record that cannot be read is still a verdict on a commit the gate refused, so it keeps the commit and says nothing more.
        let read: Option<BlockedVerdict> = match std::fs::read_to_string(path.as_path()) {
            Ok(text) => blocked_verdict_of(text.as_str(), commit.as_str()),
            Err(_unreadable) => None,
        };
        verdicts.push(read.unwrap_or(BlockedVerdict { branch_ref: String::new(), commit, reason: String::new() }));
    }
    // A held record whose own words say its gate refused the commit is a refusal as well: the command that wrote them was killed
    //  between the verdict and the verdict record, or that record could not be written. The commit must not be published by the
    //  branch's next push in that state either (`held_push_holder_and_verdict.rs`), and one verdict per commit is enough.
    for refusal in refused_while_held(work_dir) {
        let mut recorded: bool = false;
        for verdict in &verdicts {
            if verdict.commit == refusal.commit {
                recorded = true;
            }
        }
        if !recorded {
            verdicts.push(refusal);
        }
    }
    return verdicts;
}

/// What:
///  The refusals the held records of this worktree still state,
///  one per commit,
///  in name order.
/// Why:
///   A verdict record is written after the verdict is written into the held record,
/// so a command killed in that gap,
///  or a verdict record that could not be written,
///  leaves the refusal in the record alone;
///  the push of the branch is decided from the refusals this worktree can read,
///  and a held refusal is one of them.
/// A record that cannot be read,
///  or names a kind that is not a push,
/// states no refusal a push of a branch must stop before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusedWhileHeld(workDir) // for each held record whose holding says 'blocked': { branch, commit, reason }
/// ```
fn refused_while_held(work_dir: &Path) -> Vec<BlockedVerdict> {
    let held: PathBuf = state_directory(work_dir, HELD);
    let Ok(listing) = std::fs::read_dir(held.as_path()) else {
        return Vec::new();
    };
    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    for entry in listing.flatten() {
        entries.push((entry.file_name().to_string_lossy().into_owned(), entry.path()));
    }
    entries.sort();
    let mut refusals: Vec<BlockedVerdict> = Vec::new();
    for (_name, path) in entries {
        let Ok(text) = std::fs::read_to_string(path.as_path()) else { continue };
        let Ok(item) = WorkItem::decode(text.as_str()) else { continue };
        if item.kind != WorkKind::Push {
            continue;
        }
        let holding: Holding = holding_in(text.as_str());
        if holding.verdict != Verdict::Blocked {
            continue;
        }
        refusals.push(BlockedVerdict { branch_ref: item.branch_ref.clone(), commit: item.oid.clone(), reason: holding.reason.clone() });
    }
    return refusals;
}

/// What:
///  The verdict one record's text holds,
/// or nothing when it is not one.
/// Why:
///   A record an earlier release wrote holds the word `blocked` alone,
/// which names no branch and no words;
/// the caller
///  then keeps the commit and says only what the name holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parse = (text, commit) => { try { const held = JSON.parse(text); return { branchRef: held.branch, commit, reason: held.reason }; } catch { return undefined; } };
/// ```
fn blocked_verdict_of(text: &str, commit: &str) -> Option<BlockedVerdict> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    return Some(BlockedVerdict {
        branch_ref: value.get("branch").and_then(serde_json::Value::as_str).unwrap_or_default().to_owned(),
        commit: commit.to_owned(),
        reason: value.get("reason").and_then(serde_json::Value::as_str).unwrap_or_default().to_owned(),
    });
}

/// Whether the gate of this commit kept it local.
fn is_blocked(work_dir: &Path, id: &str) -> bool {
    return state_directory(work_dir, BLOCKED).join(id).exists();
}

/// What:
///  Creates every state directory a claimed item passes through.
/// Why:
///   A claim moves an item into `recovering` and then into `claimed`,
/// and its settlement into `done` or `failed`;
/// a
///  worktree whose first record was held has none of them,
/// so a promotion that made only `pending` would leave an approved
///  push that no consumer can ever claim.
/// The set is the one `enqueue_all` makes,
/// because both are the queue's own directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createStates(workDir) // pending, claimed, done, failed, recovering
/// ```
fn create_states(work_dir: &Path) -> bool {
    for state in [PENDING, HELD, BLOCKED, CLAIMED, DONE, FAILED, RECOVERING] {
        if create_private_directory_all(state_directory(work_dir, state).as_path()).is_err() {
            return false;
        }
    }
    return true;
}

/// What:
///  Put a record the consumer gave up on back where a round finds it,
/// when its id stands in `failed`,
/// and
///  say whether it moved.
/// Why:
///   Some work has an id its producer can name again without knowing what became of the last attempt:
/// the
///  delete of one path is that work,
/// and a consumer that gave up on it is no statement that the condition lasts for ever
///  (a disk fills and empties,
/// a volume is replaced,
/// a mount is remounted).
/// Nothing scans `failed`,
/// and an
///  id that reached it is answered as already known,
/// so without this a name the producer keeps naming is never
///  acted on again.
/// The count of tries goes with the record,
/// so each producer's hand-over buys one more attempt rather than
///  a fresh allowance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const requeueAfterGivingUp = (workDir, id) => rename(join(workDir, 'failed', id), join(workDir, 'pending', id));
/// ```
pub fn requeue_after_giving_up(work_dir: &Path, id: &str) -> bool {
    if !create_states(work_dir) {
        return false;
    }
    return std::fs::rename(state_directory(work_dir, FAILED).join(id), state_directory(work_dir, PENDING).join(id)).is_ok();
}

/// Forgets a held item whose producer refused it;
/// one that is already promoted is not reached.
pub fn forget_held(work_dir: &Path, id: &str) {
    let _removed: std::io::Result<()> = std::fs::remove_file(state_directory(work_dir, HELD).join(id));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_item_holding_tests.rs"]
mod tests;
