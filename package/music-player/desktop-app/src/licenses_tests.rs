// What:     Unit tests for `licenses.rs`: the embedded text and the printer's three outcomes (written, closed pipe,
//           other error).
// Why:      `--licenses` is the only way a user of the single Windows executable reaches the notices, so its output
//           and its exit behavior are part of the release promise.

use super::*;

// What:     A sink that fails every write with one fixed error kind.
// Why:      Drives the printer's error branches without a real pipe or file.
struct FailingSink {
    kind: ErrorKind,
}

impl Write for FailingSink {
    fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
        return Err(std::io::Error::from(self.kind));
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

// What:     The embedded text is either the release notices or the text a build without them carries.
// Why:      Both are legitimate builds; anything else means build.rs embedded the wrong file.
#[test]
fn embedded_text_is_the_notices_or_the_note_about_them() {
    let release = NOTICES.contains("license notices") && NOTICES.contains("Rust crates under");
    let development = NOTICES.contains("MUSIC_PLAYER_THIRD_PARTY_LICENSES");
    assert!(release || development, "unexpected embedded text: {}", &NOTICES[..NOTICES.len().min(400)]);
}

// What:     When the build embedded the release notices, they name the crates the release depends on.
// Why:      A positive control on content: Slint, symphonia and twox-hash are compiled into every desktop binary.
#[test]
fn release_notices_name_known_crates() {
    if !NOTICES.contains("Rust crates under") {
        return;
    }
    for name in ["\nslint ", "\nsymphonia ", "\ntwox-hash ", "\nturso_core "] {
        assert!(NOTICES.contains(name), "the notices do not name{name}");
    }
}

// What:     The printer writes exactly the embedded text.
// Why:      `--licenses` must print the notices byte for byte, nothing added or cut.
#[test]
fn write_notices_writes_the_whole_text() {
    let mut buffer: Vec<u8> = Vec::new();
    write_notices(&mut buffer).expect("writing to a buffer succeeds");
    assert_eq!(buffer, NOTICES.as_bytes());
}

// What:     A closed pipe ends the listing without an error.
// Why:      `music-player --licenses | head` must exit 0, not report a failure.
#[test]
fn write_notices_treats_a_closed_pipe_as_done() {
    let mut sink = FailingSink { kind: ErrorKind::BrokenPipe };
    assert!(write_notices(&mut sink).is_ok());
}

// What:     Any other write failure is returned.
// Why:      A full disk or a revoked handle must not look like a successful listing.
#[test]
fn write_notices_returns_other_errors() {
    let mut sink = FailingSink { kind: ErrorKind::PermissionDenied };
    let error = write_notices(&mut sink).expect_err("a denied write is an error");
    assert_eq!(error.kind(), ErrorKind::PermissionDenied);
}

// What:     When the build embedded the release notices, they carry the Slint license text and name every Slint crate.
// Why:      The desktop programs use Slint under its Royalty-free license, which is met by the AboutSlint widget in the
//           Licenses page and by shipping the license text with the program; GPL-3.0-only is no longer the license in use.
#[test]
fn release_notices_carry_the_slint_royalty_free_license() {
    if !NOTICES.contains("Rust crates under") {
        return;
    }
    assert!(NOTICES.contains("Slint Royalty-free Desktop, Mobile, and Web Applications License"), "the Slint license title");
    assert!(NOTICES.contains("Display the [`AboutSlint`]"), "the attribution condition of section 2");
    for name in ["\nslint 1.", "\nslint-macros 1.", "\ni-slint-core 1.", "\ni-slint-backend-winit 1."] {
        assert!(NOTICES.contains(name), "the Slint section does not name{name}");
    }
    assert!(!NOTICES.contains("(GPL-3.0-only)"), "the notices no longer say the program is used under GPL-3.0-only");
}
