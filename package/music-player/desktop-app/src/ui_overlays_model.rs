//! Pure models for the first-run pane and the feedback messages.
//!
//! What: `first_run_view` decides whether the first-run pane replaces the track area and what it says;
//! `feedback_items` turns the visible feedback messages into plain rows.
//! Why: the decisions stay testable without a window, and the window only places the values.

use music_player::feedback::{FeedbackKind, FeedbackQueue};
use music_player::first_run::{ANALYSIS_EXPLANATION, FirstRunState, first_run_copy};

use crate::ui_screen_view::ScreenState;

/// What: `FirstRunView` is the copy and actions of the first-run pane.
/// Why: the pane takes plain strings, and the Rust copy module supplies them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FirstRunView = { title: string; body: string; primaryLabel: string; secondaryLabels: string[]; explanation: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirstRunView {
    /// Headline of the pane.
    pub title: String,
    /// Explanation of the open source and what the actions do.
    pub body: String,
    /// Label of the filled action, or empty.
    pub primary_label: String,
    /// Labels of the outlined actions, in order.
    pub secondary_labels: Vec<String>,
    /// True-peak explanation, or empty when the copy has none.
    pub explanation: String,
}

/// What: `first_run_view` returns the first-run pane once the library has loaded and holds no tracks.
/// Why: an empty library is the only first-run case on desktop; the pane replaces the track area until a folder has audio.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstRunView = (s: ScreenState): FirstRunView | null => s.loaded && s.names.length === 0 ? copy : null;
/// ```
pub fn first_run_view(state: &ScreenState) -> Option<FirstRunView> {
    if !state.names.is_empty() && !state.reading {
        return None;
    }
    let kind = first_run_kind(state)?;
    let folder_name = state.source_root_name.as_deref();
    let copy = first_run_copy(kind, folder_name).ok()?;
    return Some(FirstRunView {
        title: copy.title,
        body: copy.body,
        primary_label: copy.primary_action.unwrap_or("").to_string(),
        secondary_labels: copy.secondary_actions.iter().map(|label| return label.to_string()).collect(),
        explanation: if copy.explains_analysis { ANALYSIS_EXPLANATION.to_string() } else { String::new() },
    });
}

/// What: `first_run_kind` chooses the first-run state that matches the opened folder.
/// Why: a folder being read gets the reading copy (EM1); no folder open gets the choose-a-folder copy; an opened folder with no tracks names itself; before the engine reports, nothing shows.
fn first_run_kind(state: &ScreenState) -> Option<FirstRunState> {
    if state.reading {
        return Some(FirstRunState::Reading);
    }
    if !state.root_known {
        return None;
    }
    return match (&state.source_root_name, state.loaded) {
        (None, _) => Some(FirstRunState::NoFolderOpen),
        (Some(_), true) => Some(FirstRunState::FolderNoAudio),
        (Some(_), false) => None,
    };
}

/// What: `Hold` is how long a kind of message is meant to stay: a short confirmation, or a message the user acts on or reads
/// in full.
/// Why: one fixed 4 s cut off the File details path and left too little time to press Undo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// A confirmation or a short error: at least 4 s.
    Brief,
    /// A message with an action or a long detail: at least 8 s.
    Long,
}

/// What: `visible_time_ms` gives how long a message stays on screen, in milliseconds.
/// Why: the time is the larger of the floor of its `Hold` and a reading allowance of 2 s plus 60 ms per character, so a long
/// message stays long enough to read, and no message stays longer than 15 s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const visibleTimeMs = (text: string, hold: Hold) => Math.min(15_000, Math.max(hold === "long" ? 8_000 : 4_000, 2_000 + 60 * [...text].length));
/// ```
pub fn visible_time_ms(text: &str, hold: Hold) -> i64 {
    let floor: i64 = match hold {
        Hold::Brief => 4_000,
        Hold::Long => 8_000,
    };
    let characters = i64::try_from(text.chars().count()).unwrap_or(i64::MAX / 100);
    let reading = 2_000_i64.saturating_add(characters.saturating_mul(60));
    return reading.max(floor).min(15_000);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackRow {
    /// Stable key the window reports dismiss and action with.
    pub id: String,
    /// Message text.
    pub text: String,
    /// 0 for an error, 1 for an Undo message.
    pub kind: i32,
    /// Action button label, or empty.
    pub action_label: String,
}

/// What: `feedback_rows` lists the visible messages in display order.
/// Why: the queue keeps its own order, and expired messages are already removed by the queue.
pub fn feedback_rows(queue: &FeedbackQueue) -> Vec<FeedbackRow> {
    return queue
        .visible()
        .iter()
        .map(|message| return FeedbackRow {
            id: message.id().to_string(),
            text: message.text().to_string(),
            kind: match message.kind() {
                FeedbackKind::Error => 0,
                FeedbackKind::Undo => 1,
            },
            action_label: message.action_label().unwrap_or("").to_string(),
        })
        .collect();
}

#[cfg(test)]
#[path = "ui_overlays_model_tests.rs"]
mod tests;
