//! What:
//!  What the client did to hand one request to the service and get its answer,
//!  as the commit's timing record states it.
//! Why:
//!  The queue wait,
//!  the wait for another program's hold on the index,
//!  and the landing itself all happen between the send and the answer;
//!  the service's own log line for the request splits that wait,
//!  and this record says how long the whole of it was.
//!  A service of an older release sends no figures,
//!  so the landing's own figures say they are absent rather than zero.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type SubmitTrace = { total: number; reach: ReachTrace; answerWait: number; positions: number[] };
//! ```

/// What the client did to reach a service that speaks this release,
/// with the waits that reaching cost.
use crate::commit_reach_trace::ReachTrace;
/// The landing figures the service answers with,
/// which this trace carries.
use crate::service_protocol::LandingFigures;
/// `Duration` is an elapsed stretch of time from the monotonic clock.
use std::time::Duration;

/// What:
///  What the client did to hand one request to the service and get its answer.
/// Why:
///   The queue wait,
/// the wait for a foreign index lock and the landing itself all happen between the send and the answer;
///  the service's own log line for the request splits that wait,
/// and this record says how long the whole of it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SubmitTrace = { total: number; reach: ReachTrace; answerWait: number; positions: number[]; waitingNotices: number; holderKind?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SubmitTrace {
    /// The whole of the sends,
    /// reaches included.
    pub total: Duration,
    /// How the service was reached.
    pub reach: ReachTrace,
    /// Time from a request being written to the answer (or the connection breaking),
    /// summed over the sends.
    pub answer_wait: Duration,
    /// The queue positions the service reported,
    /// in order,
    /// without repeats.
    pub positions: Vec<usize>,
    /// Notices the service sent while it waited for another program's hold on the index.
    pub waiting_notices: usize,
    /// What the last of those notices said of the holder:
    /// `proven-alive` when it named a process proven to hold the lock,
    /// `unproven` when it said the owner could not be proven alive.
    pub holder_kind: Option<&'static str>,
    /// Times the connection broke before an answer and the same request was sent again.
    pub reconnects: usize,
    /// Requests written.
    pub sends: usize,
    /// What the landing of this commit spent and held,
    /// as the service's answer stated it;
    /// `None` for a service of a release that sends no figures,
    /// so a reader never takes zeros for a figure nobody measured.
    pub landing: Option<LandingFigures>,
}
