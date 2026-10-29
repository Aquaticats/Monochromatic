//! Pixel tab-stop arithmetic and the one-byte tab projection,
//!  independent of fonts.

/// What:
///  Import the production tab arithmetic and projection through the library's public interface.
/// Why:
///  The shaper calls exactly these functions;
///  the tests must not use a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TAB_SPACES, tabAdvance, tabSpacings } from 'ide-app/tab-stop';
/// import { projectLine } from 'ide-app/text-projection';
/// ```
use ide_app::{
    tab_stop::{TAB_SPACES, tab_advance, tab_spacings},
    text_projection::project_line,
};

/// What:
///  `const` names a compile-time value;
///  `f32` is a 32-bit float (sibling `f64`).
/// Why:
///  Nine pixels is the space advance of the 15 px source font;
///  any positive value exercises the rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SPACE = 9;
/// ```
const SPACE: f32 = 9.0;

/// Stops lie two space advances apart,
///  matching the reference's `tab-size: 2`.
#[test]
fn stops_are_two_space_advances_apart() {
    assert_eq!(TAB_SPACES, 2.0);
    assert_eq!(tab_advance(0.0, SPACE), 18.0, "a leading tab");
    assert_eq!(tab_advance(9.0, SPACE), 9.0, "after one Latin letter");
    assert_eq!(tab_advance(18.0, SPACE), 18.0, "exactly on a stop");
    assert_eq!(tab_advance(27.0, SPACE), 9.0);
}

/// A tab that would be narrower than half a space runs on to the following stop.
#[test]
fn tab_narrower_than_half_a_space_uses_the_following_stop() {
    // A 15 px CJK glyph ends 3 px before the stop at 18 px; 3 px is less than half of 9 px.
    assert_eq!(
        tab_advance(15.0, SPACE),
        21.0,
        "a sliver tab was kept instead of the next stop"
    );
    assert_eq!(
        tab_advance(13.5, SPACE),
        4.5,
        "exactly half a space is wide enough"
    );
    let sliver = tab_advance(13.6, SPACE);
    assert!(
        (sliver - 22.4).abs() < 0.001,
        "a 4.4 px sliver must run on to the following stop, got {sliver}"
    );
    // Rounding noise just before a stop must not produce a zero-width tab.
    let nearly = tab_advance(17.999_998, SPACE);
    assert!(
        (nearly - 18.0).abs() < 0.001,
        "noise before a stop gave {nearly}"
    );
}

/// Positions measured in pixels,
///  not columns,
///  decide the stop:
///  mixed-script prefixes share stops.
#[test]
fn different_prefix_widths_reach_the_same_stop() {
    // Two Latin letters (18 px), one 15 px CJK glyph, and a 15 px glyph plus a Latin letter (24 px).
    for prefix in [18.0_f32, 15.0, 24.0] {
        let end = prefix + tab_advance(prefix, SPACE);
        assert_eq!(end, 36.0, "a tab after a {prefix} px prefix ended at {end}");
    }
}

/// A space advance that is zero,
///  negative,
///  or not a number has no stops and cannot loop or divide by zero.
#[test]
fn unusable_space_advance_keeps_one_space_width() {
    assert_eq!(tab_advance(5.0, 0.0), 0.0);
    assert_eq!(tab_advance(5.0, -3.0), -3.0);
    assert!(tab_advance(5.0, f32::NAN).is_nan());
    assert_eq!(tab_advance(f32::INFINITY, SPACE), SPACE);
    assert_eq!(tab_spacings(&[0.0, 9.0], 0.0), [0.0, 0.0]);
}

/// Widening an earlier tab moves the later ones,
///  so corrections accumulate from left to right.
#[test]
fn corrections_accumulate_across_the_tabs_of_one_line() {
    // "\t\tx": drawn as spaces the tabs start at 0 and 9; each must become 18 px wide.
    assert_eq!(tab_spacings(&[0.0, SPACE], SPACE), [9.0, 9.0]);
    // "a\tb\tc": tabs start at 9 and 27 as spaces; the first already ends on 18, the second follows "b" at 27.
    assert_eq!(tab_spacings(&[9.0, 27.0], SPACE), [0.0, 0.0]);
    // "abc\td\te": the first tab at 27 ends on 36 (width 9); "d" ends at 45, the second tab ends on 54.
    assert_eq!(tab_spacings(&[27.0, 45.0], SPACE), [0.0, 0.0]);
    // A 15 px glyph, then two tabs: the first is 21 px (extra 12), which moves the second from 24 to 36.
    assert_eq!(tab_spacings(&[15.0, 24.0], SPACE), [12.0, 9.0]);
    assert!(tab_spacings(&[], SPACE).is_empty());
}

/// Each source tab is one display space;
///  the maps stay exact around wide and astral characters.
#[test]
fn projection_gives_each_tab_one_display_byte() {
    let projection = project_line("\ta\t猫\t𝒳\tb\r\n");
    assert_eq!(
        projection.text, " a 猫 𝒳 b",
        "terminators are not displayed"
    );
    assert_eq!(projection.tabs, [0, 2, 6, 11]);
    // Source boundaries: tab, a, tab, 猫 (3 bytes), tab, 𝒳 (4 bytes), tab, b.
    assert_eq!(projection.source_to_byte, [0, 1, 2, 3, 6, 7, 11, 12, 13]);
    // Byte boundaries inside a multi-byte character map back to that character's start.
    assert_eq!(
        projection.byte_to_source,
        [0, 1, 2, 3, 3, 3, 4, 5, 5, 5, 5, 6, 7, 8]
    );
    assert_eq!(projection.stops, [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    let plain = project_line("no tabs");
    assert!(plain.tabs.is_empty());
    assert_eq!(plain.text, "no tabs");
}

/// Caret stops are grapheme boundaries of the visible text:
///  no stop inside a sequence or a terminator.
#[test]
fn projection_lists_grapheme_boundaries_as_caret_stops() {
    // e + acute (0..2), x (2), a joined family of five characters (3..8), a two-character flag (8..10), CRLF.
    let projection = project_line("e\u{301}x👩\u{200d}👩\u{200d}👧🇯🇵\r\n");
    assert_eq!(
        projection.stops,
        [0, 2, 3, 8, 10],
        "a caret stop splits a combining sequence, a joined emoji, a flag, or CRLF"
    );
    assert_eq!(project_line("").stops, [0]);
    assert_eq!(project_line("\n").stops, [0]);
}
