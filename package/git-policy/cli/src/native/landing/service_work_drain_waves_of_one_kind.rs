//! What:
//!  The bodies a wave of one kind asks of its handler:
//!  the relink of a new worktree's package state,
//!  the prewarm of its build directory,
//!  the emptying of a trash,
//! and the review of a landed commit.
//! Why:
//!  A wave asks each item for a future it may poll beside its siblings,
//! and a named function is a body that outlives the poll that started it
//!  and captures nothing of the caller.
//! A body whose answer is `Done` writes the one line saying how long it took,
//! which is what a person reads when one kind is slow.
//! The four bodies sit in a file of their own
//!  because the drain that starts their waves is a file's worth of steps without them
//! (`service_work_drain_pending_claims.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const askRelink = (handlers, item) => async () => { const t0 = now(); const v = await handlers.relink(item); if (v.done) log(t0); return v }
//! ```

use crate::service_work_handlers_replanning_and_acceptance::{Verdict, WorkHandlers};
use crate::work_item::WorkItem;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

/// What:
///  Ask the relink of one item and say how long it took,
///  as the wave of relinks asks it.
/// Why:
///  A named function rather than a closure,
/// because what a wave's item is asked for is a body that outlives the poll
/// that started it and captures nothing of the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function askRelink(handlers, item) { const t0 = now(); const v = await handlers.relink(item); if (v.done) log(t0); return v }
/// ```
pub(super) fn ask_relink<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move {
        let relinking: Instant = Instant::now();
        let verdict: Verdict = handlers.relink_awaiting(item).await;
        if matches!(verdict, Verdict::Done) {
            crate::service_log!("the package state of {} was relinked relink_ms={}", item.repository.display(), relinking.elapsed().as_millis());
        }
        return verdict;
    });
}

/// The prewarm of one item,
/// as its wave asks it.
pub(super) fn ask_prewarm<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move { return handlers.prewarm_awaiting(item).await; });
}

/// The emptying of one trash,
/// as its wave asks it.
pub(super) fn ask_trash<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move { return handlers.trash_awaiting(item).await; });
}

/// The review of one item and the line that says how long it took,
/// as its wave asks it.
pub(super) fn ask_review<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move {
        let review_began: Instant = Instant::now();
        let verdict: Verdict = handlers.review_awaiting(item).await;
        if matches!(verdict, Verdict::Done) {
            crate::service_log!("the review of {} is done review_ms={}", item.oid, review_began.elapsed().as_millis());
        }
        return verdict;
    });
}
