//! What:
//!  Controls for the PowerShell scanner.
//! Why:
//!  The backtick escapes,
//!  quotes double instead of escaping,
//!  and a leading `&` calls a command,
//!  none of which the POSIX scanner knows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('powershell scan', () => { /* operators, quoting, call operator, here-strings */ });
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

/// The byte range of the `nth` occurrence of `needle` in `text`,
///  counted from zero.
fn nth_span(text: &str, needle: &str, nth: usize) -> (usize, usize) {
    let start: usize = text
        .match_indices(needle)
        .nth(nth)
        .expect("the needle occurs often enough")
        .0;
    return (start, start + needle.len());
}

/// Assert that a scan finds exactly the given constructs,
///  each over exactly the given occurrence of the given text.
fn assert_hits(text: &str, expected: &[(Construct, &str, usize)]) {
    let mut wanted: Vec<(Construct, usize, usize)> = Vec::new();
    for (construct, needle, nth) in expected {
        let span: (usize, usize) = nth_span(text, needle, *nth);
        wanted.push((*construct, span.0, span.1));
    }
    let mut found: Vec<(Construct, usize, usize)> = Vec::new();
    for hit in hits(text) {
        found.push((hit.construct, hit.start, hit.end));
    }
    assert_eq!(found, wanted, "{text:?}");
}

/// Each PowerShell 7 operator is found once.
#[test]
fn operators_are_found() {
    assert_eq!(kinds("a | b"), [Construct::Pipe]);
    assert_eq!(
        kinds("a && b || c"),
        [Construct::AndChain, Construct::OrChain]
    );
    assert_eq!(kinds("a; b"), [Construct::Semicolon]);
    assert_eq!(kinds("$(Get-Date)"), [Construct::DollarParen]);
    assert_eq!(kinds("Get-Process &"), [Construct::Ampersand]);
    assert_eq!(kinds("Write-Output hi"), []);
}

/// A leading `&` calls a command and `>&` redirects a stream;
///  neither is a background operator.
#[test]
fn call_operator_and_redirection_are_not_background() {
    assert_eq!(kinds("& 'tool' arg"), []);
    assert_eq!(kinds("& $cmd | foo"), [Construct::Pipe]);
    assert_eq!(kinds("a; & b"), [Construct::Semicolon]);
    assert_eq!(kinds("a | & b"), [Construct::Pipe]);
    assert_eq!(kinds("cmd 2>&1"), []);
    assert_eq!(kinds("a\n& b"), []);
    assert_eq!(kinds("a & b"), [Construct::Ampersand]);
}

/// The backtick escapes the next byte,
///  so an escaped operator is text.
#[test]
fn backtick_escapes() {
    assert_eq!(kinds("a `| b"), []);
    assert_eq!(kinds("a `\n | b"), [Construct::Pipe]);
}

/// Single quotes double their quote and hide everything;
///  double quotes still run `$(`.
#[test]
fn quotes() {
    assert_eq!(kinds("'$(x) | y; z'"), []);
    assert_eq!(kinds("'it''s | x'"), []);
    assert_eq!(kinds("\"a | b; c\""), []);
    assert_eq!(kinds("\"$(Get-Date)\""), [Construct::DollarParen]);
    assert_eq!(kinds("\"say \"\"a | b\"\"\""), []);
    assert_eq!(kinds("\"a `\" | b\""), []);
    assert_eq!(kinds("\"unterminated | x"), []);
    assert_eq!(kinds("$(a | b)"), [Construct::DollarParen, Construct::Pipe]);
    assert_eq!(kinds("$((1 + 2) | x)").len(), 2);
}

/// Line comments and block comments hide operators.
#[test]
fn comments() {
    assert_eq!(kinds("# a | b"), []);
    assert_eq!(kinds("<# a | b #> c | d"), [Construct::Pipe]);
    assert_eq!(kinds("<# never closed | x"), []);
    assert_eq!(kinds("a#b|c"), [Construct::Pipe]);
}

/// A here-string is data,
///  but `@"` still runs `$(`.
#[test]
fn here_strings() {
    assert_eq!(kinds("@'\na | b\n'@\nc | d"), [Construct::Pipe]);
    assert_eq!(kinds("@\"\n$(x) | y\n\"@\n"), [Construct::DollarParen]);
    assert_eq!(kinds("@'\nnever ends | x"), []);
    assert_eq!(kinds("@'not a here-string' | x"), [Construct::Pipe]);
}

/// GitHub expressions are not shell.
#[test]
fn expressions_are_skipped() {
    assert_eq!(kinds("echo ${{ a && b }} | x"), [Construct::Pipe]);
}

/// Each operator is reported over exactly its own bytes.
#[test]
fn operators_are_reported_over_their_own_bytes() {
    assert_hits("abc | d", &[(Construct::Pipe, "|", 0)]);
    assert_hits("abc || d", &[(Construct::OrChain, "||", 0)]);
    assert_hits("abc && d", &[(Construct::AndChain, "&&", 0)]);
    assert_hits("abc ; d", &[(Construct::Semicolon, ";", 0)]);
    assert_hits("abc & d", &[(Construct::Ampersand, "&", 0)]);
    assert_hits("abc $(d)", &[(Construct::DollarParen, "$(", 0)]);
}

/// Single-quoted text ends at its closing quote,
///  a doubled quote is one literal quote,
///  and an unclosed quote runs to the end.
#[test]
fn single_quotes_resume_scanning_at_the_byte_after_them() {
    assert_hits("ab 'x' | y", &[(Construct::Pipe, "|", 0)]);
    assert_hits("ab 'it''s | x' | y", &[(Construct::Pipe, "|", 1)]);
    assert_hits("ab 'abc | d", &[]);
    assert_hits("ab 'a''", &[]);
    assert_hits("ab 'a'", &[]);
}

/// Double-quoted text ends at its closing quote,
///  a backtick or a doubled quote keeps it open,
///  and `$(` inside runs code.
#[test]
fn double_quotes_resume_scanning_at_the_byte_after_them() {
    assert_hits("ab \"a `\" | b\" | c", &[(Construct::Pipe, "|", 1)]);
    assert_hits(
        "ab \"say \"\"a | b\"\" x\" | c",
        &[(Construct::Pipe, "|", 1)],
    );
    assert_hits(
        "ab \"$(a | b)\" | c",
        &[
            (Construct::DollarParen, "$(", 0),
            (Construct::Pipe, "|", 0),
            (Construct::Pipe, "|", 1),
        ],
    );
    assert_hits("ab \"x\" | c", &[(Construct::Pipe, "|", 0)]);
    assert_hits(
        "$((1 + 2) | x)",
        &[(Construct::DollarParen, "$(", 0), (Construct::Pipe, "|", 0)],
    );
}

/// A here-string body is skipped from its opener line to its closer,
///  at exact bytes,
///  with every `$(` of a live one reported.
#[test]
fn here_strings_end_at_their_closer_and_report_every_live_substitution() {
    assert_hits("ab\n@'\na | b\n'@\nc | d", &[(Construct::Pipe, "|", 1)]);
    assert_hits(
        "ab\n@'  \t\r\na | b\n'@\nc | d",
        &[(Construct::Pipe, "|", 1)],
    );
    assert_hits("@'\r\na | b\r\n'@\r\nc | d", &[(Construct::Pipe, "|", 1)]);
    assert_hits(
        "ab\n@\"\n$(x)\n\"@\nc | d",
        &[(Construct::DollarParen, "$(", 0), (Construct::Pipe, "|", 0)],
    );
    assert_hits(
        "@\"\n$(a) $(b)\n\"@\nc | d",
        &[
            (Construct::DollarParen, "$(", 0),
            (Construct::DollarParen, "$(", 1),
            (Construct::Pipe, "|", 0),
        ],
    );
    assert_hits("@\"\n$(x) | y", &[(Construct::DollarParen, "$(", 0)]);
    assert_hits("@'\nnever ends | x", &[]);
}

/// An `@` that does not open a here-string is plain text.
#[test]
fn an_at_sign_that_opens_no_here_string_is_text() {
    assert_hits("@x\n a | b", &[(Construct::Pipe, "|", 0)]);
    assert_hits("@'x\n a | b", &[]);
    assert_hits("@'", &[]);
    assert_hits("@\"", &[]);
    assert_hits("@", &[]);
    assert_hits("@'a' | b", &[(Construct::Pipe, "|", 0)]);
}

/// A comment starts only at the start of a word,
///  and a block comment ends at its closer.
#[test]
fn comments_end_where_they_end() {
    assert_hits("a # b | c\nd | e", &[(Construct::Pipe, "|", 1)]);
    assert_hits("xy <# a | b #>| d", &[(Construct::Pipe, "|", 1)]);
    assert_hits("xy <# a | b", &[]);
}
