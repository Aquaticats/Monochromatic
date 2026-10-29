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

/// Import the dispatch and the tables.
use super::scan;
use crate::shell_dialect::{Construct, ShellDialect};

/// The constructs found,
///  in source order.
fn kinds(text: &str, dialect: ShellDialect) -> Vec<Construct> {
    let mut found: Vec<Construct> = Vec::new();
    for hit in scan(text, dialect) {
        found.push(hit.construct);
    }
    return found;
}

/// The same text means different things in different dialects.
#[test]
fn dialects_filter_their_own_constructs() {
    assert_eq!(kinds("echo `x`", ShellDialect::Posix), [Construct::Backtick]);
    assert_eq!(kinds("echo `x`", ShellDialect::Fish), []);
    assert_eq!(kinds("echo `x`", ShellDialect::PowerShell), []);
    assert_eq!(kinds("a |& b", ShellDialect::Posix), [Construct::PipeStderr]);
    assert_eq!(kinds("a ; b", ShellDialect::Cmd), []);
    assert_eq!(kinds("a ; b", ShellDialect::PowerShell), [Construct::Semicolon]);
    assert_eq!(kinds("a & b", ShellDialect::Cmd), [Construct::Ampersand]);
    assert_eq!(kinds("a | b", ShellDialect::Fish), [Construct::Pipe]);
}
