//! What:
//!  Controls for the lexical path:
//!  a relative `..` resolved,
//!  `.` dropped,
//!  an absolute path kept absolute,
//!  and a `..` that climbs past the root.
//! Why:
//!  A registration's records are compared against recorded paths,
//!  and a normalizer that walked past the root or dropped a leading slash would compare the wrong two names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theLexicalPath('/a/b/../c')).toBe('/a/c');
//! ```

use super::the_lexical_path;
use std::path::{
    Path,
    PathBuf,
};

/// A relative segment and a `.` resolve to the components before them.
#[test]
fn a_relative_segment_and_a_dot_resolve() {
    assert_eq!(
        the_lexical_path(Path::new("/a/b/../c")),
        PathBuf::from("/a/c")
    );
    assert_eq!(the_lexical_path(Path::new("/a/./b")), PathBuf::from("/a/b"));
    assert_eq!(the_lexical_path(Path::new("x/../y")), PathBuf::from("y"));
}

/// An absolute path stays absolute and a `..` at the root stays at the root.
#[test]
fn the_root_is_not_climbed() {
    assert_eq!(the_lexical_path(Path::new("/../a")), PathBuf::from("/a"));
    assert!(the_lexical_path(Path::new("/a")).has_root());
}

/// A relative path that climbs past its own start keeps the `..` it cannot resolve,
/// because the answer is what the caller's start makes of it.
#[test]
fn a_relative_path_keeps_the_segments_it_cannot_resolve() {
    assert_eq!(
        the_lexical_path(Path::new("../../a")),
        PathBuf::from("../../a")
    );
}
