// What:     Unit tests for `output_linux.rs`, pulled in by `#[cfg(test)] #[path = "output_linux_tests.rs"] mod tests;`.
// Why:      The backend choice itself needs a sound server or its absence, which a unit test cannot control, so the
//           Alpine container run checks that. These tests cover the pure message wording.

use super::*;

#[test]
fn the_combined_error_names_both_backends_and_both_reasons() {
    let pipewire = PlayerError::Audio("core connect: CreationFailed".to_string());
    let alsa = PlayerError::Audio("no default output device".to_string());
    let message = both_failed_message(&pipewire, &alsa);
    assert!(message.starts_with("PipeWire: "), "PipeWire comes first: {message}");
    assert!(message.contains("core connect: CreationFailed"), "the PipeWire reason is kept: {message}");
    assert!(message.contains("; ALSA: "), "the ALSA part is labelled: {message}");
    assert!(message.contains("no default output device"), "the ALSA reason is kept: {message}");
}
