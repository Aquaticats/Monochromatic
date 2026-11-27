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
