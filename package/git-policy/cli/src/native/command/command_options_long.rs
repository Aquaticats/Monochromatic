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
use crate::command_options::{Occurrence, OptionError, OptionSpec, OptionValue, ParseMode, ParsedOptions};
use crate::command_options_long_resolution::{Candidate, Resolution, resolve_long_option};
use crate::command_options_value::{Spelling, take_value};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

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
///  Read the long option at `index` and say whether it took the next token as its value.
/// Why:
///   One table walk finds an exact match first and otherwise the unique abbreviation,
///  which `resolve_long_option` answers for the tokenizer as well as for the transforms.
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
    // `?` returns a refusal to our caller, or unwraps Git's reading of the token.
    let resolution: Resolution = match resolve_long_option(token, table, mode.keep_unknown) {
        Ok(resolved) => resolved,
        Err(kind) => return Err(OptionError { kind, token: index }),
    };
    // `match` picks one arm per variant and the compiler refuses the code if one is forgotten.
    return match resolution {
        Resolution::KeptUnknown => {
            // The unknown token stands alone: the next token is not its value.
            parsed.unknown.push(index);
            Ok(false)
        }
        Resolution::Known { row, attached } => accept(arguments, index, row, attached, parsed),
    };
}

/// Abbreviation,
///  ambiguity and negation cases.
#[cfg(test)]
#[path = "command_options_long_tests.rs"]
mod tests;
