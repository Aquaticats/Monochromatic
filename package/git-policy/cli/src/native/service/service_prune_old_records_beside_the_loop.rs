//! What:
//!  The removal of the records of old finished batches,
//!  beside the service's landing loop.
//! Why:
//!  The records of the newest `DONE_RETAINED` finished batches are read into memory before any commit lands,
//! because a client whose answer was lost asks again and must be answered from the journal,
//! and that read cannot move.
//! The records *past* that count are only removed,
//! and no landing needs them gone:
//! a request the journal answers is one the newest records already name.
//! The start of 2026-11-15 at 07:58:09 spent 3,628 ms between the line that warmed the rules and the line that removed
//! ten old finished batches,
//! where the two starts before took 106 ms and 40 ms,
//! and the first commit after it took 5,577 ms
//! (`doc/troubleshooting/git-policy-cli-commit-waits-for-the-services-record-write.md`):
//! a removal of old records is worth nobody's commit.
//!  A process whose runtime cannot take the task removes them where it stands and says so,
//! which is what every start did before this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pruneOldRecordsBesideTheLoop(journal, board); // returns at once
//! ```

use crate::lazy_runtime::spawn_on_runtime;
use crate::service_busy::LandingBoard;
use crate::service_journal::{DONE_RETAINED, prune_done};
use std::path::PathBuf;
use std::sync::Arc;

/// The name the journal and a waiting client know this step by.
pub const PRUNE_STEP: &str = "it removes the records of old finished batches";

/// What:
///  Removes the records of the finished batches past `DONE_RETAINED`,
///  beside the landing loop,
///  and says in the journal how many went.
/// Why:
///   The start's own comment said a request that arrives meanwhile waits in the queue,
/// and this is one of the steps that had no reason to be waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pruneOldRecords(journal, board) // returns at once; the removal runs beside the loop
/// ```
pub fn prune_old_records_beside_the_loop(journal: PathBuf, board: Arc<LandingBoard>) {
    let started: Option<tokio::task::JoinHandle<()>> = spawn_on_runtime(prune(journal.clone(), Arc::clone(&board)));
    if started.is_none() {
        crate::service_warn!("this service has no multi-threaded runtime, so the records of old finished batches are removed where the loop stands");
        prune_old_records_where_the_loop_stands(journal, board);
    }
}

/// What:
///  Removes the records of the finished batches past `DONE_RETAINED` where it is called,
///  which is the start's own thread for a service whose runtime cannot take a task.
/// Why:
///   The removal and its report are one function, so a service that removes the records on the start's thread and one
/// that removes them beside the loop differ in nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pruneOldRecordsWhereTheLoopStands(journal, board)
/// ```
pub fn prune_old_records_where_the_loop_stands(journal: PathBuf, board: Arc<LandingBoard>) {
    let started = board.begin_starting(PRUNE_STEP);
    let removed: usize = prune_done(journal.as_path(), DONE_RETAINED, &crate::claim_time::ClaimTime::read());
    drop(started);
    if removed > 0 {
        crate::service_log!("removed the records of {removed} old finished batches from {}", journal.display());
    }
}

/// One removal of old records,
/// on a job of the blocking pool,
/// with a line when it could not run.
async fn prune(journal: PathBuf, board: Arc<LandingBoard>) {
    let removal = crate::blocking_job::run_blocking_job(move || { return prune_old_records_where_the_loop_stands(journal, board); }).await;
    if let Err(error) = removal {
        crate::service_warn!("the records of old finished batches could not be removed: {error}");
    }
}

/// The removal and its line are controlled in the test module.
#[cfg(test)]
#[path = "service_prune_old_records_beside_the_loop_tests.rs"]
mod tests;
