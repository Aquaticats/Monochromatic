//! Playback-mode conversion between the engine enum and the Slint integer.

/// What:
///      `PlaybackMode` is the engine's four-state enum.
/// Why:
///       Slint carries an integer while commands carry the named Rust value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { PlaybackMode } from "./command";
/// ```
use music_player::command::PlaybackMode;

/// Converts a named mode to the stable Slint integer order.
pub(crate) fn playback_mode_to_int(mode: PlaybackMode) -> i32 {
    if mode == PlaybackMode::Repeat {
        return 0;
    }
    if mode == PlaybackMode::InOrder {
        return 1;
    }
    if mode == PlaybackMode::ShufflePage {
        return 2;
    }
    return 3;
}

/// Converts a Slint integer to a mode with In order as the defensive default.
pub(crate) fn int_to_playback_mode(value: i32) -> PlaybackMode {
    if value == 0 {
        return PlaybackMode::Repeat;
    }
    if value == 2 {
        return PlaybackMode::ShufflePage;
    }
    if value == 3 {
        return PlaybackMode::ShuffleAll;
    }
    return PlaybackMode::InOrder;
}
