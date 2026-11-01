//! The Text size setting: one pixel size for each of four texts the human tuned (desktop only; Android follows the
//! system font size).
//!
//! What: `TextElement` names the four texts, `TextSizes` holds one size for each, and every change goes through
//! `TextSizes::stepped`, which keeps the size inside `MIN_TEXT_SIZE_PX` to `MAX_TEXT_SIZE_PX`.
//! Why: the Settings steppers, the session file and the window all need the same limits, defaults and element order,
//! so the rules live here and the glue only maps them to controls.

use serde::Serialize;

/// What: `MIN_TEXT_SIZE_PX` is the smallest size a stepper reaches.
/// Why: under 10 px the Inter glyphs stop being legible at scale 1.
pub const MIN_TEXT_SIZE_PX: u16 = 10;

/// What: `MAX_TEXT_SIZE_PX` is the largest size a stepper reaches.
/// Why: at 32 px a track title still fits the 72 px row pitch with its caption (see the layout checks in
/// `ui_text_size_tests.rs`).
pub const MAX_TEXT_SIZE_PX: u16 = 32;

/// The four texts that have a size of their own, in the order of the Settings rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextElement {
    /// The titles of the left and the right pane (the opened folder and the shown folder).
    PaneTitle,
    /// The letter that heads a group of folder names.
    GroupHeading,
    /// The folder names in the picker, and their width probe.
    FolderName,
    /// The track titles in the track list.
    TrackTitle,
}

/// Methods of `TextElement`.
impl TextElement {
    /// Every element in the order of the Settings rows, which is also the index the window uses for a stepper.
    pub const ALL: [TextElement; 4] =
        [TextElement::PaneTitle, TextElement::GroupHeading, TextElement::FolderName, TextElement::TrackTitle];

    /// What: `default_px` is the size the app drew the text at before the setting existed.
    /// Why: the defaults are today's values, so nothing changes until the human changes a stepper.
    #[must_use]
    pub const fn default_px(self) -> u16 {
        return match self {
            TextElement::PaneTitle | TextElement::GroupHeading => 20,
            TextElement::FolderName | TextElement::TrackTitle => 14,
        };
    }

    /// What: `label` is the name of the row in Settings.
    /// Why: the window and the tests share one text.
    #[must_use]
    pub const fn label(self) -> &'static str {
        return match self {
            TextElement::PaneTitle => "Pane titles",
            TextElement::GroupHeading => "Group heading letter",
            TextElement::FolderName => "Folder names",
            TextElement::TrackTitle => "Track titles",
        };
    }

    /// What: `session_key` is the name of the element's field in the session file.
    /// Why: the load path reads the four fields by name so a corrupt one is reported with its key.
    #[must_use]
    pub const fn session_key(self) -> &'static str {
        return match self {
            TextElement::PaneTitle => "text_size_pane_title_px",
            TextElement::GroupHeading => "text_size_group_heading_px",
            TextElement::FolderName => "text_size_folder_name_px",
            TextElement::TrackTitle => "text_size_track_title_px",
        };
    }

    /// What: `from_index` maps a stepper index of the window to its element.
    /// Why: the window reports a plain number; an index outside the four is no element.
    #[must_use]
    pub fn from_index(index: i32) -> Option<TextElement> {
        return usize::try_from(index).ok().and_then(|position| return TextElement::ALL.get(position).copied());
    }
}

/// One size in pixels for each element, always inside the stepper limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TextSizes {
    /// Size of the pane titles.
    #[serde(rename = "text_size_pane_title_px")]
    pane_title_px: u16,
    /// Size of the group heading letter.
    #[serde(rename = "text_size_group_heading_px")]
    group_heading_px: u16,
    /// Size of the folder names.
    #[serde(rename = "text_size_folder_name_px")]
    folder_name_px: u16,
    /// Size of the track titles.
    #[serde(rename = "text_size_track_title_px")]
    track_title_px: u16,
}

/// The defaults are today's sizes.
impl Default for TextSizes {
    /// Returns the sizes of every element at its default.
    fn default() -> TextSizes {
        return TextSizes {
            pane_title_px: TextElement::PaneTitle.default_px(),
            group_heading_px: TextElement::GroupHeading.default_px(),
            folder_name_px: TextElement::FolderName.default_px(),
            track_title_px: TextElement::TrackTitle.default_px(),
        };
    }
}

/// Reads, changes and judges sizes.
impl TextSizes {
    /// What: `get` is the size of `element` in pixels.
    /// Why: the window reads each size by element.
    #[must_use]
    pub const fn get(&self, element: TextElement) -> u16 {
        return match element {
            TextElement::PaneTitle => self.pane_title_px,
            TextElement::GroupHeading => self.group_heading_px,
            TextElement::FolderName => self.folder_name_px,
            TextElement::TrackTitle => self.track_title_px,
        };
    }

    /// What: `with` gives these sizes with `element` set to `px`, held inside the limits.
    /// Why: no caller can build a size outside the limits, so the window never draws one.
    #[must_use]
    pub fn with(self, element: TextElement, px: u16) -> TextSizes {
        let held = px.clamp(MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX);
        let mut next = self;
        match element {
            TextElement::PaneTitle => next.pane_title_px = held,
            TextElement::GroupHeading => next.group_heading_px = held,
            TextElement::FolderName => next.folder_name_px = held,
            TextElement::TrackTitle => next.track_title_px = held,
        }
        return next;
    }

    /// What: `stepped` moves `element` by `delta` pixels and stops at the limits.
    /// Why: a stepper changes a size by one pixel, and a press past the end changes nothing.
    #[must_use]
    pub fn stepped(self, element: TextElement, delta: i32) -> TextSizes {
        let moved = i32::from(self.get(element)).saturating_add(delta);
        let held = moved.clamp(i32::from(MIN_TEXT_SIZE_PX), i32::from(MAX_TEXT_SIZE_PX));
        return self.with(element, u16::try_from(held).unwrap_or(element.default_px()));
    }

    /// What: `can_decrease` tells whether the minus button of `element` still does anything.
    /// Why: the button draws as disabled at the lower limit.
    #[must_use]
    pub const fn can_decrease(&self, element: TextElement) -> bool {
        return self.get(element) > MIN_TEXT_SIZE_PX;
    }

    /// What: `can_increase` tells whether the plus button of `element` still does anything.
    /// Why: the button draws as disabled at the upper limit.
    #[must_use]
    pub const fn can_increase(&self, element: TextElement) -> bool {
        return self.get(element) < MAX_TEXT_SIZE_PX;
    }

    /// What: `is_default` tells whether every element is at its default.
    /// Why: the Reset text sizes row has nothing to do then, and says so.
    #[must_use]
    pub fn is_default(&self) -> bool {
        return *self == TextSizes::default();
    }
}

/// What: `size_or_default` reads one session field: an integer inside the limits is the size; a missing field or
/// anything else is the element's default.
/// Why: the field of one element must never cost the other fields, so a wrong type, a fraction or an out-of-range
/// number falls back to the default of that element alone, and is reported with its key and value. A missing field is a
/// session file written before the setting existed and stays quiet.
#[must_use]
pub fn size_or_default(element: TextElement, stored: Option<&serde_json::Value>) -> u16 {
    let Some(value) = stored else {
        return element.default_px();
    };
    let held = value
        .as_u64()
        .and_then(|number| return u16::try_from(number).ok())
        .filter(|number| return (MIN_TEXT_SIZE_PX..=MAX_TEXT_SIZE_PX).contains(number));
    if let Some(px) = held {
        return px;
    }
    tracing::warn!(
        key = element.session_key(),
        %value,
        default = element.default_px(),
        "the saved text size is not a whole number of pixels from {MIN_TEXT_SIZE_PX} to {MAX_TEXT_SIZE_PX}; using the default",
    );
    return element.default_px();
}

/// Tests for the limits, the defaults, the stepping and the reading of a stored field in `text_size_tests.rs`.
#[cfg(test)]
#[path = "text_size_tests.rs"]
mod tests;
