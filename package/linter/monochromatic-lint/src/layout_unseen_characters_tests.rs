//! What:
//!  Controls of which characters a reader cannot see.
//! Why:
//!  A message that spells every character a reader cannot see is only as good as the set it takes them from:
//!  too narrow and a line a person cannot see the fault in prints raw,
//!  too wide and a name a person reads normally arrives as escapes.
//!  The set is three properties, and each control below names the property it holds rather than a code point
//!  chosen by hand, so a Unicode release that moves a character between them changes a range and no control.

use super::{
    DEFAULT_IGNORABLE_CODE_POINTS,
    prints_as_nothing,
    unicode_release,
};

/// The release of the Unicode data file the ranges were read from,
///  which `DEFAULT_IGNORABLE_CODE_POINTS` names in its own doc:
///  `DerivedCoreProperties-17.0.0.txt`, `DerivedGeneralCategory-17.0.0.txt` and `PropList-17.0.0.txt`
///  from https://www.unicode.org/Public/17.0.0/ucd/.
/// Why:
///  The ranges are spelled out for one release of that file,
///  while the categories beside them come from the tables `unicode-properties` links.
///  A dependency upgrade that changes those tables must fail here rather than let one character be missed.
const DATA_FILE_RELEASE: (u64, u64, u64) = (17, 0, 0);

/// The tables this crate links hold the release the ranges were read from,
///  so a category and a range never answer from two releases.
#[test]
fn the_linked_tables_hold_the_release_the_ranges_were_read_from() {
    assert_eq!(unicode_release(), DATA_FILE_RELEASE);
}

/// Every range is in ascending order and none overlaps the next,
///  because a lookup stops at the first range that holds the character,
///  and an unsorted table would answer nothing for half of it.
#[test]
fn every_range_is_in_order_and_none_overlaps() {
    for pair in DEFAULT_IGNORABLE_CODE_POINTS.windows(2) {
        assert!(
            pair[0].1 < pair[1].0,
            "{} and {} are out of order or overlap",
            pair[0].0 as u32,
            pair[1].0 as u32
        );
    }
    for (from, to) in DEFAULT_IGNORABLE_CODE_POINTS.iter() {
        assert!(from <= to, "{}..{} is no range", *from as u32, *to as u32);
    }
}

/// The first and last code point of every range prints as nothing,
///  which is the positive control of that range:
///  a range whose answer never reaches its last code point is a range only half read.
#[test]
fn both_ends_of_every_range_print_as_nothing() {
    for (from, to) in DEFAULT_IGNORABLE_CODE_POINTS.iter() {
        for letter in [*from, *to] {
            assert!(
                prints_as_nothing(letter),
                "U+{:04X} is in a range and must print as nothing",
                letter as u32
            );
        }
    }
}

/// A code point just outside each range is answered by another property or prints as something,
///  which is the negative control:
///  a lookup that answered every character would spell a name a person reads normally.
#[test]
fn a_code_point_outside_every_range_prints_as_something() {
    for (from, to) in DEFAULT_IGNORABLE_CODE_POINTS.iter() {
        for letter in [char::from_u32(*from as u32 - 1), char::from_u32(*to as u32 + 1)] {
            let Some(letter) = letter else {
                continue;
            };
            let in_another_range: bool = DEFAULT_IGNORABLE_CODE_POINTS
                .iter()
                .any(|(other_from, other_to)| return other_from <= &letter && &letter <= other_to);
            if in_another_range {
                continue;
            }
            assert!(
                prints_as_nothing(letter) == is_format_or_control_of_the_linked_tables(letter),
                "U+{:04X} stands just outside a range and its answer is the category's alone",
                letter as u32
            );
        }
    }
}

/// Whether the linked tables give a character the `Cf` or `Cc` category,
///  which is the second of the three properties a reader's terminal acts on.
fn is_format_or_control_of_the_linked_tables(letter: char) -> bool {
    use unicode_properties::UnicodeGeneralCategory;
    let category: unicode_properties::GeneralCategory = letter.general_category();
    return category == unicode_properties::GeneralCategory::Format
        || category == unicode_properties::GeneralCategory::Control;
}

/// A format character prints as nothing and a letter, a combining accent and a space print as themselves,
///  which is the boundary between a name a message must spell and a name it must show as written.
#[test]
fn a_format_character_is_spelled_and_a_readable_one_is_shown() {
    // What: code points a reader meets in a line copied from a word processor or a web page.
    // Why: these are the faults the release review found printed raw, so each is a case a message must spell.
    for letter in [
        '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}',
        '\u{2069}', '\u{180e}', '\u{fe0f}', '\u{34f}', '\u{7f}',
    ] {
        assert!(
            prints_as_nothing(letter),
            "U+{:04X} prints as nothing",
            letter as u32
        );
    }
    // What: characters a reader sees, including one that looks like the mark beside it.
    // Why: a combining accent a reader sees is no format character, and a space between two words is no fault.
    for letter in ['a', '\u{301}', ' ', '\t', '\u{a0}', '\u{e9}', '\u{4e00}', '\u{378}'] {
        assert!(
            !prints_as_nothing(letter),
            "U+{:04X} prints as something",
            letter as u32
        );
    }
}

/// A whitespace character is answered by the rule that names a separator and not by this one,
///  even where its category is `Cc`,
///  because a message spells a tab wherever it stands and a space only at the ends of a name.
#[test]
fn a_whitespace_character_is_no_character_of_this_set() {
    for letter in ['\u{85}', '\u{b}', '\u{c}', '\r', '\u{2028}', '\u{2029}'] {
        assert!(
            !prints_as_nothing(letter),
            "U+{:04X} is whitespace and is spelled by the separator's own rule",
            letter as u32
        );
    }
}
