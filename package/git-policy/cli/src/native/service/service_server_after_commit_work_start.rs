//! What:
//!  The start of the after-commit work,
//!  the task that pushes the landed commits and keeps their records,
//!  and the end of the service when no thread can be had for it.
//! Why:
//!   The after-commit work runs as a task of the process's runtime,
//!   so a slow network never delays a landing.
//!  A service that goes on without that task never pushes the person's commits,
//!  while an ended service is started again by the manager on the next connection,
//!  so a build that gets no worker ends the service with a plain line and status 4.
//!  A build that merely outlasted its deadline is waited for,
//!  because the runtime is kept for the work it was built for.
//!  The plan and the watch those tasks start with are read before this call,
//!  by a thread of their own,
//!  so a journal that stopped answering delays them and never the greeting of a client.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const task = runtime.spawn(() => workLoopAwaiting(wakes, handlers, busy, retries, watch));
//! if (task === undefined) { log(noWorkerWords('the after-commit work')); process.exit(4); }
//! ```
use crate::push_retry::PushRetries;
use crate::service_claim_watch::ClaimWatch;
use crate::service_runtime_build_report::{
    EXIT_NO_WORKER,
    no_worker_words,
    start_what_the_service_cannot_serve_without,
};
use crate::service_server_work_loop_awaiting::work_loop_awaiting;
use crate::service_work::{
    GitHandlers,
    Wake,
};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::sync::mpsc::UnboundedReceiver;

/// Starts the after-commit work as a task of the process's runtime,
/// waiting `RUNTIME_BUILD_PATIENCE` for a build that missed its deadline,
/// or ends the service with a plain line and status 4:
/// a service that goes on
///  without its after-commit work never pushes the person's commits,
/// while an ended service is started again by the manager.
/// A build that merely outlasted `BUILD_DEADLINE` is waited for,
/// because it is kept for the work it was built for.
/// The plan and the watch are read before this call,
///  by a thread of their own (`service_journal_restored_at_start.rs`),
/// so the task starts with what the service that stopped last left and no storage wait of
///  that reading holds a worker.
pub fn start_the_after_commit_work_or_exit(
    receiver: UnboundedReceiver<Wake>,
    handlers: GitHandlers,
    busy: Arc<AtomicUsize>,
    push_retries: PushRetries,
    watch: ClaimWatch,
) {
    let work = async move {
        let mut wakes: UnboundedReceiver<Wake> = receiver;
        work_loop_awaiting(&mut wakes, &handlers, &busy, push_retries, watch).await;
    };
    if start_what_the_service_cannot_serve_without("the after-commit work", work).is_none() {
        crate::service_error!("{}", no_worker_words("the after-commit work"));
        std::process::exit(EXIT_NO_WORKER);
    }
}
