//! What:
//!  Destination classification,
//!  lexical resolution and traversal controls.
//! Why:
//!  Which repository path a destination names is frozen by
//! `package/cli/markdown-lint/src/lfs-image-context.unit.test.ts`;
//!  the added cases cover delimiters,
//! traversal tokens and non-UTF-8 components at this text-to-path boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(relativeTargetPath.name, () => { /* resolve, escape, non-relative */ });
//! ```

/// Import the classification operations under test.
use super::{
    ObjectUrlParts, apply_segments, is_relative_path, lexical_normal, object_url_parts, path_part,
    relative_link, relative_target_path, repo_relative,
};
use std::path::{Path, PathBuf};

/// Object id used by the object-URL fixtures.
const OID: &str = "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec";
/// Object base used by the object-URL fixtures.
const BASE: &str = "https://lfs.example";

/// Relative paths are accepted;
///  schemes,
///  fragments,
///  site-absolute paths and the empty destination are not.
#[test]
fn relative_paths_are_distinguished_from_other_destinations() {
    assert!(is_relative_path("asset/shot.png"));
    assert!(is_relative_path("./shot.png"));
    assert!(!is_relative_path("https://x.example/a.png"));
    assert!(!is_relative_path("mailto:x@y"));
    assert!(!is_relative_path("#top"));
    assert!(!is_relative_path("/a.png"));
    assert!(!is_relative_path(""));
    assert!(!is_relative_path("c:/windows.png"));
    assert!(is_relative_path("a b:c.png"));
    // A leading colon is not a scheme, and every scheme punctuation character is accepted.
    assert!(is_relative_path(":a.png"));
    assert!(!is_relative_path("a+b-c.d1:x"));
    assert!(is_relative_path("a_b:x"));
    assert!(is_relative_path("a/b:c"));
}

/// The cut is at the first query or fragment delimiter,
///  whichever comes first.
#[test]
fn query_and_fragment_are_removed_at_the_first_delimiter() {
    assert_eq!(path_part("a.png?x#y"), "a.png");
    assert_eq!(path_part("a.png#y?x"), "a.png");
    assert_eq!(path_part("a.png?x"), "a.png");
    assert_eq!(path_part("a.png#y"), "a.png");
    assert_eq!(path_part("a.png"), "a.png");
    assert_eq!(path_part("?x"), "");
}

/// A destination resolves against the file's directory;
///  one leaving the root,
///  or the root itself,
///  names nothing.
#[test]
fn relative_destinations_resolve_against_the_file_directory() {
    let root: &Path = Path::new("/r");
    assert_eq!(
        relative_target_path(
            "../asset/a.png?x#y",
            Path::new("/r/pkg/doc/README.md"),
            root
        ),
        Some(String::from("pkg/asset/a.png"))
    );
    assert_eq!(
        relative_target_path("../../a.png", Path::new("/r/pkg/README.md"), root),
        None
    );
    assert_eq!(
        relative_target_path("https://x.example/a.png", Path::new("/r/README.md"), root),
        None
    );
    assert_eq!(
        relative_target_path("./a//b/./c.png", Path::new("/r/README.md"), root),
        Some(String::from("a/b/c.png"))
    );
    assert_eq!(
        relative_target_path("..", Path::new("/r/pkg/README.md"), root),
        None
    );
    assert_eq!(
        relative_target_path(".", Path::new("/r/README.md"), root),
        None
    );
    // A sibling whose name merely starts with the root's name is outside the root.
    assert_eq!(
        relative_target_path("../rx/a.png", Path::new("/r/README.md"), root),
        None
    );
    // Climbing above the filesystem root stays at the root, as path.resolve does.
    assert_eq!(
        relative_target_path("../../../../r/a.png", Path::new("/r/pkg/README.md"), root),
        Some(String::from("a.png"))
    );
}

/// Lexical normalization removes `.` and resolves `..` without reading the filesystem.
#[test]
fn lexical_normalization_resolves_dot_components() {
    assert_eq!(
        lexical_normal(Path::new("/r/./pkg/../doc/a.md")),
        PathBuf::from("/r/doc/a.md")
    );
    assert_eq!(lexical_normal(Path::new("/../..")), PathBuf::from("/"));
    assert_eq!(
        apply_segments(Path::new("/r"), "a/../../x/./y//z"),
        PathBuf::from("/x/y/z")
    );
    // Path equality compares components, which hides a `.` segment and a trailing separator.
    // The operating system does not: `b.png/` and `b.png/.` name a directory. Compare the exact spelling.
    for (segments, spelled) in [
        ("a/../../x/./y//z", "/x/y/z"),
        ("a/b.png/", "/r/a/b.png"),
        ("a/b.png/.", "/r/a/b.png"),
        ("./a//b.png", "/r/a/b.png"),
    ] {
        assert_eq!(
            apply_segments(Path::new("/r"), segments).as_os_str(),
            spelled,
            "{segments}"
        );
    }
    assert_eq!(
        repo_relative(Path::new("/r"), Path::new("/r/a/b.png")),
        Some(String::from("a/b.png"))
    );
    assert_eq!(repo_relative(Path::new("/r"), Path::new("/r")), None);
    assert_eq!(
        repo_relative(Path::new("/r"), Path::new("/elsewhere/a.png")),
        None
    );
}

/// A component that is not valid UTF-8 cannot be written in a URL,
///  so the path names nothing.
#[cfg(unix)]
#[test]
fn non_utf8_components_name_nothing() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let mut path: PathBuf = PathBuf::from("/r");
    path.push(OsString::from_vec(vec![b'n', 0xff]));
    path.push("a.png");
    assert_eq!(repo_relative(Path::new("/r"), &path), None);
}

/// The oid and path are split under an exact base prefix;
///  query and fragment are not part of the path.
#[test]
fn object_urls_are_split_under_the_exact_base() {
    assert_eq!(
        object_url_parts(format!("{BASE}/{OID}/pkg/a.png?x").as_str(), BASE),
        Some(ObjectUrlParts {
            oid: String::from(OID),
            repo_relative_path: String::from("pkg/a.png"),
        })
    );
    assert_eq!(
        object_url_parts(format!("https://other.example/{OID}/a.png").as_str(), BASE),
        None
    );
    assert_eq!(
        object_url_parts(format!("{BASE}/{OID}").as_str(), BASE),
        None
    );
    assert_eq!(
        object_url_parts(format!("{BASE}/{OID}/").as_str(), BASE),
        None
    );
    assert_eq!(
        object_url_parts(format!("{BASE}/nope/a.png").as_str(), BASE),
        None
    );
    // A longer host that merely starts with the base is another origin.
    assert_eq!(
        object_url_parts(format!("{BASE}.evil/{OID}/a.png").as_str(), BASE),
        None
    );
    assert_eq!(object_url_parts(BASE, BASE), None);
    // The path is cut before a fragment even when the fragment contains a slash.
    assert_eq!(
        object_url_parts(format!("{BASE}/{OID}/a.png#x/y").as_str(), BASE),
        Some(ObjectUrlParts {
            oid: String::from(OID),
            repo_relative_path: String::from("a.png"),
        })
    );
    assert_eq!(
        object_url_parts(format!("{BASE}/{OID}?x/a.png").as_str(), BASE),
        None
    );
}

/// The link from a file's directory climbs to the shared ancestor,
///  then descends to the target.
#[test]
fn relative_links_climb_then_descend() {
    assert_eq!(
        relative_link(
            Path::new("/repo/package/player"),
            Path::new("/repo/package/player/asset/readme/plain.svg")
        ),
        "asset/readme/plain.svg"
    );
    assert_eq!(
        relative_link(
            Path::new("/repo/package/player/doc"),
            Path::new("/repo/asset/a.svg")
        ),
        "../../../asset/a.svg"
    );
    assert_eq!(
        relative_link(Path::new("/repo/a"), Path::new("/repo/ab/c.svg")),
        "../ab/c.svg"
    );
    assert_eq!(relative_link(Path::new("/repo"), Path::new("/repo")), "");
}
