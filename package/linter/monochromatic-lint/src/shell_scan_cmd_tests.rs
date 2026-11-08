//! What:
//!  Controls for the `cmd` scanner.
//! Why:
//!  A caret escapes,
//!  `rem` and `::` comment out a line,
//!  and `2>&1` is a redirection,
//!  which decides whether `&` and `|` are operators.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('cmd scan', () => { /* operators, escapes, quotes, comments */ });
//! ```

/// Import the scanner,
///  the construct names and the deadline that ends a scan which stops advancing.
use super::scan;
use crate::shell_dialect::Construct;
use crate::shell_scan::Hit;
use crate::test_deadline::finished_within_deadline;

/// The hits of a scan that must finish before the deadline.
fn hits(text: &str) -> Vec<Hit> {
    let owned: String = String::from(text);
    return finished_within_deadline(move || return scan(owned.as_str()));
}

/// The constructs found,
///  in source order.
fn kinds(text: &str) -> Vec<Construct> {
    let mut found: Vec<Construct> = Vec::new();
    for hit in hits(text) {
        found.push(hit.construct);
    }
    return found;
}

/// The constructs found with their byte ranges,
///  in source order.
fn ranges(text: &str) -> Vec<(Construct, usize, usize)> {
    let mut found: Vec<(Construct, usize, usize)> = Vec::new();
    for hit in hits(text) {
        found.push((hit.construct, hit.start, hit.end));
    }
    return found;
}

/// The four `cmd` operators are found.
#[test]
fn operators_are_found() {
    assert_eq!(kinds("a | b"), [Construct::Pipe]);
    assert_eq!(kinds("a || b"), [Construct::OrChain]);
    assert_eq!(kinds("a && b"), [Construct::AndChain]);
    assert_eq!(kinds("a & b"), [Construct::Ampersand]);
    assert_eq!(kinds("echo plain"), []);
}

/// Escapes,
///  quotes,
///  and a redirection hide `&` and `|`.
#[test]
fn escapes_quotes_and_redirections() {
    assert_eq!(kinds("echo a ^& b"), []);
    assert_eq!(kinds("echo a ^| b"), []);
    assert_eq!(kinds("echo \"a | b && c & d\""), []);
    assert_eq!(kinds("echo \"unterminated | b"), []);
    assert_eq!(kinds("dir 2>&1"), []);
}

/// A line that starts with `rem` or `::` is a comment.
#[test]
fn comment_lines() {
    assert_eq!(kinds("rem a & b\nc | d"), [Construct::Pipe]);
    assert_eq!(kinds("  REM a & b"), []);
    assert_eq!(kinds(":: a | b"), []);
    assert_eq!(kinds("rem"), []);
    assert_eq!(kinds("remove | x"), [Construct::Pipe]);
    assert_eq!(kinds("rmdir a | b"), [Construct::Pipe]);
    assert_eq!(kinds("echo x & rem y"), [Construct::Ampersand]);
}

/// GitHub expressions are not shell.
#[test]
fn expressions_are_skipped() {
    assert_eq!(kinds("echo ${{ a && b }} | x"), [Construct::Pipe]);
}

/// Each operator is reported over exactly its own bytes,
///  and the scan resumes after them.
#[test]
fn operators_are_reported_over_their_own_bytes() {
    assert_eq!(ranges("abc | d"), [(Construct::Pipe, 4, 5)]);
    assert_eq!(ranges("abc || d"), [(Construct::OrChain, 4, 6)]);
    assert_eq!(ranges("abc && d"), [(Construct::AndChain, 4, 6)]);
    assert_eq!(ranges("abc & d"), [(Construct::Ampersand, 4, 5)]);
    assert_eq!(
        ranges("a || b | c"),
        [(Construct::OrChain, 2, 4), (Construct::Pipe, 7, 8)]
    );
}

/// An ampersand at the very start of the body is an operator,
///  and one after a redirection is not.
#[test]
fn an_ampersand_at_the_start_is_an_operator_and_one_after_a_redirect_is_not() {
    assert_eq!(ranges("& x"), [(Construct::Ampersand, 0, 1)]);
    assert_eq!(kinds("a >& b | c"), [Construct::Pipe]);
}

/// A caret at the very start of the body escapes the next byte.
#[test]
fn a_caret_at_the_start_escapes_the_next_byte() {
    assert_eq!(kinds("^& x"), []);
}

/// A comment word is only a comment at the start of a line,
///  after a line break and indentation as well as at the start of the body,
///  and a body that ends inside the word is not read past its end.
#[test]
fn a_comment_word_is_a_comment_only_at_the_start_of_a_line() {
    assert_eq!(kinds("x\nrem a & b"), []);
    assert_eq!(kinds("x\n \t rem a & b"), []);
    assert_eq!(kinds("x\n:: a | b\ny & z"), [Construct::Ampersand]);
    assert_eq!(
        kinds("echo x & rem y | z"),
        [Construct::Ampersand, Construct::Pipe]
    );
    assert_eq!(kinds("re"), []);
    assert_eq!(kinds("x\nr"), []);
    assert_eq!(kinds("x\nrem"), []);
}
