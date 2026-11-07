//! Key hint badges and shortcut texts on the window (D106).
//!
//! What:
//! `modifiers_changed` feeds the held modifiers to the pure `KeyHints` state,
//! starts a one-shot timer for the
//! 300 ms hold,
//! and writes the badge of every control into the window's `Keys` global;
//! `install_texts` writes the
//! shortcut text of every control once,
//! for the tooltips.
//! Why:
//! the timing and narrowing rules are tested in `key_hints`;
//! this file only connects them to the clock,
//! to a Slint
//! timer,
//! and to the window.

/// Imports the cell that holds the hint state between ticks.
use std::cell::Cell;
/// Imports the duration and instant types that time the hold delay.
use std::time::{Duration, Instant};

/// Imports the hint state,
/// its targets and the badge text.
use music_player::key_hints::{HintTarget, KeyHints, badge_text};
/// Imports the keyboard platform and the shortcut action that the texts describe.
use music_player::keymap::{KeyboardPlatform, ShortcutAction};
/// Imports the held modifiers and the shortcut text lookup.
use music_player::keymap_bindings::{HeldModifiers, shortcut_text};
/// Imports the component handle,
/// the timer and its mode that run the hold delay.
use slint::{ComponentHandle, SharedString, Timer, TimerMode};

/// Imports the window and the generated badge,
/// key and text globals that hold the hints.
use crate::{AppWindow, KeyBadges, Keys, ShortcutTexts};

thread_local! {
    /// What:
    /// the badge state of this window thread.
    /// Why:
    /// the key handler and the timer both read and write it on the event-loop thread only.
    static HINTS: Cell<KeyHints> = Cell::new(KeyHints::default());
    /// What:
    /// the one-shot timer that shows the badges after the hold time.
    /// Why:
    /// a Slint timer runs on the event loop,
    /// so the badges appear even when no key event arrives.
    static HOLD_TIMER: &'static Timer = Box::leak(Box::new(Timer::default()));
    /// What:
    /// the moment the hint clock started.
    /// Why:
    /// `KeyHints` takes millisecond timestamps,
    /// so every call uses this origin.
    static CLOCK_START: Instant = Instant::now();
}

#[cfg(test)]
thread_local! {
    /// What:
    /// the clock a window test sets,
    /// in milliseconds.
    /// Why:
    /// the tests move Slint's timers with a mock clock,
    /// so the hint state and the double tap of Shift must read a
    /// clock that moves with it.
    static TEST_CLOCK_MS: Cell<Option<u64>> = const { Cell::new(None) };
}

/// What:
/// `set_test_clock_ms` makes `now_ms` return a value the test controls.
/// Why:
/// see `TEST_CLOCK_MS`.
#[cfg(test)]
pub(crate) fn set_test_clock_ms(milliseconds: u64) {
    TEST_CLOCK_MS.set(Some(milliseconds));
}

/// What:
/// `now_ms` gives the hint clock in milliseconds since the first call.
/// Why:
/// the state compares timestamps,
/// so the origin only has to be stable.
pub(crate) fn now_ms() -> u64 {
    #[cfg(test)]
    if let Some(milliseconds) = TEST_CLOCK_MS.get() {
        return milliseconds;
    }
    return u64::try_from(CLOCK_START.with(Instant::elapsed).as_millis()).unwrap_or(u64::MAX);
}

/// What:
/// `badge` is the badge text of one target as the window's string type.
/// Why:
/// `badges_of` fills eighteen badge fields the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const badge = (target: HintTarget, platform: KeyboardPlatform, visible: HeldModifiers | null): string => badgeText(target, platform, visible);
/// ```
fn badge(target: HintTarget, platform: KeyboardPlatform, visible: Option<HeldModifiers>) -> SharedString {
    return badge_text(target, platform, visible).as_str().into();
}

/// What:
/// `badges_of` builds the window struct of badge texts for the modifiers that are showing.
/// Why:
/// one value feeds every control's badge,
/// and an empty text draws none.
fn badges_of(platform: KeyboardPlatform, visible: Option<HeldModifiers>) -> KeyBadges {
    return KeyBadges {
        search: badge(HintTarget::Search, platform, visible),
        settings: badge(HintTarget::Settings, platform, visible),
        open_folder: badge(HintTarget::OpenFolder, platform, visible),
        folder_trigger: badge(HintTarget::FolderTrigger, platform, visible),
        folder_list: badge(HintTarget::FolderList, platform, visible),
        track_list: badge(HintTarget::TrackList, platform, visible),
        previous: badge(HintTarget::Previous, platform, visible),
        next: badge(HintTarget::Next, platform, visible),
        seek_back: badge(HintTarget::SeekBack, platform, visible),
        seek_forward: badge(HintTarget::SeekForward, platform, visible),
        mode_repeat: badge(HintTarget::ModeRepeat, platform, visible),
        mode_in_order: badge(HintTarget::ModeInOrder, platform, visible),
        mode_shuffle_folder: badge(HintTarget::ModeShuffleFolder, platform, visible),
        mode_shuffle_all: badge(HintTarget::ModeShuffleAll, platform, visible),
        mode_cycle: badge(HintTarget::ModeCycle, platform, visible),
        deck_title: badge(HintTarget::DeckTitle, platform, visible),
        deck_folder: badge(HintTarget::DeckFolder, platform, visible),
        editor_reset: badge(HintTarget::EditorReset, platform, visible),
    };
}

/// What:
/// `hold_timer` returns this thread's hold timer.
/// Why:
/// the timer is leaked once per thread,
/// so it outlives every borrow,
/// so the schedule can call its methods directly instead of passing a closure to `LocalKey::with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdTimer = (): Timer => HOLD_TIMER;
/// ```
fn hold_timer() -> &'static Timer {
    return HOLD_TIMER.with(Clone::clone);
}

/// What:
/// `refresh` writes the badges the current state shows into the window.
/// Why:
/// every change of the state ends here,
/// so the window never disagrees with the state.
fn refresh(window: &AppWindow) {
    let visible = HINTS.get().visible();
    window.global::<Keys>().set_badges(badges_of(crate::ui_keys::platform(), visible));
    tracing::debug!(visible = ?visible, "key hint badges updated");
    crate::ui_state_log::emit(window, "badges");
}

/// What:
/// `schedule` starts the one-shot timer that shows the badges when the hold time has passed.
/// Why:
/// the badges must appear without another key event,
/// and a new held set restarts the wait.
fn schedule(window: &AppWindow) {
    let wait = HINTS.get().wait_ms(now_ms());
    let timer = hold_timer();
    timer.stop();
    let Some(wait_ms) = wait else {
        return;
    };
    let weak = window.as_weak();
    timer.start(TimerMode::SingleShot, Duration::from_millis(wait_ms.max(1)), move || {
        let Some(strong) = weak.upgrade() else {
            return;
        };
        HINTS.set(HINTS.get().on_tick(now_ms()));
        refresh(&strong);
        schedule(&strong);
    });
}

/// What:
/// `modifiers_changed` records the modifiers held after a key event and updates the badges.
/// Why:
/// every key press and release carries the full held set,
/// so a stale set (a release missed while the window had no
/// focus) is corrected by the next key event.
pub(crate) fn modifiers_changed(window: &AppWindow, held: HeldModifiers) {
    let before = HINTS.get();
    let after = before.on_modifiers(held, now_ms());
    if after == before {
        return;
    }
    HINTS.set(after);
    refresh(window);
    schedule(window);
}

/// What:
/// `shortcut` is the shortcut text of one action as the window's string type.
/// Why:
/// `texts_of` fills eighteen text fields the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortcut = (action: ShortcutAction, platform: KeyboardPlatform): string => shortcutText(action, platform);
/// ```
fn shortcut(action: ShortcutAction, platform: KeyboardPlatform) -> SharedString {
    return shortcut_text(action, platform).as_str().into();
}

/// What:
/// `texts_of` builds the window struct of shortcut texts for a map.
/// Why:
/// tooltips show the chord of each button,
/// written once from the same table the classifier reads.
fn texts_of(platform: KeyboardPlatform) -> ShortcutTexts {
    return ShortcutTexts {
        search: shortcut(ShortcutAction::Search, platform),
        settings: shortcut(ShortcutAction::Settings, platform),
        open_folder: shortcut(ShortcutAction::OpenFolderDialog, platform),
        folder_trigger: shortcut(ShortcutAction::Picker, platform),
        previous: shortcut(ShortcutAction::Previous, platform),
        play: shortcut(ShortcutAction::Toggle, platform),
        next: shortcut(ShortcutAction::Next, platform),
        seek_back: shortcut(ShortcutAction::SeekBack, platform),
        seek_forward: shortcut(ShortcutAction::SeekForward, platform),
        mode_repeat: shortcut(ShortcutAction::ModeRepeat, platform),
        mode_in_order: shortcut(ShortcutAction::ModeInOrder, platform),
        mode_shuffle_folder: shortcut(ShortcutAction::ModeShuffleFolder, platform),
        mode_shuffle_all: shortcut(ShortcutAction::ModeShuffleAll, platform),
        mode_cycle: shortcut(ShortcutAction::Mode, platform),
        deck_title: shortcut(ShortcutAction::Reveal, platform),
        deck_folder: shortcut(ShortcutAction::RevealFolder, platform),
        back: shortcut(ShortcutAction::Back, platform),
        editor_reset: shortcut(ShortcutAction::EditorReset, platform),
    };
}

/// What:
/// `install_texts` writes the shortcut texts of the running platform into the window.
/// Why:
/// the tooltips read them,
/// and they never change while the program runs.
pub(crate) fn install_texts(window: &AppWindow) {
    window.global::<Keys>().set_texts(texts_of(crate::ui_keys::platform()));
}
