//! What:
//!  Controls for the line a gitignore pattern is compiled as:
//!  a literal brace stays literal,
//!  an author's own escape is kept,
//!  and a character class is copied whole.
//! Why:
//!  A brace read as an alternate group excludes a file Git checks and checks a file Git leaves
//!  alone,
//!  which is a wrong exclusion with no error,
//!  or a pattern that cannot compile.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(forMatcher('a{b.md')).toBe('a\\{b.md');
//! ```

/// The translation under test.
use super::for_matcher;

/// A literal brace is escaped for the glob compiler,
///  and a brace the author escaped is not escaped twice.
#[test]
fn a_literal_brace_is_escaped_once() {
    assert_eq!(for_matcher("a{b.md"), "a\\{b.md");
    assert_eq!(for_matcher("a}b.md"), "a\\}b.md");
    assert_eq!(for_matcher("notes{old,new}.md"), "notes\\{old,new\\}.md");
    // A brace the author already escaped is one the glob compiler reads as text already.
    assert_eq!(for_matcher("a\\{b.md"), "a\\{b.md");
    // A pattern with no brace is unchanged, so every incumbent control keeps its meaning.
    assert_eq!(for_matcher("*.md"), "*.md");
    assert_eq!(for_matcher("package/ssg/"), "package/ssg/");
    assert_eq!(for_matcher("!keep.md"), "!keep.md");
    assert_eq!(for_matcher("space\\ .md"), "space\\ .md");
}

/// A brace inside a character class is copied with the class,
///  because the class parser reads a backslash as itself.
#[test]
fn a_class_is_copied_whole() {
    assert_eq!(for_matcher("[{]x.md"), "[{]x.md");
    assert_eq!(for_matcher("[!}]x.md"), "[!}]x.md");
    // `]` first in the class is a member, not the end.
    assert_eq!(for_matcher("[]}]x.md"), "[]}]x.md");
    assert_eq!(for_matcher("[a-z]{b}.md"), "[a-z]\\{b\\}.md");
    // An unclosed class is a literal `[`, so what follows it is ordinary pattern text.
    assert_eq!(for_matcher("[a{b.md"), "[a\\{b.md");
}

/// The translation ends at the pattern's own characters:
///  an empty pattern stays empty,
///  and a trailing escape is left for the compiler to judge.
#[test]
fn the_edges_of_a_pattern_are_kept() {
    assert_eq!(for_matcher(""), "");
    assert_eq!(for_matcher("\\"), "\\");
    assert_eq!(for_matcher("{"), "\\{");
    assert_eq!(for_matcher("}"), "\\}");
}

/// A range that runs backwards is reduced to the character that opens it,
///  and a range the right way round is copied as it is.
/// Why:
///  Git tests every member character as itself before it can open a range,
///  so `[z-a]` names `z` and nothing else;
///  the glob compiler refuses the range itself,
///  which failed a policy for a pattern Git accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(forMatcher('[z-a].md')).toBe('[z].md');
/// ```
#[test]
fn a_backwards_range_keeps_only_the_character_that_opens_it() {
    assert_eq!(for_matcher("[z-a].md"), "[z].md");
    assert_eq!(for_matcher("[!z-a].md"), "[!z].md");
    assert_eq!(for_matcher("[c-a9-0].md"), "[c9].md");
    // A range the right way round is copied as it is, and so is everything around it.
    assert_eq!(for_matcher("[a-z].md"), "[a-z].md");
    assert_eq!(for_matcher("[!a-z0-9].md"), "[!a-z0-9].md");
    assert_eq!(for_matcher("a[z-a]b[a-c]c.md"), "a[z]b[a-c]c.md");
}
