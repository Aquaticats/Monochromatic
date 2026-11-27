//! What:
//!  Controls of how findings show a path.
//! Why:
//!  A path inside the package is shown relative to it,
//!  and the package folder itself must not be shown as an empty name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a path inside the base is relative', () => expect(shown('/p/src/a.rs', '/p')).toBe('src/a.rs'));
//! ```

use super::{
    key_path,
    shown,
    spelled_unreadable,
    unseen_character_name,
};
use std::path::Path;

/// A path below the base is shown without the base.
#[test]
fn layout_a_path_inside_the_package_is_shown_relative_to_it() {
    let base: &Path = Path::new("/package");
    assert_eq!(shown(&base.join("src").join("a.rs"), base), "src/a.rs");
}

/// A path outside the base is shown whole.
#[test]
fn layout_a_path_outside_the_package_is_shown_whole() {
    let base: &Path = Path::new("/package");
    assert_eq!(shown(Path::new("/elsewhere/a.rs"), base), "/elsewhere/a.rs");
}

/// The base itself is shown whole,
///  because nothing is left of it once the base is removed.
#[test]
fn layout_the_package_folder_itself_is_shown_whole() {
    let base: &Path = Path::new("/package");
    assert_eq!(shown(base, base), "/package");
}

/// A name that ends in a space spells that space even when an earlier character takes more than one byte,
///  because the last character's position is counted in characters and not in bytes.
#[test]
fn a_name_that_ends_in_a_space_after_a_wide_character_spells_it() {
    assert_eq!(spelled_unreadable("é "), "é\\u{20}");
    // What: the control, a space between two words that the message can show.
    // Why: a spelling that escaped every space would pass the line above and make every message unreadable.
    assert_eq!(spelled_unreadable("a b"), "a b");
    assert_eq!(spelled_unreadable("é\tx"), "é\\u{9}x");
}

/// A character a reader cannot see is spelled wherever a name holds it,
///  because nothing on a screen shows which word it joined.
#[test]
fn a_name_that_holds_a_character_a_reader_cannot_see_spells_it_anywhere() {
    assert_eq!(spelled_unreadable("a\u{feff}b"), "a\\u{feff}b");
    assert_eq!(spelled_unreadable("src/\u{200b}a"), "src/\\u{200b}a");
    assert_eq!(spelled_unreadable("a\u{ad}b"), "a\\u{ad}b");
    // What: the control, an ordinary name the message reads as words.
    // Why: a spelling that escaped every character would pass the lines above and make every message unreadable.
    assert_eq!(spelled_unreadable("src/ab"), "src/ab");
}

/// A key writes every path with `/` between its segments,
///  so a list written on one platform matches on another,
///  and a name that holds a backslash of its own keeps it,
///  because a file name and an entry of a `uses` folder are no paths.
#[test]
fn a_key_path_uses_forward_slashes_at_every_boundary() {
    let base: std::path::PathBuf = std::path::PathBuf::from("pkg");
    let key: String = key_path(&base.join("src/native/mod.rs"), base.as_path());
    assert_eq!(key, "src/native/mod.rs");
    assert!(!key.contains('\\'), "{key}");
    // What: a file name that holds a backslash, which the file system of this machine allows.
    // Why: a file name is no path, so the key writes it as the source does rather than as a separator.
    let named: String = key_path(&base.join("src").join("a\\b.rs"), base.as_path());
    assert_eq!(named, "src/a\\b.rs");
    // A folder outside the package keeps the path it has, still with `/` between its parts.
    let outside: String = key_path(std::path::Path::new("/elsewhere/a/b"), base.as_path());
    assert!(outside.ends_with("a/b"), "{outside}");
}

/// Every character this crate gives a plain name is named by the phrase a message shows,
///  written out one by one.
/// Why:
///  A message that called a right-to-left override "a character a reader cannot see" leaves its reader
///  no way to tell it from a zero-width space, and the two need different edits.
///  A control that asked the table for its own answers could not see an edit to a name.
#[test]
fn every_character_this_crate_names_is_named_by_the_phrase_a_message_shows() {
    for (letter, name) in [
        ('\u{feff}', "a byte order mark or zero-width no-break space (`\\u{feff}`)"),
        ('\u{200b}', "a zero-width space (`\\u{200b}`)"),
        ('\u{200c}', "a zero-width non-joiner (`\\u{200c}`)"),
        ('\u{200d}', "a zero-width joiner (`\\u{200d}`)"),
        ('\u{2060}', "a word joiner (`\\u{2060}`)"),
        ('\u{ad}', "a soft hyphen (`\\u{ad}`)"),
        ('\u{200e}', "a left-to-right mark (`\\u{200e}`)"),
        ('\u{200f}', "a right-to-left mark (`\\u{200f}`)"),
        ('\u{202a}', "a left-to-right embedding (`\\u{202a}`)"),
        ('\u{202b}', "a right-to-left embedding (`\\u{202b}`)"),
        ('\u{202c}', "a pop directional formatting (`\\u{202c}`)"),
        ('\u{202d}', "a left-to-right override (`\\u{202d}`)"),
        ('\u{202e}', "a right-to-left override (`\\u{202e}`)"),
        ('\u{2066}', "a left-to-right isolate (`\\u{2066}`)"),
        ('\u{2067}', "a right-to-left isolate (`\\u{2067}`)"),
        ('\u{2068}', "a first strong isolate (`\\u{2068}`)"),
        ('\u{2069}', "a pop directional isolate (`\\u{2069}`)"),
        ('\u{61c}', "an arabic letter mark (`\\u{61c}`)"),
        ('\u{180e}', "a mongolian vowel separator (`\\u{180e}`)"),
        ('\u{34f}', "a combining grapheme joiner (`\\u{34f}`)"),
        ('\u{7f}', "a delete character (`\\u{7f}`)"),
    ] {
        assert_eq!(
            unseen_character_name(letter),
            Some(name),
            "U+{:04X} must be called `{name}`",
            letter as u32
        );
    }
    // What: the control, a character this crate gives no name, which the sentence spells by its code point instead.
    // Why: a name table that answered every character would pass the lines above and call an unknown one by a wrong name.
    assert_eq!(unseen_character_name('\u{70f}'), None);
}
