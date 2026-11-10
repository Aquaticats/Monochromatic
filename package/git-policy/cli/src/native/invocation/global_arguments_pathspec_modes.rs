//! What:
//!  The pathspec-mode options of a global prefix:
//!  which of them the caller wrote,
//!  and what the prefix is without them.
//! Why:
//!  A mode the caller chose governs the caller's own pathspecs and never one the wrapper generated,
//! and a generated pathspec must reach Git without a mode that would redirect it.
//!  The scan reads the value a separated-value option consumes as a value,
//! so a directory that spells a mode option is still that directory.
//!  It is kept out of `global_arguments.rs` so each file stays inside the line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pathspecModeOptions(['-C', 'dir', '--icase-pathspecs']); // ['--icase-pathspecs']
//! withoutPathspecModeOptions(['-C', 'dir', '--icase-pathspecs']); // ['-C', 'dir']
//! ```

/// The prefix scan that knows an option's value from the next option.
use crate::global_arguments::takes_value;
/// `OsString` is owned operating-system text.
use std::ffi::OsString;

/// What:
///  The caller's global prefix without the options that set a pathspec mode.
/// Why:
///   A mode the caller chose governs the caller's own pathspecs,
/// never a pathspec the wrapper generated:
///  under `--icase-pathspecs` the wrapper's `:(top,literal)a.txt` matches `A.txt` too,
///  under `--literal-pathspecs` it names a file that is not there,
///  and the two together make Git refuse every pathspec.
///  A value a separated-value option consumes is kept,
///  so a directory that spells a mode option is still that directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutPathspecModeOptions = (prefix: string[]) => prefix.filter((token, index) => isValue(prefix, index) || !MODE_OPTIONS.includes(token));
/// ```
pub fn without_pathspec_mode_options(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::with_capacity(global_prefix.len());
    // True while the token being visited is the value of the separated-value option before it.
    let mut is_value: bool = false;
    for token in global_prefix {
        if is_value {
            is_value = false;
            kept.push(token.clone());
            continue;
        }
        is_value = takes_value(token.as_encoded_bytes());
        if is_value || !is_pathspec_mode_option(token.as_encoded_bytes()) {
            kept.push(token.clone());
        }
    }
    return kept;
}

/// What:
///  The caller's pathspec-mode options,
/// in the order the caller wrote them.
/// Why:
///   A private index start runs without the caller's global prefix (`GitRequest::without_prefix`),
/// so the caller's own pathspecs reach Git without the mode the caller chose:
///  `--noglob-pathspecs commit -- 'st*r.txt'` then stages the untracked `stxr.txt`,
///  and `--icase-pathspecs commit -- a.txt` misses a staged `A.TXT`.
///  A mode the caller set as a variable is already in the environment of every private start;
/// the mode written as an option is forwarded by this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathspecModeOptions = (prefix: string[]) => prefix.filter((token) => MODE_OPTIONS.includes(token));
/// ```
pub fn pathspec_mode_options(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::new();
    // True while the token being visited is the value of the separated-value option before it, so a directory that spells a
    // mode option is still that directory and not a mode.
    let mut is_value: bool = false;
    for token in global_prefix {
        if is_value {
            is_value = false;
            continue;
        }
        is_value = takes_value(token.as_encoded_bytes());
        if !is_value && is_pathspec_mode_option(token.as_encoded_bytes()) {
            kept.push(token.clone());
        }
    }
    return kept;
}

/// What:
///  Whether Git reads this token as a global option that sets a pathspec mode.
/// Why:
///   `conflict_side_modes::MODE_OPTIONS` names those five options for the conflict-side guard,
/// and this file reads the same list rather than keeping a second one that could disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPathspecModeOption = (token: string) => MODE_OPTIONS.includes(token);
/// ```
fn is_pathspec_mode_option(token: &[u8]) -> bool {
    for option in crate::conflict_side_modes::MODE_OPTIONS {
        if token == option.as_bytes() {
            return true;
        }
    }
    return false;
}
