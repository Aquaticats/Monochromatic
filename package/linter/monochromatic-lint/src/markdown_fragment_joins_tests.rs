//! What:
//!  Fragment-join decisions on literal paragraph sources.
//! Why:
//!  A join must change one line ending and the spaces around it,
//!  and must decline every line the rule itself owns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Assert each declining reason and one join, then apply the answers as edits.
//! ```

/// Import the join the module builds and its limit.
use super::{
    FragmentJoin,
    JOINED_LINE_LIMIT,
    fragment_joins,
};
/// Import the grouped edit application the rule's fix goes through.
use crate::edits::{
    Edit,
    Fix,
    FixClass,
    apply_fixes,
};

/// The offset a fragment starts at,
///  taken from the source the test writes rather than counted by hand.
fn fragment_at(
    source: &str,
    written: &str,
) -> usize {
    return source.find(written).expect("the test writes this fragment");
}

/// Apply the answers exactly as the rule's fix does,
///  one space for the ending and indentation of each joined fragment.
fn joined(
    source: &str,
    fragments: &[usize],
    indent_width: usize,
) -> String {
    let answers: Vec<Option<FragmentJoin>> =
        fragment_joins(source, source.len(), fragments, indent_width);
    let mut edits: Vec<Edit> = Vec::<Edit>::new();
    for answer in answers.into_iter().flatten() {
        edits.push(Edit {
            start: answer.start,
            end: answer.end,
            replacement: String::from(" "),
        });
    }
    let fix: Fix = Fix {
        class: FixClass::Whitespace,
        edits,
    };
    return apply_fixes(source, &[fix]).expect("apply").source;
}

/// A word the old wrap cut off joins the line that continues its clause.
#[test]
fn a_fragment_the_old_wrap_cut_joins_the_clause_that_continues_it() {
    let source: &str = "Owns the record family, the\nmanaged lock's words, the finder.\n";
    let fragment: usize = fragment_at(source, "the\n");
    assert_eq!(
        joined(source, &[fragment], 0),
        "Owns the record family, the managed lock's words, the finder.\n"
    );
}

/// A fragment ending at a break-point character is where the rule puts the line break,
///  so joining it would only have the rule break it again.
#[test]
fn a_fragment_ending_at_a_break_point_stays_on_its_line() {
    let source: &str = "The counts were\n4,\n5,\nheld,\nfor the pass.\n";
    for written in ["4,\n", "5,\n", "held,\n"] {
        assert_eq!(
            joined(source, &[fragment_at(source, written)], 0),
            source,
            "{written}"
        );
    }
    // A closing delimiter moves the rule's break past it, so it counts as the same ending.
    let quoted: &str = "The counts were\n\"4.\"\nfor the pass.\n";
    assert_eq!(
        joined(quoted, &[fragment_at(quoted, "\"4.\"\n")], 0),
        quoted
    );
}

/// A following line that opens a block carries a marker a whitespace-only fix may not delete.
#[test]
fn a_following_line_that_opens_a_block_keeps_its_marker() {
    for source in [
        "Owns the record family, the\n> managed lock's words.\n",
        "Owns the record family, the\n- managed lock's words.\n",
        "Owns the record family, the\n# managed lock's words.\n",
        "Owns the record family, the\n```\nmanaged lock's words.\n",
    ] {
        assert_eq!(
            joined(source, &[fragment_at(source, "the\n")], 0),
            source,
            "{source}"
        );
    }
}

/// A line ending the author marked as a hard break,
///  and a line that holds no word,
///  are the author's own line structure.
#[test]
fn a_marked_or_empty_line_is_not_a_fragment_to_join() {
    for source in [
        "Owns the record family, the  \nmanaged lock's words.\n",
        "Owns the record family, the\\\nmanaged lock's words.\n",
        "Owns the record family, the\n\nmanaged lock's words.\n",
        "Owns the record family, the\n",
    ] {
        assert_eq!(
            joined(source, &[fragment_at(source, "the")], 0),
            source,
            "{source:?}"
        );
    }
}

/// A join may not write a line longer than the limit,
///  which counts the indent the line is written with.
#[test]
fn a_join_that_would_pass_the_limit_is_declined() {
    let written: &str = &"x".repeat(JOINED_LINE_LIMIT);
    let source: String = format!("Owns the record family, the\n{written}\n");
    assert_eq!(
        joined(source.as_str(), &[fragment_at(source.as_str(), "the\n")], 0),
        source
    );
    let short: String = format!("Owns the record family, the\n{}\n", &"x".repeat(3));
    assert_ne!(
        joined(short.as_str(), &[fragment_at(short.as_str(), "the\n")], 0),
        short
    );
}

/// One answer arrives per fragment,
///  and a CRLF ending loses both of its bytes.
#[test]
fn each_answer_belongs_to_its_fragment_and_crlf_is_removed_whole() {
    let source: &str = "One clause, the\r\nnext clause here\r\nand the rest, the\r\nlast clause.\r\n";
    let first: usize = fragment_at(source, "the\r");
    let second: usize = fragment_at(&source[first + 1..], "the\r") + first + 1;
    let answers: Vec<Option<FragmentJoin>> = fragment_joins(source, source.len(), &[first, second], 0);
    assert_eq!(answers.len(), 2);
    let joined_first: &FragmentJoin = answers[0].as_ref().expect("the first fragment joins");
    assert_eq!(&source[joined_first.start..joined_first.end], "\r\n");
    assert_eq!(
        joined(source, &[first, second], 0),
        "One clause, the next clause here\r\nand the rest, the last clause.\r\n"
    );
}
