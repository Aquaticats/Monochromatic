//! What:
//!  Pure decision of the add-explicit policy:
//!  `git add` names the paths it stages.
//! Why:
//!  `git add .`,
//!  `git add -A` and `git add -u` sweep up every changed path,
//!  including
//!      ones the caller never looked at,
//!  so one commit stops describing one change.
//!  The
//!      policy rejects those forms.
//!  This file decides from the arguments alone;
//!  it reads
//!      no file and starts no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // decideAddExplicit(['add', '.']) => 'cli-git: git add rejects bulk-staging patterns (.) ...'
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Which tokens stage in bulk comes from Git's own option table of `git add`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ADD_ESCAPE_HATCH, parseAddRegion } from './command_add.ts';
/// ```
use super::command_add::{ADD_ESCAPE_HATCH, AddRegion, parse_add_region};
use super::command_options::OptionError;
use super::global_arguments::command_tokens;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments are compared as bytes;
///  only the rejection text renders them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The finding code of an add-explicit rejection.
///  `&str` is borrowed text baked into
///       the program.
/// Why:
///   Callers identify the finding by this stable code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BULK_ADD_CODE = 'bulk-add-rejected';
/// ```
pub const BULK_ADD_CODE: &str = "bulk-add-rejected";

/// What:
///  Decide from the argument list:
///  the rejection text,
///  or nothing to let the command
///       through.
///  `&[OsString]` borrows the arguments,
///  already free of wrapper controls;
///       `Option<String>` is "owned text or nothing".
/// Why:
///   Only `git add` is judged,
///  and only when Git accepts its options.
///  The text quotes
///       the caller's own tokens,
///  so `-vA` is named as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideAddExplicit(args: string[]): string | undefined;
/// ```
pub fn decide_add_explicit(arguments: &[OsString]) -> Option<String> {
    // `let Some((a, b)) = ... else { ... };` unpacks the command word and the tokens after
    // it, or returns when Git runs no subcommand.
    let Some((word, region)) = command_tokens(arguments) else {
        // `None` is the "absent" case of `Option`.
        return None;
    };
    // `.as_encoded_bytes()` lends the raw bytes of the subcommand word.
    if word.as_encoded_bytes() != b"add" {
        return None;
    }
    // `Result<A, B>` is "either success `A` or failure `B`"; `&[]` is an empty flag list.
    let parsed: Result<AddRegion, OptionError> = parse_add_region(region, &[]);
    // `let Ok(x) = ... else { ... };` unwraps the facts or returns for a refused region.
    let Ok(facts) = parsed else {
        return None;
    };
    if facts.bulk_matches.is_empty() {
        return None;
    }
    // `Vec<String>` is an owned list of owned text; `mut` allows pushing.
    let mut tokens: Vec<String> = Vec::<String>::new();
    // `Option<usize>` is "the index of the token named last, or nothing".
    let mut named_last: Option<usize> = None;
    // `for found in &facts.bulk_matches` borrows each match in argument order.
    for found in &facts.bulk_matches {
        // One cluster such as `-Au` matches twice; it is named once.
        if named_last != Some(found.token) {
            // `.to_string_lossy()` renders a token as text; `.into_owned()` makes it owned.
            tokens.push(region[found.token].to_string_lossy().into_owned());
        }
        named_last = Some(found.token);
    }
    // `Some(x)` is the "present" case; `.join(", ")` builds one text with separators.
    return Some(format!(
        "cli-git: git add rejects bulk-staging patterns ({}) because they sweep up paths the \
         caller did not intend to stage, leaving the index in a state that does not match a \
         single logical change. Name the paths explicitly, or pass {ADD_ESCAPE_HATCH} to bypass \
         for this invocation.",
        tokens.join(", ")
    ));
}

/// Bulk forms,
///  explicit forms and the rejection text.
#[cfg(test)]
#[path = "rule_add_explicit_tests.rs"]
mod tests;
