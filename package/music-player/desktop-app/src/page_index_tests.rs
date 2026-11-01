// What:     Tests for `page_index.rs`, pulled in by `#[cfg(test)] #[path = "page_index_tests.rs"] mod tests;`.
//           The index must hold exactly the pages `paginate` makes, in the same order, with the same identities and
//           positions, whether built at once or extended in pieces.
// Why:      The window now reads pages from the index instead of calling `paginate` on every projection.
use super::*;
use crate::pagination::{page_identity, paginate};

// What:     Names that cover folder pages, letter pages, the catch-all page and awkward folder names.
// Why:      Each name decides a page; the list repeats folders so pages have several positions.
fn sample_names() -> Vec<String> {
    return [
        "Abba/one.mp3", "abba/two.mp3", "ABBA/three.mp3", "Abba/Sub/four.mp3", "zeta.mp3", "Zulu.mp3", "alpha.mp3", "1 first.mp3",
        "#hash.mp3", "/rooted.mp3", "Ёж/five.mp3", "ёлка/six.mp3", "Camellia/seven.mp3", "camellia/eight.mp3", ".hidden", "b.mp3",
        "A.mp3", "Unfiled/nine.mp3", "Z/ten.mp3", "z/eleven.mp3",
    ]
    .iter()
    .map(|name| return name.to_string())
    .collect();
}

// What:     `assert_same_pages`, compares the index with `paginate` page by page.
// Why:      Order, label, identity and the positions of every page must match.
fn assert_same_pages(index: &PageIndex, names: &[String]) {
    let expected = paginate(names);
    assert_eq!(index.page_count(), expected.len());
    for (position, page) in expected.iter().enumerate() {
        let actual = index.page(position).expect("the index has this page");
        assert_eq!(actual.label, page.label, "label of page {position}");
        assert_eq!(actual.identity, page_identity(page), "identity of page {position}");
        let wanted: Vec<u32> = page.entries.iter().map(|entry| return u32::try_from(entry.index).unwrap_or(0)).collect();
        assert_eq!(actual.indices(), wanted.as_slice(), "positions of page {position}");
        for entry in &page.entries {
            assert_eq!(index.page_of_index(entry.index), Some(position));
        }
    }
}

// What:     A whole-library build equals `paginate`.
// Why:      This is the basic parity case.
#[test]
fn build_matches_paginate() {
    let names = sample_names();
    assert_same_pages(&PageIndex::build(&names), &names);
}

// What:     Extending in pieces equals one build, at every split point.
// Why:      A library that grows by appending must produce the same pages as one that arrived whole.
#[test]
fn extending_in_pieces_matches_paginate() {
    let names = sample_names();
    for split in 0..=names.len() {
        let mut index = PageIndex::build(&names[..split]);
        assert_eq!(index.covered(), split);
        index.extend(&names);
        assert_eq!(index.covered(), names.len());
        assert_same_pages(&index, &names);
    }
}

// What:     Lookups by identity and label find the same pages, and a missing identity is `None`.
// Why:      The selection is stored as an identity and Search chooses a folder by label.
#[test]
fn lookups_find_pages_by_identity_and_label() {
    let names = sample_names();
    let index = PageIndex::build(&names);
    let position = index.position_of_identity("folder:Camellia").expect("Camellia is a folder page");
    assert_eq!(index.page(position).map(|page| return page.label.as_str()), Some("Camellia"));
    assert_eq!(index.position_of_label("Camellia"), Some(position));
    assert_eq!(index.position_of_identity("folder:Missing"), None);
    assert_eq!(index.page_of_index(names.len()), None);
}

// What:     An empty library has no pages.
// Why:      The first-run state has no names.
#[test]
fn an_empty_library_has_no_pages() {
    let index = PageIndex::build(&[]);
    assert_eq!(index.page_count(), 0);
    assert!(index.page(0).is_none());
}
