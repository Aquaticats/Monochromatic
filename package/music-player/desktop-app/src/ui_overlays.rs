//! Window binding for the feedback messages and the first-run pane.
//!
//! What:
//! `wire` registers the feedback dismiss and action callbacks and the first-run actions,
//! and starts
//! the expiry timer;
//! `show_error` and `show_undo` add messages from other modules.
//! Why:
//! the feedback queue is a pure value (`feedback.rs`);
//! this module owns the one copy and its timer.

/// Imports the cell type that holds the overlay state on the window thread.
use std::cell::RefCell;
/// Imports the path type that names a trashed file.
use std::path::PathBuf;
/// Imports the shared ownership type that shares the engine with the callbacks.
use std::rc::Rc;
/// Imports the duration and instant types that time the message expiry.
use std::time::{Duration, Instant};

/// Imports the engine that receives the commands.
use music_player::engine::Engine;
/// Imports the feedback kinds,
/// messages and queue that the overlays show.
use music_player::feedback::{FeedbackKind, FeedbackMessage, FeedbackQueue};
/// Imports the component handle that reaches the window.
use slint::ComponentHandle;

/// Imports the file error type and the restore action.
use crate::ui_files::{FileActionError, restore_from_trash};
/// Imports the pure models of the overlays.
use crate::ui_overlays_model::{Hold, feedback_rows, first_run_view, visible_time_ms};
/// Imports the model builder and the state reader of the screen.
use crate::ui_screen::{model_of, read_state};
/// Imports the window and the generated feedback row type.
use crate::{AppWindow, FeedbackItem};

/// What:
/// `OverlayState` is the feedback queue,
/// its id counter and the Undo targets it can still restore.
/// Why:
/// one copy on the UI thread keeps message ids unique and lets Undo find its file.
struct OverlayState {
    /// The visible feedback messages.
    queue: FeedbackQueue,
    /// Counter that makes every message id unique.
    next_id: u64,
    /// Undo messages and the path each one restores.
    undo_targets: Vec<(String, PathBuf)>,
}

thread_local! {
    /// What:
    /// the UI thread's feedback state.
    /// Why:
    /// messages are added and expired only on the event-loop thread.
    static OVERLAYS: &'static RefCell<OverlayState> = Box::leak(Box::new(RefCell::new(OverlayState {
        queue: FeedbackQueue::EMPTY,
        next_id: 0,
        undo_targets: Vec::new(),
    })));
    /// What:
    /// the repeating timer that expires feedback messages;
    /// kept alive for the window's lifetime.
    static EXPIRY_TIMER: RefCell<Option<slint::Timer>> = const { RefCell::new(None) };
    /// What:
    /// the moment the feedback clock started.
    /// Why:
    /// `FeedbackQueue` takes millisecond timestamps,
    /// so every call uses this origin.
    static CLOCK_START: Instant = Instant::now();
}

/// What:
/// `overlays` returns this thread's overlay state cell.
/// Why:
/// the cell is leaked once per thread,
/// so it outlives every borrow,
/// so each function borrows it directly instead of passing a closure to `LocalKey::with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const overlays = (): Cell<OverlayState> => OVERLAYS;
/// ```
fn overlays() -> &'static RefCell<OverlayState> {
    return OVERLAYS.with(Clone::clone);
}

/// What:
/// `now_ms` gives the feedback clock in milliseconds since the first feedback call.
/// Why:
/// the queue compares timestamps,
/// so the origin only has to be stable.
fn now_ms() -> i64 {
    return i64::try_from(CLOCK_START.with(Instant::elapsed).as_millis()).unwrap_or(i64::MAX);
}

/// What:
/// `add_message` shows one message for the time its text and `hold` call for,
/// and returns its id.
/// Why:
/// every feedback path goes through this,
/// so ids and timestamps stay consistent;
/// a long message or one with an action
/// stays longer than the fixed 4 s the queue would give it.
fn add_message(kind: FeedbackKind, text: &str, hold: Hold, action_label: Option<&str>, undo_path: Option<PathBuf>) -> Option<String> {
    let mut state = overlays().borrow_mut();
    state.next_id += 1;
    let prefix = if kind == FeedbackKind::Undo { "undo" } else { "error" };
    let id = format!("{prefix}-{}", state.next_id);
    let message = FeedbackMessage::new(&id, text, kind, action_label).ok()?;
    let duration_ms = visible_time_ms(text, hold);
    state.queue = match state.queue.show_for(message, now_ms(), duration_ms) {
        Ok(queue) => queue,
        Err(error) => {
            tracing::warn!(%error, duration_ms, "feedback duration rejected");
            return None;
        }
    };
    if let Some(path) = undo_path {
        state.undo_targets.push((id.clone(), path));
    }
    // The text is not logged: it can name a file. The id and kind show that a message appeared.
    tracing::info!(id = %id, kind = ?kind, duration_ms, "feedback message shown in the window");
    return Some(id);
}

/// What:
/// `first_message_expires_in_ms` gives how long the oldest visible message still has to stay,
/// or None with no message.
/// Why:
/// the window tests check how long each kind of message is meant to stay without waiting for it to expire.
#[cfg(test)]
pub(crate) fn first_message_expires_in_ms() -> Option<i64> {
    let expiry = overlays().borrow().queue.next_expiry_ms()?;
    return Some(expiry - now_ms());
}

/// What:
/// `show_error` shows an error message with the failure text.
/// Why:
/// a failed file action must be reported instead of looking successful.
pub(crate) fn show_error(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Error, text, Hold::Brief, None, None);
    project(window);
}

/// What:
/// `show_file_error` shows the text of a file action failure.
/// Why:
/// the error variants already name the detail in plain words.
pub(crate) fn show_file_error(window: &AppWindow, error: &FileActionError) {
    show_error(window, &error.to_string());
}

/// What:
/// `show_undo` shows the message that restores `path` from the trash,
/// with an Undo action.
/// Why:
/// D8 offers Undo for Move to trash;
/// the target path is kept so Undo can find the trashed copy.
pub(crate) fn show_undo(window: &AppWindow, text: &str, path: PathBuf) {
    add_message(FeedbackKind::Undo, text, Hold::Long, Some("Undo"), Some(path));
    project(window);
}

/// What:
/// `show_trash_notice` shows a trash move message that has no Undo action.
/// Why:
/// a platform that cannot restore from the trash still reports the move,
/// without offering Undo.
pub(crate) fn show_trash_notice(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Undo, text, Hold::Brief, None, None);
    project(window);
}

/// What:
/// `show_notice` shows a plain message with no action.
/// Why:
/// a keyboard action such as File details reports its result as a message;
/// this is the same message kind as the
/// trash notice,
/// named for what it is used for.
pub(crate) fn show_notice(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Undo, text, Hold::Brief, None, None);
    project(window);
}

/// What:
/// `show_long_notice` shows a plain message with no action that stays long enough to read in full.
/// Why:
/// File details lists a file name,
/// its size and its folder,
/// which takes longer to read than a one-line confirmation.
pub(crate) fn show_long_notice(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Undo, text, Hold::Long, None, None);
    project(window);
}

/// What:
/// `dismiss` removes the message with `id` and its Undo target.
/// Why:
/// a dismissed or acted-on message must not stay visible or restorable.
fn dismiss(id: &str) {
    let mut state = overlays().borrow_mut();
    state.queue = state.queue.dismiss(id);
    let mut kept_targets: Vec<(String, PathBuf)> = Vec::new();
    for target in state.undo_targets.drain(..) {
        if target.0 != id {
            kept_targets.push(target);
        }
    }
    state.undo_targets = kept_targets;
}

/// What:
/// `take_undo_target` removes and returns the path that the Undo message `id` restores.
/// Why:
/// an Undo can run once,
/// so the target leaves the list when it is taken.
fn take_undo_target(id: &str) -> Option<PathBuf> {
    let mut state = overlays().borrow_mut();
    let mut found: Option<usize> = None;
    for (at, target) in state.undo_targets.iter().enumerate() {
        if target.0 == id {
            found = Some(at);
            break;
        }
    }
    let position = found?;
    return Some(state.undo_targets.remove(position).1);
}

/// What:
/// `expire` drops messages whose time has run out,
/// and reprojects when any were removed.
/// Why:
/// the timer calls this periodically,
/// so messages leave without a user action.
fn expire(window: &AppWindow) {
    let changed = {
        let mut state = overlays().borrow_mut();
        let before = state.queue.visible().len();
        state.queue = state.queue.expire(now_ms());
        let remaining = state.queue.visible();
        let mut ids: Vec<String> = Vec::with_capacity(remaining.len());
        for message in &remaining {
            ids.push(message.id().to_string());
        }
        let mut kept_targets: Vec<(String, PathBuf)> = Vec::new();
        for target in state.undo_targets.drain(..) {
            if ids.contains(&target.0) {
                kept_targets.push(target);
            }
        }
        state.undo_targets = kept_targets;
        before != remaining.len()
    };
    if changed {
        project(window);
    }
}

/// What:
/// `project` pushes the feedback rows and the first-run pane to the window.
/// Why:
/// both sit above the track area,
/// so they are projected from the same state read.
pub(crate) fn project(window: &AppWindow) {
    let rows = feedback_rows(&overlays().borrow().queue);
    let mut items: Vec<FeedbackItem> = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(FeedbackItem {
            id: row.id.as_str().into(),
            text: row.text.as_str().into(),
            kind: row.kind,
            action_label: row.action_label.as_str().into(),
        });
    }
    window.set_feedback_items(model_of(items));
    let first_run = read_state(first_run_view);
    window.set_first_run_state(if first_run.is_some() { 1 } else { 0 });
    if let Some(view) = first_run {
        window.set_first_run_title(view.title.as_str().into());
        window.set_first_run_body(view.body.as_str().into());
        window.set_first_run_primary_label(view.primary_label.as_str().into());
        let mut secondary_labels: Vec<slint::SharedString> = Vec::with_capacity(view.secondary_labels.len());
        for label in &view.secondary_labels {
            secondary_labels.push(label.as_str().into());
        }
        window.set_first_run_secondary_labels(model_of(secondary_labels));
        window.set_first_run_explanation(view.explanation.as_str().into());
    }
}

/// What:
/// `wire` registers the feedback,
/// first-run and expiry behaviour on the window.
/// Why:
/// the first-run primary action opens the folder dialog (D10 wording),
/// and Undo restores from the trash.
pub(crate) fn wire(app: &AppWindow, engine: &Rc<Engine>) {
    let dismiss_weak = app.as_weak();
    app.on_feedback_dismiss(move |id| {
        if let Some(window) = dismiss_weak.upgrade() {
            dismiss(id.as_str());
            project(&window);
        }
    });
    let action_weak = app.as_weak();
    app.on_feedback_action(move |id| {
        if let Some(window) = action_weak.upgrade() {
            run_feedback_action(&window, id.as_str());
        }
    });
    let primary_sender = engine.sender();
    app.on_first_run_primary(move || {
        crate::ui_wire::open_folder_dialog(primary_sender.clone());
    });
    let settings_weak = app.as_weak();
    app.on_first_run_secondary(move |position| {
        if let Some(window) = settings_weak.upgrade()
            && position == 0
        {
            crate::ui_settings::open_settings(&window);
        }
    });
    start_expiry_timer(app);
}

/// What:
/// `run_feedback_action` runs the action of the message `id`;
/// only Undo has one.
/// Why:
/// Undo restores the trashed file,
/// and a failed restore is reported as its own error.
fn run_feedback_action(window: &AppWindow, id: &str) {
    let Some(path) = take_undo_target(id) else {
        dismiss(id);
        project(window);
        return;
    };
    dismiss(id);
    match restore_from_trash(&path) {
        Ok(()) => project(window),
        Err(error) => show_file_error(window, &error),
    }
}

/// What:
/// `start_expiry_timer` runs `expire` every quarter second for the window's lifetime.
/// Why:
/// a repeating timer is simpler than scheduling one timer per message,
/// and the queue is small.
fn start_expiry_timer(app: &AppWindow) {
    let timer = slint::Timer::default();
    let tick_weak = app.as_weak();
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(250), move || {
        if let Some(window) = tick_weak.upgrade() {
            expire(&window);
        }
    });
    EXPIRY_TIMER.set(Some(timer));
}
