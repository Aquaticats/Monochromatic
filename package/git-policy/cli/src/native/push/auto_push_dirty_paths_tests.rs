//! What:
//!  The reading of Git's NUL-separated status and diff listings, and the overlap between them.
//! Why:
//!  A rename read as one path would let a move overwrite the staged work under its old name; a count that is off
//!  by one would name the wrong files to the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(statusPaths('R  new\0old\0')).toEqual(['new', 'old']);
//! ```

/// The readers and the report text.
use super::{diff_paths, listed, overlapping_paths, status_paths};

/// Byte paths from text.
fn paths(values: &[&str]) -> Vec<Vec<u8>> {
    return values.iter().map(|value| { return value.as_bytes().to_vec(); }).collect();
}

/// Each entry gives its path; a rename or copy gives its original path too, in either status column.
#[test]
fn status_entries_give_their_paths_and_renames_give_both() {
    assert_eq!(status_paths(b""), Vec::<Vec<u8>>::new());
    assert_eq!(status_paths(b" M a.txt\0A  b.txt\0"), paths(&["a.txt", "b.txt"]));
    assert_eq!(status_paths(b"R  new.txt\0old.txt\0 M a.txt\0"), paths(&["new.txt", "old.txt", "a.txt"]));
    assert_eq!(status_paths(b"C  copy.txt\0src.txt\0"), paths(&["copy.txt", "src.txt"]));
    assert_eq!(status_paths(b" R new.txt\0old.txt\0"), paths(&["new.txt", "old.txt"]));
    assert_eq!(status_paths(b" C copy.txt\0src.txt\0"), paths(&["copy.txt", "src.txt"]));
    assert_eq!(status_paths(b"UU both.txt\0"), paths(&["both.txt"]));
    assert_eq!(status_paths(b"R  new.txt\0"), paths(&["new.txt"]));
    assert_eq!(status_paths(b"M \0"), Vec::<Vec<u8>>::new());
    assert_eq!(status_paths(b" M a b.txt\0"), paths(&["a b.txt"]));
}

/// Paths that are not UTF-8 stay bytes.
#[test]
fn paths_that_are_not_utf8_stay_bytes() {
    assert_eq!(status_paths(b" M t\xffp.txt\0"), vec![b"t\xffp.txt".to_vec()]);
    assert_eq!(diff_paths(b"t\xffp.txt\0"), vec![b"t\xffp.txt".to_vec()]);
}

/// The diff listing is the non-empty NUL-separated names.
#[test]
fn the_diff_listing_is_the_names() {
    assert_eq!(diff_paths(b""), Vec::<Vec<u8>>::new());
    assert_eq!(diff_paths(b"a.txt\0dir/b.txt\0"), paths(&["a.txt", "dir/b.txt"]));
}

/// Only a path in both lists overlaps, once.
#[test]
fn only_paths_in_both_lists_overlap() {
    assert_eq!(
        overlapping_paths(&paths(&["a", "b", "c"]), &paths(&["c", "a", "z"])),
        paths(&["a", "c"])
    );
    assert_eq!(overlapping_paths(&paths(&["a", "a"]), &paths(&["a"])), paths(&["a"]));
    assert_eq!(overlapping_paths(&paths(&["a"]), &paths(&["b"])), Vec::<Vec<u8>>::new());
    assert_eq!(overlapping_paths(&[], &paths(&["b"])), Vec::<Vec<u8>>::new());
}

/// Five names are listed in full, more are counted.
#[test]
fn the_report_names_five_and_counts_the_rest() {
    assert_eq!(listed(&paths(&["a"])), "a");
    assert_eq!(listed(&paths(&["a", "b", "c", "d", "e"])), "a, b, c, d, e");
    assert_eq!(listed(&paths(&["a", "b", "c", "d", "e", "f"])), "a, b, c, d, e, and 1 more");
    assert_eq!(listed(&paths(&["a", "b", "c", "d", "e", "f", "g", "h"])), "a, b, c, d, e, and 3 more");
    assert_eq!(listed(&[b"t\xffp".to_vec()]), "t\u{fffd}p");
}

/// An entry of two letters, a space and a one-byte path is the shortest valid one; anything shorter has no path.
#[test]
fn the_shortest_valid_entry_is_four_bytes() {
    assert_eq!(status_paths(b"M  a\0"), paths(&["a"]));
    assert_eq!(status_paths(b" M a\0?? b\0"), paths(&["a", "b"]));
    assert_eq!(status_paths(b"M  \0"), Vec::<Vec<u8>>::new());
}
