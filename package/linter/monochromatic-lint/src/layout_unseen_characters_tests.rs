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
        for neighbour in [
            char::from_u32(*from as u32 - 1),
            char::from_u32(*to as u32 + 1),
        ] {
            let Some(letter) = neighbour else {
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
        '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}',
        '\u{2066}', '\u{2069}', '\u{180e}', '\u{fe0f}', '\u{34f}', '\u{7f}',
    ] {
        assert!(
            prints_as_nothing(letter),
            "U+{:04X} prints as nothing",
            letter as u32
        );
    }
    // What: characters a reader sees, including one that looks like the mark beside it.
    // Why: a combining accent a reader sees is no format character, and a space between two words is no fault.
    for letter in [
        'a', '\u{301}', ' ', '\t', '\u{a0}', '\u{e9}', '\u{4e00}', '\u{378}',
    ] {
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

/// The first and last code point of every range, an interior code point of it, and the code points just outside each end,
///  written out one by one rather than read from the table the answer is built from.
/// Why:
///  A control that walks `DEFAULT_IGNORABLE_CODE_POINTS` itself cannot see an edit to that table:
///  deleting a range, cutting an end, splitting a range in two or dropping a range leaves every assertion it makes true.
///  The code points below were taken from the Unicode 17.0.0 data files on 2026-11-27:
///  `DerivedCoreProperties-17.0.0.txt` (property `Default_Ignorable_Code_Point`, date 2025-07-30),
///  `DerivedGeneralCategory-17.0.0.txt` (`Cf` and `Cc`) and `PropList-17.0.0.txt` (`White_Space`),
///  all three from https://www.unicode.org/Public/17.0.0/ucd/,
///  taking the property's code points and leaving out every `Cf`, `Cc` and whitespace one.
///  The code point before the first end is the one the general categories give it:
///  U+FDFF stands before the variation selectors, and U+FDFE is two away from them.
#[test]
fn every_range_end_an_interior_point_and_the_code_point_just_outside_it_is_written_out() {
    // What: each range as (first, interior, last, the code point before it, the code point after it),
    // with no interior point for a range of one or two code points, whose ends are all of it.
    // Why: the five literals are the control; a table edit cannot change what this list asserts.
    for (first, interior, last, before, after) in [
        ('\u{34f}', None, '\u{34f}', '\u{34e}', '\u{350}'),
        ('\u{115f}', None, '\u{1160}', '\u{115e}', '\u{1161}'),
        ('\u{17b4}', None, '\u{17b5}', '\u{17b3}', '\u{17b6}'),
        (
            '\u{180b}',
            Some('\u{180c}'),
            '\u{180d}',
            '\u{180a}',
            '\u{180e}',
        ),
        ('\u{180f}', None, '\u{180f}', '\u{180e}', '\u{1810}'),
        ('\u{2065}', None, '\u{2065}', '\u{2064}', '\u{2066}'),
        ('\u{3164}', None, '\u{3164}', '\u{3163}', '\u{3165}'),
        (
            '\u{fe00}',
            Some('\u{fe07}'),
            '\u{fe0f}',
            '\u{fdff}',
            '\u{fe10}',
        ),
        ('\u{ffa0}', None, '\u{ffa0}', '\u{ff9f}', '\u{ffa1}'),
        (
            '\u{fff0}',
            Some('\u{fff4}'),
            '\u{fff8}',
            '\u{ffef}',
            '\u{fff9}',
        ),
        ('\u{e0000}', None, '\u{e0000}', '\u{dffff}', '\u{e0001}'),
        (
            '\u{e0002}',
            Some('\u{e0010}'),
            '\u{e001f}',
            '\u{e0001}',
            '\u{e0020}',
        ),
        (
            '\u{e0080}',
            Some('\u{e0500}'),
            '\u{e0fff}',
            '\u{e007f}',
            '\u{e1000}',
        ),
    ] {
        // What: both ends of the range, which the table must answer as printing as nothing.
        // Why: a cut end or a deleted range leaves one of these answered as a visible character.
        for end in [first, last] {
            assert!(
                prints_as_nothing(end),
                "U+{:04X} is the end of a range and must print as nothing",
                end as u32
            );
        }
        // What: a code point inside the range that is neither end.
        // Why: a range split in two around this point answers it as a visible character although the data file names it.
        if let Some(inside) = interior {
            assert!(
                prints_as_nothing(inside),
                "U+{:04X} stands inside a range and must print as nothing",
                inside as u32
            );
        }
        // What: the two code points just outside the range, whose answer must be the category's alone.
        // Why: a lookup that reached one character too far would pass the ends above and spell a name a reader reads.
        for neighbour in [before, after] {
            assert_eq!(
                prints_as_nothing(neighbour),
                is_format_or_control_of_the_linked_tables(neighbour),
                "U+{:04X} stands just outside U+{:04X}..U+{:04X} and its answer is the category's alone",
                neighbour as u32,
                first as u32,
                last as u32
            );
        }
    }
}

/// The blank cell of the Braille block has a glyph of its own,
///  so it is no character a reader cannot see,
///  and the set's three properties say so: it is `So`, and `Default_Ignorable_Code_Point` does not name it.
///  A README that listed it among the characters a message spells would be wrong,
///  and the reason it is left as written stands in the README beside that list.
#[test]
fn a_braille_blank_cell_prints_as_a_character_a_reader_can_see() {
    assert!(
        !prints_as_nothing('\u{2800}'),
        "U+2800 is a symbol with a glyph, not a character a message spells"
    );
    // What: the control, the code points on either side of it that the set does and does not name.
    // Why: an answer of `false` for every character would pass the line above and spell nothing.
    assert!(!prints_as_nothing('\u{2801}'));
    assert!(prints_as_nothing('\u{fe0f}'));
}
