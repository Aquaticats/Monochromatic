//! What:
//!  Which row of a Git option table one long token names,
//!  how the token is written and where an attached value starts.
//! Why:
//!  Git accepts any unambiguous prefix of a long option (`--mess` for `--message`)
//!  and a `--no-` form that takes no value,
//!  so the tokenizer and the argument transforms read one function's answer
//!  instead of each keeping a list of spellings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // resolveLongOption({ token, table, keepUnknown }): Resolution  (throws OptionErrorKind)
//! ```

/// What:
///  Bring the table row type,
///  its value arity and the refusal categories into this file.
/// Why:
///   A resolution is a row with a spelling,
/// and a refusal is Git's own category rather than rendered text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, OptionErrorKind, OptionSpec } from './command_options.ts';
/// ```
use crate::command_options::{Arity, OptionErrorKind, OptionSpec};

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
pub(crate) struct Candidate {
    /// The candidate row.
    pub(crate) spec: OptionSpec,
    /// True when the token turns the option off.
    pub(crate) unset: bool,
}

/// What:
///  What one long token turned out to be.
/// Why:
///   An unset form and an attached value leave the next token in option position,
/// and a token no row names is only kept under `PARSE_OPT_KEEP_UNKNOWN_OPT`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Resolution = { kind: 'known'; row: Candidate; attached?: number } | { kind: 'kept-unknown' };
/// ```
pub(crate) enum Resolution {
    /// A row of the table,
    /// with the spelling the token used.
    Known {
        /// The matched row and whether the token negated it.
        row: Candidate,
        /// Byte offset in the token where an attached value starts,
        /// when the token continues after the option.
        attached: Option<usize>,
    },
    /// A token no row names,
    /// kept because the mode keeps unknown options.
    KeptUnknown,
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
///  Resolve one `--name[=value]` token against a Git option table as Git 2.56.0 does.
///  `token` holds both dashes;
///  `keep_unknown` is `PARSE_OPT_KEEP_UNKNOWN_OPT`.
/// Why:
///   One table walk finds an exact match first and otherwise the unique abbreviation,
/// and both the tokenizer and the transforms that must not remove a person's value read it here.
/// Gotcha:
///  `OPT_ALIAS` pairs are not modeled.
///  No table in this crate holds an alias and its
///         source that share a prefix,
///  so the alias exception to ambiguity never applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveLongOption({ token, table, keepUnknown }): Resolution; // throws OptionErrorKind
/// ```
pub(crate) fn resolve_long_option(
    token: &[u8],
    table: &[OptionSpec],
    keep_unknown: bool,
) -> Result<Resolution, OptionErrorKind> {
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
                // `Ok(x)` is the success case of `Result`.
                return Ok(Resolution::Known {
                    row: Candidate { spec: *spec, unset },
                    attached: None,
                });
            }
            if rest[0] == b'=' {
                // Two dashes, the name, then `=`.
                let offset: usize = token.len() - rest.len() + 1;
                return Ok(Resolution::Known {
                    row: Candidate { spec: *spec, unset },
                    attached: Some(offset),
                });
            }
            // A longer name sharing this prefix: Git moves to the next row (558-559).
            continue;
        }
        if keep_unknown {
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
        return Err(OptionErrorKind::AmbiguousOption);
    }
    // `if let Some(chosen) = candidate` runs only when one abbreviation survived.
    if let Some(chosen) = candidate {
        let attached: Option<usize> = if argument_name_end < argument.len() {
            Some(2 + argument_name_end + 1)
        } else {
            None
        };
        return Ok(Resolution::Known { row: chosen, attached });
    }
    if keep_unknown {
        return Ok(Resolution::KeptUnknown);
    }
    return Err(OptionErrorKind::UnknownOption);
}

/// What:
///  Whether one long option token takes the token after it as its value.
/// Why:
///   A transform that removes argument tokens must know which of them are the person's values,
///  and only Git's own reading of an abbreviation and of a `--no-` form answers that.
///  A line Git refuses (an unknown or an ambiguous spelling) takes none:
///  the command fails as the person's spelling asks,
///  and no token of it is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// longOptionTakesNextToken(Buffer.from('--mess'), COMMIT_TABLE) // true
/// ```
pub(crate) fn long_option_takes_next_token(token: &[u8], table: &[OptionSpec]) -> bool {
    if !token.starts_with(b"--") || token.len() < 3 {
        return false;
    }
    // `let Ok(x) = ... else` leaves an unknown or ambiguous token with its answer "takes none".
    let Ok(Resolution::Known { row, attached }) = resolve_long_option(token, table, false) else {
        return false;
    };
    // An unset form never reads a value (201,
    //  241), and an attached value leaves the next token in option position.
    if row.unset || attached.is_some() {
        return false;
    }
    return row.spec.arity == Arity::Required || row.spec.arity == Arity::LastArgDefault;
}
