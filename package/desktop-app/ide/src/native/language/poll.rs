//! One language tick:
//!  synchronize,
//!  read status,
//!  send due requests,
//!  apply replies,
//!  store snapshots,
//! and repaint the source when a snapshot was stored.

/// The state,
///  the request record,
///  and the steps of a tick.
use super::{Language, guard, message, outcome, rest_step, surface, sync};
/// The window,
///  the source,
///  the navigation that opens other files,
///  and the source repaint.
use crate::native::{
    AppWindow, State,
    navigation::{Navigation, feed_servers},
    render,
};
/// The identity of the displayed text and the requests the worker takes.
use ide_app::language::{identity::DocumentStamp, reply::PositionRequest};
/// What:
///  `Rc<RefCell<T>>` is the window's shared,
///  borrow-checked state.
/// Why:
///  A tick reads the displayed document and changes the popup,
///  the list,
///  and the caret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::RefCell, rc::Rc};

/// What:
///  Turn language support off after the worker stopped,
///  keeping the reason for later notes.
/// Why:
///  A stopped worker must never take the window down;
///  a waiting explicit action is
///      answered with the reason at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stop(window, source, language, error: Error): void
/// ```
pub(super) fn stop(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    language: &mut Language,
    error: &anyhow::Error,
) {
    tracing::error!(?error, "language support stopped");
    let reason = format!("{error:#}");
    language.worker = None;
    language.failure = Some(reason.clone());
    language.hover = None;
    source.borrow_mut().language_reload = None;
    // `take` moves the waiting action out; it is answered with the reason.
    if let Some(pending) = language.action.take()
        && pending.action.explicit()
    {
        language.shown = surface::popup(
            window,
            source,
            pending.position,
            &message::stopped(&reason),
            (true, false),
        );
    }
}

/// What:
///  Send a request that is not queued yet.
///  `Ok(())` also when the queue was full;
///  it is
///       sent again next tick.
/// Why:
///  Requests go out only for the text the worker holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function send(language: Language, slot: Pending | undefined, displayed: DocumentStamp): void
/// ```
fn send(language: &mut Language, hover: bool, displayed: DocumentStamp) -> anyhow::Result<()> {
    if language.synced != Some(displayed) {
        return Ok(());
    }
    let Some(worker) = language.worker.as_mut() else {
        return Ok(());
    };
    // `as_mut` lends the chosen slot's request for modification.
    let slot = if hover {
        language.hover.as_mut()
    } else {
        language.action.as_mut()
    };
    let Some(pending) = slot else {
        return Ok(());
    };
    if pending.number.is_some() || pending.stamp != displayed {
        return Ok(());
    }
    let number = worker.request(PositionRequest {
        stamp: pending.stamp,
        kind: pending.action.kind(),
        position: pending.position,
    })?;
    if let Some(sent) = number {
        tracing::debug!(action = ?pending.action, number = sent, position = pending.position, "sent a language request");
    }
    pending.number = number;
    return Ok(());
}

/// What:
///  Drop a waiting request whose text is no longer displayed.
/// Why:
///  The worker drops it too and never replies;
///  a reload of the same file is explained,
///      because the user is still looking at the place they asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cancelStale(window, source, language, displayed: DocumentStamp): void
/// ```
fn cancel_stale(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    language: &mut Language,
    displayed: DocumentStamp,
) {
    let hover_stale = match language.hover.as_ref() {
        Some(pending) => pending.stamp != displayed,
        None => false,
    };
    if hover_stale {
        language.hover = None;
    }
    let action_stale = match language.action.as_ref() {
        Some(pending) => pending.stamp != displayed,
        None => false,
    };
    let dropped = if action_stale {
        language.action.take()
    } else {
        None
    };
    let Some(pending) = dropped else {
        return;
    };
    tracing::debug!(action = ?pending.action, ?displayed, "dropped a language request for text no longer displayed");
    if pending.stamp.file == displayed.file && pending.action.explicit() {
        let sentence = message::changed(pending.action);
        let position = source.borrow().document.position().head;
        language.shown = surface::popup(window, source, position, &sentence, (true, false));
    }
}

/// What:
///  Read every reply now waiting and attach the admitted ones to their requests.
/// Why:
///  All replies are drained before any request is finished,
///  because answers needing no
///      server traffic arrive together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replies(language: Language, displayed: DocumentStamp): void
/// ```
fn replies(language: &mut Language, displayed: DocumentStamp) -> anyhow::Result<()> {
    loop {
        let Some(worker) = language.worker.as_mut() else {
            return Ok(());
        };
        let Some(reply) = worker.try_take_reply()? else {
            return Ok(());
        };
        let mut admitted = false;
        // `[...]` builds a fixed two-element array of the two slots, walked in order.
        for slot in [&mut language.action, &mut language.hover] {
            let Some(pending) = slot.as_mut() else {
                continue;
            };
            let verdict = guard::verdict(pending, displayed, &reply);
            if verdict != guard::Verdict::Apply {
                tracing::debug!(?verdict, request = reply.request, action = ?pending.action, "language reply not applied to this request");
                continue;
            }
            admitted = true;
            pending.complete = reply.remaining == 0;
            // `push` records the server's answer with the server that gave it.
            pending
                .outcomes
                .push((reply.server.clone(), reply.outcome.clone()));
        }
        if !admitted {
            tracing::debug!(request = reply.request, stamp = ?reply.stamp, "dropped a language reply no request waits for");
        }
    }
}

/// Finish whichever requests are complete.
fn finish(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &mut Language,
) {
    // A request moves out only when its last answer arrived.
    let hover_complete = match language.hover.as_ref() {
        Some(pending) => pending.complete,
        None => false,
    };
    if hover_complete && let Some(pending) = language.hover.take() {
        outcome::finish(window, source, navigation, language, pending);
    }
    let action_complete = match language.action.as_ref() {
        Some(pending) => pending.complete,
        None => false,
    };
    if action_complete && let Some(pending) = language.action.take() {
        outcome::finish(window, source, navigation, language, pending);
    }
}

/// What:
///  Store new hints and diagnostics that describe the displayed text,
///  and answer whether
///       either was stored.
///  `anyhow::Result<bool>` is "yes or no,
///  or the worker's error".
/// Why:
///  Storing draws nothing:
///  the source renderer reads the store from `State` when it runs,
///      so the tick repaints once when this answers yes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshots(language: Language, source: Shared<State>, displayed: DocumentStamp): boolean
/// ```
fn snapshots(
    language: &mut Language,
    source: &Rc<RefCell<State>>,
    displayed: DocumentStamp,
) -> anyhow::Result<bool> {
    let Some(worker) = language.worker.as_mut() else {
        return Ok(false);
    };
    let hints = worker.try_take_hints()?;
    let diagnostics = worker.try_take_diagnostics()?;
    let mut current = source.borrow_mut();
    // What: Destructuring `&mut State` lends two fields separately.
    // Why: The store groups what it accepts by the lines of the displayed text, which is only read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { annotations, document } = source.current;
    // ```
    let State {
        annotations,
        document,
        ..
    } = &mut *current;
    // `mut` lets either branch record that the store now holds something new to paint.
    let mut accepted = false;
    if let Some(snapshot) = hints
        && annotations.accept_hints(displayed, document.text(), snapshot)
    {
        accepted = true;
        // The accessor the renderer uses reports what was stored.
        let count = match annotations.hints(displayed) {
            Some(stored) => stored.hints.len(),
            None => 0,
        };
        tracing::debug!(
            count,
            ?displayed,
            "stored inlay hints for the displayed text"
        );
    }
    if let Some(snapshot) = diagnostics
        && annotations.accept_diagnostics(displayed, document.text(), snapshot)
    {
        accepted = true;
        let groups = match annotations.diagnostics(displayed) {
            Some(stored) => stored.groups.len(),
            None => 0,
        };
        tracing::debug!(
            groups,
            ?displayed,
            "stored diagnostics for the displayed text"
        );
    }
    return Ok(accepted);
}

/// What:
///  Watch the project's source folders for the servers while one registered file watchers,
///  and stop
///       when none does or language support stopped.
///  `then` makes the sender only when it is wanted.
/// Why:
///  The change watcher sends changes straight to the worker;
///  this thread only says whether to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relayWatching(language, navigation) { feedServers(navigation, worker?.wantsFileChanges() ? worker.fileChangeSender() : undefined); }
/// ```
fn relay_watching(language: &Language, navigation: &Rc<RefCell<Navigation>>) {
    let mut feed = None;
    if let Some(worker) = language.worker.as_ref()
        && worker.wants_file_changes()
    {
        feed = Some(worker.file_change_sender());
    }
    feed_servers(&mut navigation.borrow_mut(), feed);
}

/// What:
///  The steps of one tick in order,
///  stopping at the first worker error;
///  `repaint` is set when new hints or diagnostics were stored.
/// Why:
///  `?` inside one function lets the first error end the steps and the caller turn support off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(window, source, navigation, language, displayed): { repaint: boolean } // throws
/// ```
fn steps(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &mut Language,
    displayed: DocumentStamp,
    repaint: &mut bool,
) -> anyhow::Result<()> {
    sync::update(language, source)?;
    // A status for an earlier file generation is ignored.
    if let Some(worker) = language.worker.as_mut()
        && let Some(status) = worker.try_take_status()?
        && status.file == Some(displayed.file)
    {
        language.status = status;
    }
    cancel_stale(window, source, language, displayed);
    rest_step::rest(window, source, language, displayed);
    send(language, false, displayed)?;
    send(language, true, displayed)?;
    replies(language, displayed)?;
    finish(window, source, navigation, language);
    // A references request made by a finished definition goes out in the same tick.
    send(language, false, displayed)?;
    *repaint = snapshots(language, source, displayed)?;
    if language.synced == Some(displayed) {
        sync::hints(language, window, source, displayed)?;
    }
    return Ok(());
}

/// What:
///  One tick.
///  Every worker error turns language support off for the rest of the session.
/// Why:
///  Order matters:
///  the worker is told about the displayed text before anything is asked,
///      and status is read before replies so a reply's explanation sees the newest states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tick(window, source, navigation, language): void
/// ```
pub(super) fn tick(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &mut Language,
) {
    relay_watching(language, navigation);
    if language.worker.is_none() {
        // Nothing will send the record; dropping it keeps the source state small.
        source.borrow_mut().language_reload = None;
        return;
    }
    let displayed = surface::displayed(&source.borrow());
    // Set by the steps when new hints or diagnostics were stored; read after them to repaint once.
    let mut repaint = false;
    // `?` inside the steps stops them at the first worker error.
    let stepped = steps(
        window,
        source,
        navigation,
        language,
        displayed,
        &mut repaint,
    );
    if let Err(error) = stepped {
        stop(window, source, language, &error);
        return;
    }
    // What: `render` rebuilds the source image and its overlays from `State`.
    // Why: Accepted hints and diagnostics are only stored by the steps; without this call they
    //      would stay invisible until some unrelated event repainted the source.
    //      No `State` borrow is held here, which `render` needs because it borrows mutably.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (repaint) render(window, source);
    // ```
    if repaint {
        render(window, source);
    }
    let state = source.borrow();
    let reason = surface::stale(window, &state, &language.shown, language.rest.over());
    drop(state);
    if let Some(why) = reason {
        language.shown = surface::dismiss(window, &language.shown, why);
    }
}
