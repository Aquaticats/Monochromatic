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

use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::work_item::{CLAIMED, DONE, FAILED, HELD, PENDING, RECOVERING, WorkItem, state_directory};
use std::path::{Path, PathBuf};

/// What:
///  Holds one item where nothing drains it,
/// until its producer promotes it.
/// Why:
///   A landed commit's push must be durable before the command that owes it can be killed,
/// and it must not run
///  before that command's post-commit gate has decided:
/// a held record is both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await hold(workDir, item)
/// ```
///
/// # Errors
///
/// The work queue's own words when the record cannot be written.
pub fn hold(work_dir: &Path, item: &WorkItem) -> Result<(), String> {
    // A gate that blocked this commit keeps it local: no later hold, recovery included, may publish it.
    if is_blocked(work_dir, item.id().as_str()) {
        return Ok(());
    }
    if let Err(error) = create_private_directory_all(state_directory(work_dir, HELD).as_path()) {
        return Err(format!("creating the {HELD} directory failed: {error}"));
    }
    let path: PathBuf = state_directory(work_dir, HELD).join(item.id());
    return match write_private_file_unsynced(path.as_path(), item.encode().as_bytes()) {
        Ok(()) => Ok(()),
        // The same commit's push is already held; the gate has not decided either way.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("writing the held record failed: {error}")),
    };
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
/// and gives the held record up.
/// Why:
///   `forget_the_push` alone leaves no trace of the verdict,
/// so a later recovery of the same batch would hold the push again
///  and publish a commit the gate refused.
///   The verdict keeps the branch and the reason as well,
/// because the automatic push must stop before this commit and every
///  command must say why,
/// and a digest of the branch is not the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// block(workDir, item, reason) // write `blocked/<id>` with the branch and the words, hold nothing
/// ```
pub fn block(work_dir: &Path, item: &WorkItem, reason: &str) {
    let id: String = item.id();
    if create_private_directory_all(state_directory(work_dir, BLOCKED).as_path()).is_err() {
        crate::service_error!("the blocked verdict of the push record {id} cannot be kept");
        return;
    }
    let record: String = blocked_record(item.branch_ref.as_str(), item.oid.as_str(), reason);
    let _written: std::io::Result<()> = write_private_file_unsynced(state_directory(work_dir, BLOCKED).join(id.as_str()).as_path(), record.as_bytes());
    forget_held(work_dir, id.as_str());
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
/// in name order,
/// each naming its branch,
/// its commit and the gate's words.
/// Why:
///   The records are the only statement of a verdict,
/// so the automatic push and every command's notice read them here rather than
///  remembering anything.
/// A record that cannot be read is still a verdict on a commit the gate refused,
/// so it is
///  named without its branch and words:
/// silence would let that commit be published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// blockedVerdicts(workDir) // one entry per file in `blocked`, oldest name first
/// ```
pub fn blocked_verdicts(work_dir: &Path) -> Vec<BlockedVerdict> {
    let blocked: PathBuf = state_directory(work_dir, BLOCKED);
    let Ok(listing) = std::fs::read_dir(blocked.as_path()) else {
        return Vec::new();
    };
    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        entries.push((name, entry.path()));
    }
    entries.sort();
    let mut verdicts: Vec<BlockedVerdict> = Vec::new();
    for (name, path) in entries {
        let commit: String = match crate::work_item_ids_and_names::object_of_a_record_name(name.as_str()) {
            Some(object) => object.to_owned(),
            None => continue,
        };
        verdicts.push(match std::fs::read_to_string(path.as_path()).ok().and_then(|text| blocked_verdict_of(text.as_str(), commit.as_str())) {
            Some(verdict) => verdict,
            None => BlockedVerdict { branch_ref: String::new(), commit, reason: String::new() },
        });
    }
    return verdicts;
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

/// Forgets a held item whose producer refused it;
/// one that is already promoted is not reached.
pub fn forget_held(work_dir: &Path, id: &str) {
    let _removed: std::io::Result<()> = std::fs::remove_file(state_directory(work_dir, HELD).join(id));
}
