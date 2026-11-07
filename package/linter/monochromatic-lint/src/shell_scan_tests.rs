//! What:
//!  Controls for the dialect dispatch of the scan.
//! Why:
//!  The dispatch must drop constructs a dialect gives no composing meaning,
//!  or the rules would flag a PowerShell backtick escape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('scan dispatch', () => { /* per-dialect filtering */ });
//! ```

/// Import the dispatch,
///  the tables and the deadline that ends a scan which stops advancing.
use super::{expression_end, scan};
use crate::shell_dialect::{Construct, ShellDialect};
use crate::shell_scan::Hit;
use crate::test_deadline::finished_within_deadline;

/// The constructs found,
///  in source order.
fn kinds(text: &str, dialect: ShellDialect) -> Vec<Construct> {
    let mut found: Vec<Construct> = Vec::new();
    let owned: String = String::from(text);
    let scanned: Vec<Hit> = finished_within_deadline(move || return scan(owned.as_str(), dialect));
    for hit in scanned {
        found.push(hit.construct);
    }
    return found;
}

/// The same text means different things in different dialects.
#[test]
fn dialects_filter_their_own_constructs() {
    assert_eq!(
        kinds("echo `x`", ShellDialect::Posix),
        [Construct::Backtick]
    );
    assert_eq!(kinds("echo `x`", ShellDialect::Fish), []);
    assert_eq!(kinds("echo `x`", ShellDialect::PowerShell), []);
    assert_eq!(
        kinds("a |& b", ShellDialect::Posix),
        [Construct::PipeStderr]
    );
    assert_eq!(kinds("a ; b", ShellDialect::Cmd), []);
    assert_eq!(
        kinds("a ; b", ShellDialect::PowerShell),
        [Construct::Semicolon]
    );
    assert_eq!(kinds("a & b", ShellDialect::Cmd), [Construct::Ampersand]);
    assert_eq!(kinds("a | b", ShellDialect::Fish), [Construct::Pipe]);
}

/// An expression ends after its closing braces,
///  or at the end of the body when they never come,
///  and no expression starts where its opening does not.
#[test]
fn shell_an_expression_ends_after_its_closing_braces_or_at_the_end_of_the_body() {
    let closed: &[u8] = b"${{ a }} b";
    let after_closing: usize = closed
        .windows(2)
        .position(|pair| return pair == b"}}")
        .expect("closing braces")
        + 2;
    assert_eq!(expression_end(closed, 0), Some(after_closing));
    let later: &[u8] = b"x ${{ a }}";
    assert_eq!(expression_end(later, 2), Some(later.len()));
    let empty: &[u8] = b"${{}}";
    assert_eq!(expression_end(empty, 0), Some(empty.len()));
    let open: &[u8] = b"${{ a";
    assert_eq!(expression_end(open, 0), Some(open.len()));
    let one_brace: &[u8] = b"${{ a }";
    assert_eq!(expression_end(one_brace, 0), Some(one_brace.len()));
    assert_eq!(expression_end(b"echo", 0), None);
}

/// The end of an expression is found after text before it and with text after it,
///  and a single closing brace does not end it.
#[test]
fn an_expression_ends_after_its_own_closing_braces_whatever_surrounds_it() {
    let inside: &[u8] = b"abcd${{}} x";
    let end_of_inside: usize = inside
        .windows(2)
        .position(|pair| return pair == b"}}")
        .expect("closing braces")
        + 2;
    assert_eq!(expression_end(inside, 4), Some(end_of_inside));
    let single: &[u8] = b"${{ a } b }}";
    assert_eq!(expression_end(single, 0), Some(single.len()));
}
