//! Controls for cutting a `--zdiff3` line merge apart:
//! the settled text and the regions of an answer,
//! the shape without a common ancestor section,
//! the answers that cannot be cut,
//! the split-shaped lines Git's own split is chosen from,
//! and where a region sits in a stage.

use super::*;

/// The labels the second pass gives one merge:
/// a fresh value per call,
/// so this is one of them.
const LABELS: MarkerLabels<'_> = MarkerLabels { ours: b"ours-0f8b1c", base: b"base-0f8b1c", theirs: b"theirs-0f8b1c" };

/// A `--zdiff3` answer with two regions,
/// as Git writes one,
/// with the labels the second pass gives it.
const TWO_REGIONS: &[u8] = b"head\n<<<<<<< ours-0f8b1c\nours one\n||||||| base-0f8b1c\nbase one\n=======\ntheirs one\n>>>>>>> theirs-0f8b1c\nbetween\n<<<<<<< ours-0f8b1c\nours two\n||||||| base-0f8b1c\n=======\ntheirs two\n>>>>>>> theirs-0f8b1c\ntail\n";

/// The three stages the two-region answer came from:
/// each side of each region is a run of whole lines of its own,
/// in the order the answer holds them.
const TWO_REGIONS_BASE: &[u8] = b"head\nbase one\nbetween\ntail\n";
const TWO_REGIONS_OURS: &[u8] = b"head\nours one\nbetween\nours two\ntail\n";
const TWO_REGIONS_THEIRS: &[u8] = b"head\ntheirs one\nbetween\ntheirs two\ntail\n";

/// An answer with its two regions and their places,
/// cut and placed against the stages above.
fn two_regions_placed() -> (Cut, Vec<Region>) {
    let cut: Cut = cut(TWO_REGIONS, &LABELS).expect("a two-region answer cuts");
    let regions: Vec<Region> = cut.place([TWO_REGIONS_BASE, TWO_REGIONS_OURS, TWO_REGIONS_THEIRS]).expect("both regions are explained by the stages");
    return (cut, regions);
}

/// Both regions are cut out with their three sides and their lines,
/// and the settled text keeps the order of the file.
#[test]
fn an_answer_is_cut_into_its_settled_text_and_its_regions() {
    let (cut, regions) = two_regions_placed();
    assert_eq!(cut.settled, vec![b"head\n".to_vec(), b"between\n".to_vec(), b"tail\n".to_vec()]);
    assert_eq!(regions.len(), 2);
    assert_eq!(cut.settled.len(), regions.len() + 1);
    assert_eq!(
        cut.regions[0].markers,
        b"<<<<<<< ours-0f8b1c\nours one\n||||||| base-0f8b1c\nbase one\n=======\ntheirs one\n>>>>>>> theirs-0f8b1c\n".to_vec()
    );
    // The region's lines are the markers' and the three sides':
    // the opening marker, the first side, the common ancestor's marker and side, the split, the second side, the close.
    assert_eq!(cut.regions[0].line_starts.len(), 7);
    assert_eq!(cut.regions[0].base_line, Some(2));
    assert_eq!(cut.regions[0].split_lines, vec![4]);
    assert_eq!((regions[0].base.as_slice(), regions[0].ours.as_slice(), regions[0].theirs.as_slice()), (b"base one\n".as_slice(), b"ours one\n".as_slice(), b"theirs one\n".as_slice()));
    // Each side is found where its stage holds it.
    assert_eq!(regions[0].starts, [5, 5, 5]);
    // A region both sides inserted at one point has no common ancestor text.
    assert_eq!(cut.regions[1].base_line, Some(2));
    assert!(regions[1].base.is_empty());
    assert_eq!(regions[1].theirs, b"theirs two\n".to_vec());
}

/// An answer whose region carries no common ancestor section,
/// as `merge` style writes it,
/// has an empty base and the second side where `=======` says.
#[test]
fn a_region_without_a_common_ancestor_section_is_cut_with_an_empty_base() {
    let plain: &[u8] = b"<<<<<<< ours-0f8b1c\none\n=======\ntwo\n>>>>>>> theirs-0f8b1c\n";
    let cut: Cut = cut(plain, &LABELS).expect("an answer of two sides cuts");
    assert_eq!(cut.regions.len(), 1);
    assert_eq!(cut.regions[0].base_line, None);
    assert_eq!(cut.regions[0].split_lines, vec![2]);
    assert_eq!(cut.settled, vec![Vec::<u8>::new(), Vec::new()]);
    let regions: Vec<Region> = cut.place([b"", b"one\n", b"two\n"]).expect("the sides of the region are its stages");
    assert!(regions[0].base.is_empty());
    assert_eq!((regions[0].ours.as_slice(), regions[0].theirs.as_slice()), (b"one\n".as_slice(), b"two\n".as_slice()));
}

/// A marker out of order inside a region and an answer that ends inside one are refused rather than guessed:
/// a half-read region would lose one side.
/// A line that only looks like a marker outside a region is the file's own text.
#[test]
fn an_answer_that_cannot_be_cut_is_refused() {
    assert!(cut(b"<<<<<<< ours-0f8b1c\n>>>>>>> theirs-0f8b1c\n", &LABELS).is_err(), "a region with no split line cannot be cut");
    assert!(cut(b"<<<<<<< ours-0f8b1c\none\n=======\ntwo\n", &LABELS).is_err(), "an answer that ends inside a region cannot be cut");
    assert!(cut(b"<<<<<<< ours-0f8b1c\none\n<<<<<<< ours-0f8b1c\ntwo\n=======\nthree\n>>>>>>> theirs-0f8b1c\n", &LABELS).is_err(), "a region inside a region cannot be cut");
    // A second common ancestor marker is not an answer Git writes, so it is refused rather than read as text.
    assert!(
        cut(b"<<<<<<< ours-0f8b1c\none\n||||||| base-0f8b1c\none\n||||||| base-0f8b1c\ntwo\n=======\nthree\n>>>>>>> theirs-0f8b1c\n", &LABELS).is_err(),
        "a second common ancestor marker cannot be cut"
    );
    // A split-shaped line before the common ancestor marker is the first side's own text,
    // so an answer with none behind the marker cannot be cut.
    assert!(
        cut(b"<<<<<<< ours-0f8b1c\none\n=======\n||||||| base-0f8b1c\ntwo\n>>>>>>> theirs-0f8b1c\n", &LABELS).is_err(),
        "a split-shaped line before the common ancestor marker leaves no split"
    );
    // A line that only starts like a marker is text, as Git's own seven-character run makes it,
    // and so is one outside a region: Git does not escape the markers it writes.
    let text: Cut = cut(b"<<<<<<\n>>>>>>\n=======\n", &LABELS).expect("six characters and a marker outside a region are text");
    assert!(text.regions.is_empty());
    assert_eq!(text.settled[0], b"<<<<<<\n>>>>>>\n=======\n".to_vec());
    // A line with the seven characters and another merge's label is the file's own text.
    let other: Cut = cut(b"<<<<<<< ours-other\n>>>>>>> theirs-other\n", &LABELS).expect("another merge's labels are text");
    assert!(other.regions.is_empty());
    assert_eq!(other.settled[0], b"<<<<<<< ours-other\n>>>>>>> theirs-other\n".to_vec());
}

/// What:
///  A split-shaped line the first side inserted before the common ancestor marker is that side's text,
///  and the one split-shaped line behind the marker is Git's.
/// Why:
///   Git writes the split marker as seven equals signs with no label,
///   so a line of the person's file that looks exactly like it reads as a split;
///   the region was cut at the inserted line,
///   the common ancestor's marker then arrived behind the split,
///   and the answer was refused as out of order although its three sides are exactly what the stages hold
///   (cross-vendor review of round 198,
///   finding 1).
#[test]
fn a_split_shaped_line_before_the_common_ancestor_marker_is_the_first_sides_text() {
    // What real Git 2.56.0 writes for these three stages:
    // the first side's inserted `=======` line comes before Git's own common ancestor and split markers.
    let answer: &[u8] = b"<<<<<<< ours-0f8b1c\nalpha uno\n=======\nbeta one\n||||||| base-0f8b1c\nalpha one\nbeta one\n=======\nalpha one\nbeta uno\n>>>>>>> theirs-0f8b1c\n";
    let cut: Cut = cut(answer, &LABELS).expect("the answer cuts");
    assert_eq!(cut.regions.len(), 1);
    assert_eq!(cut.regions[0].base_line, Some(4));
    assert_eq!(cut.regions[0].split_lines, vec![2, 7]);
    // Only the line behind the common ancestor marker can be Git's split, so the stages are not asked.
    let regions: Vec<Region> = cut
        .place([b"alpha one\nbeta one\n", b"alpha uno\n=======\nbeta one\n", b"alpha one\nbeta uno\n"])
        .expect("the region is explained by its stages");
    assert_eq!(regions[0].base, b"alpha one\nbeta one\n".to_vec());
    assert_eq!(regions[0].ours, b"alpha uno\n=======\nbeta one\n".to_vec());
    assert_eq!(regions[0].theirs, b"alpha one\nbeta uno\n".to_vec());
}

/// What:
///  Where a region holds more than one split-shaped line behind the common ancestor marker,
///  the stages decide,
///  and a line that is really the first side's text cannot be mistaken for the split.
/// Why:
///   A split-shaped line of the common ancestor's section and one of the second side's section both read as splits too,
///   and only the sides their stages hold tell them apart.
#[test]
fn a_split_shaped_line_of_a_section_is_told_apart_by_its_stage() {
    // The common ancestor's second line and the second side's first line are both split-shaped.
    let answer: &[u8] = b"<<<<<<< ours-0f8b1c\nalpha uno\nbeta one\n||||||| base-0f8b1c\nalpha one\n=======\nbeta one\n=======\n=======\nbeta uno\n>>>>>>> theirs-0f8b1c\n";
    let cut: Cut = cut(answer, &LABELS).expect("the answer cuts");
    assert_eq!(cut.regions[0].split_lines, vec![5, 7, 8]);
    let regions: Vec<Region> = cut
        .place([b"alpha one\n=======\nbeta one\n", b"alpha uno\nbeta one\n", b"=======\nbeta uno\n"])
        .expect("only the split whose sides the stages hold is taken");
    assert_eq!(regions[0].base, b"alpha one\n=======\nbeta one\n".to_vec());
    assert_eq!(regions[0].ours, b"alpha uno\nbeta one\n".to_vec());
    assert_eq!(regions[0].theirs, b"=======\nbeta uno\n".to_vec());
}

/// Two splits that both explain the region are no answer at all:
/// the file is left as Git left it rather than merged on a guess.
#[test]
fn a_region_two_splits_explain_is_left() {
    let answer: &[u8] = b"<<<<<<< ours-0f8b1c\na\n=======\nb\n=======\nc\n>>>>>>> theirs-0f8b1c\n";
    let cut: Cut = cut(answer, &LABELS).expect("the answer cuts");
    assert_eq!(cut.regions[0].split_lines, vec![2, 4]);
    // Each split's two sides are runs of the stages, so neither is Git's for certain.
    assert!(
        cut.place([b"", b"a\n=======\nb\n", b"b\n=======\nc\n"]).is_none(),
        "both splits place, so neither is Git's for certain"
    );
}

/// A region with nothing on any side is refused:
/// Git writes no such region,
/// and cutting it would drop the text it holds.
#[test]
fn a_region_with_no_text_on_any_side_is_left() {
    let answer: &[u8] = b"<<<<<<< ours-0f8b1c\n=======\n>>>>>>> theirs-0f8b1c\n";
    let cut: Cut = cut(answer, &LABELS).expect("the answer cuts");
    assert!(cut.place([b"", b"", b""]).is_none(), "a region of markers alone is no region");
}

/// What:
///  An answer's split-shaped lines are only searched against the stages a bounded number of times,
///  so a file whose regions hold more of them than that is left as Git left it,
///  while a region at the bound is still explained.
/// Why:
///   Each try searches the three stages,
///   so a file whose every line looked like a split would otherwise hold the merge for as long as it liked.
#[test]
fn more_split_shaped_lines_than_the_tries_is_left() {
    let explained = |splits: usize| -> Option<Vec<Region>> {
        let mut answer: Vec<u8> = b"<<<<<<< ours-0f8b1c\n".to_vec();
        let mut ours: Vec<u8> = Vec::new();
        for index in 0..splits {
            let line = format!("ours {index}\n");
            ours.extend_from_slice(line.as_bytes());
            ours.extend_from_slice(b"=======\n");
            answer.extend_from_slice(line.as_bytes());
            answer.extend_from_slice(b"=======\n");
        }
        answer.extend_from_slice(b"theirs\n>>>>>>> theirs-0f8b1c\n");
        let cut: Cut = cut(answer.as_slice(), &LABELS).expect("the answer cuts");
        assert_eq!(cut.regions[0].split_lines.len(), splits);
        return cut.place([b"", ours.as_slice(), b"theirs\n"]);
    };
    // At the bound the last split is the only one whose sides the stages hold, and it is found.
    assert!(explained(SPLIT_TRIES).is_some(), "a region at the bound is explained");
    // One more is past the bound, so the split the stages hold is never reached.
    assert!(explained(SPLIT_TRIES + 1).is_none(), "a region past the bound is left");
}

/// An answer whose lines end with a carriage return and a line feed is cut,
/// with the markers Git writes for such a file,
/// which carry that ending too.
#[test]
fn an_answer_whose_lines_end_with_both_bytes_is_cut() {
    let answer: &[u8] = b"head\r\n<<<<<<< ours-0f8b1c\r\nours one\r\n||||||| base-0f8b1c\r\nbase one\r\n=======\r\ntheirs one\r\n>>>>>>> theirs-0f8b1c\r\ntail\r\n";
    let cut: Cut = cut(answer, &LABELS).expect("an answer whose lines end with both bytes cuts");
    assert_eq!(cut.regions.len(), 1);
    assert_eq!(cut.settled, vec![b"head\r\n".to_vec(), b"tail\r\n".to_vec()]);
    assert_eq!(cut.regions[0].split_lines, vec![4]);
    let regions: Vec<Region> = cut
        .place([b"head\r\nbase one\r\ntail\r\n", b"head\r\nours one\r\ntail\r\n", b"head\r\ntheirs one\r\ntail\r\n"])
        .expect("the region is explained by its stages");
    assert_eq!(regions[0].ours, b"ours one\r\n".to_vec());
    assert_eq!(regions[0].base, b"base one\r\n".to_vec());
    assert_eq!(regions[0].theirs, b"theirs one\r\n".to_vec());
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
