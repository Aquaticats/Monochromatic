//! Controls for the names the reset refuses:
//! the port of Git's `is_ntfs_dotgitmodules` on the spellings its own `t0060-path.sh` uses.

use super::*;

/// The port of `is_ntfs_dotgitmodules` on the spellings of Git's own `t0060-path.sh` and the edges of its three branches.
#[test]
fn the_dotgitmodules_predicate_follows_git() {
    for name in [".gitmodules", ".GITMODULES", ".gitmodules . ", ".gitmodules:stream", "gitmod~1", "GITMOD~4 ", "GI7EBA~1", "gi7eba~9", "GI7EBA~1 ."] {
        assert!(is_ntfs_dotgitmodules(name.as_bytes()), "{name:?}");
    }
    for name in ["", ".gitmodule", ".gitmodulesx", "gitmod~5", "gitmod~", "gitmodules", "GI7EBA~0", "GI7EBA~1x", "gi7eba", "GI7EBB~1", "xi7eba~1", ".gitignore", "GI250A~1", "GI7EBA~12:x"] {
        assert!(!is_ntfs_dotgitmodules(name.as_bytes()), "{name:?}");
    }
}
