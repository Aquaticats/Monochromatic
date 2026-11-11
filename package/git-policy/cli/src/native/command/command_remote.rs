//! What:
//!  The subcommand of a `git remote` region,
//!  read as Git 2.56.0 reads it.
//! Why:
//!  `git remote` reads `-v`/`--verbose` before its subcommand and nothing else,
//!  so the first
//!       token that is not one of those spellings names the subcommand,
//!  and every token after that
//!       belongs to the subcommand's own parser.
//!  A rule that reads the first token blindly misses `git remote -v
//!       set-url origin <url>`,
//!  and a rule that parses the whole region with only this table would refuse
//!       the subcommand's own options as unknown.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseRemoteRegion(['-v', 'set-url', 'origin', url]).subcommand === 'set-url'
//! ```

/// What:
///  Bring the table,
///  the tokenizer and its default mode into this file.
/// Why:
///   Git's own grammar decides which spellings of `-v` the wrapper accepts;
///  this module only
///       finds where the subcommand starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { REMOTE_TABLE } from './command_remote_table.ts';
/// import { DEFAULT_MODE, parse_options } from './command_options.ts';
/// ```
use crate::command_options::{DEFAULT_MODE, OptionError, parse_options};
use crate::command_remote_table::REMOTE_TABLE;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The subcommand word of one `git remote` region.
///  `Option<OsString>` is "a word or nothing":
///  nothing when the command
///       runs no subcommand,
///  as `git remote -v` does.
/// Why:
///   The caller decides what the subcommand changes;
///  this module only says which word Git will run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoteRegion = { subcommand: string | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteRegion {
    /// The subcommand Git dispatches to,
    ///  exactly as it was written.
    pub subcommand: Option<OsString>,
}

/// What:
///  The index of the first token `git remote` reads as an argument rather than its own option.
///  `&[OsString]` borrows the region.
/// Why:
///   The options before a subcommand take no value,
///  so the first token that does not look like an option
///       starts the subcommand,
///  and the parser below only has to judge the options before it.
///  A lone
///       `-` is an argument to Git,
///  and `--` ends the options without naming a subcommand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstNonOption = (region: string[]): number => region.findIndex((t) => !t.startsWith('-') || t === '-');
/// ```
fn first_non_option(region: &[OsString]) -> usize {
    // `index` walks the tokens once; `while` stops at the first argument.
    let mut index: usize = 0;
    while index < region.len() {
        // `.as_encoded_bytes()` lends the raw bytes; nothing is decoded or copied.
        let token: &[u8] = region[index].as_encoded_bytes();
        // A token shorter than two bytes cannot be an option: the empty token and a lone `-` are arguments.
        if token.len() < 2 || token.first() != Some(&b'-') {
            return index;
        }
        // `--` ends the options; `git remote -- add ...` runs no subcommand.
        if token == b"--" {
            return index;
        }
        index += 1;
    }
    return index;
}

/// What:
///  Read the region after `git remote`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region whose options Git refuses runs nothing,
///  so its refusal is reported rather than
///       guessed at,
///  with the token index counted inside those option words;
///  the subcommand after them is
///       returned exactly as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemoteRegion(region: string[]): RemoteRegion; // throws OptionError
/// ```
pub fn parse_remote_region(region: &[OsString]) -> Result<RemoteRegion, OptionError> {
    let first: usize = first_non_option(region);
    // `&region[..first]` borrows the option words; a `?` returns Git's refusal to our caller.
    parse_options(&region[..first], REMOTE_TABLE, DEFAULT_MODE, &[])?;
    // `region.get(first)` is the word the subcommand starts at, when there is one.
    let subcommand: Option<OsString> = match region.get(first) {
        // A `--` there names no subcommand.
        Some(word) if word.as_encoded_bytes() != b"--" => Some(word.clone()),
        Some(_) | None => None,
    };
    return Ok(RemoteRegion { subcommand });
}

/// Subcommand positions and refusals,
///  with real Git 2.56.0 controls.
#[cfg(test)]
#[path = "command_remote_tests.rs"]
mod tests;
