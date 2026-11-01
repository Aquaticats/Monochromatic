//! What:
//!  The complete `git status` option table of Git 2.56.0,
//!  the machine-readable fact
//!       read from it,
//!  and the reading of `advice.statusHints` set before the subcommand.
//! Why:
//!  The status-hints transform must not override an explicit caller choice,
//!  and the
//!      wrapper adds no note to output a program will parse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseStatusPreRegion(pre).hasStatusHintsOverride; parseStatusPostRegion(post).isMachineReadable
//! ```

/// What:
///  Bring the tokenizer,
///  its table builder,
///  its questions and the list of global
///       options that take a separate value into this file.
/// Why:
///   The post-subcommand region uses Git's status table;
///  the pre-subcommand region is
///       walked with Git's global option arity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions, row } from './command_options.ts';
/// ```
use super::command_options::{
    Arity, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, UNREAD, WrapperOccurrence,
    parse_options, row,
};
use super::command_options_query::is_enabled;
use super::global_arguments::VALUE_OPTIONS;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// `-s`,
///  `--short`.
pub const SHORT: u16 = 1;
/// `--porcelain[=<version>]`.
pub const PORCELAIN: u16 = 2;
/// `--long`.
pub const LONG: u16 = 3;
/// `-z`,
///  `--null`.
pub const NULL: u16 = 4;

/// What:
///  Rows of `builtin_status_options` (builtin/commit.c:1548-1599),
///  in source order.
///       `&[OptionSpec]` is a borrowed table baked into the program.
/// Why:
///   Four rows take an optional value (`PARSE_OPT_OPTARG`),
///  so a token after them is a
///       pathspec,
///  not their value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const STATUS_TABLE = [row(UNREAD, 'v', 'verbose', 'none', true), /* ... */] as const;
/// ```
pub const STATUS_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true),
    row(SHORT, Some(b's'), Some("short"), Arity::None, true),
    row(UNREAD, Some(b'b'), Some("branch"), Arity::None, true),
    row(UNREAD, None, Some("show-stash"), Arity::None, true),
    row(UNREAD, None, Some("ahead-behind"), Arity::None, true),
    row(PORCELAIN, None, Some("porcelain"), Arity::Optional, true),
    row(LONG, None, Some("long"), Arity::None, true),
    row(NULL, Some(b'z'), Some("null"), Arity::None, true),
    row(
        UNREAD,
        Some(b'u'),
        Some("untracked-files"),
        Arity::Optional,
        true,
    ),
    row(UNREAD, None, Some("ignored"), Arity::Optional, true),
    row(
        UNREAD,
        None,
        Some("ignore-submodules"),
        Arity::Optional,
        true,
    ),
    row(UNREAD, None, Some("column"), Arity::Optional, true),
    row(UNREAD, None, Some("no-renames"), Arity::None, true),
    // `PARSE_OPT_OPTARG | PARSE_OPT_NONEG` (commit.c:1596-1598).
    row(
        UNREAD,
        Some(b'M'),
        Some("find-renames"),
        Arity::Optional,
        false,
    ),
];

/// Config key,
///  lower-cased,
///  whose explicit setting the transform respects.
const ADVICE_KEY: &[u8] = b"advice.statushints";

/// What:
///  Facts of the region after `status`.
///  `bool` is true or false.
/// Why:
///   Short,
///  porcelain and NUL-terminated output is parsed by programs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StatusPostRegion = { isMachineReadable: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusRegion {
    /// The final format is short or porcelain,
    ///  or `-z` is on.
    pub machine_readable: bool,
    /// Wrapper-only flags in option position;
    ///  `flag` indexes the caller's list.
    pub wrapper: Vec<WrapperOccurrence>,
}

/// What:
///  Whether the final status format is short or porcelain.
/// Why:
///   `--short`,
///  `--porcelain` and `--long` write one shared variable,
///  so only the last
///       counts;
///  `--long` and every negated form select human-readable output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let machine = false; for (const o of occurrences) if (isFormat(o.id)) machine = o.id !== LONG && !o.negated;
/// ```
fn has_machine_format(parsed: &ParsedOptions) -> bool {
    let mut machine: bool = false;
    // `for occurrence in &parsed.occurrences` borrows each record in the order Git applied it.
    for occurrence in &parsed.occurrences {
        if occurrence.id == SHORT || occurrence.id == PORCELAIN {
            machine = !occurrence.negated;
        }
        if occurrence.id == LONG {
            machine = false;
        }
    }
    return machine;
}

/// What:
///  Parse the region after `status`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal,
///  never a guessed fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStatusPostRegion(region: string[]): StatusPostRegion; // throws
/// ```
pub fn parse_status_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<StatusRegion, OptionError> {
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions = parse_options(region, STATUS_TABLE, DEFAULT_MODE, wrapper_flags)?;
    // `Ok(x)` is the success case of `Result`.
    return Ok(StatusRegion {
        machine_readable: has_machine_format(&parsed) || is_enabled(&parsed, NULL),
        wrapper: parsed.wrapper,
    });
}

/// What:
///  Whether `key` names `advice.statusHints`.
///  `&[u8]` borrows the key bytes.
/// Why:
///   Git compares section and variable names without regard to ASCII case,
///  and the
///       comparison must not decode bytes that may not be UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAdviceKey = key.toLowerCase() === 'advice.statushints';
/// ```
fn is_advice_key(key: &[u8]) -> bool {
    return key.eq_ignore_ascii_case(ADVICE_KEY);
}

/// What:
///  Byte index of the first (`last == false`) or last (`last == true`) `=` in `bytes`.
///       `Option<usize>` is "an index or nothing".
/// Why:
///   `-c name=value` splits at the first `=` (config.c:495);
///  `--config-env=name=var`
///       splits at the last (config.c:511).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = last ? text.lastIndexOf('=') : text.indexOf('=');
/// ```
fn equals_index(bytes: &[u8], last: bool) -> Option<usize> {
    let mut found: Option<usize> = None;
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index] == b'=' {
            // `Some(x)` is the "present" case of `Option`.
            found = Some(index);
            if !last {
                return found;
            }
        }
        index += 1;
    }
    return found;
}

/// What:
///  Whether one configuration parameter sets `advice.statusHints`.
///  `parameter` is the
///       text after `-c` or after `--config-env`.
/// Why:
///   A bare `-c advice.statusHints` is Git's boolean-true spelling and is as explicit
///       as the valued form;
///  `--config-env` always needs `=<variable>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const key = parameter.split('=')[0]; return key.toLowerCase() === 'advice.statushints';
/// ```
fn sets_advice_key(parameter: &[u8], from_environment: bool) -> bool {
    // `if let Some(at) = ...` runs only when the parameter contains `=`.
    if let Some(at) = equals_index(parameter, from_environment) {
        // `&parameter[..at]` borrows the bytes before the `=`.
        return is_advice_key(&parameter[..at]);
    }
    return !from_environment && is_advice_key(parameter);
}

/// What:
///  Whether the caller configured `advice.statusHints` in the global options before
///       the subcommand.
///  `global_prefix` is exactly those tokens.
/// Why:
///   `git -c advice.statusHints=true status` keeps Git's hints;
///  the wrapper neither
///       overrides the choice nor adds its own note.
/// Gotcha:
///  Divergence from the incumbent:
///  `--config-env` sets the same key and counts too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStatusPreRegion(pre: readonly string[]): { hasStatusHintsOverride: boolean };
/// ```
pub fn has_status_hints_override(global_prefix: &[OsString]) -> bool {
    // The value-taking option whose value is the token being visited, when there is one.
    // `Option<&[u8]>` is "borrowed bytes or nothing"; `mut` allows setting and clearing it.
    let mut awaiting_value: Option<&[u8]> = None;
    // `for argument in global_prefix` borrows each token once, in order, so the scan
    // always ends: an option only marks the next token as its value.
    for argument in global_prefix {
        // `.as_encoded_bytes()` lends the raw bytes; nothing is decoded or copied.
        let token: &[u8] = argument.as_encoded_bytes();
        // `if let Some(option) = ...` runs only while a value is awaited.
        if let Some(option) = awaiting_value {
            // `None` is the "absent" case: this token settles the option.
            awaiting_value = None;
            if option == b"-c" && sets_advice_key(token, false) {
                return true;
            }
            if option == b"--config-env" && sets_advice_key(token, true) {
                return true;
            }
            continue;
        }
        // `.strip_prefix(...)` returns the remainder when the bytes start with the prefix.
        if let Some(joined) = token.strip_prefix(b"--config-env=") {
            if sets_advice_key(joined, true) {
                return true;
            }
            continue;
        }
        if VALUE_OPTIONS.contains(&token) {
            // `Some(x)` is the "present" case: the next token is this option's value.
            awaiting_value = Some(token);
        }
    }
    return false;
}

/// Facts in every accepted spelling,
///  and real-Git controls of the table.
#[cfg(test)]
#[path = "command_status_tests.rs"]
mod tests;
