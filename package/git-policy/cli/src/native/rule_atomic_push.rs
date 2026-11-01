//! What:
//!  The pure decision of the atomic-push transform:
//!  insert `--atomic` after `git push`
//!       unless the caller already chose.
//! Why:
//!  An atomic push updates every ref on the remote or none,
//!  which prevents partially
//!      applied pushes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // atomicPush(['push', 'origin', 'main']) => ['push', '--atomic', 'origin', 'main']
//! ```

/// What:
///  Bring the push facts,
///  the global-option boundary and the rewrite helpers into
///       this file.
/// Why:
///   The decision combines them;
///  it owns no parsing of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parsePushRegion } from '../parser/push.ts';
/// import { parseGlobalOptions } from '../parse-global-options.ts';
/// ```
use super::command_options::OptionError;
use super::command_push::{PushRegion, parse_push_region};
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use super::rule_argument_rewrite::{ArgumentRewrite, insert_tokens};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Decide the transform for one command line.
///  `Result<A, B>` is "either success `A`
///       or failure `B`";
///  `&[&[u8]]` borrows the caller's wrapper-only spellings.
/// Why:
///   The flag goes immediately after `push`,
///  so global options before it survive.
/// Gotcha:
///  `Err` means Git 2.56.0 itself refuses the push options;
///  the caller forwards such
///         a command unchanged so Git prints its own error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function atomicPush(args: readonly string[]): readonly string[];
/// ```
pub fn atomic_push(
    arguments: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<ArgumentRewrite, OptionError> {
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command
        || arguments[layout.prefix_len].as_encoded_bytes() != b"push"
    {
        // `Ok(x)` is the success case of `Result`.
        return Ok(ArgumentRewrite::Unchanged);
    }
    // `usize` is the list index type; the region starts after the `push` token.
    let region_start: usize = layout.prefix_len + 1;
    // `&arguments[region_start..]` borrows the tokens after `push`; `?` returns Git's
    // refusal to our caller, or unwraps the facts.
    let region: PushRegion = parse_push_region(&arguments[region_start..], wrapper_flags)?;
    if region.atomic_stated {
        return Ok(ArgumentRewrite::Unchanged);
    }
    return Ok(ArgumentRewrite::Rewritten(insert_tokens(
        arguments,
        region_start,
        &["--atomic"],
    )));
}

/// Every `atomic-push.unit.test.ts` case and the spellings the incumbent missed.
#[cfg(test)]
#[path = "rule_atomic_push_tests.rs"]
mod tests;
