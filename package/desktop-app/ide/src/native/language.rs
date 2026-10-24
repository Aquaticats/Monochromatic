//! Native language navigation:
//!  go to definition,
//!  references,
//!  and hover through the Language module.
//!
//! This module owns the window's one `LanguageWorker`.
//!  A 20 ms timer keeps the worker told about
//! the displayed file,
//!  polls status,
//!  replies,
//!  and snapshots,
//!  and applies a reply only while it
//! still answers the request it was sent for and describes the displayed text.
//!  Accepted inlay hints
//! and diagnostics go into `State::annotations`,
//!  the library's `ide_app::annotation::Annotations`,
//! which the source renderer reads with the stamp of the text it draws;
//!  this module does not draw them.

/// One source owner,
///  one window,
///  and the navigation state that opens other files.
use super::{AppWindow, State, navigation::Navigation};
/// What:
///  `anyhow::Result<T>` is a value or an error with a readable chain of causes.
/// Why:
///  The worker may fail to start;
///  the window still opens and explains it when asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Result<T> = T; // a failure is a thrown Error
/// ```
use anyhow::Result;
/// The handle,
///  the identities results carry,
///  and the states and replies it reports.
use ide_app::language::{
    LanguageWorker,
    identity::{DocumentStamp, ServerIdentity},
    reply::{RequestKind, RequestOutcome},
    status::LanguageStatus,
};
/// Weak window handles and a retained repeating timer.
use slint::{ComponentHandle, Timer, TimerMode};
/// What:
///  `Rc<RefCell<T>>` is one shared,
///  run-time borrow-checked owner on this thread;
///  `Arc` is a
///       thread-safe shared pointer;
///  `Path` and `PathBuf` are borrowed and owned paths;
///       `Duration` and `Instant` are a time span and a point on a monotonic clock.
/// Why:
///  The timer and every callback change the same language state on the interface thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const language = { current: createLanguage() };
/// ```
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

/// Window callbacks:
///  keys,
///  pointer,
///  list choices,
///  and dismissal.
mod actions;
/// The native checks every reply passes before it is applied.
mod guard;
/// Hover content as plain text.
mod hover_text;
/// Sentences for requests that cannot be satisfied.
mod message;
/// What a finished request does:
///  navigate,
///  list,
///  show,
///  or explain.
mod outcome;
/// The resting pointer and the character under it.
mod pointer;
/// Polling:
///  status,
///  replies,
///  snapshots,
///  and due requests.
mod poll;
/// The popup and the location list:
///  showing and dismissing.
mod surface;
/// Telling the worker about the displayed file and its visible lines.
mod sync;
/// Places a target names,
///  and opening them.
mod targets;

/// The file-open path places the caret at a target once its file is shown.
pub(super) use targets::{Jump, place};

/// Server hints and diagnostics painted by the poll,
///  and the problem card yielding to the popup.
#[cfg(test)]
mod annotation_wiring_tests;
/// Ctrl+B,
///  Ctrl+click,
///  and the references fallback through real window events.
#[cfg(test)]
mod definition_tests;
/// Reply admission rules on fabricated replies.
#[cfg(test)]
mod guard_tests;
/// A view whose hint request would ask only lines that stored answers already asked about sends none.
#[cfg(test)]
mod hint_skip_tests;
/// Hover by key and pointer,
///  its placement,
///  and every dismissal.
#[cfg(test)]
mod hover_tests;
/// Message sentences and hover text rendering without a window.
#[cfg(test)]
mod message_tests;
/// Server states,
///  stale replies,
///  and closing while a request is pending.
#[cfg(test)]
mod state_tests;
/// A complete reader with the scripted language server,
///  shared by the window tests.
#[cfg(test)]
mod test_support;
/// Waiting for timers and for the scripted server,
///  shared by the window tests.
#[cfg(test)]
mod test_waits;
/// An external change reaches a server through the application's relay to the change watcher.
#[cfg(test)]
mod watched_tests;

/// What:
///  The user action a request serves.
///  A plain `enum` is a closed set of names.
/// Why:
///  The same reply leads to different results:
///  Ctrl+B falls back to references,
///  a
///      Ctrl+click does not,
///  and a resting pointer never shows a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Action = 'definition' | 'pointerDefinition' | 'references' | 'hover' | 'pointerHover';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    /// Ctrl+B at the caret.
    Definition,
    /// Ctrl+click on a character.
    PointerDefinition,
    /// Ctrl+B at a definition:
    ///  its references.
    References,
    /// Ctrl+Q at the caret.
    Hover,
    /// The pointer rested on a character.
    PointerHover,
}

/// Request kind and wording per action.
impl Action {
    /// The Language module request this action sends.
    pub(super) fn kind(self) -> RequestKind {
        return match self {
            Self::Definition | Self::PointerDefinition => RequestKind::Definition,
            Self::References => RequestKind::References,
            Self::Hover | Self::PointerHover => RequestKind::Hover,
        };
    }

    /// True for an action the user asked for;
    ///  only those explain a failure.
    pub(super) fn explicit(self) -> bool {
        return self != Self::PointerHover;
    }

    /// The input that repeats this action,
    ///  as a message names it.
    pub(super) fn key(self) -> &'static str {
        return match self {
            Self::Definition | Self::References => "Ctrl+B",
            Self::PointerDefinition => "Ctrl+click",
            Self::Hover | Self::PointerHover => "Ctrl+Q",
        };
    }
}

/// What:
///  One request the window waits for.
///  `Option<u64>` is the request number,
///  or nothing
///       before the worker accepted it;
///  the list holds each server's answer with its name.
/// Why:
///  Replies are matched by number and stamp,
///  and a request with several servers finishes
///      only when the last of them answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pending = { action: Action; stamp: DocumentStamp; position: number; number?: number;
///                  outcomes: [ServerIdentity | undefined, RequestOutcome][]; complete: boolean };
/// ```
#[derive(Clone, Debug)]
pub(super) struct Pending {
    /// What the user did.
    pub(super) action: Action,
    /// Displayed file generation and revision the position belongs to.
    pub(super) stamp: DocumentStamp,
    /// Character offset asked about.
    pub(super) position: usize,
    /// Number the worker's replies carry;
    ///  nothing until the request was queued.
    pub(super) number: Option<u64>,
    /// Every accepted answer so far.
    pub(super) outcomes: Vec<(Option<ServerIdentity>, RequestOutcome)>,
    /// The last expected answer arrived.
    pub(super) complete: bool,
}

/// What the window's language state holds between ticks.
pub(super) struct Language {
    /// The handle;
    ///  nothing after it stopped or after the window closed.
    worker: Option<LanguageWorker>,
    /// Why language support is unavailable,
    ///  once it is.
    failure: Option<String>,
    /// Canonical project root,
    ///  for location labels.
    root: PathBuf,
    /// Displayed text the worker was last told about.
    synced: Option<DocumentStamp>,
    /// An explicit action asked to display the file again,
    ///  so the worker re-resolves its servers.
    reopen: bool,
    /// Latest status for the displayed file.
    status: Arc<LanguageStatus>,
    /// The request of the latest key or Ctrl+click action.
    action: Option<Pending>,
    /// The request of the resting pointer,
    ///  kept apart so hovering never replaces an action.
    hover: Option<Pending>,
    /// What is on screen.
    shown: surface::Shown,
    /// The resting pointer.
    rest: pointer::Rest,
    /// The file generation for which a resting pointer already explained why hover is unavailable;
    ///  the explanation is shown once for each displayed file.
    explained: Option<u64>,
    /// The visible lines reported for inlay hints.
    hints: sync::HintLines,
}

/// The running binding:
///  the timer and the shared state,
///  closed explicitly at window shutdown.
pub(super) struct LanguageBinding {
    /// Polls every 20 ms while it lives.
    timer: Timer,
    /// Shared with the timer and the window callbacks.
    language: Rc<RefCell<Language>>,
}

/// Shutdown.
impl LanguageBinding {
    /// What:
    ///  Stop polling and drop the worker,
    ///  which stops every server and waits up to about a second.
    /// Why:
    ///  Called after the window closed,
    ///  so the wait never freezes a visible window.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// close() { timer.stop(); language.current.worker?.dispose(); }
    /// ```
    pub(super) fn close(self) {
        self.timer.stop();
        let started = Instant::now();
        // `take` moves the handle out of the state; dropping it joins the worker thread.
        let worker = self.language.borrow_mut().worker.take();
        drop(worker);
        tracing::info!(elapsed = ?started.elapsed(), "language servers stopped at window close");
    }
}

/// What:
///  Bind the language callbacks and start polling.
///  `worker` is the started handle or the
///       reason it could not start.
/// Why:
///  A worker that failed to start leaves every other feature working;
///  the reason is shown
///      when the user asks for a language feature.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bind(window, source, navigation, root: string, worker: LanguageWorker | Error): LanguageBinding
/// ```
pub(super) fn bind(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    root: &Path,
    worker: Result<LanguageWorker>,
) -> LanguageBinding {
    // What: `match` unpacks the started handle (`Ok`) or the start error (`Err`).
    // Why: Both are valid starting points; only the second records a failure to explain later.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [handle, failure] = worker instanceof Error ? [undefined, worker.message] : [worker, undefined];
    // ```
    let (handle, failure) = match worker {
        Ok(started) => (Some(started), None),
        Err(error) => {
            tracing::error!(?error, "language support is unavailable for this window");
            (None, Some(format!("{error:#}")))
        }
    };
    let language = Rc::new(RefCell::new(Language {
        worker: handle,
        failure,
        root: root.to_path_buf(),
        synced: None,
        reopen: false,
        status: Arc::new(LanguageStatus::closed()),
        action: None,
        hover: None,
        shown: surface::Shown::Nothing,
        rest: pointer::Rest::default(),
        explained: None,
        hints: sync::HintLines::default(),
    }));
    actions::bind(window, source, navigation, &language);
    let timer = Timer::default();
    let tick_language = Rc::clone(&language);
    let tick_source = Rc::clone(source);
    let tick_navigation = Rc::clone(navigation);
    let tick_window = window.as_weak();
    timer.start(TimerMode::Repeated, Duration::from_millis(20), move || {
        let Some(active) = tick_window.upgrade() else {
            return;
        };
        // A callback that is changing the state right now is finished first; the next tick polls.
        let Ok(mut current) = tick_language.try_borrow_mut() else {
            tracing::debug!("skipped a language tick while a callback held the state");
            return;
        };
        poll::tick(&active, &tick_source, &tick_navigation, &mut current);
    });
    return LanguageBinding { timer, language };
}
