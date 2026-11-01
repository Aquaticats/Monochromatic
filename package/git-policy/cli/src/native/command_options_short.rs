//! What:
//!  Read one `-abc` token as Git 2.56.0 reads a cluster of short options.
//! Why:
//!  `-am` is `-a -m`,
//!  `-mhello` is `-m hello`,
//!  and `-ua` is `-u` with the value `a`;
//!      only the option table can tell these apart (`parse_short_opt`,
//!  parse-options.c:426-461).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // scanShortCluster({ args, index, table, mode, parsed }): number  // tokens consumed
//! ```

/// What:
///  Bring the tokenizer types and the shared value decision into this file.
/// Why:
///   A cluster appends to the same result the long-option scan appends to.
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
///  Find the table row whose short letter is `letter`.
///  `u8` is one byte (siblings:
///       `char`,
///  a Unicode scalar;
///  `u16`).
///  `Option<OptionSpec>` is "a row or nothing".
/// Why:
///   Git compares option letters as single bytes,
///  so a byte is the exact unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spec = table.find(row => row.short === letter);
/// ```
fn find_short(table: &[OptionSpec], letter: u8) -> Option<OptionSpec> {
    // `for spec in table` borrows each row in table order.
    for spec in table {
        if spec.short == Some(letter) {
            // `*spec` copies the small row out of the borrowed table.
            return Some(*spec);
        }
    }
    return None;
}

/// What:
///  Report whether the letters after one dash spell the start of a long option
///       (`check_typos`,
///  parse-options.c:622-640).
/// Why:
///   Git refuses `-amend` and asks about two dashes instead of reading `-a -m end`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const typo = cluster.length >= 3 && (cluster.startsWith('no-') || table.some(r => r.long?.startsWith(cluster)));
/// ```
fn spells_long_option(table: &[OptionSpec], cluster: &[u8]) -> bool {
    if cluster.len() < 3 {
        return false;
    }
    if cluster.starts_with(b"no-") {
        return true;
    }
    for spec in table {
        // `if let Some(long) = spec.long` skips rows that have no long spelling.
        if let Some(long) = spec.long {
            // `.as_bytes()` lends the text as bytes for a byte-wise prefix test.
            if long.as_bytes().starts_with(cluster) {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  Handle a letter no row declares.
///  `&mut ParsedOptions` lends the result for writing.
/// Why:
///   Git tries the typo check,
///  then `-h`,
///  then either keeps or refuses the token
///       (parse-options.c:1067-1072,
///  1088-1099,
///  1149-1167).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unknownLetter({ table, token, position, index, mode, parsed }): boolean; // throws
/// ```
fn unknown_letter(
    table: &[OptionSpec],
    token: &[u8],
    position: usize,
    index: usize,
    mode: ParseMode,
    parsed: &mut ParsedOptions,
) -> Result<bool, OptionError> {
    // The typo check runs for an unknown first letter only (1068-1069).
    if position == 1 && spells_long_option(table, &token[1..]) {
        return Err(OptionError {
            kind: OptionErrorKind::SingleDashLongOption,
            token: index,
        });
    }
    if token[position] == b'h' {
        return Err(OptionError {
            kind: OptionErrorKind::HelpRequested,
            token: index,
        });
    }
    if mode.keep_unknown {
        // Git keeps the rest of the token as one argument for the command to forward;
        // the next token is not its value.
        parsed.unknown.push(index);
        return Ok(false);
    }
    return Err(OptionError {
        kind: OptionErrorKind::UnknownOption,
        token: index,
    });
}

/// What:
///  Read the cluster at `index` and say whether it took the next token as a value.
/// Why:
///   A value-taking letter ends the cluster:
///  the rest of the token,
///  or else the next
///       token,
///  is its value.
///  Only the second case makes the caller pass over a token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanShortCluster({ args, index, table, mode, parsed }): boolean; // throws OptionError
/// ```
pub(crate) fn scan_short_cluster(
    arguments: &[OsString],
    index: usize,
    table: &[OptionSpec],
    mode: ParseMode,
    parsed: &mut ParsedOptions,
) -> Result<bool, OptionError> {
    // `.as_encoded_bytes()` lends the raw bytes of the token without decoding them.
    let token: &[u8] = arguments[index].as_encoded_bytes();
    // Byte 0 is the dash; letters start at byte 1.
    let mut position: usize = 1;
    while position < token.len() {
        let found: Option<OptionSpec> = find_short(table, token[position]);
        // `if let ... else` either unwraps the row or leaves through the unknown path.
        let spec: OptionSpec = if let Some(declared) = found {
            declared
        } else {
            return unknown_letter(table, token, position, index, mode, parsed);
        };
        position += 1;
        let attached: Option<usize> = if position < token.len() {
            Some(position)
        } else {
            None
        };
        // `?` returns a refusal to our caller, or unwraps the value position.
        let value: Option<OptionValue> =
            take_value(arguments, index, spec.arity, attached, Spelling::Short)?;
        parsed.occurrences.push(Occurrence {
            id: spec.id,
            negated: false,
            value,
            token: index,
        });
        if let Some(taken) = value {
            // `matches!(x, pattern)` is true when `x` has that shape: the next token was
            // the value. Otherwise the rest of this token was.
            return Ok(matches!(taken, OptionValue::Detached { .. }));
        }
        // After the first letter, remaining letters get the typo check once (1082-1083).
        // A cluster without remaining letters is too short to spell a long option, which
        // the check itself decides, so no length is compared here.
        if position == 2 && spells_long_option(table, &token[1..]) {
            return Err(OptionError {
                kind: OptionErrorKind::SingleDashLongOption,
                token: index,
            });
        }
    }
    return Ok(false);
}

/// Cluster arity,
///  typo-check and help cases.
#[cfg(test)]
#[path = "command_options_short_tests.rs"]
mod tests;
