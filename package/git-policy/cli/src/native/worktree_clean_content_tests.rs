//! Controls for the matcher of attributes patterns and the blob names the content decision compares with.

use super::*;

/// `*` and `?` stop at a slash, every other byte stands for itself.
#[test]
fn a_wildcard_matches_like_a_shell_pattern_without_crossing_a_slash() {
    assert!(wildcard(b"*.dat", b"data.dat"));
    assert!(!wildcard(b"*.dat", b"data.txt"));
    assert!(wildcard(b"a?c", b"abc"));
    assert!(!wildcard(b"a?c", b"a/c"));
    assert!(!wildcard(b"*.dat", b"dir/data.dat"));
    assert!(wildcard(b"dir/*.dat", b"dir/data.dat"));
    assert!(wildcard(b"*", b""));
    assert!(!wildcard(b"", b"x"));
}

/// A pattern without a slash matches the name at any depth below its directory, one with a slash is anchored there, and a
///  spelling the matcher does not model answers yes.
#[test]
fn a_pattern_applies_by_name_or_by_path_below_its_directory() {
    assert!(may_apply("*.dat", "", "deep/er/data.dat"));
    assert!(!may_apply("*.dat", "", "deep/er/data.txt"));
    assert!(may_apply("*.dat", "deep", "deep/er/data.dat"));
    assert!(!may_apply("*.dat", "other", "deep/er/data.dat"));
    assert!(may_apply("deep/*.dat", "", "deep/data.dat"));
    assert!(!may_apply("deep/*.dat", "", "other/deep/data.dat"));
    assert!(may_apply("/data.dat", "", "data.dat"));
    assert!(may_apply("**/data.dat", "", "x/data.dat"), "an unmodelled spelling answers yes");
    assert!(may_apply("[ab].dat", "", "z.dat"), "a bracket class is not modelled");
    assert!(may_apply("\"quoted name\"", "", "z.dat"));
    assert!(may_apply("DATA.DAT", "", "data.dat"), "case never makes a match less likely");
}

/// A comment, a blank line and a macro definition are told apart from a pattern line, and a quoted pattern keeps its quotes.
#[test]
fn attribute_lines_are_split_into_patterns_macros_and_tokens() {
    let lines: Vec<AttributeLine> = parse_attribute_lines("# comment\n\n*.dat filter=x -text\n[attr]scrambled ident\n\"a b.dat\" ident\n");
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], AttributeLine { subject: String::from("*.dat"), is_macro: false, tokens: vec![String::from("filter=x"), String::from("-text")] });
    assert_eq!(lines[1], AttributeLine { subject: String::from("scrambled"), is_macro: true, tokens: vec![String::from("ident")] });
    assert_eq!(lines[2].subject, "\"a b.dat\"");
    assert_eq!(attribute_name("-filter"), "filter");
    assert_eq!(attribute_name("working-tree-encoding=UTF-16"), "working-tree-encoding");
}

/// A blob is named by its framed bytes: the empty blob and a short one have the names Git gives them.
#[test]
fn a_blob_is_named_as_git_names_it() {
    let hex = |name: Vec<u8>| -> String { return name.iter().map(|byte: &u8| { return format!("{byte:02x}"); }).collect(); };
    assert_eq!(hex(blob_name(b"", ObjectFormat::Sha1)), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    assert_eq!(hex(blob_name(b"hello\n", ObjectFormat::Sha1)), "ce013625030ba8dba906f756967f9e9ca394464a");
    assert_eq!(blob_name(b"hello\n", ObjectFormat::Sha256).len(), 32);
}
