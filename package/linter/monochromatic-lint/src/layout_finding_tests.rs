//! What:
//!  Controls of how findings show a path.
//! Why:
//!  A path inside the package is shown relative to it,
//!  and the package folder itself must not be shown as an empty name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a path inside the base is relative', () => expect(shown('/p/src/a.rs', '/p')).toBe('src/a.rs'));
//! ```

use super::{key_path, shown};
use std::path::Path;

/// A path below the base is shown without the base.
#[test]
fn layout_a_path_inside_the_package_is_shown_relative_to_it() {
    let base: &Path = Path::new("/package");
    assert_eq!(shown(&base.join("src").join("a.rs"), base), "src/a.rs");
}

/// A path outside the base is shown whole.
#[test]
fn layout_a_path_outside_the_package_is_shown_whole() {
    let base: &Path = Path::new("/package");
    assert_eq!(shown(Path::new("/elsewhere/a.rs"), base), "/elsewhere/a.rs");
}

/// The base itself is shown whole,
///  because nothing is left of it once the base is removed.
#[test]
fn layout_the_package_folder_itself_is_shown_whole() {
    let base: &Path = Path::new("/package");
    assert_eq!(shown(base, base), "/package");
}

/// A key writes every path with `/`,
///  so a list written on one platform matches on another.
#[test]
fn a_key_path_uses_forward_slashes_at_every_boundary() {
    let base: std::path::PathBuf = std::path::PathBuf::from("pkg");
    let key: String = key_path(&base.join("src/native/mod.rs"), base.as_path());
    assert_eq!(key, "src/native/mod.rs");
    assert!(!key.contains('\\'), "{key}");
    // A folder outside the package keeps the path it has, still with `/` between its parts.
    let outside: String = key_path(std::path::Path::new("/elsewhere/a/b"), base.as_path());
    assert!(outside.ends_with("a/b"), "{outside}");
}
