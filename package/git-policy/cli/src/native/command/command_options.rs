//! What:
//!  Read one Git command's arguments the way Git 2.56.0 `parse-options.c` reads them.
//! Why:
//!  A policy that guesses which token is an option,
//!  a value or a path can be bypassed by a
//!      spelling Git accepts.
//!  Every per-command module declares Git's own option table and
//!      this module walks the arguments exactly once with Git's rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseOptions({ args, table, mode, wrapperFlags }): ParsedOptions  (throws OptionError)
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  which is where every sibling file of this crate is declared.
/// Why:
///   Long and short spellings follow different Git functions,
///  so each lives in its own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { scanLongOption } from './command_options_long.ts';
/// import { scanShortCluster } from './command_options_short.ts';
/// ```
use crate::command_options_long::scan_long_option;
use crate::command_options_short::scan_short_cluster;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   File names and messages may hold bytes that are not UTF-8;
///  `String` would reject them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The shapes the walk fills and hands back,
///  re-exported so a caller keeps naming them from the module that reads an option region.
/// Why:
///   Every file outside this folder spells `crate::command_options::OptionError` and its siblings,
///  and those names must keep resolving when the shapes move into a file of the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { Arity, OptionSpec, ParsedOptions } from './command_options_types.ts';
/// ```
pub use crate::command_options_types::{
    Arity,
    Boundary,
    DEFAULT_MODE,
    Occurrence,
    OptionError,
    OptionErrorKind,
    OptionSpec,
    OptionValue,
    ParseMode,
    ParsedOptions,
    UNREAD,
    WrapperOccurrence,
    row,
};

/// What:
///  Find a wrapper-only flag by exact byte spelling.
///  `&[u8]` borrows bytes;
///  `&[&[u8]]`
///       borrows a list of byte spellings.
///  The result is its list index or nothing.
/// Why:
///   Wrapper flags are never abbreviated and never carry `=value`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flag = wrapperFlags.indexOf(token); // -1 becomes "nothing"
/// ```
fn wrapper_flag_index(
    token: &[u8],
    wrapper_flags: &[&[u8]],
) -> Option<usize> {
    let mut index: usize = 0;
    while index < wrapper_flags.len() {
        if wrapper_flags[index] == token {
            // `Some(x)` is the "present" case of `Option`.
            return Some(index);
        }
        index += 1;
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  Tokenize one region in a single forward pass (`parse_options_step`,
///       parse-options.c:995-1169).
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   One faithful scan replaces per-command guesses about option arity.
/// Gotcha:
///  Indexes in the result are relative to `arguments`,
///  the region after the subcommand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseOptions({ args, table, mode, wrapperFlags }): ParsedOptions; // throws OptionError
/// ```
pub fn parse_options(
    arguments: &[OsString],
    table: &[OptionSpec],
    mode: ParseMode,
    wrapper_flags: &[&[u8]],
) -> Result<ParsedOptions, OptionError> {
    // `mut` allows the lists to grow; `Vec::new()` is an empty list.
    let mut parsed: ParsedOptions = ParsedOptions {
        occurrences: Vec::<Occurrence>::new(),
        wrapper: Vec::<WrapperOccurrence>::new(),
        unknown: Vec::<usize>::new(),
        leading: Vec::<usize>::new(),
        boundary: Boundary::End,
    };
    // True while the token being visited is the value of the option before it.
    let mut is_value: bool = false;
    // `.iter().enumerate()` visits every token once with its position, so the scan always
    // ends: an option only says whether the next token is its value.
    for (index, argument) in arguments.iter().enumerate() {
        if is_value {
            is_value = false;
            continue;
        }
        // `.as_encoded_bytes()` lends the raw bytes; nothing is decoded or copied.
        let token: &[u8] = argument.as_encoded_bytes();
        // `if let Some(flag) = ...` runs the block only when a value is present.
        if let Some(flag) = wrapper_flag_index(token, wrapper_flags) {
            parsed.wrapper.push(WrapperOccurrence {
                flag,
                token: index,
            });
            continue;
        }
        // A token not starting with `-`, the empty token and a lone `-` are not options (1011).
        if token.first() != Some(&b'-') || token.len() == 1 {
            if mode.stop_at_non_option {
                parsed.boundary = Boundary::NonOption(index);
                break;
            }
            parsed.leading.push(index);
            continue;
        }
        // A lone `-h` asks for usage (1050); a lone completion request prints the table
        // (1057-1060). Git exits 0 for both without running the command.
        if arguments.len() == 1
            && (token == b"-h"
                || token == b"--git-completion-helper"
                || token == b"--git-completion-helper-all")
        {
            // `Err(x)` is the failure case of `Result`.
            return Err(OptionError {
                kind: OptionErrorKind::HelpRequested,
                token: index,
            });
        }
        if token[1] != b'-' {
            // `?` returns the failure to our caller, or unwraps the success value.
            is_value = scan_short_cluster(arguments, index, table, mode, &mut parsed)?;
            continue;
        }
        if token.len() == 2 {
            parsed.boundary = Boundary::DashDash(index);
            break;
        }
        // `&token[2..]` borrows the bytes after the two dashes.
        let name: &[u8] = &token[2..];
        if name == b"end-of-options" {
            parsed.boundary = Boundary::EndOfOptions(index);
            break;
        }
        if name == b"help" || name == b"help-all" {
            return Err(OptionError {
                kind: OptionErrorKind::HelpRequested,
                token: index,
            });
        }
        is_value = scan_long_option(arguments, index, table, mode, &mut parsed)?;
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(parsed);
}

/// Synthetic-table cases,
///  ported incumbent cases and typed refusals.
#[cfg(test)]
#[path = "command_options_tests.rs"]
mod tests;

/// Differential controls against `git rev-parse --parseopt` of the real Git 2.56.0.
#[cfg(test)]
#[path = "command_options_git_tests.rs"]
mod git_tests;
