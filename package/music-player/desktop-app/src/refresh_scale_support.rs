// What:     A library of a million tracks in the folder walk's order, for the refresh tests.
// Why:      1,000 artists and 1,000,000 tracks are normal operation (D112), so the planner and the controller are measured
//           at that size in the normal suite. The shared generator lists artist folders in the order it drew them, not in
//           the walk's order, so the artist groups are sorted by folder name here and the whole list is then checked
//           against the comparison the planner searches with.

use std::path::{Path, PathBuf};

use crate::natural_order::natural_cmp;
use crate::scale_library_tests::build_library;
use crate::walk_order::walk_cmp;

/// What: the folder the pretend library lives in.
/// Why: the planner needs a root to strip from paths.
pub(crate) const ROOT: &str = "/library";

/// What: `million_library` is 1,000 artists of 1,000 tracks, absolute paths, in the walk's order.
/// Why: the planner binary-searches the list, so the order must be exactly the walk's; the function proves it.
pub(crate) fn million_library() -> Vec<PathBuf> {
    let library = build_library(1000, 1000);
    let mut loose: Vec<PathBuf> = Vec::new();
    let mut groups: Vec<(String, Vec<PathBuf>)> = Vec::new();
    for name in &library.names {
        let path: PathBuf = Path::new(ROOT).join(name);
        match name.split_once('/') {
            None => loose.push(path),
            Some((folder, _)) => match groups.last_mut() {
                Some((last, members)) if last == folder => members.push(path),
                _ => groups.push((folder.to_string(), vec![path])),
            },
        }
    }
    loose.sort_by(|left, right| return walk_cmp(Path::new(ROOT), left, right));
    groups.sort_by(|left, right| return natural_cmp(&left.0, &right.0).then_with(|| return left.0.cmp(&right.0)));
    let mut ordered: Vec<PathBuf> = loose;
    for (_folder, mut members) in groups {
        members.sort_by(|left, right| return walk_cmp(Path::new(ROOT), left, right));
        ordered.extend(members);
    }
    assert_eq!(ordered.len(), library.names.len(), "no track was lost");
    let root: &Path = Path::new(ROOT);
    let misplaced: Option<usize> = ordered.windows(2).position(|pair| return walk_cmp(root, &pair[0], &pair[1]) != std::cmp::Ordering::Less);
    assert_eq!(misplaced, None, "every track sorts strictly before the next in the walk's order");
    return ordered;
}
