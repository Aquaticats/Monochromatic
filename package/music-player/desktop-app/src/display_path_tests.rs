// What:     Unit tests for `display_path.rs`, pulled in by
//           `#[cfg(test)] #[path = "display_path_tests.rs"] mod tests;` at the bottom of
//           `display_path.rs`. Cases mirror the `sanitizeComponent` and `joinDisplayPath`
//           cases of `DisplayPathTest.kt`. Reaches the module items (including private ones)
//           via `use super::*` because this file is the `tests` CHILD of display_path.
// Why:      Keep the tests beside the code without inflating `display_path.rs` or its
//           max-lines budget (sibling `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module (this file) into the
//           test scope. `super` means "one level up".
// Why:      Tests need `sanitize_component` and `join_display_path`.
use super::*;

// What:     `#[test]` marks the next function as a test case.
// Why:      A separator inside a name becomes the U+2215 look-alike, so it adds no folder level.
#[test]
fn separator_in_name_becomes_division_slash() {
    assert_eq!(sanitize_component("AC/DC - song.flac"), "AC\u{2215}DC - song.flac");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Newline, carriage return and tab each collapse to one space, so the path stays on one line.
#[test]
fn control_characters_become_spaces() {
    assert_eq!(sanitize_component("a\nb\tc"), "a b c");
    assert_eq!(sanitize_component("side a\rside b"), "side a side b");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Ordinary names, including accented Unicode, pass through unchanged.
#[test]
fn ordinary_names_pass_through_unchanged() {
    assert_eq!(sanitize_component("Café del Mar"), "Café del Mar");
    assert_eq!(sanitize_component("track [01].opus"), "track [01].opus");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A name joined under an empty prefix returns the bare sanitized segment.
#[test]
fn join_under_empty_prefix_gives_bare_segment() {
    assert_eq!(join_display_path("", "loose.flac"), "loose.flac");
    assert_eq!(join_display_path("", "a\nb\tc"), "a b c");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A name joins under a prefix with exactly one separator.
#[test]
fn join_appends_name_under_prefix() {
    assert_eq!(join_display_path("Artist/Album", "01 song.flac"), "Artist/Album/01 song.flac");
    assert_eq!(join_display_path("Rock", "AC/DC - song.flac"), "Rock/AC\u{2215}DC - song.flac");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A `..` name is kept as text, because only separators and control characters change.
#[test]
fn parent_directory_name_is_kept_as_text() {
    assert_eq!(join_display_path("Music", ".."), "Music/..");
}
