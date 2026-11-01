//! What:
//!  Match one `--name[=value]` token against a Git option table as Git 2.56.0 does.
//! Why:
//!  Git accepts any unambiguous prefix (`--am` for `--amend`),
//!  `--no-` negation and
//!      `--no-no-` forms (`parse_long_opt`,
//!  parse-options.c:519-594).
//!  A list of exact
//!      spellings misses forms Git accepts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // scanLongOption({ args, index, table, mode, parsed }): number  // tokens consumed
//! ```

/// What:
///  Bring the tokenizer types and the shared value decision into this file.
/// Why:
///   A long option appends to the same result the short-cluster scan appends to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Occurrence, OptionError, OptionSpec, ParsedOptions, ParseMode } from './command_options.ts';
/// import { takeValue } from './command_options_value.ts';
/// ```
use super::command_options::{
    Occurrence, OptionError, OptionErrorKind, OptionSpec, OptionValue, ParseMode, ParsedOptions,
};
use super::command_options_value::{Spelling, take_value};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The row an abbreviation resolved to,
///  with whether it is used in negated form.
/// Why:
///   Git remembers the latest abbreviation candidate until the whole table is checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Candidate = { spec: OptionSpec; unset: boolean };
/// ```
#[derive(Clone, Copy)]
struct Candidate {
    /// The candidate row.
    spec: OptionSpec,
    /// True when the token turns the option off.
    unset: bool,
}

/// What:
///  Record a matched option and say whether it took the next token as its value.
///       `&mut ParsedOptions` lends the result for writing;
///  `bool` is true or false.
/// Why:
///   Exact and abbreviated matches end the same way (`get_value`).
///  The caller visits
///       every token once and only needs to know whether to pass over the next one,
///  so
///       no count is returned that a mistake could turn into "stay on this token".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function accept({ args, index, spec, unset, attached, parsed }): boolean; // throws OptionError
/// ```
fn accept(
    arguments: &[OsString],
    index: usize,
    chosen: Candidate,
    attached: Option<usize>,
    parsed: &mut ParsedOptions,
) -> Result<bool, OptionError> {
    let spelling: Spelling = if chosen.unset {
        Spelling::NegatedLong
    } else {
        Spelling::Long
    };
    // `?` returns a refusal to our caller, or unwraps the value position.
    let value: Option<OptionValue> =
        take_value(arguments, index, chosen.spec.arity, attached, spelling)?;
    parsed.occurrences.push(Occurrence {
        id: chosen.spec.id,
        negated: chosen.unset,
        value,
        token: index,
    });
    // `matches!(x, pattern)` is true when `x` has that shape: here, a next-token value.
    return Ok(matches!(value, Some(OptionValue::Detached { .. })));
}

/// What:
///  Byte index of the first `=` in `bytes`,
///  or the length when there is none.
/// Why:
///   Git compares abbreviations up to the `=` only (`strchrnul`,
///  parse-options.c:523).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const end = bytes.indexOf('=') === -1 ? bytes.length : bytes.indexOf('=');
/// ```
fn name_end(bytes: &[u8]) -> usize {
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index] == b'=' {
            return index;
        }
        index += 1;
    }
    return bytes.len();
}

/// What:
///  Read the long option at `index` and return how many tokens it used (1 or 2).
/// Why:
///   One table walk finds an exact match first and otherwise the unique abbreviation.
/// Gotcha:
///  `OPT_ALIAS` pairs are not modeled.
///  No table in this crate holds an alias and its
///         source that share a prefix,
///  so the alias exception to ambiguity never applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanLongOption({ args, index, table, mode, parsed }): boolean; // throws OptionError
/// ```
pub(crate) fn scan_long_option(
    arguments: &[OsString],
    index: usize,
    table: &[OptionSpec],
    mode: ParseMode,
    parsed: &mut ParsedOptions,
) -> Result<bool, OptionError> {
    // `.as_encoded_bytes()` lends the raw bytes of the token without decoding them.
    let token: &[u8] = arguments[index].as_encoded_bytes();
    // Git's `arg`: everything after the two dashes, `=value` included.
    let argument: &[u8] = &token[2..];
    let argument_name_end: usize = name_end(argument);
    // Git's `arg_start`: `argument` without one or two leading `no-` (530-535).
    let mut stem: &[u8] = argument;
    let mut negated: bool = false;
    let mut double_negated: bool = false;
    // `.strip_prefix(...)` returns the remainder when the bytes start with the prefix.
    if let Some(once) = stem.strip_prefix(b"no-") {
        stem = once;
        if let Some(twice) = stem.strip_prefix(b"no-") {
            stem = twice;
            double_negated = true;
        } else {
            negated = true;
        }
    }
    // The part of `stem` before any `=`: what an abbreviation is compared with.
    let stem_name: &[u8] = &stem[..argument_name_end - (argument.len() - stem.len())];
    let mut candidate: Option<Candidate> = None;
    let mut ambiguous: bool = false;
    for spec in table {
        // Rows without a long spelling cannot match a long token.
        let long: &str = if let Some(spelling) = spec.long {
            spelling
        } else {
            continue;
        };
        let mut long_name: &[u8] = long.as_bytes();
        let mut spec_negative: bool = false;
        if let Some(positive) = long_name.strip_prefix(b"no-") {
            // A row such as `no-verify` is reached by `--verify` in unset form (547-548).
            long_name = positive;
            spec_negative = true;
        } else if double_negated {
            continue;
        }
        // Git's `flags ^ opt_flags`: the token and the row disagree about `no-`.
        let unset: bool = negated != spec_negative;
        if unset && !spec.negatable {
            continue;
        }
        if let Some(rest) = stem.strip_prefix(long_name) {
            if rest.is_empty() {
                let exact: Candidate = Candidate { spec: *spec, unset };
                return accept(arguments, index, exact, None, parsed);
            }
            if rest[0] == b'=' {
                let exact: Candidate = Candidate { spec: *spec, unset };
                // The value starts one byte after the `=`.
                let offset: usize = token.len() - rest.len() + 1;
                return accept(arguments, index, exact, Some(offset), parsed);
            }
            // A longer name sharing this prefix: Git moves to the next row (558-559).
            continue;
        }
        if mode.keep_unknown {
            // `register_abbrev` does nothing under `PARSE_OPT_KEEP_UNKNOWN_OPT` (502-503).
            continue;
        }
        if long_name.starts_with(stem_name) {
            ambiguous = ambiguous || candidate.is_some();
            candidate = Some(Candidate { spec: *spec, unset });
        }
        // "Negated and abbreviated very much": `--n`, `--no` and `--no-` (568-571).
        if spec.negatable && b"no-".starts_with(argument) {
            ambiguous = ambiguous || candidate.is_some();
            candidate = Some(Candidate {
                spec: *spec,
                unset: !spec_negative,
            });
        }
    }
    if ambiguous {
        return Err(OptionError {
            kind: OptionErrorKind::AmbiguousOption,
            token: index,
        });
    }
    if let Some(chosen) = candidate {
        let attached: Option<usize> = if argument_name_end < argument.len() {
            // Two dashes, the name, then `=`.
            Some(2 + argument_name_end + 1)
        } else {
            None
        };
        return accept(arguments, index, chosen, attached, parsed);
    }
    if mode.keep_unknown {
        // The unknown token stands alone: the next token is not its value.
        parsed.unknown.push(index);
        return Ok(false);
    }
    return Err(OptionError {
        kind: OptionErrorKind::UnknownOption,
        token: index,
    });
}

/// Abbreviation,
///  ambiguity and negation cases.
#[cfg(test)]
#[path = "command_options_long_tests.rs"]
mod tests;
