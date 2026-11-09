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
/// A disposable repository and its removal.
use crate::test_support::{fixture, git, git_output, remove};

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

/// What:
///  A pattern the compiler will not read is reported with the pattern,
///  never dropped.
/// Why:
///  A refusal must name the caller's own pattern,
///  and the pattern must not be silently dropped with it;
///  a trailing escape is the shape left that Git matches
///  nothing with,
///  so it is the one this wrapper refuses rather than reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(() => compileExclude(['ok/', 'a\\'])).toThrow(/a\\/)
/// ```
#[test]
fn a_pattern_that_cannot_compile_is_reported() {
    let refused: String =
        compile_exclude(&[String::from("ok/"), String::from("a\\")]).expect_err("refused");
    assert!(
        refused.starts_with(
            "monochromatic-git-policy-cli could not compile the markdown/autofix exclude pattern \"a\\\\\": "
        ),
        "{refused}"
    );
    // A literal brace compiles, and excludes exactly the file Git excludes.
    assert!(excluded(&["a{b.md"], b"a{b.md"));
    assert!(!excluded(&["a{b.md"], b"ab.md"));
}

/// What:
///  A range that runs backwards excludes exactly what native Git excludes with it,
///  and never fails the policy.
/// Why:
///  The option is documented as gitignore syntax,
///  and Git reads a member character as itself before it can open
///  a range,
///  so `[z-a]` names `z` and nothing else;
///  the glob compiler refuses that range,
///  which failed the
///  whole Markdown policy for a pattern Git accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(excluded(['[z-a].md'], 'z.md')).toBe(true); expect(excluded(['[z-a].md'], 'a.md')).toBe(false)
/// ```
#[test]
fn a_backwards_range_matches_what_native_git_matches() {
    assert!(excluded(&["[z-a].md"], b"z.md"), "the character that opens the range is a member");
    assert!(!excluded(&["[z-a].md"], b"a.md"), "the character that closes it is not");
    assert!(!excluded(&["[z-a].md"], b"README.md"));
    assert!(excluded(&["[c-a9-0].md"], b"9.md"), "each backwards range keeps its own first character");
    assert!(excluded(&["[c-a9-0].md"], b"c.md"));
    assert!(!excluded(&["[c-a9-0].md"], b"a.md"));
    assert!(!excluded(&["[!z-a].md"], b"z.md"), "a negated backwards range excludes what it names");
    assert!(excluded(&["[!z-a].md"], b"a.md"));
}

/// A directory-only pattern never excludes a file of the same name,
///  because the last segment of a pathname is the file itself.
#[test]
fn a_directory_only_pattern_never_excludes_a_file_of_that_name() {
    assert!(!excluded(&["package/ssg/"], b"package/ssg"));
    assert!(!excluded(&["ssg/"], b"a/ssg"));
    assert!(excluded(&["ssg/"], b"a/ssg/b.md"));
}

/// What:
///  One pattern and a pathname,
///  with what native Git says about them.
/// Why:
///  `git check-ignore` is the oracle of the grammar these patterns are documented as;
///  a differential control is the only way to know that the matcher reads the same language.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shape = { patterns: string[]; name: string };
/// ```
struct Shape {
    /// The gitignore lines,
    ///  in authored order.
    patterns: &'static [&'static str],
    /// The pathname asked about,
    ///  relative to the repository top level.
    name: &'static str,
}

/// The shapes a gitignore pattern can take here:
///  a wildcard,
///  a negation,
///  a negated child of an excluded directory,
///  a root anchor,
///  literal braces in both their unclosed and closed spellings,
///  the alternate-group reading brace syntax would give,
///  an escaped leading exclamation mark,
///  and an escaped space.
const SHAPES: [Shape; 14] = [
    Shape { patterns: &["*.md"], name: "README.md" },
    Shape { patterns: &["*.md", "!keep.md"], name: "keep.md" },
    Shape { patterns: &["doc/", "!doc/keep.md"], name: "doc/keep.md" },
    Shape { patterns: &["/README.md"], name: "doc/README.md" },
    Shape { patterns: &["a{b.md"], name: "a{b.md" },
    Shape { patterns: &["notes{old,new}.md"], name: "notes{old,new}.md" },
    Shape { patterns: &["notes{old,new}.md"], name: "notesold.md" },
    Shape { patterns: &["\\!literal.md"], name: "!literal.md" },
    Shape { patterns: &["space\\ .md"], name: "space .md" },
    // What: A range that runs backwards, whose first character Git reads as a member of the class.
    // Why:  The glob compiler refuses the range, so a wrapper that asked it directly failed the policy instead of
    //       matching what Git matches.
    //
    // In TS you'd write (pseudocode):
    // expect(excluded(['[z-a].md'], 'z.md')).toBe(await nativeGitIgnores('[z-a].md', 'z.md'));
    Shape { patterns: &["[z-a].md"], name: "z.md" },
    Shape { patterns: &["[z-a].md"], name: "a.md" },
    Shape { patterns: &["[z-a].md"], name: "README.md" },
    Shape { patterns: &["[c-a9-0].md"], name: "9.md" },
    Shape { patterns: &["[!z-a].md"], name: "a.md" },
];

/// What:
///  Whether native Git ignores `shape`'s name under `shape`'s patterns.
/// Why:
///  The oracle writes the patterns as the repository's own `.gitignore` and asks `git check-ignore`,
///  which reads the language the option is documented in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // writeFileSync('.gitignore', patterns.join('\n')); return spawnSync('git', ['check-ignore', '--no-index', '--quiet', '--', name]).status === 0;
/// ```
fn native_git_ignores(repository_root: &std::path::Path, shape: &Shape) -> bool {
    let ignore_file: std::path::PathBuf = repository_root.join(".gitignore");
    std::fs::write(&ignore_file, format!("{}\n", shape.patterns.join("\n"))).expect("a .gitignore");
    let asked: std::process::Output = git_output(
        repository_root,
        &["check-ignore", "--no-index", "--quiet", "--", shape.name],
    );
    let ignored: bool = asked.status.success();
    // A status above one is an error, not "not ignored", and must not read as agreement.
    assert!(
        ignored || asked.status.code() == Some(1),
        "git check-ignore failed: {}",
        String::from_utf8_lossy(&asked.stderr)
    );
    std::fs::remove_file(&ignore_file).expect("the oracle's file is removed");
    return ignored;
}

/// What:
///  The matcher reads the same pattern language native Git reads,
///  for every shape a pattern can take here.
/// Why:
///  The option is documented as gitignore syntax,
///  so a literal character must stay literal:
///  a brace read as an alternate group excludes a file Git checks and silently checks a file Git
///  leaves alone,
///  which is a wrong exclusion or a policy that exits `2`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // for (const shape of SHAPES) expect(excluded(shape.patterns, shape.name)).toBe(await nativeGitIgnores(shape));
/// ```
#[test]
fn the_matcher_reads_the_pattern_language_native_git_reads() {
    let root: std::path::PathBuf = fixture("exclude-differential");
    let repository_root: std::path::PathBuf = crate::test_support::repository(root.as_path(), "repo");
    // Every shape has a file of its own name, so the oracle removes only its own pattern file.
    for shape in &SHAPES {
        let named: std::path::PathBuf = repository_root.join(shape.name);
        if let Some(parent) = named.parent() {
            std::fs::create_dir_all(parent).expect("the shape's directory");
        }
        std::fs::write(&named, b"text\n").expect("the shape's file");
    }
    git(repository_root.as_path(), &["add", "--", "."]);
    for shape in &SHAPES {
        let oracle: bool = native_git_ignores(repository_root.as_path(), shape);
        assert_eq!(
            excluded(shape.patterns, shape.name.as_bytes()),
            oracle,
            "patterns {:?} and name {:?}",
            shape.patterns,
            shape.name
        );
    }
    remove(root.as_path());
}
