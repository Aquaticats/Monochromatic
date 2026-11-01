//! The sending ends of the channels the interface thread polls.

/// Commands and one-shot replies.
use crate::language::reply::LanguageReply;
/// Latest-value results the worker publishes.
use crate::language::{
    diagnostics::DiagnosticsSnapshot, hints::HintsSnapshot, status::LanguageStatus,
};
/// `Arc` is a thread-safe shared pointer.
use std::sync::Arc;
/// `mpsc` is a queue,
///  and `watch` a single-slot latest-value cell.
use tokio::sync::{mpsc, watch};

/// The sending ends of every channel the interface thread polls.
pub(in crate::language) struct Outputs {
    /// One-shot replies to position requests.
    pub(in crate::language) replies: mpsc::Sender<LanguageReply>,
    /// Latest status.
    pub(in crate::language) status: watch::Sender<Arc<LanguageStatus>>,
    /// Latest diagnostics;
    ///  nothing while no file is displayed.
    pub(in crate::language) diagnostics: watch::Sender<Option<Arc<DiagnosticsSnapshot>>>,
    /// Latest hints;
    ///  nothing until a server answered for the displayed text.
    pub(in crate::language) hints: watch::Sender<Option<Arc<HintsSnapshot>>>,
    /// Whether some server registered file watchers,
    ///  so the project's folders are worth watching.
    pub(in crate::language) watching: watch::Sender<bool>,
}

/// Publishing.
impl Outputs {
    /// What:
    ///  Queue one reply without waiting.
    ///  `try_send` returns `Err` when the queue is full or
    ///       the handle is gone.
    /// Why:
    ///  The worker must never block on an interface thread that stopped polling.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reply(reply: LanguageReply) { if (!queue.offer(reply)) log.warn('dropped'); }
    /// ```
    pub(in crate::language) fn reply(&self, reply: LanguageReply) {
        if let Err(error) = self.replies.try_send(reply) {
            tracing::warn!(%error, "dropped a language reply because the reply queue is full or closed");
        }
    }
}
