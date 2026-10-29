//! The conditions a step waits for:
//!  each one reads the session after a poll
//!  and says whether the step may stop waiting.

/// The session whose polled state the conditions read.
use super::Session;
/// The stamp of the displayed text,
///  which latest-value snapshots must carry.
use ide_app::language::identity::DocumentStamp;
/// A server's state is compared with `Ready`.
use ide_app::language::status::ServerState;

/// What:
///  Something a step waits for.
///  A type that implements this trait holds the values it compares with,
///  and `holds` reads the session once per poll.
/// Why:
///  Every step names its own condition,
///  and a named type carries the values a step compares with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Condition { holds(session: Session): boolean }
/// ```
pub(super) trait Condition {
    /// Whether the session is in the state the step waits for.
    fn holds(&self, session: &Session) -> bool;
}

/// The last server answered the request with this number.
pub(super) struct Answered {
    /// The number the worker gave the request.
    pub(super) request: u64,
}

/// The condition reads the polled session.
impl Condition for Answered {
    /// The last server answered the request.
    fn holds(&self, session: &Session) -> bool {
        for reply in session.replies.iter() {
            if reply.request == self.request && reply.remaining == 0 {
                return true;
            }
        }
        return false;
    }
}

/// Some server is ready.
pub(super) struct ServerReady;

/// The condition reads the polled session.
impl Condition for ServerReady {
    /// Some server is ready.
    fn holds(&self, session: &Session) -> bool {
        for row in session.status.servers.iter() {
            if row.state == ServerState::Ready {
                return true;
            }
        }
        return false;
    }
}

/// The held hints describe the displayed revision and number at least `least`.
pub(super) struct HintsEnough {
    /// The stamp of the displayed text.
    pub(super) stamp: DocumentStamp,
    /// The fewest hints that count.
    pub(super) least: usize,
}

/// The condition reads the polled session.
impl Condition for HintsEnough {
    /// Enough hints are held for the displayed revision.
    fn holds(&self, session: &Session) -> bool {
        return match session.hints.as_ref() {
            Some(hints) => hints.stamp == self.stamp && hints.hints.len() >= self.least,
            None => false,
        };
    }
}

/// The held diagnostics describe the displayed revision
///  and number between `least` and `most`.
pub(super) struct DiagnosticsWithin {
    /// The stamp of the displayed text.
    pub(super) stamp: DocumentStamp,
    /// The fewest diagnostics that count.
    pub(super) least: usize,
    /// The most diagnostics that count.
    pub(super) most: usize,
}

/// The condition reads the polled session.
impl Condition for DiagnosticsWithin {
    /// The diagnostics count is within the bounds.
    fn holds(&self, session: &Session) -> bool {
        let Some(found) = session.diagnostics.as_ref() else {
            return false;
        };
        let mut count: usize = 0;
        for group in found.groups.iter() {
            count += group.items.len();
        }
        return found.stamp == self.stamp && count >= self.least && count <= self.most;
    }
}

/// The condition that is never met,
///  so a step waits for its whole limit.
pub(super) struct Never;

/// The condition reads the polled session.
impl Condition for Never {
    /// Never holds.
    fn holds(&self, _session: &Session) -> bool {
        return false;
    }
}

/// At least `minimum` folders are watched for the servers.
pub(super) struct FoldersWatched {
    /// The fewest folders that count.
    pub(super) minimum: usize,
}

/// The condition reads the polled session.
impl Condition for FoldersWatched {
    /// Enough folders are watched.
    fn holds(&self, session: &Session) -> bool {
        return match session.watcher.as_ref() {
            Some(watcher) => watcher.server_folders() >= self.minimum,
            None => false,
        };
    }
}
