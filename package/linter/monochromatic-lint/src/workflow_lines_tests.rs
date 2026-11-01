//! What:
//!  Controls for the workflow line reader.
//! Why:
//!  A wrong column or value offset moves a finding to the wrong byte,
//!  and a misread key hides a step from the rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('workflow lines', () => { /* indentation, dashes, keys, values, comments */ });
//! ```

/// Import the reader under test.
use super::{Content, Entry, parse_entry, read_lines, without_comment};

/// Read one line and split it.
fn entry_of(text: &str) -> (usize, Vec<usize>, String) {
    let lines = read_lines(text).expect("lines");
    let entry: Entry<'_> = parse_entry(&lines[0]);
    let shape: String = match &entry.content {
        Content::Nothing => String::from("nothing"),
        Content::OddKey { .. } => String::from("odd"),
        Content::Scalar => String::from("scalar"),
        Content::Key {
            name,
            column,
            value,
            value_offset,
        } => format!("key {name} @{column} = {value:?} +{value_offset}"),
    };
    return (lines[0].indent, entry.dashes, shape);
}

/// Indentation counts spaces and a tab is refused at its byte.
#[test]
fn lines_carry_indentation() {
    let lines = read_lines("a:\n  b: 1\n\n   # c\n").expect("lines");
    assert_eq!(lines.len(), 4);
    assert_eq!((lines[1].indent, lines[1].text), (2, "b: 1"));
    assert!(lines[2].is_blank());
    assert!(lines[3].is_skippable());
    assert_eq!(lines[1].start, 3);
    assert_eq!(lines[1].text_start(), 5);
    let error = read_lines("a:\n \tb: 1\n").expect_err("tab");
    assert_eq!(error.offset, 4);
    assert!(error.message.contains("tab"));
}

/// A key reports its column and its value's offset.
#[test]
fn keys_report_column_and_value_offset() {
    assert_eq!(
        entry_of("  key:   val # c"),
        (2, vec![], String::from("key key @2 = \"val # c\" +7"))
    );
    assert_eq!(
        entry_of("k:"),
        (0, vec![], String::from("key k @0 = \"\" +2"))
    );
    assert_eq!(
        entry_of("url: http://a:b"),
        (0, vec![], String::from("key url @0 = \"http://a:b\" +5"))
    );
}

/// Dashes shift the key's column.
#[test]
fn dashes_shift_the_column() {
    assert_eq!(
        entry_of("      - run: x"),
        (6, vec![6], String::from("key run @8 = \"x\" +7"))
    );
    assert_eq!(entry_of("-   run: x").1, vec![0]);
    assert_eq!(entry_of("- - a: b").1, vec![0, 2]);
    assert_eq!(entry_of("  -"), (2, vec![2], String::from("nothing")));
    assert_eq!(entry_of("- # c"), (0, vec![0], String::from("nothing")));
}

/// Text without a key colon is a scalar.
#[test]
fn scalars_have_no_key() {
    assert_eq!(entry_of("- main"), (0, vec![0], String::from("scalar")));
    assert_eq!(entry_of("[a, b]").2, "scalar");
    assert_eq!(entry_of("key:value").2, "scalar");
    assert_eq!(entry_of("-x: 1").2, "key -x @0 = \"1\" +4");
}

/// Quoted,
///  anchored,
///  tagged,
///  aliased and merge keys are not read.
#[test]
fn odd_keys_are_flagged() {
    for text in [
        "\"a\": 1",
        "'a': 1",
        "&anchor key: 1",
        "*alias : 1",
        "!tag key: 1",
        "<<: *base",
        "? complex: 1",
        "- \"a: b\"",
    ] {
        assert_eq!(entry_of(text).2, "odd", "{text}");
    }
}

/// A comment needs a space before the hash and no quote around it.
#[test]
fn comments_are_removed_outside_quotes() {
    assert_eq!(without_comment("a # b"), "a");
    assert_eq!(without_comment("a#b"), "a#b");
    assert_eq!(without_comment("\"a # b\" # c"), "\"a # b\"");
    assert_eq!(without_comment("'a # b'"), "'a # b'");
    assert_eq!(without_comment("# only"), "");
    assert_eq!(without_comment("a   "), "a");
}
