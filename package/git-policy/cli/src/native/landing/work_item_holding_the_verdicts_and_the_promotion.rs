//! What:
//!  What is written into a held record and what moves the record on:
//!  the verdict a post-commit gate or a refused commit leaves beside it,
//!  the promotion that makes it runnable,
//!  the requeue of a record the consumer gave up on,
//!  the forgetting of one its producer refused,
//!  and the state directories a promotion makes.
//! Why:
//!   The verdict must be in the record before it is promoted or forgotten,
//!  so a command killed between its gate and that step leaves a record whose own words say what it decided,
//!  and a refusal that a crash must not lose waits for the disk while a pass does not.
//!  They live beside the holds they rewrite,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/workItemHoldingTheVerdictsAndThePromotion.ts
//! export const BLOCKED = "blocked";
//! export const recordVerdict = (workDir, id, holder, verdict, reason): boolean => ...;
//! export const promote = (workDir, id): boolean => rename(held/id, pending/id);
//! ```

use crate::held_push_holder_and_verdict::{
    Holding,
    Verdict,
    this_command,
    with_holding,
};
use crate::private_storage::{
    create_private_directory_all,
    replace_private_file,
    replace_private_file_unsynced,
};
use crate::work_item_state_directories_and_claim_names::{
    CLAIMED,
    DONE,
    FAILED,
    HELD,
    PENDING,
    RECOVERING,
    state_directory,
};
use std::path::{
    Path,
    PathBuf,
};
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
pub fn record_verdict(
    work_dir: &Path,
    id: &str,
    holder: &str,
    verdict: Verdict,
    reason: &str,
) -> bool {
    return write_verdict(
        work_dir,
        id,
        &Holding {
            holder: holder.to_string(),
            verdict,
            reason: reason.to_string(),
        },
        false,
    );
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
fn write_verdict(
    work_dir: &Path,
    id: &str,
    holding: &Holding,
    durable: bool,
) -> bool {
    let path: PathBuf = state_directory(work_dir, HELD).join(id);
    // A named pipe at a held record's name is a record this pass cannot read, answered at once rather than waited on.
    let Ok(text) = crate::plain_file::read_plain_text(path.as_path()) else {
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
pub fn record_pass(
    work_dir: &Path,
    id: &str,
) -> bool {
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
pub fn record_refusal(
    work_dir: &Path,
    id: &str,
    reason: &str,
) -> bool {
    return write_verdict(
        work_dir,
        id,
        &Holding {
            holder: this_command(),
            verdict: Verdict::Blocked,
            reason: reason.to_string(),
        },
        true,
    );
}

/// What:
///  Makes a held item runnable,
/// whole by one rename,
/// and answers whether anything was held.
/// Why:
///   The promotion is the producer's approval,
/// so a record that is not there is no approval and the caller says so.
/// The rename is tried before anything is created:
/// the service made the record it is promoting,
/// so the state directories are there,
/// and creating seven of them on every commit cost a dozen metadata calls on a machine where one of those can take a second
/// (`doc/planning/cli-git-commit-latency-alpha-11-against-alpha-10.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promote(workDir, id) // rename held/<id> to pending/<id>
/// ```
pub fn promote(
    work_dir: &Path,
    id: &str,
) -> bool {
    let held: PathBuf = state_directory(work_dir, HELD).join(id);
    let pending: PathBuf = state_directory(work_dir, PENDING).join(id);
    if std::fs::rename(&held, &pending).is_ok() {
        return true;
    }
    // A work tree the queue has never used has nowhere to move the record into: the states are made and the rename tried again.
    if !create_states(work_dir) {
        return false;
    }
    return std::fs::rename(&held, &pending).is_ok();
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
pub fn requeue_after_giving_up(
    work_dir: &Path,
    id: &str,
) -> bool {
    if !create_states(work_dir) {
        return false;
    }
    return std::fs::rename(
        state_directory(work_dir, FAILED).join(id),
        state_directory(work_dir, PENDING).join(id),
    )
    .is_ok();
}

/// Forgets a held item whose producer refused it;
/// one that is already promoted is not reached.
pub fn forget_held(
    work_dir: &Path,
    id: &str,
) {
    let _removed: std::io::Result<()> =
        std::fs::remove_file(state_directory(work_dir, HELD).join(id));
}
