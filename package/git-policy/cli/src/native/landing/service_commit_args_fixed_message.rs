//! What:
//!  The arguments of a commit Git built,
//! rewritten so that the message the person already typed is used as it is and no editor opens.
//! Why:
//!   After the editor has run the message is fixed
//! (`service_commit_args.rs`):
//! a commit built on a tip that has since moved is built again from `-F <file>` with the message kept verbatim,
//! and a line that borrowed its author or its parents from another commit cannot be rebuilt that way.
//! The tables of long options it reads, and the abbreviation rule that keeps an unknown spelling from being guessed at,
//! are this rewrite's alone,
//! so they stand beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const rebuilt = withFixedMessage(args, subcommandIndex, file) // undefined when the line cannot be rebuilt from the message alone
//! ```

use crate::commit_args_token_bytes::{
    is_an_equals_sign,
    is_one_of,
};
use crate::path_bytes::{
    argument_bytes,
    os_from_path_bytes,
};
use std::ffi::OsString;

/// Long options that name where the message comes from,
/// or edit it,
/// or add to it.
const MESSAGE_VALUE_LONG: &[&str] = &[
    "--message",
    "--file",
    "--template",
    "--trailer",
    "--cleanup",
];

/// Long options without a value that open the editor or add a sign-off.
const MESSAGE_FLAG_LONG: &[&str] = &["--edit", "--no-edit", "--signoff", "--no-signoff"];

/// Long options that take the author,
/// or the parents,
/// or the whole commit from another commit.
const BORROWING_LONG: &[&str] = &[
    "--reedit-message",
    "--reuse-message",
    "--squash",
    "--fixup",
    "--amend",
];

/// What:
///  Whether a name is the start of one of the known long options.
/// Why:
///  Git accepts an unambiguous abbreviation of a long option,
/// so a name that starts a known option is not a safe option to pass on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// known.some((option) => option.startsWith(name))
/// ```
fn abbreviates_a_known_option(
    known: &[&str],
    name: &[u8],
) -> bool {
    for option in known {
        if option.as_bytes().starts_with(name) {
            return true;
        }
    }
    return false;
}

/// What:
///  The arguments of a commit Git built,
/// rewritten so that the message the person already typed is used as it is and no
///  editor opens.
/// Why:
///   After the editor has run the message is fixed,
/// so a commit built on a tip that has since moved is built again from
///  `-F <file>` with the message kept verbatim.
/// A line that takes its author or its parents from another commit (`-c`,
/// `-C`,
///  `--fixup`,
/// `--squash`,
/// `--amend`) cannot be rebuilt from the message alone,
/// and an abbreviated option that may be one of the dropped
///  ones is not guessed at:
/// both give nothing.
/// The caller that can supply what such a line borrowed reads it back from the commit Git built
///  (`service_commit_line_reissued.rs`,
/// which hands the line over here once it holds nothing of its own).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withFixedMessage(args, subcommandIndex, file) // undefined when the line cannot be rebuilt from the message alone
/// ```
pub fn with_fixed_message(
    arguments: &[OsString],
    subcommand_index: usize,
    file: &std::path::Path,
) -> Option<Vec<OsString>> {
    let region: Vec<OsString> =
        crate::commit_args::normalise_commit_args(arguments.get(subcommand_index + 1..)?);
    let split: usize =
        crate::commit_args::separator_index(region.as_slice()).unwrap_or(region.len());
    let (options, tail): (&[OsString], &[OsString]) = region.split_at(split);
    let mut kept: Vec<OsString> = arguments[..=subcommand_index].to_vec();
    let mut index: usize = 0;
    while index < options.len() {
        let token: &OsString = &options[index];
        index += 1;
        let bytes: &[u8] = argument_bytes(token.as_os_str());
        if bytes.starts_with(b"--") {
            // The name ends where the value an `=` attached begins, and the name is the caller's bytes so a token that is
            //  not text is never taken for one of these options.
            let (rest, attached): (&[u8], bool) = match bytes.iter().position(is_an_equals_sign) {
                Some(at) => (&bytes[..at], true),
                None => (bytes, false),
            };
            let all: Vec<&str> = MESSAGE_VALUE_LONG
                .iter()
                .chain(MESSAGE_FLAG_LONG)
                .chain(BORROWING_LONG)
                .copied()
                .collect();
            if is_one_of(rest, BORROWING_LONG)
                || (!is_one_of(rest, all.as_slice())
                    && abbreviates_a_known_option(all.as_slice(), rest))
            {
                return None;
            }
            if is_one_of(rest, MESSAGE_VALUE_LONG) {
                if !attached {
                    index += 1;
                }
                continue;
            }
            if is_one_of(rest, MESSAGE_FLAG_LONG) {
                continue;
            }
            kept.push(token.clone());
            continue;
        }
        if bytes.starts_with(b"-") && bytes.len() > 1 {
            let mut cluster: Vec<u8> = vec![b'-'];
            let mut value_next: bool = false;
            let letters: &[u8] = &bytes[1..];
            for (at, letter) in letters.iter().enumerate() {
                match letter {
                    b'c' | b'C' => return None,
                    b'm' | b'F' | b't' => value_next = true,
                    b'e' | b's' => {}
                    b'S' => {
                        cluster.extend_from_slice(&letters[at..]);
                        break;
                    }
                    other => cluster.push(*other),
                }
            }
            if cluster != b"-" {
                kept.push(os_from_path_bytes(cluster.as_slice()));
            }
            if value_next {
                index += 1;
            }
            continue;
        }
        kept.push(token.clone());
    }
    kept.push(OsString::from("-F"));
    kept.push(file.as_os_str().to_os_string());
    kept.push(OsString::from("--cleanup=verbatim"));
    kept.push(OsString::from("--no-edit"));
    kept.extend(tail.iter().cloned());
    return Some(kept);
}
