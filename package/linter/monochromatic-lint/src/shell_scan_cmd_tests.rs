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

/// Import the scanner and the construct names.
use super::scan;
use crate::shell_dialect::Construct;

/// The constructs found,
///  in source order.
fn kinds(text: &str) -> Vec<Construct> {
    let mut found: Vec<Construct> = Vec::new();
    for hit in scan(text) {
        found.push(hit.construct);
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
