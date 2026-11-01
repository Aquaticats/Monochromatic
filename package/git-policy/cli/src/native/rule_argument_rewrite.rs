//! What:
//!  The shared result and builder of fixed transforms that add tokens to a command line.
//! Why:
//!  A transform either leaves the caller's arguments alone or returns a new owned list;
//!      saying which lets the caller keep the original vector without comparing contents.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type ArgumentRewrite = { kind: 'unchanged' } | { kind: 'rewritten'; args: string[] };
//! ```

/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The outcome of one transform.
///  An `enum` is a closed set of named alternatives;
///       `Rewritten` carries the new owned argument list,
///  `Vec<OsString>`.
/// Why:
///   The TypeScript rules returned the same array object to mean "unchanged";
///  Rust
///       says it with a variant instead of object identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ArgumentRewrite = { kind: 'unchanged' } | { kind: 'rewritten'; args: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentRewrite {
    /// Forward the caller's arguments as they are.
    Unchanged,
    /// Forward this list instead.
    Rewritten(Vec<OsString>),
}

/// What:
///  Copy `arguments` with `inserted` placed before index `at`.
///  `&[&str]` borrows a
///       list of text spellings;
///  `usize` is the list index type.
/// Why:
///   Injected tokens are ASCII flags,
///  while every existing argument keeps its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = [...args.slice(0, at), ...inserted, ...args.slice(at)];
/// ```
pub fn insert_tokens(arguments: &[OsString], at: usize, inserted: &[&str]) -> Vec<OsString> {
    // `with_capacity` reserves room for the known final length.
    let mut result: Vec<OsString> =
        Vec::<OsString>::with_capacity(arguments.len() + inserted.len());
    let mut index: usize = 0;
    while index < arguments.len() {
        if index == at {
            // `for token in inserted` borrows each spelling; `OsString::from` copies it.
            for token in inserted {
                result.push(OsString::from(token));
            }
        }
        // `.clone()` copies the argument bytes into the owned result.
        result.push(arguments[index].clone());
        index += 1;
    }
    if at >= arguments.len() {
        for token in inserted {
            result.push(OsString::from(token));
        }
    }
    return result;
}

/// Insertion at the start,
///  middle and end.
#[cfg(test)]
#[path = "rule_argument_rewrite_tests.rs"]
mod tests;
