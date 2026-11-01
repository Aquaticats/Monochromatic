//! Key hint badges and shortcut texts on the window (D106).
//!
//! What: `modifiers_changed` feeds the held modifiers to the pure `KeyHints` state, starts a one-shot timer for the
//! 300 ms hold, and writes the badge of every control into the window's `Keys` global; `install_texts` writes the
//! shortcut text of every control once, for the tooltips.
//! Why: the timing and narrowing rules are tested in `key_hints`; this file only connects them to the clock, to a Slint
//! timer, and to the window.

use std::cell::Cell;
use std::time::{Duration, Instant};

use music_player::key_hints::{HintTarget, KeyHints, badge_text};
use music_player::keymap::{KeyboardPlatform, ShortcutAction};
use music_player::keymap_bindings::{HeldModifiers, shortcut_text};
use slint::{ComponentHandle, Timer, TimerMode};

use crate::{AppWindow, KeyBadges, Keys, ShortcutTexts};

thread_local! {
    /// What: the badge state of this window thread.
    /// Why: the key handler and the timer both read and write it on the event-loop thread only.
    static HINTS: Cell<KeyHints> = Cell::new(KeyHints::default());
    /// What: the one-shot timer that shows the badges after the hold time.
    /// Why: a Slint timer runs on the event loop, so the badges appear even when no key event arrives.
    static HOLD_TIMER: Timer = Timer::default();
    /// What: the moment the hint clock started.
    /// Why: `KeyHints` takes millisecond timestamps, so every call uses this origin.
    static CLOCK_START: Instant = Instant::now();
}

#[cfg(test)]
thread_local! {
    /// What: the clock a window test sets, in milliseconds.
    /// Why: the tests move Slint's timers with a mock clock, so the hint state and the double tap of Shift must read a
    /// clock that moves with it.
    static TEST_CLOCK_MS: Cell<Option<u64>> = const { Cell::new(None) };
}

/// What: `set_test_clock_ms` makes `now_ms` return a value the test controls.
/// Why: see `TEST_CLOCK_MS`.
#[cfg(test)]
pub(crate) fn set_test_clock_ms(milliseconds: u64) {
    TEST_CLOCK_MS.set(Some(milliseconds));
}

/// What: `now_ms` gives the hint clock in milliseconds since the first call.
/// Why: the state compares timestamps, so the origin only has to be stable.
pub(crate) fn now_ms() -> u64 {
    #[cfg(test)]
    if let Some(milliseconds) = TEST_CLOCK_MS.get() {
        return milliseconds;
    }
    return CLOCK_START.with(|start| return u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX));
}

/// What: `badges_of` builds the window struct of badge texts for the modifiers that are showing.
/// Why: one value feeds every control's badge, and an empty text draws none.
fn badges_of(platform: KeyboardPlatform, visible: Option<HeldModifiers>) -> KeyBadges {
    let text = |target: HintTarget| return badge_text(target, platform, visible).as_str().into();
    return KeyBadges {
        search: text(HintTarget::Search),
        settings: text(HintTarget::Settings),
        open_folder: text(HintTarget::OpenFolder),
        folder_trigger: text(HintTarget::FolderTrigger),
        folder_list: text(HintTarget::FolderList),
        track_list: text(HintTarget::TrackList),
        previous: text(HintTarget::Previous),
        next: text(HintTarget::Next),
        seek_back: text(HintTarget::SeekBack),
        seek_forward: text(HintTarget::SeekForward),
        mode_repeat: text(HintTarget::ModeRepeat),
        mode_in_order: text(HintTarget::ModeInOrder),
        mode_shuffle_folder: text(HintTarget::ModeShuffleFolder),
        mode_shuffle_all: text(HintTarget::ModeShuffleAll),
        mode_cycle: text(HintTarget::ModeCycle),
        deck_title: text(HintTarget::DeckTitle),
        deck_folder: text(HintTarget::DeckFolder),
        editor_reset: text(HintTarget::EditorReset),
    };
}

/// What: `refresh` writes the badges the current state shows into the window.
/// Why: every change of the state ends here, so the window never disagrees with the state.
fn refresh(window: &AppWindow) {
    let visible = HINTS.with(|cell| return cell.get().visible());
    window.global::<Keys>().set_badges(badges_of(crate::ui_keys::platform(), visible));
    tracing::debug!(visible = ?visible, "key hint badges updated");
    crate::ui_state_log::emit(window, "badges");
}

/// What: `schedule` starts the one-shot timer that shows the badges when the hold time has passed.
/// Why: the badges must appear without another key event, and a new held set restarts the wait.
fn schedule(window: &AppWindow) {
    let wait = HINTS.with(|cell| return cell.get().wait_ms(now_ms()));
    HOLD_TIMER.with(|timer| {
        timer.stop();
        let Some(wait_ms) = wait else {
            return;
        };
        let weak = window.as_weak();
        timer.start(TimerMode::SingleShot, Duration::from_millis(wait_ms.max(1)), move || {
            let Some(strong) = weak.upgrade() else {
                return;
            };
            HINTS.with(|cell| cell.set(cell.get().on_tick(now_ms())));
            refresh(&strong);
            schedule(&strong);
        });
    });
}

/// What: `modifiers_changed` records the modifiers held after a key event and updates the badges.
/// Why: every key press and release carries the full held set, so a stale set (a release missed while the window had no
/// focus) is corrected by the next key event.
pub(crate) fn modifiers_changed(window: &AppWindow, held: HeldModifiers) {
    let before = HINTS.with(|cell| return cell.get());
    let after = before.on_modifiers(held, now_ms());
    if after == before {
        return;
    }
    HINTS.with(|cell| cell.set(after));
    refresh(window);
    schedule(window);
}

/// What: `texts_of` builds the window struct of shortcut texts for a map.
/// Why: tooltips show the chord of each button, written once from the same table the classifier reads.
fn texts_of(platform: KeyboardPlatform) -> ShortcutTexts {
    let text = |action: ShortcutAction| return shortcut_text(action, platform).as_str().into();
    return ShortcutTexts {
        search: text(ShortcutAction::Search),
        settings: text(ShortcutAction::Settings),
        open_folder: text(ShortcutAction::OpenFolderDialog),
        folder_trigger: text(ShortcutAction::Picker),
        previous: text(ShortcutAction::Previous),
        play: text(ShortcutAction::Toggle),
        next: text(ShortcutAction::Next),
        seek_back: text(ShortcutAction::SeekBack),
        seek_forward: text(ShortcutAction::SeekForward),
        mode_repeat: text(ShortcutAction::ModeRepeat),
        mode_in_order: text(ShortcutAction::ModeInOrder),
        mode_shuffle_folder: text(ShortcutAction::ModeShuffleFolder),
        mode_shuffle_all: text(ShortcutAction::ModeShuffleAll),
        mode_cycle: text(ShortcutAction::Mode),
        deck_title: text(ShortcutAction::Reveal),
        deck_folder: text(ShortcutAction::RevealFolder),
        back: text(ShortcutAction::Back),
        editor_reset: text(ShortcutAction::EditorReset),
    };
}

/// What: `install_texts` writes the shortcut texts of the running platform into the window.
/// Why: the tooltips read them, and they never change while the program runs.
pub(crate) fn install_texts(window: &AppWindow) {
    window.global::<Keys>().set_texts(texts_of(crate::ui_keys::platform()));
}
