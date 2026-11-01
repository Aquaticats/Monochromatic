//! Window binding for the feedback messages and the first-run pane.
//!
//! What: `wire` registers the feedback dismiss and action callbacks and the first-run actions, and starts
//! the expiry timer; `show_error` and `show_undo` add messages from other modules.
//! Why: the feedback queue is a pure value (`feedback.rs`); this module owns the one copy and its timer.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use music_player::engine::Engine;
use music_player::feedback::{FeedbackKind, FeedbackMessage, FeedbackQueue};
use slint::ComponentHandle;

use crate::ui_files::{FileActionError, restore_from_trash};
use crate::ui_overlays_model::{Hold, feedback_rows, first_run_view, visible_time_ms};
use crate::ui_screen::{model_of, read_state};
use crate::{AppWindow, FeedbackItem};

/// What: `OverlayState` is the feedback queue, its id counter and the Undo targets it can still restore.
/// Why: one copy on the UI thread keeps message ids unique and lets Undo find its file.
struct OverlayState {
    /// The visible feedback messages.
    queue: FeedbackQueue,
    /// Counter that makes every message id unique.
    next_id: u64,
    /// Undo messages and the path each one restores.
    undo_targets: Vec<(String, PathBuf)>,
}

thread_local! {
    /// What: the UI thread's feedback state.
    /// Why: messages are added and expired only on the event-loop thread.
    static OVERLAYS: RefCell<OverlayState> = const { RefCell::new(OverlayState {
        queue: FeedbackQueue::EMPTY,
        next_id: 0,
        undo_targets: Vec::new(),
    }) };
    /// What: the repeating timer that expires feedback messages; kept alive for the window's lifetime.
    static EXPIRY_TIMER: RefCell<Option<slint::Timer>> = const { RefCell::new(None) };
    /// What: the moment the feedback clock started.
    /// Why: `FeedbackQueue` takes millisecond timestamps, so every call uses this origin.
    static CLOCK_START: Instant = Instant::now();
}

/// What: `now_ms` gives the feedback clock in milliseconds since the first feedback call.
/// Why: the queue compares timestamps, so the origin only has to be stable.
fn now_ms() -> i64 {
    return CLOCK_START.with(|start| return i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX));
}

/// What: `add_message` shows one message for the time its text and `hold` call for, and returns its id.
/// Why: every feedback path goes through this, so ids and timestamps stay consistent; a long message or one with an action
/// stays longer than the fixed 4 s the queue would give it.
fn add_message(kind: FeedbackKind, text: &str, hold: Hold, action_label: Option<&str>, undo_path: Option<PathBuf>) -> Option<String> {
    let outcome = OVERLAYS.with(|cell| {
        let mut state = cell.borrow_mut();
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
    });
    return outcome;
}

/// What: `first_message_expires_in_ms` gives how long the oldest visible message still has to stay, or None with no message.
/// Why: the window tests check how long each kind of message is meant to stay without waiting for it to expire.
#[cfg(test)]
pub(crate) fn first_message_expires_in_ms() -> Option<i64> {
    return OVERLAYS.with(|cell| return cell.borrow().queue.next_expiry_ms()).map(|expiry| return expiry - now_ms());
}

/// What: `show_error` shows an error message with the failure text.
/// Why: a failed file action must be reported instead of looking successful.
pub(crate) fn show_error(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Error, text, Hold::Brief, None, None);
    project(window);
}

/// What: `show_file_error` shows the text of a file action failure.
/// Why: the error variants already name the detail in plain words.
pub(crate) fn show_file_error(window: &AppWindow, error: &FileActionError) {
    show_error(window, &error.to_string());
}

/// What: `show_undo` shows the message that restores `path` from the trash, with an Undo action.
/// Why: D8 offers Undo for Move to trash; the target path is kept so Undo can find the trashed copy.
pub(crate) fn show_undo(window: &AppWindow, text: &str, path: PathBuf) {
    add_message(FeedbackKind::Undo, text, Hold::Long, Some("Undo"), Some(path));
    project(window);
}

/// What: `show_trash_notice` shows a trash move message that has no Undo action.
/// Why: a platform that cannot restore from the trash still reports the move, without offering Undo.
pub(crate) fn show_trash_notice(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Undo, text, Hold::Brief, None, None);
    project(window);
}

/// What: `show_notice` shows a plain message with no action.
/// Why: a keyboard action such as File details reports its result as a message; this is the same message kind as the
/// trash notice, named for what it is used for.
pub(crate) fn show_notice(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Undo, text, Hold::Brief, None, None);
    project(window);
}

/// What: `show_long_notice` shows a plain message with no action that stays long enough to read in full.
/// Why: File details lists a file name, its size and its folder, which takes longer to read than a one-line confirmation.
pub(crate) fn show_long_notice(window: &AppWindow, text: &str) {
    add_message(FeedbackKind::Undo, text, Hold::Long, None, None);
    project(window);
}

/// What: `dismiss` removes the message with `id` and its Undo target.
/// Why: a dismissed or acted-on message must not stay visible or restorable.
fn dismiss(id: &str) {
    OVERLAYS.with(|cell| {
        let mut state = cell.borrow_mut();
        state.queue = state.queue.dismiss(id);
        state.undo_targets.retain(|(target_id, _)| return target_id != id);
    });
}

/// What: `take_undo_target` removes and returns the path that the Undo message `id` restores.
/// Why: an Undo can run once, so the target leaves the list when it is taken.
fn take_undo_target(id: &str) -> Option<PathBuf> {
    return OVERLAYS.with(|cell| {
        let mut state = cell.borrow_mut();
        let position = state.undo_targets.iter().position(|(target_id, _)| return target_id == id)?;
        return Some(state.undo_targets.remove(position).1);
    });
}

/// What: `expire` drops messages whose time has run out, and reprojects when any were removed.
/// Why: the timer calls this periodically, so messages leave without a user action.
fn expire(window: &AppWindow) {
    let changed = OVERLAYS.with(|cell| {
        let mut state = cell.borrow_mut();
        let before = state.queue.visible().len();
        state.queue = state.queue.expire(now_ms());
        let remaining = state.queue.visible();
        let ids: Vec<String> = remaining.iter().map(|message| return message.id().to_string()).collect();
        state.undo_targets.retain(|(target_id, _)| return ids.contains(target_id));
        return before != remaining.len();
    });
    if changed {
        project(window);
    }
}

/// What: `project` pushes the feedback rows and the first-run pane to the window.
/// Why: both sit above the track area, so they are projected from the same state read.
pub(crate) fn project(window: &AppWindow) {
    let rows = OVERLAYS.with(|cell| return feedback_rows(&cell.borrow().queue));
    window.set_feedback_items(model_of(
        rows.into_iter()
            .map(|row| return FeedbackItem {
                id: row.id.as_str().into(),
                text: row.text.as_str().into(),
                kind: row.kind,
                action_label: row.action_label.as_str().into(),
            })
            .collect(),
    ));
    let first_run = read_state(first_run_view);
    window.set_first_run_state(if first_run.is_some() { 1 } else { 0 });
    if let Some(view) = first_run {
        window.set_first_run_title(view.title.as_str().into());
        window.set_first_run_body(view.body.as_str().into());
        window.set_first_run_primary_label(view.primary_label.as_str().into());
        window.set_first_run_secondary_labels(model_of(
            view.secondary_labels.iter().map(|label| return label.as_str().into()).collect(),
        ));
        window.set_first_run_explanation(view.explanation.as_str().into());
    }
}

/// What: `wire` registers the feedback, first-run and expiry behaviour on the window.
/// Why: the first-run primary action opens the folder dialog (D10 wording), and Undo restores from the trash.
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

/// What: `run_feedback_action` runs the action of the message `id`; only Undo has one.
/// Why: Undo restores the trashed file, and a failed restore is reported as its own error.
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

/// What: `start_expiry_timer` runs `expire` every quarter second for the window's lifetime.
/// Why: a repeating timer is simpler than scheduling one timer per message, and the queue is small.
fn start_expiry_timer(app: &AppWindow) {
    let timer = slint::Timer::default();
    let tick_weak = app.as_weak();
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(250), move || {
        if let Some(window) = tick_weak.upgrade() {
            expire(&window);
        }
    });
    EXPIRY_TIMER.with(|cell| {
        *cell.borrow_mut() = Some(timer);
    });
}
