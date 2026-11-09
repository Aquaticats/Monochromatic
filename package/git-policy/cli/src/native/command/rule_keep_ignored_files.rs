//! What:
//!  The pure decision of the ignored-files transform:
//!  add `--no-overwrite-ignore` to a `checkout` or `switch`
//!       that changes worktree content.
//! Why:
//!  By default Git replaces an ignored file in the worktree with the version the tree it moves to carries,
//!  and an ignored file is exactly where a person keeps their environment and their local secrets.
//!  With the option Git answers instead:
//! it names the file and refuses the switch,
//!  so the decision is Git's own,
//! taken inside the process that changes the files.
//!  A forced `checkout -f` and a `reset --hard` overwrite the file anyway,
//!  and the guard around those forms is the worktree measurement instead
//!  (`src/native/command/rule_linked_worktree.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // keepIgnoredFiles(['checkout', 'other']) => ['checkout', '--no-overwrite-ignore', 'other']
//! ```

/// What:
///  Bring the reading of a `checkout` and a `switch` region,
///  the global-option boundary
///       and the rewrite helper into this file.
/// Why:
///   The decision combines them;
///  it owns no parsing of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { checkoutDiscard, switchDiscard } from './command_discard.ts';
/// ```
use crate::command_discard::{Discard, checkout_discard, switch_discard};
use crate::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use crate::rule_argument_rewrite::{ArgumentRewrite, insert_tokens};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// The token inserted directly after the subcommand.
pub const NO_OVERWRITE_IGNORE: &[&str] = &["--no-overwrite-ignore"];

/// What:
///  Whether the caller already asked about ignored files,
///  in either direction.
///  `bool` is true or false.
/// Why:
///   The inserted token goes first and Git applies the last one,
///  so a caller's own
///       `--overwrite-ignore` already decides;
///  adding the token beside an answered question would put noise in the
///       arguments Git receives and in the audit line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function answersIgnoredFiles(region: string[]): boolean;
/// ```
fn answers_ignored_files(region: &[OsString]) -> bool {
    // `for token in region` borrows each argument in turn.
    for token in region {
        // `.as_encoded_bytes()` lends the raw bytes; `.starts_with` compares them without decoding.
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes.starts_with(b"--overwrite-ignore") || bytes.starts_with(b"--no-overwrite-ignore") {
            return true;
        }
    }
    return false;
}

/// What:
///  The region's reading as one of the three answers a checkout region has,
///  or nothing when
///       Git would refuse the region.
///  `Option<Discard>` is "a reading or nothing";
///  `&[u8]` borrows the
///       subcommand word.
/// Why:
///   A region Git refuses runs nothing,
///  so it needs no token;
///  the reading also says whether the
///       region changes any file at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reading(word: Buffer, region: string[]): Discard | undefined;
/// ```
fn reading(word: &[u8], region: &[OsString]) -> Option<Discard> {
    let parsed: Result<Discard, crate::command_options::OptionError> = if word == b"checkout" {
        checkout_discard(region)
    } else if word == b"switch" {
        switch_discard(region)
    } else {
        return None;
    };
    return parsed.ok();
}

/// What:
///  Decide the transform for one command line.
/// Why:
///   The token goes directly after the subcommand,
///  where Git reads it as an option of that subcommand and
///       every caller option after it still wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepIgnoredFiles(args: readonly string[]): ArgumentRewrite;
/// ```
pub fn keep_ignored_files(arguments: &[OsString]) -> ArgumentRewrite {
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        return ArgumentRewrite::Unchanged;
    }
    // `&arguments[prefix_len]` is the subcommand word; `&arguments[prefix_len + 1..]` is its region.
    let word: &[u8] = arguments[layout.prefix_len].as_encoded_bytes();
    let region: &[OsString] = &arguments[layout.prefix_len + 1..];
    // A harmful reading of an option a switch does not know is `None`: Git refuses it and runs nothing.
    let Some(answer) = reading(word, region) else {
        return ArgumentRewrite::Unchanged;
    };
    if answer == Discard::Harmless || answers_ignored_files(region) {
        return ArgumentRewrite::Unchanged;
    }
    return ArgumentRewrite::Rewritten(insert_tokens(
        arguments,
        layout.prefix_len + 1,
        NO_OVERWRITE_IGNORE,
    ));
}

/// Every spelling,
///  the forced forms and real-Git controls stay out of the release executable.
#[cfg(test)]
#[path = "rule_keep_ignored_files_tests.rs"]
mod tests;
