// What:     Unit tests for `ui_notices.rs`, pulled in by `#[cfg(test)] #[path = "ui_notices_tests.rs"] mod tests;`.
// Why:      The message names the file and avoids developer words.

use super::*;

#[test]
fn a_removed_playing_file_says_playback_stopped() {
    assert_eq!(file_gone_message(Path::new("/lib/Tiny/02.wav"), true), "02.wav was removed from the folder, so playback stopped.");
}

#[test]
fn a_removed_idle_file_does_not_claim_a_stop() {
    assert_eq!(file_gone_message(Path::new("/lib/Tiny/02.wav"), false), "02.wav was removed from the folder.");
}

#[test]
fn a_path_without_a_file_name_is_shown_whole() {
    assert_eq!(file_gone_message(Path::new("/"), false), "/ was removed from the folder.");
}

#[test]
fn the_missing_audio_message_names_the_cause_and_both_ways_to_fix_it() {
    let message = audio_unavailable_message();
    assert!(message.contains("No sound output"), "the cause is stated: {message}");
    assert!(message.contains("PipeWire"), "the sound server option is named: {message}");
    assert!(message.contains("ALSA"), "the sound card option is named: {message}");
    assert!(message.contains("restart"), "the person is told when it takes effect: {message}");
}
