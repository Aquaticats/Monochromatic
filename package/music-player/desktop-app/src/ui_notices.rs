//! Plain-language message for an event the user did not ask about: a file that left the folder.
//!
//! What: a pure function that turns an engine report into the one or two lines a message shows.
//! Why: D83 says a failure is explained in a floating message that names the file and fits two lines; keeping the words
//! apart from the window code lets tests read them without a window.
//! The report of files that cannot be played is no longer a message in the window (D122):
//! its words are in `unplayable_notification_logic` and it goes to the desktop as a system notification.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fileGoneMessage = (path: string, wasPlaying: boolean) => `${basename(path)} was removed from the folder.`;
//! ```

use std::path::Path;

use music_player::unplayable_notification_logic::file_name_text;

/// What: `file_gone_message` words the report that the selected track's file left the folder.
/// Why: the row disappears and playback stops, so the message says which file and, when it was playing, that playback
/// stopped because of it (PB2).
pub(crate) fn file_gone_message(path: &Path, was_playing: bool) -> String {
    let name: String = file_name_text(path);
    if was_playing {
        return format!("{name} was removed from the folder, so playback stopped.");
    }
    return format!("{name} was removed from the folder.");
}

/// What: `audio_unavailable_message` words the report that no audio output could be opened.
/// Why: the player keeps working without sound, so the person needs the cause and what to start; the wording
/// names both Linux sound systems the player can use and avoids developer words.
pub(crate) fn audio_unavailable_message() -> &'static str {
    return "No sound output is available, so nothing will be heard. Start a sound server such as PipeWire, or set up an ALSA sound card, then restart the player.";
}

#[cfg(test)]
#[path = "ui_notices_tests.rs"]
mod tests;
