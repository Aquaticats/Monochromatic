//! What:
//!  Whether one `git worktree prune` removes nothing:
//!  a dry run by Git's own reading of its own options,
//!  or a form Git refuses before it prunes anything.
//! Why:
//!  The pass that restores a managed clone's pointer must not write on a
//!  command that removes nothing,
//!  and Git answers that question with its own option parser rather than the
//!  words the person typed:
//!  `prune` declares `-n`/`--dry-run` as one boolean
//!  (`OPT__DRY_RUN`,
//!  `OPT_SET_INT` with `PARSE_OPT_NOARG` and no
//!  `PARSE_OPT_NONEG`),
//!  and `parse_long_opt` accepts any unique prefix of a long name and any
//!  prefix of it after `no-`,
//!  so `--dry`,
//!  `--d`,
//!  `--no-dry` and `--no-dru` are all Git's own spellings
//!  of it,
//!  while `--dry-run=<value>` and `--no-dry-run=<value>` are refused as
//!  "takes no value" (`do_get_value`).
//!  The last of them wins,
//!  as Git applies them in the order the person typed
//!  them,
//!  and Git reads a word after `--` as an operand of a command that takes none,
//!  so that command is its usage error and removes nothing either.
//!  A word this reading does not know is not a removal of nothing:
//!  the pass still guards it,
//!  because a spelling a later Git adds must never be
//!  read as a dry run (Git 2.56.1's `builtin/worktree.c`,
//!  `parse-options.c`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writesNothing = aPruneThatRemovesNothing(['prune', '--no-dry', '--dry']); // true: the last word wins
//! ```

use std::ffi::OsString;

/// Whether Git reads one word as an option rather than as an operand.
///
/// Why:
///  Git reads a word of one `-` as a name and every longer word that starts
///  with `-` as an option,
///  so the subcommand word is the first word this answers `false` for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const option = (word: string) => word.length > 1 && word.startsWith('-');
/// ```
fn is_an_option(word: &OsString) -> bool {
    let bytes: &[u8] = word.as_encoded_bytes();
    return bytes.len() > 1 && bytes.starts_with(b"-");
}

/// Whether one long option names the dry run,
/// with or without Git's `no-` that unsets it.
/// Why:
///  Git matches a unique prefix of the declared long name,
///  so every abbreviation of `dry-run` is the same option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = name.startsWith('dry-run'.slice(0, name.length));
/// ```
fn names_the_dry_run(name: &[u8]) -> bool {
    return name.len() <= b"dry-run".len() && b"dry-run".starts_with(name);
}

/// Whether one long option names Git's expiry word,
/// which takes the next word as its value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = name.startsWith('expire'.slice(0, name.length));
/// ```
fn names_the_expiry(name: &[u8]) -> bool {
    return name.len() <= b"expire".len() && b"expire".starts_with(name);
}

/// What:
///  Whether one `git worktree prune` removes nothing,
///  read from the words after the `worktree` word.
/// Why:
///  A dry run must not write,
///  and the pass that writes is what this answer keeps out of it,
///  so the reading follows Git's own parser rather than the exact spellings it
///  started with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aPruneThatRemovesNothing(words: string[]): boolean;
/// ```
pub fn a_prune_that_removes_nothing(words: &[OsString]) -> bool {
    let mut tokens = words.iter();
    // The subcommand word is the first word that is no option, and the options this answers about come after it.
    if tokens.find(|word| return !is_an_option(word)).is_none() {
        return false;
    }
    let mut dry: bool = false;
    let mut options_end: bool = false;
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !options_end {
            if bytes == b"--" {
                options_end = true;
                continue;
            }
            if is_an_option(token) {
                if let Some(long) = bytes
                    .strip_prefix(b"--")
                    .filter(|long| return !long.starts_with(b"-"))
                {
                    let (name, value): (&[u8], Option<&[u8]>) =
                        match long.iter().position(|byte| return *byte == b'=') {
                            Some(at) => (&long[..at], Some(&long[at + 1..])),
                            None => (long, None),
                        };
                    if let Some(abbreviated) = name.strip_prefix(b"no-") {
                        if names_the_dry_run(abbreviated) {
                            // Git refuses a value on this option and prunes nothing.
                            if value.is_some() {
                                return true;
                            }
                            dry = false;
                        }
                        continue;
                    }
                    if names_the_dry_run(name) {
                        if value.is_some() {
                            return true;
                        }
                        dry = true;
                        continue;
                    }
                    // `--expire <time>` and its abbreviations take the next word as their value, which is no option of this reading.
                    if names_the_expiry(name) && value.is_none() {
                        tokens.next();
                    }
                    continue;
                }
                for short in &bytes[1..] {
                    if *short == b'n' {
                        dry = true;
                    }
                }
                continue;
            }
        }
        // `git worktree prune` takes no operand, so its own parser answers the usage and nothing is pruned.
        return true;
    }
    return dry;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "prune_removes_nothing_tests.rs"]
mod tests;
