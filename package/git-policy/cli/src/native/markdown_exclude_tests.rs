//! What:
//!  Controls for the `exclude` patterns of `markdown/autofix`.
//! Why:
//!  Gitignore semantics decide which candidates the policy leaves alone:
//!  directory
//!      patterns,
//!  anchoring,
//!  negation,
//!  and a negated file below an excluded directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(ignore().add(['package/ssg/']).ignores('package/ssg/a.mdx')).toBe(true);
//! ```

/// The matcher under test.
use super::compile_exclude;

/// Whether `patterns` exclude `path`.
fn excluded(patterns: &[&str], path: &[u8]) -> bool {
    let owned: Vec<String> = patterns.iter().map(ToString::to_string).collect();
    return compile_exclude(owned.as_slice())
        .expect("compiles")
        .excludes(path);
}

/// A directory pattern excludes everything below it and nothing beside it.
#[test]
fn a_directory_pattern_excludes_what_is_below_it() {
    let patterns: &[&str] = &["package/ssg/"];
    for (path, expected) in [
        (&b"package/ssg/a.md"[..], true),
        (b"package/ssg/deep/b.mdx", true),
        (b"package/ssg.md", false),
        (b"package/ssgx/a.md", false),
        (b"other/package/ssg/a.md", false),
        (b"a.md", false),
    ] {
        assert_eq!(excluded(patterns, path), expected, "{path:?}");
    }
}

/// A pattern without a slash matches in any directory;
///  a leading slash anchors it.
#[test]
fn anchoring_follows_gitignore() {
    assert!(excluded(&["*.mdx"], b"a/b/c.mdx"));
    assert!(!excluded(&["*.mdx"], b"a/b/c.md"));
    assert!(excluded(&["/top.md"], b"top.md"));
    assert!(!excluded(&["/top.md"], b"sub/top.md"));
    assert!(excluded(&["doc/**/draft.md"], b"doc/a/b/draft.md"));
    assert!(excluded(&["doc/**/draft.md"], b"doc/draft.md"));
}

/// A later negation re-includes a file,
///  but never a file below an excluded directory.
#[test]
fn negation_cannot_reach_below_an_excluded_directory() {
    assert!(!excluded(&["*.md", "!keep.md"], b"keep.md"));
    assert!(excluded(&["*.md", "!keep.md"], b"drop.md"));
    assert!(excluded(&["!keep.md", "*.md"], b"keep.md"));
    assert!(excluded(
        &["package/ssg/", "!package/ssg/keep.md"],
        b"package/ssg/keep.md"
    ));
}

/// No patterns,
///  comments and blank lines exclude nothing;
///  an empty pathname,
///  which names
/// no file,
///  is never excluded even by a pattern that matches everything.
#[test]
fn nothing_excludes_nothing() {
    assert!(!excluded(&[], b"a.md"));
    assert!(!excluded(&["# a.md", "", "   "], b"a.md"));
    assert!(excluded(&["*"], b"a.md"));
    assert!(!excluded(&["*"], b""));
}

/// A name that is not UTF-8 is matched by its directories and by wildcards,
///  byte for byte.
#[test]
fn a_name_that_is_not_utf8_is_matched_by_its_bytes() {
    assert!(excluded(&["x/"], b"x/caf\xe9.md"));
    assert!(excluded(&["*.md"], b"caf\xe9.md"));
    assert!(!excluded(&["caf?.txt"], b"caf\xe9.md"));
}

/// A pattern the compiler refuses is reported with the pattern,
///  never ignored.
#[test]
fn a_pattern_that_cannot_compile_is_reported() {
    let refused: String =
        compile_exclude(&[String::from("ok/"), String::from("a{b")]).expect_err("refused");
    assert!(
        refused.starts_with(
            "cli-git could not compile the markdown/autofix exclude pattern \"a{b\": "
        ),
        "{refused}"
    );
}

/// A directory-only pattern never excludes a file of the same name,
///  because the last segment of a pathname is the file itself.
#[test]
fn a_directory_only_pattern_never_excludes_a_file_of_that_name() {
    assert!(!excluded(&["package/ssg/"], b"package/ssg"));
    assert!(!excluded(&["ssg/"], b"a/ssg"));
    assert!(excluded(&["ssg/"], b"a/ssg/b.md"));
}
