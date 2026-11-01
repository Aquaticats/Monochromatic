//! What:
//!  The pure decision of the status-hints transform:
//!  insert
//!       `-c advice.statusHints=false` before `git status` unless the caller set that key.
//! Why:
//!  Git's stock status hints suggest `git add <file>...` and `git commit -a`,
//!  forms the
//!      wrapper's policies reject;
//!  the wrapper prints accurate guidance instead.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // statusHintsOff(['status']) => ['-c', 'advice.statusHints=false', 'status']
//! ```

/// What:
///  Bring the override reading,
///  the global-option boundary and the rewrite helpers
///       into this file.
/// Why:
///   The decision combines them;
///  it owns no parsing of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseStatusPreRegion } from '../parser/status.ts';
/// import { parseGlobalOptions } from '../parse-global-options.ts';
/// ```
use super::command_status::has_status_hints_override;
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use super::rule_argument_rewrite::{ArgumentRewrite, insert_tokens};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// Tokens inserted before `status` so Git itself does not print its stock hints.
pub const QUIET_INJECTION: &[&str] = &["-c", "advice.statusHints=false"];

/// What:
///  Whether the caller configured `advice.statusHints` before the subcommand,
///       whatever the subcommand is.
///  `&[OsString]` borrows the full argument list.
/// Why:
///   The entry point also asks this to decide whether to print its own note,
///  and it
///       asks about the raw arguments,
///  before any transform ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasExplicitStatusHintsOverride(args: readonly string[]): boolean;
/// ```
pub fn has_explicit_status_hints_override(arguments: &[OsString]) -> bool {
    let layout: GlobalLayout = global_layout(arguments);
    // `&arguments[..n]` borrows the first `n` tokens: the completed global options.
    return has_status_hints_override(&arguments[..layout.prefix_len]);
}

/// What:
///  Decide the transform for one command line.
/// Why:
///   The tokens go immediately before `status`,
///  after every caller-supplied global
///       option,
///  so those survive and Git still reads the inserted pair as global.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusHintsOff(args: readonly string[]): readonly string[];
/// ```
pub fn status_hints_off(arguments: &[OsString]) -> ArgumentRewrite {
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command
        || arguments[layout.prefix_len].as_encoded_bytes() != b"status"
    {
        return ArgumentRewrite::Unchanged;
    }
    if has_status_hints_override(&arguments[..layout.prefix_len]) {
        return ArgumentRewrite::Unchanged;
    }
    return ArgumentRewrite::Rewritten(insert_tokens(
        arguments,
        layout.prefix_len,
        QUIET_INJECTION,
    ));
}

/// Every `status-hints-off.unit.test.ts` case and a real-Git control of the injection.
#[cfg(test)]
#[path = "rule_status_hints_tests.rs"]
mod tests;
