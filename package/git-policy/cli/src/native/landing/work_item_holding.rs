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

use crate::held_push_holder_and_verdict::{
    Holding,
    Verdict,
    holding_in,
    proven_gone,
    with_holding,
};
use crate::private_storage::{
    create_private_directory_all,
    sync_directory,
    write_private_file,
    write_private_file_unsynced,
};
use crate::work_item::{
    DONE,
    FAILED,
    HELD,
    WorkItem,
    WorkKind,
    state_directory,
};
use std::path::{
    Path,
    PathBuf,
};

/// What is written into a held record and what moves it on,
/// and the state directory a refused commit is kept in,
/// are declared beside this file
/// (`work_item_holding_the_verdicts_and_the_promotion.rs`),
/// because the holds and their read-back verdicts are at their line bound.
pub use crate::work_item_holding_the_verdicts_and_the_promotion::{
    BLOCKED,
    BlockedVerdict,
    forget_held,
    promote,
    record_pass,
    record_refusal,
    record_verdict,
    requeue_after_giving_up,
};

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
pub fn hold(
    work_dir: &Path,
    item: &WorkItem,
    holder: &str,
) -> Result<(), String> {
    // A gate that blocked this commit keeps it local: no later hold, recovery included, may publish it.
    if is_blocked(work_dir, item.id().as_str()) {
        return Ok(());
    }
    // Work the queue has already settled is work that was done: a batch replayed after a restart must not hold it again.
    //  A hold of a settled id outlives the command that could have promoted it, so every command in the worktree would name a
    //  commit that was already pushed (`held_push_line`, `service_handoff.rs`).
    if settled_or_taken(work_dir, item.id().as_str()) {
        return Ok(());
    }
    if let Err(error) = create_private_directory_all(state_directory(work_dir, HELD).as_path()) {
        return Err(format!("creating the {HELD} directory failed: {error}"));
    }
    let holding: Holding = Holding {
        holder: holder.to_string(),
        verdict: Verdict::Undecided,
        reason: String::new(),
    };
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
pub fn undecided_commits(
    work_dir: &Path,
    branch_ref: &str,
) -> Vec<String> {
    let held: PathBuf = state_directory(work_dir, HELD);
    let Ok(listing) = std::fs::read_dir(held.as_path()) else {
        return Vec::new();
    };
    let mut commits: Vec<String> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(commit) = crate::work_item_ids_and_names::object_of_a_record_name(name.as_str())
        else {
            continue;
        };
        // A record that cannot be read names no branch: a pipe there answers at once instead of waiting for a writer.
        let Ok(text) = crate::plain_file::read_plain_text(&entry.path()) else {
            continue;
        };
        // The branch is the record's,
        // and a record this module cannot read names no branch:
        // it is left to the line every command prints for it.
        let Ok(item) = WorkItem::decode(text.as_str()) else {
            continue;
        };
        if item.branch_ref != branch_ref {
            continue;
        }
        let holding: Holding = holding_in(text.as_str());
        if holding.verdict != Verdict::Undecided
            || holding.holder.is_empty()
            || proven_gone(holding.holder.as_str())
        {
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
pub fn block(
    work_dir: &Path,
    item: &WorkItem,
    reason: &str,
) -> bool {
    let id: String = item.id();
    let blocked: PathBuf = state_directory(work_dir, BLOCKED);
    if let Err(error) = create_private_directory_all(blocked.as_path()) {
        crate::service_error!(
            "the blocked verdict of the push record {id} {} cannot be kept, because its directory cannot be made ({error}); the held record stays, so nothing here publishes that commit",
            crate::service_timing::in_the_repository_at(item.repository.as_path())
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
                "the blocked verdict of the push record {id} {} could not be written ({error}); the held record stays, so nothing here publishes \
                 that commit",
                crate::service_timing::in_the_repository_at(item.repository.as_path())
            );
            return false;
        }
    }
    // The name is the last part of the verdict a crash can lose, so the directory that holds it is flushed too.
    if let Err(error) = sync_directory(blocked.as_path()) {
        crate::service_error!(
            "the blocked verdict of the push record {id} {} cannot be flushed: {error}",
            crate::service_timing::in_the_repository_at(item.repository.as_path())
        );
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
fn blocked_record(
    branch_ref: &str,
    commit: &str,
    reason: &str,
) -> String {
    let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    fields.insert(String::from("v"), serde_json::Value::from(1));
    for (name, text) in [
        ("branch", branch_ref),
        ("commit", commit),
        ("reason", reason),
    ] {
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
        let commit: String =
            match crate::work_item_ids_and_names::object_of_a_record_name(name.as_str()) {
                Some(object) => object.to_owned(),
                None => continue,
            };
        // A record that cannot be read is still a verdict on a commit the gate refused, so it keeps the commit and says nothing more.
        let read: Option<BlockedVerdict> = match crate::plain_file::read_plain_text(path.as_path())
        {
            Ok(text) => blocked_verdict_of(text.as_str(), commit.as_str()),
            Err(_unreadable) => None,
        };
        verdicts.push(read.unwrap_or(BlockedVerdict {
            branch_ref: String::new(),
            commit,
            reason: String::new(),
        }));
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
        entries.push((
            entry.file_name().to_string_lossy().into_owned(),
            entry.path(),
        ));
    }
    entries.sort();
    let mut refusals: Vec<BlockedVerdict> = Vec::new();
    for (_name, path) in entries {
        let Ok(text) = crate::plain_file::read_plain_text(path.as_path()) else {
            continue;
        };
        let Ok(item) = WorkItem::decode(text.as_str()) else {
            continue;
        };
        if item.kind != WorkKind::Push {
            continue;
        }
        let holding: Holding = holding_in(text.as_str());
        if holding.verdict != Verdict::Blocked {
            continue;
        }
        refusals.push(BlockedVerdict {
            branch_ref: item.branch_ref.clone(),
            commit: item.oid.clone(),
            reason: holding.reason.clone(),
        });
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
fn blocked_verdict_of(
    text: &str,
    commit: &str,
) -> Option<BlockedVerdict> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    return Some(BlockedVerdict {
        branch_ref: value
            .get("branch")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        commit: commit.to_owned(),
        reason: value
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    });
}

/// Whether the gate of this commit kept it local.
fn is_blocked(
    work_dir: &Path,
    id: &str,
) -> bool {
    return state_directory(work_dir, BLOCKED).join(id).exists();
}

/// What:
///  Whether the queue has already finished with this id,
///  or another instance is running it now.
/// Why:
///   `done` is the answer that the work was done,
/// and `failed` the answer that it was given up on,
/// with a retry kept by the branch's retry plan when one can pass;
/// `claimed` and `recovering` are another holder's work.
/// A hold written over any of them is a second record nothing can promote,
/// because the command that would have promoted it answered before it was written,
///  and no round drains a held record
/// (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
/// `held_push_line`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = exists(join(workDir, 'done', id)) || exists(join(workDir, 'failed', id)) || findClaim(workDir, id) !== undefined;
/// ```
fn settled_or_taken(
    work_dir: &Path,
    id: &str,
) -> bool {
    // `done` and `failed` name the record itself, so they are one look each.
    if state_directory(work_dir, DONE).join(id).exists()
        || state_directory(work_dir, FAILED).join(id).exists()
    {
        return true;
    }
    // A claim is named by its id and the generation and reading of its holder, so it is found by its id.
    return crate::work_item::find_claim(work_dir, id).is_some();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_item_holding_tests.rs"]
mod tests;
