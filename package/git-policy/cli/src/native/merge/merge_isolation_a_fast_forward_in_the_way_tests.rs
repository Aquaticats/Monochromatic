//! Controls for the question a fast-forward asks about the paths it writes:
//! which answers name a path the merge would write over,
//! and how a merge of any size is asked in bounded starts.

use super::*;

/// A path at or below a written path is a path the merge writes over;
/// one beside them is not;
/// a name that merely starts the same letters names nothing below it.
#[test]
fn only_the_paths_at_or_below_a_written_path_are_the_merges() {
    let written: Vec<Vec<u8>> = vec![b"a.txt".to_vec(), b"dir/x.txt".to_vec()];
    for (path, expected) in [
        (b"a.txt".to_vec(), true),
        (b"dir/x.txt".to_vec(), true),
        (b"dir".to_vec(), false),
        (b"dir/y.txt".to_vec(), false),
        (b"dir/x.txt.bak".to_vec(), false),
        (b"ax.txt".to_vec(), false),
        (b"a.txt/inside".to_vec(), true),
    ] {
        assert_eq!(at_or_below_a_written_path(path.as_slice(), written.as_slice()), expected, "{}", String::from_utf8_lossy(path.as_slice()));
    }
}

/// Every path is asked about exactly once,
/// in order,
/// and one start never carries more path text than the bound the ignored-paths read uses.
#[test]
fn the_batches_cover_every_path_once_under_the_bound() {
    assert_eq!(batches(&[]), [(0, 0)]);
    // As many paths as the bound holds, and two more, so the list cannot fit in one start.
    let example: String = format!("package/git-policy/cli/src/native/merge/an_example_file_name_{}.rs", 0);
    let carried: usize = example.len() + TOP_LITERAL_PATHSPEC.len() + 1;
    let many: Vec<Vec<u8>> = (0..IGNORED_PATHS_QUERY_BYTES / carried + 2)
        .map(|index| { return format!("package/git-policy/cli/src/native/merge/an_example_file_name_{index}.rs").into_bytes(); })
        .collect();
    let spans: Vec<(usize, usize)> = batches(many.as_slice());
    assert!(spans.len() > 1, "a list of this size is asked in more than one start");
    assert_eq!(spans.first().map(|(start, _)| *start), Some(0), "the first start begins at the first path");
    assert_eq!(spans.last().map(|(_, end)| *end), Some(many.len()));
    let mut next: usize = 0;
    for (start, end) in spans {
        assert_eq!(start, next, "the ranges cover the list in order");
        assert!(end > start);
        let bytes: usize = many[start..end].iter().map(|path| { return path.len() + TOP_LITERAL_PATHSPEC.len() + 1; }).sum();
        assert!(bytes <= IGNORED_PATHS_QUERY_BYTES, "a batch's path text stays under the bound: {bytes}");
        next = end;
    }
    assert_eq!(next, many.len(), "every path is asked about once");
}
