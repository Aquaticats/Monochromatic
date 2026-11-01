//! Key hint badges: when they show and what they say (D106).
//!
//! What: `KeyHints` tracks which modifiers are held and for how long; `badge_text` gives the badge of one control for
//! the modifiers that are held.
//! Why: D106 shows a small badge on every control that has a shortcut once the modifiers of a shortcut have all been
//! held for 300 ms, narrows the badges as more modifiers are held, and hides them when a required modifier is
//! released. The timing and narrowing rules are pure logic with injected time, so they are tested without a window.

use crate::keymap::{KeyboardPlatform, ShortcutAction};
use crate::keymap_bindings::{HeldModifiers, binding_for, key_label};

/// What:
///      `HINT_HOLD_MS` is how long the modifiers must stay held before the badges appear.
/// Why:
///      D106 fixes it at 300 ms, so a quick chord never flashes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HINT_HOLD_MS = 300;
/// ```
pub const HINT_HOLD_MS: u64 = 300;

/// What:
///      `pub struct KeyHints` is the badge state: the modifiers held now, since when, and whether the badges are showing.
/// Why:
///      The badges show after the hold time and then follow the held set until every modifier is released.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeyHints = { held: HeldModifiers; sinceMs: number; shown: boolean };
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyHints {
    /// Modifiers held now.
    held: HeldModifiers,
    /// When the held set last changed.
    since_ms: u64,
    /// Whether the hold time has passed, so badges show.
    shown: bool,
}

/// Methods of `KeyHints`.
impl KeyHints {
    /// What:
    ///      `on_modifiers` records the held set after a key event.
    /// Why:
    ///      A change of the set restarts the hold time while the badges are hidden, so a quick chord never flashes
    ///      them; once shown they stay while any modifier is held and follow the set, and releasing everything hides them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function onModifiers(hints: KeyHints, held: HeldModifiers, nowMs: number): KeyHints;
    /// ```
    pub fn on_modifiers(self, held: HeldModifiers, now_ms: u64) -> Self {
        if held == self.held {
            return self;
        }
        if held.is_empty() {
            return Self::default();
        }
        return Self { held, since_ms: now_ms, shown: self.shown };
    }

    /// What:
    ///      `on_tick` shows the badges once the held set has been unchanged for the hold time.
    /// Why:
    ///      A timer calls it; before the hold time it changes nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function onTick(hints: KeyHints, nowMs: number): KeyHints;
    /// ```
    pub fn on_tick(self, now_ms: u64) -> Self {
        if self.shown || self.held.is_empty() || now_ms.saturating_sub(self.since_ms) < HINT_HOLD_MS {
            return self;
        }
        return Self { shown: true, ..self };
    }

    /// What:
    ///      `wait_ms` gives how long until the badges are due, or `None` when no timer is needed.
    /// Why:
    ///      The window starts one timer for exactly this long after a modifier goes down.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function waitMs(hints: KeyHints, nowMs: number): number | null;
    /// ```
    pub fn wait_ms(self, now_ms: u64) -> Option<u64> {
        if self.shown || self.held.is_empty() {
            return None;
        }
        return Some(HINT_HOLD_MS.saturating_sub(now_ms.saturating_sub(self.since_ms)));
    }

    /// What:
    ///      `visible` gives the held set whose chords the badges show, or `None` while the badges are hidden.
    /// Why:
    ///      Badges narrow to the rows whose modifiers equal the held set.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function visible(hints: KeyHints): HeldModifiers | null;
    /// ```
    pub fn visible(self) -> Option<HeldModifiers> {
        return if self.shown && !self.held.is_empty() { Some(self.held) } else { None };
    }
}

/// What:
///      `pub enum HintTarget` names a control that can carry a badge.
/// Why:
///      Each control has its own badge slot in the window, and the table says which actions it shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HintTarget = "search" | "settings" | "open-folder" | /* ... */;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintTarget {
    /// The Search button.
    Search,
    /// The Settings button.
    Settings,
    /// The Open pill, which opens the system folder dialog.
    OpenFolder,
    /// The folder title trigger of the folded layout, which opens the picker.
    FolderTrigger,
    /// The folder picker list, focused by Alt+1.
    FolderList,
    /// The track list, focused by Alt+2.
    TrackList,
    /// The previous track button.
    Previous,
    /// The next track button.
    Next,
    /// The back end of the seek bar.
    SeekBack,
    /// The forward end of the seek bar.
    SeekForward,
    /// The Repeat segment of the mode group.
    ModeRepeat,
    /// The In order segment.
    ModeInOrder,
    /// The Shuffle the folder segment.
    ModeShuffleFolder,
    /// The Shuffle all segment.
    ModeShuffleAll,
    /// The whole mode group, which Control+M cycles.
    ModeCycle,
    /// The playing track's title on the deck.
    DeckTitle,
    /// The playing track's folder on the deck.
    DeckFolder,
    /// The Reset to default control of the template editor.
    EditorReset,
}

/// Methods of `HintTarget`.
impl HintTarget {
    /// What:
    ///      `ALL` lists every control that can carry a badge.
    /// Why:
    ///      The window fills one badge slot per target, and tests walk the list.
    pub const ALL: [HintTarget; 18] = [
        Self::Search,
        Self::Settings,
        Self::OpenFolder,
        Self::FolderTrigger,
        Self::FolderList,
        Self::TrackList,
        Self::Previous,
        Self::Next,
        Self::SeekBack,
        Self::SeekForward,
        Self::ModeRepeat,
        Self::ModeInOrder,
        Self::ModeShuffleFolder,
        Self::ModeShuffleAll,
        Self::ModeCycle,
        Self::DeckTitle,
        Self::DeckFolder,
        Self::EditorReset,
    ];

    /// What:
    ///      `actions` lists the actions whose chords this control shows,
    ///  most plain first.
    /// Why:
    ///      A control can have more than one chord, such as the seek ends with their 10 and 60 second variants.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const actions = (target: HintTarget): ShortcutAction[] => ...;
    /// ```
    pub fn actions(self) -> &'static [ShortcutAction] {
        return match self {
            Self::Search => &[ShortcutAction::Search],
            Self::Settings => &[ShortcutAction::Settings],
            Self::OpenFolder => &[ShortcutAction::OpenFolderDialog],
            Self::FolderTrigger => &[ShortcutAction::Picker],
            Self::FolderList => &[ShortcutAction::FocusFolders],
            Self::TrackList => &[ShortcutAction::FocusTracks],
            Self::Previous => &[ShortcutAction::Previous],
            Self::Next => &[ShortcutAction::Next],
            Self::SeekBack => &[ShortcutAction::SeekBack, ShortcutAction::SeekBackLong],
            Self::SeekForward => &[ShortcutAction::SeekForward, ShortcutAction::SeekForwardLong],
            Self::ModeRepeat => &[ShortcutAction::ModeRepeat],
            Self::ModeInOrder => &[ShortcutAction::ModeInOrder],
            Self::ModeShuffleFolder => &[ShortcutAction::ModeShuffleFolder],
            Self::ModeShuffleAll => &[ShortcutAction::ModeShuffleAll],
            Self::ModeCycle => &[ShortcutAction::Mode],
            Self::DeckTitle => &[ShortcutAction::Reveal],
            Self::DeckFolder => &[ShortcutAction::RevealFolder],
            Self::EditorReset => &[ShortcutAction::EditorReset],
        };
    }
}

/// What:
///      `badge_text` gives the badge of one control for the held modifiers,
///  or an empty string when it shows none.
/// Why:
///      A control shows its key only when the held set equals the modifiers of one of its chords,
///  and the key alone is shown because the modifiers are already down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function badgeText(target: HintTarget, platform: KeyboardPlatform, visible: HeldModifiers | null): string {
///   if (!visible) return "";
///   for (const action of target.actions()) {
///     const binding = bindingFor(action, platform);
///     if (binding && sameModifiers(binding, platform, visible)) return keyLabel(binding.keys[0]);
///   }
///   return "";
/// }
/// ```
pub fn badge_text(target: HintTarget, platform: KeyboardPlatform, visible: Option<HeldModifiers>) -> String {
    let Some(held) = visible else {
        return String::new();
    };
    for action in target.actions() {
        let Some(binding) = binding_for(*action, platform) else {
            continue;
        };
        if binding.required(platform) == held {
            return binding.keys.first().map_or_else(String::new, |name| return key_label(name));
        }
    }
    return String::new();
}

/// Unit tests in `key_hints_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "key_hints_tests.rs"]
mod tests;
