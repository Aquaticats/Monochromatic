//! What:
//!  Attribute-line translation and gitignore-syntax matching controls.
//! Why:
//!  Which paths are LFS-tracked is frozen by `package/cli/markdown-lint/src/lfs-tracked.unit.test.ts`;
//! the added cases cover the `exclude` option's directory patterns and excluded-parent semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(lfsTrackedMatcher.name, () => { /* extension, scoped, unset, empty */ });
//! ```

/// Import the matcher and the attribute translation.
use super::{PathPatterns, lfs_tracked_patterns};

/// Attribute lines mirroring the repository's `.gitattributes`.
const ATTRIBUTES: &str = "* text=auto eol=lf\n\n# raster formats\n*.png filter=lfs diff=lfs merge=lfs -text\n*.jpg\tfilter=lfs diff=lfs merge=lfs -text\ndesign/*.bin filter=lfs\npackage/*/asset/readme/*.png !filter !diff !merge";

/// Compile the tracked-path predicate from attribute text.
fn tracked(text: &str) -> PathPatterns {
    let patterns: Vec<String> = lfs_tracked_patterns(text);
    return PathPatterns::new(patterns.as_slice()).expect("patterns compile");
}

/// Compile a literal pattern list.
fn patterns(lines: &[&str]) -> PathPatterns {
    let mut owned: Vec<String> = Vec::<String>::new();
    for line in lines {
        owned.push(String::from(*line));
    }
    return PathPatterns::new(owned.as_slice()).expect("patterns compile");
}

/// Only `filter=lfs` lines select,
///  unset lines negate,
///  and other attribute lines contribute nothing.
#[test]
fn attribute_lines_become_ordered_patterns() {
    assert_eq!(
        lfs_tracked_patterns(ATTRIBUTES),
        [
            "*.png",
            "*.jpg",
            "design/*.bin",
            "!package/*/asset/readme/*.png"
        ]
    );
    assert_eq!(
        lfs_tracked_patterns(
            "a.bin -filter\r\n  b.bin \t filter=lfs  \r\nc.bin filter=other\nd.bin"
        ),
        ["!a.bin", "b.bin"]
    );
    assert!(lfs_tracked_patterns("").is_empty());
    assert!(lfs_tracked_patterns("# *.png filter=lfs\n \n").is_empty());
}

/// An extension pattern matches anywhere in the tree;
///  lines without `filter=lfs` match nothing.
#[test]
fn extension_patterns_match_at_any_depth() {
    let matcher: PathPatterns = tracked(ATTRIBUTES);
    assert!(matcher.matches("deep/dir/shot.png"));
    assert!(matcher.matches("shot.jpg"));
    assert!(!matcher.matches("README.md"));
    assert!(!matcher.matches("src/index.ts"));
}

/// A pattern with a slash is anchored at the repository root,
///  and a later unset line wins.
#[test]
fn scoped_patterns_and_later_unsets_are_honoured() {
    let matcher: PathPatterns = tracked(ATTRIBUTES);
    assert!(matcher.matches("design/a.bin"));
    assert!(!matcher.matches("other/a.bin"));
    assert!(!matcher.matches("design/deep/a.bin"));
    assert!(!matcher.matches("package/player/asset/readme/shot.png"));
    assert!(matcher.matches("package/player/design/shot.png"));
    assert!(!tracked("").matches("shot.png"));
}

/// A directory pattern selects every file below it,
///  and a file under an excluded directory cannot be re-included.
#[test]
fn directory_patterns_cover_descendants_and_outrank_file_negations() {
    let directory: PathPatterns = patterns(&["package/ssg/"]);
    assert!(directory.matches("package/ssg/README.md"));
    assert!(directory.matches("package/ssg/deep/page.mdx"));
    assert!(!directory.matches("package/ssg"));
    assert!(!directory.matches("package/other/README.md"));
    let reincluded: PathPatterns = patterns(&["docs/", "!docs/keep.md"]);
    assert!(reincluded.matches("docs/keep.md"));
    let file_level: PathPatterns = patterns(&["*.md", "!keep.md"]);
    assert!(file_level.matches("docs/other.md"));
    assert!(!file_level.matches("docs/keep.md"));
    assert!(!patterns(&[]).matches("docs/keep.md"));
}

/// A pattern the compiler rejects is an error naming the pattern;
///  an unclosed class is a literal,
///  as in git.
#[test]
fn invalid_patterns_are_errors() {
    let error = PathPatterns::new(&[String::from("a/[z-a].png")]).expect_err("reversed range");
    assert!(error.message.contains("a/[z-a].png"), "{}", error.message);
    assert_eq!(error.to_string(), error.message);
    let literal: PathPatterns = patterns(&["a/["]);
    assert!(literal.matches("a/["));
    assert!(!literal.matches("a/b"));
}
