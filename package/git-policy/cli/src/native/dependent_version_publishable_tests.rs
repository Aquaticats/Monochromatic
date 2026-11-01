//! What: Publishable-name reading, with every `readPublishableNames` case of
//!       `dependent-version-bump-policy.unit.test.ts` and JavaScript's trim set.
//! Why: A name missing from the list is never bumped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(readPublishableNames("packages:\n  - '@s/a'")).toEqual(['@s/a']);
//! ```

/// The reader under test.
use super::read_publishable_names;

/// Read names as borrowed text for comparison.
fn read(text: &str) -> Vec<String> {
    return read_publishable_names(text);
}

/// Ported: the list is read up to the first line that is not an item.
#[test]
fn reads_the_list_and_stops_at_the_first_non_item_line() {
    assert_eq!(
        read("a: 1\n  packages:\n    - '@s/a'\n    - '@s/b'\n  other: x\n    - '@s/c'\n"),
        ["@s/a", "@s/b"]
    );
    assert_eq!(read("packages:\n  - '@s/a'"), ["@s/a"]);
    assert!(read("no list here\n").is_empty());
    assert!(read("packages:\n  - ''\n").is_empty());
}

/// Only the first key line opens the list; an item needs both quotes and a name.
#[test]
fn reads_items_after_the_first_key_only() {
    assert!(read("packages:\n  x\npackages:\n  - 'a'\n").is_empty());
    assert!(read("packages:\n  - '\n").is_empty());
    assert!(read("packages:\n  - 'a\n").is_empty());
    assert!(read("packages:\n  - a'\n").is_empty());
    assert_eq!(read("packages:\n  - '''\n  - 'a b'\n"), ["'", "a b"]);
    assert!(read("packages: x\n  - 'a'\n").is_empty());
}

/// Lines are trimmed with JavaScript's set: CR, BOM and wide spaces go, NEL stays.
#[test]
fn trims_as_javascript_does() {
    assert_eq!(read("packages:\r\n  - 'a'\r\n"), ["a"]);
    assert_eq!(read("\u{feff}packages:\n\u{3000}- 'a'\u{a0}\n"), ["a"]);
    assert_eq!(
        read(
            "packages:\n\u{b}\u{c}\u{1680}\u{2000}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}- 'a'\n"
        ),
        ["a"]
    );
    assert!(read("\u{85}packages:\n  - 'a'\n").is_empty());
    assert_eq!(read("packages:\n  - 'a'\u{85}\n"), Vec::<String>::new());
}
