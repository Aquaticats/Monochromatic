//! Controls for lexical normalization.

use super::*;

/// `.` and `..` fold away, repeated separators collapse, and a `..` above the root stays at the root.
#[test]
fn paths_normalize_lexically() {
    assert_eq!(normalized_path(Path::new("/repo/./wt//.git/")), PathBuf::from("/repo/wt/.git"));
    assert_eq!(normalized_path(Path::new("/repo/a/../b")), PathBuf::from("/repo/b"));
    assert_eq!(normalized_path(Path::new("/repo/../../x/")), PathBuf::from("/x"));
}
