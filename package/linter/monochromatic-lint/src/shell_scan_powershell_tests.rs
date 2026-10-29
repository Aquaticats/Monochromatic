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

/// Each PowerShell 7 operator is found once.
#[test]
fn operators_are_found() {
    assert_eq!(kinds("a | b"), [Construct::Pipe]);
    assert_eq!(kinds("a && b || c"), [Construct::AndChain, Construct::OrChain]);
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
