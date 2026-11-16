//! What:
//!  How many threads this process started through `spawn_named` have not ended,
//!  and the value each started thread holds while it runs.
//! Why:
//!  A job `spawn_named` starts belongs to no caller:
//!  the service starts one beside a landing and returns to its loop
//!  (`service_server_land_group.rs`,
//!  the pool fill,
//!  the repository scratch collection,
//!  and the draw),
//!  so the job writes into the repository the landing was given after the call that started it returned.
//!  A control that removes that repository's fixture therefore removed it under a writer still writing,
//!  and met `DirectoryNotEmpty`
//!  (`landing::service_land::owed_push_tests::a_landed_commit_holds_its_push_where_no_round_runs_it`,
//!  the gate of 2026-11-14).
//!  The count is what a fixture removal waits on (`remove` in `testing/test_support.rs`),
//!  and a thread lowers it after its last write,
//!  so a removal that has seen the count reach zero meets no job started through `spawn_named`
//!  that was still running when it looked.
//!  What the count does not cover,
//! and no fixture removal waits for:
//!  work a job hands to a task of the runtime,
//!  the note a landing writes on a thread of its own
//!  (`service_claim_watch_note_within_a_ceiling.rs`,
//!  which may still write when its ceiling passed),
//!  a child process a control starts,
//!  and a new job started while the removal is under way.
//!  It counts one process's threads,
//! so it is the count of one test only while one test runs in the process:
//!  this repository's suite runs one test per process (`cargo nextest`),
//!  and a threaded run of the same tests in one process would have one test wait for another's jobs.
//!  Kept in a file of its own so the shipped `scoped_start.rs` holds no inline test module (`rust/tests-in-sidecars`),
//!  and it exists only in a test build.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const counted = begun(); // a thread holds this until it ends
//! running(); // how many started threads have not ended
//! ```

use std::sync::Mutex;
use std::sync::PoisonError;

/// Lock kept:
///  how many threads this process started through `spawn_named` have not ended,
///  which a started thread raises before it runs and lowers after its last write,
///  and which a wait reads;
///  nothing else touches it,
///  and it is one word behind a mutex a thread holds for the two instructions that change it.
static RUNNING: Mutex<usize> = Mutex::new(0);

/// What:
///  One started thread,
/// counted from the moment it starts until this value is dropped.
/// Why:
///  The value is built before the thread starts and carried into it,
///  so a start the system refuses drops it on the spot and leaves no count behind,
///  and the drop runs after the job's last write because it is the last thing the thread's body holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using counted = begun(); // lowered when this scope ends
/// ```
pub(crate) struct StartedThread(());

impl Drop for StartedThread {
    fn drop(&mut self) {
        let mut running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
        *running = running.saturating_sub(1);
    }
}

/// What:
///  Count one thread that is about to start,
///  and answer the value it holds until it ends.
/// Why:
///   A thread counted before it starts is counted for everything it does,
///  which is what a fixture removal needs:
///  the thread writes nothing after its value is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const counted = begun();
/// ```
pub(crate) fn begun() -> StartedThread {
    let mut running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
    *running += 1;
    return StartedThread(());
}

/// How many threads this process started through `spawn_named` have not ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// running(); // 0 when nothing this process started is still writing
/// ```
pub(crate) fn running() -> usize {
    return *RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
}

/// The count and its control are exercised through the fixture removal a control calls.
#[path = "scoped_start_started_threads_test_support_tests.rs"]
mod tests;
