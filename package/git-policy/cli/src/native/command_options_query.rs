//! What:
//!  Small read-only questions over a tokenized Git argument region.
//! Why:
//!  Git applies options in order and the last use wins,
//!  so command modules ask for
//!      final states and positions instead of counting spellings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lastOccurrence(parsed, id), isEnabled(parsed, id), positionalTokens(parsed, length), ...
//! ```

/// What:
///  Bring the tokenizer result types into this file.
/// Why:
///   Every question reads the lists the tokenizer produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Boundary, Occurrence, OptionValue, ParsedOptions } from './command_options.ts';
/// ```
use super::command_options::{Boundary, Occurrence, OptionValue, ParsedOptions, WrapperOccurrence};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The final use of the option `id`,
///  or nothing.
///  `&ParsedOptions` borrows the result
///       read-only;
///  `u16` is the identifier type of table rows.
/// Why:
///   Git's variables keep what the last matching option wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const last = parsed.occurrences.findLast(o => o.id === id);
/// ```
pub fn last_occurrence(parsed: &ParsedOptions, id: u16) -> Option<Occurrence> {
    // `mut` lets later matches replace earlier ones during the single pass.
    let mut last: Option<Occurrence> = None;
    // `for occurrence in &parsed.occurrences` borrows each item in order.
    for occurrence in &parsed.occurrences {
        if occurrence.id == id {
            // `*occurrence` copies the small record out of the borrowed list.
            last = Some(*occurrence);
        }
    }
    return last;
}

/// What:
///  Whether the option was written at all,
///  in either form.
/// Why:
///   An explicit `--no-only` is a stated choice even though it turns the option off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stated = parsed.occurrences.some(o => o.id === id);
/// ```
pub fn is_stated(parsed: &ParsedOptions, id: u16) -> bool {
    return last_occurrence(parsed, id).is_some();
}

/// What:
///  Whether the option is on after every use was applied.
/// Why:
///   `--all --no-all` leaves `all` off,
///  exactly as Git's `OPT_BOOL` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const enabled = last !== undefined && !last.negated;
/// ```
pub fn is_enabled(parsed: &ParsedOptions, id: u16) -> bool {
    // `if let Some(last) = ...` runs only when the option was written.
    if let Some(last) = last_occurrence(parsed, id) {
        return !last.negated;
    }
    return false;
}

/// What:
///  Every non-option token index in order:
///  those before the boundary,
///  then every
///       token after it.
///  `length` is the region's token count.
/// Why:
///   Git hands the command both groups as its remaining arguments.
/// Gotcha:
///  Under `Boundary::NonOption` the boundary token itself is positional;
///  under
///         `--` and `--end-of-options` it is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const positionals = [...parsed.leading, ...indexesAfterBoundary];
/// ```
pub fn positional_tokens(parsed: &ParsedOptions, length: usize) -> Vec<usize> {
    // `.clone()` copies the index list so the result is independently owned.
    let mut tokens: Vec<usize> = parsed.leading.clone();
    // What: `match` picks one arm per variant and unpacks the index a variant holds; the
    //       compiler refuses the code if a variant is forgotten.
    // Why:  `next` is the first index from which every token is positional.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (boundary.kind) { case 'end': next = length; break; /* ... */ }
    // ```
    let mut next: usize = match parsed.boundary {
        Boundary::End => length,
        Boundary::DashDash(at) => at + 1,
        Boundary::EndOfOptions(at) => at + 1,
        Boundary::NonOption(at) => at,
    };
    while next < length {
        tokens.push(next);
        next += 1;
    }
    return tokens;
}

/// What:
///  The exact bytes of an option value.
///  `&[u8]` borrows them from `arguments`.
/// Why:
///   Values are compared as bytes;
///  converting to `String` would reject non-UTF-8 text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = 'offset' in value ? args[value.token].slice(value.offset) : args[value.token];
/// ```
pub fn value_bytes(arguments: &[OsString], value: OptionValue) -> &[u8] {
    // `(usize, usize)` is a pair; a detached value is the whole token, so it starts at byte 0.
    let (token, offset): (usize, usize) = match value {
        OptionValue::Attached { token, offset } => (token, offset),
        OptionValue::Detached { token } => (token, 0),
    };
    // `&bytes[offset..]` borrows from `offset` to the end.
    return &arguments[token].as_encoded_bytes()[offset..];
}

/// What:
///  Wrapper-only flags found in option position,
///  split into the command's own escape
///       hatch and every other flag the caller listed.
/// Why:
///   A rule asks "was my hatch written?",
///  while the caller removes all of them by position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrapperFlags = { escape: number[]; other: WrapperOccurrence[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrapperFlags {
    /// Region token indexes of the command's own escape hatch.
    pub escape: Vec<usize>,
    /// Other wrapper flags;
    ///  `flag` indexes the caller's list.
    pub other: Vec<WrapperOccurrence>,
}

/// What:
///  Split the wrapper occurrences of a parse whose flag list was built as
///       "own escape hatch first,
///  then the caller's flags".
///  `&[WrapperOccurrence]` borrows
///       the occurrence list of that parse.
/// Why:
///   Every command module builds its list that way,
///  so the caller's indexes are the
///       tokenizer's indexes minus one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const escape = wrapper.filter(w => w.flag === 0).map(w => w.token);
/// const other = wrapper.filter(w => w.flag > 0).map(w => ({ flag: w.flag - 1, token: w.token }));
/// ```
pub fn split_wrapper_flags(occurrences: &[WrapperOccurrence]) -> WrapperFlags {
    let mut escape: Vec<usize> = Vec::<usize>::new();
    let mut other: Vec<WrapperOccurrence> = Vec::<WrapperOccurrence>::new();
    for occurrence in occurrences {
        if occurrence.flag == 0 {
            escape.push(occurrence.token);
        } else {
            other.push(WrapperOccurrence {
                flag: occurrence.flag - 1,
                token: occurrence.token,
            });
        }
    }
    return WrapperFlags { escape, other };
}

/// What:
///  Copy `arguments` without the tokens at `removed`,
///  which are indexes relative to
///       `offset` (the index of the first region token).
/// Why:
///   Wrapper-only flags are deleted by position,
///  so an equal-looking value or path stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = args.filter((_, i) => !removed.includes(i - offset));
/// ```
pub fn without_tokens(arguments: &[OsString], offset: usize, removed: &[usize]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::<OsString>::with_capacity(arguments.len());
    let mut index: usize = 0;
    while index < arguments.len() {
        // `index >= offset` first, so the subtraction cannot go below zero.
        let dropped: bool = index >= offset && removed.contains(&(index - offset));
        if !dropped {
            // `.clone()` copies the argument bytes into the owned result.
            kept.push(arguments[index].clone());
        }
        index += 1;
    }
    return kept;
}
