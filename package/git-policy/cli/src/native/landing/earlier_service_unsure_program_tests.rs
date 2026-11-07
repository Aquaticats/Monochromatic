//! Controls for the printable form of a command line that names a program in a message.

use super::*;
use crate::lfs_output_mask::CREDENTIALS_MASK;

/// A URL's userinfo is masked,
/// and text that has nothing to hide is returned as it was,
/// non-ASCII letters included.
#[test]
fn credentials_in_a_url_are_masked_and_plain_text_is_kept() {
    let shown: String = printable("tool --remote https://user:s3cret@example.invalid/x café".as_bytes());
    assert_eq!(shown, format!("tool --remote https://{CREDENTIALS_MASK}@example.invalid/x café"));
    assert_eq!(printable(b""), "");
}

/// A newline,
/// a tab and an escape byte are written as escapes,
/// so the text stays on one line and starts no terminal sequence.
#[test]
fn control_characters_are_written_as_escapes() {
    let shown: String = printable(b"a\nb\tc\x1b[31m");
    assert_eq!(shown, "a\\nb\\tc\\u{1b}[31m");
    assert!(shown.chars().all(|character| { return !character.is_control(); }), "{shown}");
}

/// Bytes that are not UTF-8 become the replacement character instead of failing,
/// and a secret next to them is still masked.
#[test]
fn bytes_that_are_not_utf8_are_replaced_and_a_secret_beside_them_is_masked() {
    let mut bytes: Vec<u8> = b"https://user:s3cret@host/ ".to_vec();
    bytes.push(0xff);
    let shown: String = printable(bytes.as_slice());
    assert_eq!(shown, format!("https://{CREDENTIALS_MASK}@host/ \u{fffd}"));
}

/// The program is the first word:
/// a path stays whole,
/// a bare name stays as it is,
/// a rewritten title is cut at its first space,
///  nothing at all gives nothing,
/// and the words that follow are never part of it.
#[test]
fn the_program_is_the_first_word_cut_at_its_first_space() {
    assert_eq!(program_of(Some(b"/usr/bin/tool")), "/usr/bin/tool");
    assert_eq!(program_of(Some(b"tool")), "tool");
    assert_eq!(program_of(Some(b"nginx: master process --key s3cret")), "nginx:");
    assert_eq!(program_of(Some(b"")), "");
    assert_eq!(program_of(None), "");
    assert_eq!(program_of(Some(b"/tmp/a\x1bb")), "/tmp/a\\u{1b}b", "a control character is escaped");
    assert_eq!(program_of(Some(b"/tmp/a\nb")), "/tmp/a", "a newline is white space and ends the program");
}
