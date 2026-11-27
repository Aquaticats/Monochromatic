//! The requests the worker makes on its own:
//!  inlay hints for the lines the interface last reported,
//!  pull diagnostics,
//!  and asking again when a server that owed an answer sends anything.

/// The hint window the interface reported,
///  and the lines a request asks for.
use super::super::hints::{
    HintWindow,
    request_lines,
};
/// The revision an answer is stamped with.
use super::super::identity::DocumentStamp;
/// The session whose document and owed answers these steps read.
use super::super::session::Session;
/// The worker whose state these steps change.
use super::super::worker::Worker;
/// What a ticket asks for,
///  and what became of the one before.
use super::{
    Ask,
    Sent,
    dispatch,
};

/// What:
///  Ask one server for diagnostics of the displayed document,
///  if it offers pull diagnostics.
///       `matches!` is true when the outcome is the "request is on its way" variant.
/// Why:
///  The outcome needs no reply:
///  an unsupported or unready server simply contributes nothing.
///      Once a request is on its way,
///  the server owes nothing until that request ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pullDiagnostics(worker: Worker, index: number, attempt: number): void {
///   if (dispatch(worker, index, 0, { kind: 'pull' }, 0, attempt) === 'spawned') worker.session.servers[index].owed.pull = false;
/// }
/// ```
pub(in crate::language) fn pull_diagnostics(
    worker: &mut Worker,
    index: usize,
    attempt: u32,
) {
    if matches!(
        dispatch(worker, index, 0, Ask::Pull, 0, attempt),
        Sent::Spawned
    ) {
        worker.session.servers[index].owed.pull = false;
    }
}

/// What:
///  The hint request for the last reported window against the displayed text,
///  or nothing
///       when no file is displayed or no window was reported.
///  `&Session` lends the session
///       read-only;
///  `Option<Ask>` is "a request,
///  or nothing".
/// Why:
///  First asks,
///  retries,
///  and catch-up asks all name the lines wanted now,
///  never the lines an
///      earlier request named,
///  so a request sent again after a wait cannot bring back hints for
///      lines the reader has scrolled away from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hintAsk(session: Session): Ask | undefined {
///   const window = session.document?.window;
///   if (!window) return undefined;
///   const [firstLine, lastLine] = requestLines(window, session.document.text.lineCount);
///   return { kind: 'hints', firstLine, lastLine };
/// }
/// ```
pub(in crate::language) fn hint_ask(session: &Session) -> Option<Ask> {
    // The trailing `?` returns `None` when no document is displayed, or no window was reported.
    let document = session.document.as_ref()?;
    let window = document.window?;
    let (first_line, last_line) = request_lines(window, document.text.len_lines());
    return Some(Ask::Hints {
        first_line,
        last_line,
    });
}

/// Ask one server for hints of the last reported window,
///  if any window was reported.
pub(in crate::language) fn hints(
    worker: &mut Worker,
    index: usize,
    attempt: u32,
) {
    let Some(ask) = hint_ask(&worker.session) else {
        return;
    };
    // A request on its way means the server owes nothing until that request ends.
    if matches!(dispatch(worker, index, 0, ask, 0, attempt), Sent::Spawned) {
        worker.session.servers[index].owed.hints = false;
    }
}

/// What:
///  The server at `index` just sent something:
///  ask it again for the hints and pull
///       diagnostics it still owes for the displayed text,
///  if the catch-up rule allows.
/// Why:
///  A request that stayed unanswered through its retries is not repeated on a timer.
///  A
///      message of any kind shows that the server processes its input again,
///  which is the
///      moment a new request can succeed.
///  `Owed::catch_up` bounds how often this happens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function catchUp(worker: Worker, index: number): void {
///   const owed = worker.session.servers[index].owed.catchUp();
///   if (!owed) return;
///   if (owed.pull) pullDiagnostics(worker, index, 0);
///   if (owed.hints) hints(worker, index, 0);
/// }
/// ```
pub(in crate::language) fn catch_up(
    worker: &mut Worker,
    index: usize,
) {
    let Some(owed) = worker.session.servers[index].owed.catch_up() else {
        return;
    };
    tracing::debug!(server = %worker.session.servers[index].identity.name, ?owed, "the server sent a message while it owed answers; asking again");
    if owed.pull {
        pull_diagnostics(worker, index, 0);
    }
    if owed.hints {
        hints(worker, index, 0);
    }
}

/// What:
///  The interface reported the visible lines:
///  remember them and ask every server for hints.
/// Why:
///  The window is kept so hints can be asked for again after a reload and after a late start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function window(worker: Worker, stamp: DocumentStamp, window: HintWindow): void
/// ```
pub(in crate::language) fn window(
    worker: &mut Worker,
    stamp: DocumentStamp,
    window: HintWindow,
) {
    let Some(document) = worker.session.document.as_mut() else {
        return;
    };
    if document.stamp != stamp {
        tracing::debug!(
            ?stamp,
            "dropped a hint window for text that is no longer displayed"
        );
        return;
    }
    document.window = Some(window);
    for index in 0..worker.session.servers.len() {
        if worker.session.servers[index].attached {
            hints(worker, index, 0);
        }
    }
}
