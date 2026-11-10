//! What:
//!  The settle of a fast-forward Git refused,
//! awaited by a caller that is itself a task of the runtime.
//! Why:
//!  `settle_failed_merge` and the folder it settles through read Git with the synchronous starts,
//! and a synchronous start is refused from inside the runtime,
//! so a task cannot call it.
//!  The settle runs on a thread of its own and its answer is awaited,
//! which is a labelled workaround with a removal condition:
//! the recovery folder's awaited Git seam replaces it and the thread goes with it
//! (`doc/planning/git-policy-cli-full-tokio.md`,
//! "Step T12, the second pass awaits its own Git, and the reconcile's chain stops at the branch move").
//!  The thread is a plain one,
//! never a job of the blocking pool:
//! a job of the pool is inside the runtime and would refuse the same start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = await settleFailedMergeOnAThreadOfItsOwn({ place, record, intent });
//! ```

use crate::merge_isolation_recovery::settle_failed_merge;
use crate::merge_isolation_recovery_steps::{MergeRecoveryAction, MergeRefusal, RecoveryPlace};
use crate::merge_isolation_journal::MergeIntent;
use crate::recovery_error::RecoveryError;
use std::path::PathBuf;

/// What:
///  Settle a merge record from a thread of its own,
/// and answer what the settle decided.
/// Why:
///   The work starts Git synchronously,
/// and only a thread outside the runtime may start one from a task's work.
/// A thread the system refuses is not run here:
/// the settle's own starts would be refused in this task's place,
/// so the record stays and the next wrapper command settles it,
/// which is said in the answer rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [answer] = await Promise.all([once(settleFailedMerge)]);
/// ```
pub(crate) async fn settle_failed_merge_awaiting(
    place: RecoveryPlace,
    record: PathBuf,
    intent: MergeIntent,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let (tell, heard) = tokio::sync::oneshot::channel();
    let started = std::thread::Builder::new()
        .name(String::from("git-policy-settle"))
        .spawn(move || {
            let answered: Result<MergeRecoveryAction, MergeRefusal> =
                settle_failed_merge(&place, record.as_path(), &intent, &mut std::io::stderr());
            // A receiver the caller dropped is a caller that stopped waiting: the answer has nowhere to go, and the settle
            //  itself is done either way.
            let _told = tell.send(answered);
        });
    if let Err(error) = started {
        return Err(MergeRefusal {
            error: RecoveryError(crate::scoped_start::refused_thread_sentence(
                "settle the merge record",
                &error,
            )),
            began: true,
        });
    }
    return match heard.await {
        Ok(answered) => answered,
        // The thread ended without an answer: a panic is reported by the thread itself, and the record is left for the next
        //  command, which is what a settle that did not run leaves.
        Err(_dropped) => Err(MergeRefusal {
            error: RecoveryError(String::from(
                "the merge record could not be settled: the thread that settles it ended without an answer. \
                 Nothing was removed; the next git command settles it",
            )),
            began: true,
        }),
    };
}
