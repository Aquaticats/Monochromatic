//! The one rule for activating a track: a click, Enter or a Search result on a track row (D124, D127, D129).
//!
//! What: `row_activation_for` turns the Auto-play on select setting and whether the track is the current one into what
//! the activation does.
//! Why: a click on a row, Enter on the ringed row and the activation of a Search result must not differ, so the
//! window wiring asks this one function and only maps its answer to engine commands.

/// What the activation of a track does to the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowActivation {
    /// The track becomes current and loads paused at its start; whatever played stops (D124).
    SelectOnly,
    /// The track becomes current and starts playing at once (D129, with Auto-play on select on).
    SelectAndPlay,
    /// The track is already current: it plays when paused and pauses when playing (D124).
    TogglePlay,
}

/// What: `row_activation_for` decides what activating a track does.
/// Why: the current track always toggles; any other track is only selected unless Auto-play on select is on, in which
/// case it also starts (the behavior before D124). The folder rail, the letter rail and the folder results do not use
/// this rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rowActivationFor = (autoPlayOnSelect: boolean, isCurrent: boolean): RowActivation =>
///   isCurrent ? "togglePlay" : autoPlayOnSelect ? "selectAndPlay" : "selectOnly";
/// ```
#[must_use]
pub fn row_activation_for(auto_play_on_select: bool, is_current: bool) -> RowActivation {
    if is_current {
        return RowActivation::TogglePlay;
    }
    if auto_play_on_select {
        return RowActivation::SelectAndPlay;
    }
    return RowActivation::SelectOnly;
}

/// What: `AUTO_PLAY_ON_SELECT_TITLE` is the Settings row's title.
/// Why: the window and the tests share one text.
pub const AUTO_PLAY_ON_SELECT_TITLE: &str = "Auto-play on select";

/// What: `auto_play_on_select_supporting_text` is the line under the setting's title.
/// Why: the line states what the current choice does, so the text is a label of the state besides the switch itself
/// (D129); both texts fit one line at the narrowest wide list pane, 218 px, or wrap to a second.
#[must_use]
pub fn auto_play_on_select_supporting_text(enabled: bool) -> &'static str {
    if enabled {
        return "Select and play at once";
    }
    return "Select first, then Play";
}

/// Tests for the rule and the texts in `row_activation_tests.rs`.
#[cfg(test)]
#[path = "row_activation_tests.rs"]
mod tests;
