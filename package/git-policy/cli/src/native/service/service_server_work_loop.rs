//! What:
//!  The loop of the commit service's after-commit work asked by a caller outside the runtime:
//! it waits for wakes and due retries,
//! runs a round of repository work for them,
//!  and keeps the service from exiting idle while a repository waits to be looked at again.
//! Why:
//!  The body is `work_loop_awaiting`
//! (`service_server_work_loop_awaiting.rs`),
//! which the service starts as a task of the runtime,
//! so a
//!  loop cannot run differently depending on which face asked it.
//! This face is for a caller on a thread of its own:
//! it enters the runtime once for the whole loop,
//!  and a call from inside the runtime is refused there,
//! which ends the loop with the reason in the service log
//! instead of
//!  holding a worker for the life of the service.
//! The loop that lands commits (`serve` in `service_server.rs`) and this loop run side by side;
//! separate files keep each
//!  one readable,
//! and this one does not use the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const dir of wakesOrDueRetries) await drain(dir, handlers)
//! ```

use crate::push_retry::PushRetries;
use crate::service_claim_watch::ClaimWatch;
use crate::service_server_work_loop_awaiting::work_loop_awaiting;
use crate::service_work::{GitHandlers, Wake};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::sync::mpsc::UnboundedReceiver;

/// What:
///  Runs the after-commit work of each repository named,
/// entering the runtime once for it.
/// Why:
///   A caller on a thread of its own cannot await,
/// and the loop is awaited work:
/// its waits are the runtime's own,
///  its rounds are lanes of tasks,
/// and its storage is on the blocking pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(workLoopAwaiting(receiver, handlers, busy, retries, watch))
/// ```
pub fn work_loop(receiver: &mut UnboundedReceiver<Wake>, handlers: &GitHandlers, busy: &Arc<AtomicUsize>, push_retries: PushRetries, watch: ClaimWatch) {
    if let Err(refusal) = crate::lazy_runtime::run_on_runtime(work_loop_awaiting(receiver, handlers, busy, push_retries, watch)) {
        crate::service_error!("the after-commit work of this service did not run: {refusal}");
    }
}
