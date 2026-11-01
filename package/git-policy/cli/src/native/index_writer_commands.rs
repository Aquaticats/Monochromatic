//! What:
//!  Whether a resolved forwarded command writes the real index.
//! Why:
//!  Index writers take the landing lock so they never interleave with a landing commit.
//!      The set follows `SPEC.md` "Index-writer coordination":
//!  `add`,
//!  `rm`,
//!  `mv`,
//!      `restore --staged`,
//!  `reset` except `--soft`,
//!  `stash`,
//!  `checkout`,
//!  `switch`,
//!  `merge`,
//!      `rebase`,
//!  `cherry-pick`,
//!  `revert`,
//!  `apply --cached` and `apply --index`,
//!  `update-index`,
//!      `read-tree`,
//!  `am`,
//!  `pull` and `sparse-checkout`.
//!  Options count only before Git's `--`;
//!      long options match in full or as the abbreviations Git accepts
//!      (`src/index-lock/index-writer-commands.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isIndexWriter(await resolveForwardedCommand({ args, gitPath }))
//! ```

/// The resolved command.
use super::forwarded_command::ResolvedCommand;

/// Commands that write the index in every form.
pub const ALWAYS_WRITING_COMMANDS: [&[u8]; 15] = [
    b"add",
    b"am",
    b"checkout",
    b"cherry-pick",
    b"merge",
    b"mv",
    b"pull",
    b"read-tree",
    b"rebase",
    b"revert",
    b"rm",
    b"sparse-checkout",
    b"stash",
    b"switch",
    b"update-index",
];

/// What:
///  A long option with the shortest abbreviation Git accepts for it in its command.
/// Why:
///   Git's `parse_options` accepts unambiguous prefixes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LongOption = { name: string; minimumLength: number };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LongOption {
    /// Full spelling.
    pub name: &'static [u8],
    /// Shortest accepted prefix length,
    ///  `--` included.
    pub minimum_length: usize,
}

/// `git restore --staged`;
///  `--s` is ambiguous with `--source`.
pub const RESTORE_STAGED: LongOption = LongOption { name: b"--staged", minimum_length: 4 };
/// `git reset --soft`;
///  `--s` is accepted.
pub const RESET_SOFT: LongOption = LongOption { name: b"--soft", minimum_length: 3 };
/// `git apply --cached`;
///  `--c` is ambiguous with `--check`.
pub const APPLY_CACHED: LongOption = LongOption { name: b"--cached", minimum_length: 4 };
/// `git apply --index`;
///  `--in` is ambiguous with `--intent-to-add` and `--inaccurate-eof`.
pub const APPLY_INDEX: LongOption = LongOption { name: b"--index", minimum_length: 5 };

/// What:
///  The option tokens after the subcommand and before `--`.
/// Why:
///   `optionTokens`:
///  a token is an option when it starts with `-` and is not `-` alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionTokens(command: ResolvedGitCommand): string[];
/// ```
pub fn option_tokens(command: &ResolvedCommand) -> Vec<&[u8]> {
    // `mut` allows collecting tokens up to the separator.
    let mut options: Vec<&[u8]> = Vec::new();
    for token in command.region() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"--" {
            break;
        }
        if bytes.starts_with(b"-") && bytes != b"-" {
            options.push(bytes);
        }
    }
    return options;
}

/// What:
///  Whether a token spells a long option or an accepted abbreviation of it.
/// Why:
///   `matchesLong`:
///  an attached `=value` is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function matchesLong({ token, option }): boolean;
/// ```
pub fn matches_long(token: &[u8], option: LongOption) -> bool {
    let name: &[u8] = match token.iter().position(is_equals) {
        Some(end) => &token[..end],
        None => token,
    };
    return name.len() >= option.minimum_length && option.name.starts_with(name);
}

/// What:
///  Whether a byte is `=`.
/// Why:
///   A named predicate keeps the search free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x3d
/// ```
fn is_equals(byte: &u8) -> bool {
    return *byte == b'=';
}

/// What:
///  Whether a short-option cluster sets `letter` before a value-taking letter consumes the
///       rest.
/// Why:
///   `clusterHas`:
///  `-WS` sets `--staged`,
///  `-sS` does not (`-s` takes the rest as a value).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clusterHas({ token, letter, valueLetters }): boolean;
/// ```
pub fn cluster_has(token: &[u8], letter: u8, value_letters: &[u8]) -> bool {
    if !token.starts_with(b"-") || token.starts_with(b"--") {
        return false;
    }
    for byte in &token[1..] {
        if *byte == letter {
            return true;
        }
        if value_letters.contains(byte) {
            return false;
        }
    }
    return false;
}

/// What:
///  Whether a resolved forwarded command writes the real index.
/// Why:
///   `isIndexWriter`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isIndexWriter(command: ResolvedGitCommand): boolean;
/// ```
pub fn is_index_writer(command: &ResolvedCommand) -> bool {
    if !command.is_builtin() {
        return false;
    }
    let word: &[u8] = command.word();
    if ALWAYS_WRITING_COMMANDS.contains(&word) {
        return true;
    }
    let options: Vec<&[u8]> = option_tokens(command);
    if word == b"restore" {
        for token in &options {
            if matches_long(token, RESTORE_STAGED) || cluster_has(token, b'S', b"s") {
                return true;
            }
        }
        return false;
    }
    if word == b"reset" {
        for token in &options {
            if matches_long(token, RESET_SOFT) {
                return false;
            }
        }
        return true;
    }
    if word == b"apply" {
        for token in &options {
            if matches_long(token, APPLY_CACHED) || matches_long(token, APPLY_INDEX) {
                return true;
            }
        }
    }
    return false;
}

/// Classification controls stay out of the release executable.
#[cfg(test)]
#[path = "index_writer_commands_tests.rs"]
mod tests;
