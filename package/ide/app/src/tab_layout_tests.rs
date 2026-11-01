//! The work the tab correction of a wrapped line may do:
//!  the passes a line gets fall with its length,
//! so the characters shaped stay near a fixed budget,
//!  and the count of passes a long line really shaped is checked against that bound.
//! Work is counted in passes,
//! never timed,
//! so the checks do not depend on the host's load.

/// The bound under test.
use super::{TAB_BUDGET, TAB_MIN_PASSES, TAB_PASSES, passes_for};
/// Canonical source and the shaper whose passes are counted.
use crate::{document::Document, shaped_text::TextShaper};

/// A line of about `length` characters with a tab every few characters,
/// which keeps correcting for many passes.
fn tabbed(length: usize) -> String {
    let mut text = String::new();
    let mut number = 0;
    while text.len() < length {
        text.push_str(&format!("w{number}\tx "));
        number += 1;
    }
    text.truncate(length);
    return text;
}

/// A short line gets every pass,
/// a long one the budget's share,
/// and no line fewer than the minimum.
#[test]
fn passes_fall_with_the_length_of_the_line() {
    assert_eq!(passes_for(0), TAB_PASSES);
    assert_eq!(passes_for(3_000), TAB_PASSES);
    assert_eq!(passes_for(TAB_BUDGET / TAB_PASSES), TAB_PASSES);
    assert_eq!(passes_for(50_000), TAB_BUDGET / 50_000);
    assert_eq!(passes_for(100_000), 10);
    assert_eq!(passes_for(10_000_000), TAB_MIN_PASSES);
    let mut length = 1;
    let mut earlier = passes_for(length);
    while length < 20_000_000 {
        length *= 3;
        let now = passes_for(length);
        assert!(now <= earlier, "a longer line got more passes at {length}");
        earlier = now;
    }
}

/// A long line with tabs inside shapes no more corrective passes than its length allows,
/// and a short one is not
///  held back:
/// the count of shaped passes is the bound's,
/// in the same units.
#[test]
fn a_long_tabbed_line_shapes_no_more_passes_than_its_length_allows() {
    let mut shaper = TextShaper::new();
    let long = tabbed(50_000);
    let allowed = passes_for(long.len());
    assert!(
        allowed < TAB_PASSES,
        "the fixture is long enough to be limited"
    );
    let before = shaper.tab_passes();
    let document = Document::new(&long);
    let row = shaper.wrapped_row(&document, 0, 1.0, 782.0);
    assert!(row.visual_rows() > 100, "the line wraps to many rows");
    let shaped = shaper.tab_passes() - before;
    assert!(
        shaped as usize <= allowed,
        "the line shaped {shaped} corrective passes, more than the {allowed} its length allows"
    );
    assert!(
        shaped as usize >= allowed,
        "the fixture settled after {shaped} passes, so it does not test the bound ({allowed})"
    );
    // A short line of the same pattern is held to the full count, and not below what it needs.
    let short = tabbed(2_000);
    let earlier = shaper.tab_passes();
    let short_document = Document::new(&short);
    shaper.wrapped_row(&short_document, 0, 1.0, 782.0);
    let used = shaper.tab_passes() - earlier;
    assert!(
        used > 4 && used as usize <= TAB_PASSES,
        "a short line shaped {used} corrective passes"
    );
}
