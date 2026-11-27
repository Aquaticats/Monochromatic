//! What:
//!  The looks at Git's `index.lock` and what each proves:
//!  the last look's proof (a holder proven alive,
//!  a PID number this process table does not have,
//!  or nothing),
//!  the two looks a command makes without waiting,
//!  and the sentences those looks are reported in.
//! Why:
//!  A merge that has nothing of itself in the checkout must not wait for a lock,
//!  and a merge about to apply must know who holds it,
//!  so both read the same proof the wait reads
//!  and a lock a gone process left is treated the same way by all three.
//!  The proof is one classification and one set of words,
//!  so a caller reads a lock through the mechanism the wait uses and never through its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = lookAtTheIndexLockOnce(realIndex); // 'free' | 'held' | { leftByAGoneProcess: pid }
//! ```

/// Evidence about the holder of a lock:
/// the two records a look reads,
/// and the words they are reported in.
use crate::index_lock_evidence::{
    Evidence,
    Verdict,
    classify_index_lock,
    gather_with,
    host_pid_process,
};
/// The sentences a look reports the evidence in.
use crate::index_lock_evidence_words::{
    blocker_words,
    dead_owner_diagnostic,
};
/// The bounded scan of open descriptors.
use crate::index_lock_holders::scan_platform_holders_bounded;
/// What came of a wait,
/// which a look also answers in when it does not wait.
use crate::index_wait_outcome::IndexWait;
/// The place of the lock.
use crate::service_index_lock::lock_path_of;
/// Writing the lines a person reads.
use std::io::Write;
/// `Path` is a borrowed filesystem path,
/// and not the owned `PathBuf`.
use std::path::Path;
/// `Duration` is a span of monotonic time,
/// and not a wall-clock `SystemTime`.
use std::time::Duration;

/// The sentence a merge's dead-owner line ends on:
/// nothing of the merge is in the checkout until that lock is gone,
/// and the lock is not that merge's failure,
/// because it is cleared on the proof the next clause states or the merge is saved on a side ref and exits 0.
const MERGE_CONSEQUENCE: &str = "the merge does not apply until that lock is gone";

/// The sentence a command's dead owner's line ends on when nothing else holds the lock:
/// the command clears it and goes on.
pub const CLEARS_A_DEAD_OWNERS_LOCK: &str = "the merge clears that lock when nothing else holds it and goes on; if anything else holds it, nothing was written by this step and the lock was left in place";

/// What the last look at the lock found.
pub(crate) enum Proof {
    /// A holder is proven alive.
    Alive(Evidence, i64, bool, Option<String>),
    /// The PID file names a process that is gone.
    Dead(i64),
    /// Nothing proves a holder alive or dead.
    Nothing(Option<Evidence>),
}

/// What the evidence for `real_index` proves now.
pub(crate) fn look_at(
    real_index: &Path,
    gather: &dyn Fn(&Path) -> std::io::Result<Option<Evidence>>,
) -> Proof {
    let Ok(Some(evidence)) = gather(real_index) else {
        return Proof::Nothing(None);
    };
    return match classify_index_lock(&evidence) {
        Verdict::ProvenAlive {
            pid,
            by_descriptor,
            command,
        } => Proof::Alive(evidence, pid, by_descriptor, command),
        Verdict::Dead(pid) => Proof::Dead(pid),
        Verdict::EvidenceFree => Proof::Nothing(Some(evidence)),
    };
}

/// The evidence of the host:
/// the Git lock PID file and the bounded scan of open descriptors.
pub(crate) fn gather_here(real_index: &Path) -> std::io::Result<Option<Evidence>> {
    return gather_with(real_index, host_pid_process, scan_platform_holders_bounded);
}

/// Write `bytes` to `out`,
/// then flush it only if the write succeeded;
/// the callers ignore the result,
/// because a closed terminal
///  is not a reason to stop the command the person asked for.
pub(crate) fn write_then_flush<W: Write + ?Sized>(
    out: &mut W,
    bytes: &[u8],
) -> std::io::Result<()> {
    out.write_all(bytes)?;
    return out.flush();
}

/// What the one look at the index lock found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLook = { kind: 'free' } | { kind: 'held' } | { kind: 'left-by-a-gone-process'; pid: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexLook {
    /// Nobody holds it.
    Free,
    /// A program holds it,
    /// or nothing proves which:
    /// either way a merge with nothing of itself in the checkout does not wait.
    Held,
    /// The PID file names a process that no longer runs,
    /// so nothing will release it.
    LeftByAGoneProcess(
        /// The number the PID file holds.
        i64,
    ),
}

/// What:
///  The index lock,
/// looked at once and never waited for.
/// Why:
///  A merge that is saved on its record and its side ref has nothing of itself in the checkout,
/// so a lock cannot be in
///  its way:
/// the next command finishes the merge once the lock is free.
/// Waiting cost every command (a `git rev-parse HEAD`
///  included) the whole ceiling again for as long as the lock stood,
/// and a lock a killed Git left never goes by
///  itself (finding 3 of the merge-and-pull audit of 2026-11-09).
/// The holder is classified exactly as the wait
///  classifies it,
/// so a lock a gone process left is treated the same way by both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = lookAtTheIndexLockOnce(realIndex); // 'free' | 'held' | { leftByAGoneProcess: pid }
/// ```
pub fn look_at_the_index_lock_once(real_index: &Path) -> IndexLook {
    if std::fs::symlink_metadata(lock_path_of(real_index)).is_err() {
        return IndexLook::Free;
    }
    return match look_at(real_index, &gather_here) {
        Proof::Dead(pid) => IndexLook::LeftByAGoneProcess(pid),
        Proof::Alive(..) | Proof::Nothing(_) => IndexLook::Held,
    };
}

/// What:
///  The index lock before a merge's own apply,
/// looked at once and never waited for:
///  the dead owner whose lock the merge may clear is named,
///  a lock a holder proven alive holds ends the apply there,
///  and a lock nothing proves the owner of ends it there too.
/// Why:
///   Three owners are met here,
///  and none of them is waited for.
///  A number this process table does not have is what a killed Git leaves,
///  and no process here will release it,
///  so the lock is named and the merge clears it on the commit service's proof of disuse
///  (`remove_if_the_owner_is_gone`).
///  A holder proven alive is another program's command,
///  and the merge has an answer that needs no wait:
///  its commit is already made,
///  so the merge is saved on a side ref for the next command in the worktree to finish
///  (`SPEC.md`,
///  "Merges and commits never fail").
///  Waiting on another agent's command is what the owner forbids
///  (`SPEC.md`,
///  "Dozens of agents at once is normal operation"),
///  and a lock that stayed held for the whole ceiling reached the same save,
///  so the minute bought nothing a person asked for
///  (a cross-vendor audit of alpha.12,
///  "A merge waits for another agent's live index holder").
///  An owner nothing proves is the third case,
///  and it is waited for no more than the other two,
///  because it is exactly as likely to be another program's live command:
///  native Git keeps the whole index in `index.lock` through its hooks and editor with no descriptor open,
///  and writes no PID file unless `core.lockfilePid` is on,
///  so a person's own `git commit` at its editor and a killed Git leave the same evidence.
///  A merge that waited for that case would wait a minute for the commit of the person sitting at the machine.
///  The holder is classified exactly as the wait classifies it,
///  so a lock a gone process left is cleared on the same proof by both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // 'not-held' | { ownerGone } | { heldBy: 'PID 42' }
/// lookAtTheIndexBeforeAMerge(gitDir.resolve('index'), stderr)
/// ```
pub fn look_at_the_index_before_a_merge<W: Write + ?Sized>(
    real_index: &Path,
    out: &mut W,
) -> IndexWait {
    let lock: std::path::PathBuf = lock_path_of(real_index);
    if std::fs::symlink_metadata(&lock).is_err() {
        return IndexWait::NotHeld;
    }
    return match look_at(real_index, &gather_here) {
        Proof::Dead(pid) => {
            let _noted = write_then_flush(
                out,
                dead_owner_diagnostic(
                    lock.as_path(),
                    pid,
                    MERGE_CONSEQUENCE,
                    CLEARS_A_DEAD_OWNERS_LOCK,
                )
                .as_bytes(),
            );
            IndexWait::OwnerGone(Duration::ZERO)
        }
        // A holder proven alive and an owner nothing proves are both another program's command until they let go, and the
        //  merge does not wait for either: it is saved on its side ref and the next command settles it.
        Proof::Alive(evidence, ..) | Proof::Nothing(Some(evidence)) => {
            IndexWait::HeldByAnotherCommand(blocker_words(&evidence))
        }
        Proof::Nothing(None) => {
            IndexWait::HeldByAnotherCommand(String::from("a program whose owner could not be read"))
        }
    };
}
