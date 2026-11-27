//! Controls for what a command line's words are made of before a sentence shows them:
//! the URL userinfo masked and bytes that are not UTF-8 replaced,
//! and the program a command line runs cut from its first word.
//! Why:
//!  The escape for the sentence's own wrapping is not made here but where the sentence is written
//!  (`foundation/shown_so_it_cannot_rewrite_the_line.rs`),
//!  so this file holds the masking alone and the controls that would otherwise pass on it are beside
//!  the two sentences that show a name (`earlier_service_journal_adopted_tests.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(maskedWords('tool --remote https://user:s3cret@example.invalid/x')).toBe('tool --remote https://<credentials>@example.invalid/x');
//! ```

use super::*;
use crate::lfs_output_mask::CREDENTIALS_MASK;

/// A URL's userinfo is masked,
/// and text that has nothing to hide is returned as it was,
/// non-ASCII letters included.
#[test]
fn credentials_in_a_url_are_masked_and_plain_text_is_kept() {
    let shown: String =
        masked_words("tool --remote https://user:s3cret@example.invalid/x café".as_bytes());
    assert_eq!(
        shown,
        format!("tool --remote https://{CREDENTIALS_MASK}@example.invalid/x café")
    );
    assert_eq!(masked_words(b""), "");
}

/// Bytes that are not UTF-8 become the replacement character instead of failing,
/// and a secret next to them is still masked.
#[test]
fn bytes_that_are_not_utf8_are_replaced_and_a_secret_beside_them_is_masked() {
    let mut bytes: Vec<u8> = b"https://user:s3cret@host/ ".to_vec();
    bytes.push(0xff);
    let shown: String = masked_words(bytes.as_slice());
    assert_eq!(shown, format!("https://{CREDENTIALS_MASK}@host/ \u{fffd}"));
}

/// The masking is not an escape:
/// a control character of another program's text is handed on as it is,
/// and the sentence that shows it is what writes it out.
#[test]
fn a_control_character_is_handed_on_for_the_sentence_to_escape() {
    let shown: String = masked_words(b"a\nb\x1b[31m");
    assert_eq!(shown, "a\nb\u{1b}[31m");
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
    assert_eq!(
        program_of(Some(b"nginx: master process --key s3cret")),
        "nginx:"
    );
    assert_eq!(program_of(Some(b"")), "");
    assert_eq!(program_of(None), "");
    assert_eq!(
        program_of(Some(b"/tmp/a\nb")),
        "/tmp/a",
        "a newline is white space and ends the program"
    );
}
