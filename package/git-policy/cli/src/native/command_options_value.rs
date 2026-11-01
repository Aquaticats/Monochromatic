//! What:
//!  Decide which bytes,
//!  if any,
//!  are the value of one matched Git option.
//! Why:
//!  Git's `do_get_value` and `get_arg` (parse-options.c:47-62,
//!  130-336) are the only
//!      authority on whether the next token is a value;
//!  both spellings share this decision.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // takeValue({ args, index, arity, attached, spelling }): OptionValue | undefined
//! ```

/// What:
///  Bring the shared tokenizer types into this file.
/// Why:
///   The value position and the refusal are reported with the tokenizer's own types.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, OptionError, OptionErrorKind, OptionValue } from './command_options.ts';
/// ```
use super::command_options::{Arity, OptionError, OptionErrorKind, OptionValue};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  How the option was written.
///  `pub(crate)` means "visible inside this crate only".
/// Why:
///   A short letter cannot be negated,
///  and only a long spelling can carry `=value`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Spelling = 'short' | 'long' | 'negated-long';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Spelling {
    /// `-m`
    Short,
    /// `--message`
    Long,
    /// `--no-message`
    NegatedLong,
}

/// What:
///  Return where the value of the option at `index` sits.
///  `attached` is the byte offset
///       inside the same token where a value would start,
///  when the token continues.
///       `Result<Option<OptionValue>, OptionError>` is "a value position,
///  no value,
///  or a refusal".
/// Why:
///   Negated options never take a value;
///  a required value takes the next token even when
///       it starts with a dash;
///  an optional value never takes the next token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeValue({ args, index, arity, attached, spelling }): OptionValue | undefined; // throws
/// ```
pub(crate) fn take_value(
    arguments: &[OsString],
    index: usize,
    arity: Arity,
    attached: Option<usize>,
    spelling: Spelling,
) -> Result<Option<OptionValue>, OptionError> {
    if spelling == Spelling::NegatedLong {
        // `--no-name=value`: "takes no value" (parse-options.c:138-139).
        if attached.is_some() {
            // `Err(x)` is the failure case of `Result`.
            return Err(OptionError {
                kind: OptionErrorKind::UnexpectedValue,
                token: index,
            });
        }
        // `Ok(None)`: success, and there is no value. Unset forms never read one (201, 241).
        return Ok(None);
    }
    if arity == Arity::None {
        // `--flag=value` is refused (142-143); letters after a short flag are more flags.
        if spelling == Spelling::Long && attached.is_some() {
            return Err(OptionError {
                kind: OptionErrorKind::UnexpectedValue,
                token: index,
            });
        }
        return Ok(None);
    }
    // `if let Some(offset) = attached` runs only when the token continues after the option.
    if let Some(offset) = attached {
        // `Some(x)` is the "present" case of `Option`.
        return Ok(Some(OptionValue::Attached {
            token: index,
            offset,
        }));
    }
    if arity == Arity::Optional {
        // `PARSE_OPT_OPTARG` without an attached value uses its default (203-204).
        return Ok(None);
    }
    // `usize` indexes the argument list; `index + 1` cannot overflow a real argument count.
    let following: usize = index + 1;
    if following < arguments.len() {
        return Ok(Some(OptionValue::Detached { token: following }));
    }
    if arity == Arity::LastArgDefault {
        // The option is the last argument and has a default (54-55).
        return Ok(None);
    }
    return Err(OptionError {
        kind: OptionErrorKind::MissingValue,
        token: index,
    });
}

/// Per-arity value cases and the real-Git control for the last-argument default.
#[cfg(test)]
#[path = "command_options_value_tests.rs"]
mod tests;
