//! Controls for cutting a `--zdiff3` line merge apart:
//! the settled text and the regions of an answer,
//! the shape without a common ancestor section,
//! the answers that cannot be cut,
//! and where a region sits in a stage.

use super::*;

/// A `--zdiff3` answer with two regions,
/// as Git writes one,
/// with the labels the second pass gives it.
const TWO_REGIONS: &[u8] = b"head\n<<<<<<< ours\nours one\n||||||| base\nbase one\n=======\ntheirs one\n>>>>>>> theirs\nbetween\n<<<<<<< ours\nours two\n||||||| base\n=======\ntheirs two\n>>>>>>> theirs\ntail\n";

/// Both regions are cut out with their three sides and their markers,
/// and the settled text keeps the order of the file.
#[test]
fn an_answer_is_cut_into_its_settled_text_and_its_regions() {
    let cut: Cut = cut(TWO_REGIONS).expect("a two-region answer cuts");
    assert_eq!(cut.settled, vec![b"head\n".to_vec(), b"between\n".to_vec(), b"tail\n".to_vec()]);
    assert_eq!(cut.regions.len(), 2);
    assert_eq!(cut.settled.len(), cut.regions.len() + 1);
    let first: &Region = &cut.regions[0];
    assert_eq!((first.base.as_slice(), first.ours.as_slice(), first.theirs.as_slice()), (b"base one\n".as_slice(), b"ours one\n".as_slice(), b"theirs one\n".as_slice()));
    assert_eq!(first.markers, b"<<<<<<< ours\nours one\n||||||| base\nbase one\n=======\ntheirs one\n>>>>>>> theirs\n".to_vec());
    // A region both sides inserted at one point has no common ancestor text.
    assert!(cut.regions[1].base.is_empty());
    assert_eq!(cut.regions[1].theirs, b"theirs two\n".to_vec());
}

/// An answer whose region carries no common ancestor section,
/// as `merge` style writes it,
/// has an empty base and the second side where `=======` says.
#[test]
fn a_region_without_a_common_ancestor_section_is_cut_with_an_empty_base() {
    let plain: &[u8] = b"<<<<<<< ours\none\n=======\ntwo\n>>>>>>> theirs\n";
    let cut: Cut = cut(plain).expect("an answer of two sides cuts");
    assert_eq!(cut.regions.len(), 1);
    assert!(cut.regions[0].base.is_empty());
    assert_eq!((cut.regions[0].ours.as_slice(), cut.regions[0].theirs.as_slice()), (b"one\n".as_slice(), b"two\n".as_slice()));
    assert_eq!(cut.settled, vec![Vec::<u8>::new(), Vec::new()]);
}

/// A marker out of order inside a region and an answer that ends inside one are refused rather than guessed:
/// a half-read region would lose one side.
/// A line that only looks like a marker outside a region is the file's own text.
#[test]
fn an_answer_that_cannot_be_cut_is_refused() {
    assert!(cut(b"<<<<<<< ours\n>>>>>>> theirs\n").is_err(), "a region with no split line cannot be cut");
    assert!(cut(b"<<<<<<< ours\none\n=======\ntwo\n").is_err(), "an answer that ends inside a region cannot be cut");
    assert!(cut(b"<<<<<<< ours\none\n<<<<<<< ours\ntwo\n=======\nthree\n>>>>>>> theirs\n").is_err(), "a region inside a region cannot be cut");
    // A line that only starts like a marker is text, as Git's own seven-character run makes it,
    // and so is one outside a region: Git does not escape the markers it writes.
    let text: Cut = cut(b"<<<<<<\n>>>>>>\n=======\n").expect("six characters and a marker outside a region are text");
    assert!(text.regions.is_empty());
    assert_eq!(text.settled[0], b"<<<<<<\n>>>>>>\n=======\n".to_vec());
}

/// A region's place is found at a line start after the region before it,
/// an empty side sits where the search starts,
/// and a passage that is not there is nothing.
#[test]
fn a_region_sits_at_a_line_start_of_its_stage() {
    let text: &[u8] = b"one short\ntwo\nthree\none short\ntwo\n";
    // The first line holds the passage but only from its middle, which a region cannot start at.
    assert_eq!(offset_from_line(text, b"short\ntwo\n", 0), None);
    assert_eq!(offset_from_line(text, b"one short\ntwo\n", 0), Some(0));
    // The search starts where the region before it ended, so a repeated passage is taken in order.
    assert_eq!(offset_from_line(text, b"one short\ntwo\n", 1), Some(20));
    assert_eq!(offset_from_line(text, b"", 7), Some(7));
    assert_eq!(offset_from_line(text, b"four\n", 0), None);
}

/// The lines before a place are the line ends before it.
#[test]
fn the_lines_before_a_place_count_from_the_top_of_the_stage() {
    let text: &[u8] = b"one\ntwo\nthree\n";
    assert_eq!(lines_before(text, 0), 0);
    assert_eq!(lines_before(text, 4), 1);
    assert_eq!(lines_before(text, 8), 2);
    // A place past the end counts every line, as a region after the last one does.
    assert_eq!(lines_before(text, 99), 3);
}
