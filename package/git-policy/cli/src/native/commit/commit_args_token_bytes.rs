//! What:
//!  One token of a commit command line,
//! read as the bytes it is:
//! its option name,
//! its cluster letters and the value a cluster carries inline.
//! Why:
//!  A message or a file name the person typed may hold a byte that is not UTF-8,
//! so an option is compared as the bytes Git compares,
//!  and a value is moved as the bytes it is,
//! never spelled.
//!  These readings are the byte-level half of the argument transforms,
//! so they stand in a file of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isOneOf(token, ['-m']) // token.equals(Buffer.from('-m'))
//! ```

/// The argument bytes of one token,
/// and the operating-system string they came from.
use crate::path_bytes::{argument_bytes, os_from_path_bytes};
/// A command line is made of operating-system strings.
use std::ffi::{OsStr, OsString};

/// Commit options whose separated value never names a flag.
pub(crate) const COMMIT_VALUE_OPTIONS: [&str; 20] = [
    "-m",
    "-F",
    "-C",
    "-c",
    "-t",
    "-U",
    "--message",
    "--file",
    "--reuse-message",
    "--reedit-message",
    "--squash",
    "--fixup",
    "--author",
    "--date",
    "--cleanup",
    "--trailer",
    "--template",
    "--unified",
    "--inter-hunk-context",
    "--pathspec-from-file",
];

/// Short-value options whose clustered form can carry the value in the same token.
const SHORT_VALUE_OPTIONS: [&str; 6] = ["-m", "-F", "-C", "-c", "-t", "-U"];

/// Short commit options with an optional attached value (`-S[<keyid>]`,
/// `-u[<mode>]`).
pub(crate) const OPTIONAL_VALUE_LETTERS: &[u8] = b"Su";

/// What:
///  Whether one token is one of `options`,
/// compared as the bytes Git compares.
/// Why:
///   An option's spelling is text,
/// so a token holding a byte that is not UTF-8 is no option at all,
///  and the comparison never reads the bytes as characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// options.some((option) => token.equals(Buffer.from(option)))
/// ```
pub(crate) fn is_one_of(token: &[u8], options: &[&str]) -> bool {
    for option in options {
        if option.as_bytes() == token {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether one byte is a letter whose short option takes an optional value.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand,
/// and `S` and `u` end a cluster rather than naming a flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOptionalValueLetter = (letter: number) => OPTIONAL_VALUE_LETTERS.includes(letter);
/// ```
pub(crate) fn is_optional_value_letter(letter: &u8) -> bool {
    return OPTIONAL_VALUE_LETTERS.contains(letter);
}

/// What:
///  Whether one byte is a letter whose short option takes the next token as its value.
/// Why:
///   The letters are read from the one list of short value options,
/// so the list and the cluster rule cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carriesAValue = (letter) => ['-m', '-F', '-C', '-c', '-t', '-U'].includes('-' + letter)
/// ```
pub(crate) fn is_a_short_value_letter(letter: u8) -> bool {
    // The second byte of each spelling is the letter; the first is the dash.
    for option in SHORT_VALUE_OPTIONS {
        if option.as_bytes()[1] == letter {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether one byte is the sign that ends an option's name and starts its value.
/// Why:
///   `--git-dir=/somewhere` and `--message=x` carry their value after it,
/// and the name is compared without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnEqualsSign = (byte: number) => byte === 61
/// ```
pub(crate) fn is_an_equals_sign(byte: &u8) -> bool {
    return *byte == b'=';
}

/// What:
///  The letters of a cluster token,
/// without its leading dash.
/// Why:
///  Every rule about a cluster counts letters from the one after the dash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// token.toString().slice(1)
/// ```
pub(crate) fn cluster_letters(token: &[u8]) -> &[u8] {
    return &token[1..];
}

/// What:
///  Whether a token is a short cluster:
/// one dash and at least one letter.
/// Why:
///   A lone `-` is a path and a `--…` token is a long option,
/// and neither is a cluster.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// token[0] === '-' && token[1] !== '-' && token.length > 1
/// ```
pub(crate) fn is_a_short_cluster(token: &[u8]) -> bool {
    return token.starts_with(b"-") && !token.starts_with(b"--") && token.len() > 1;
}

/// What:
///  Finds the first value-taking short option letter inside a cluster token.
/// Why:
///   A cluster such as `-am` carries `m`,
/// whose value is the following token or the rest of the cluster.
/// A letter with an attached value (`-S<keyid>`,
/// `-u<mode>`) ends the search instead:
/// what follows it is its value,
/// so a key ID or mode that spells `m`,
/// `F`,
/// `C`,
/// `c`,
/// `t` or `U` is never read as flags.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// findValueOptionIndex(token)
/// ```
pub(crate) fn find_value_option_index(token: &[u8]) -> Option<usize> {
    // The dash is not a letter, so the search starts after it.
    for (index, letter) in token.iter().enumerate().skip(1) {
        if OPTIONAL_VALUE_LETTERS.contains(letter) {
            return None;
        }
        if is_a_short_value_letter(*letter) {
            return Some(index);
        }
    }
    return None;
}

/// What:
///  Splits one short cluster that carries its value inline into the separated form.
/// Why:
///   `-amhello` becomes `-a`,
/// `-m`,
/// `hello`,
/// so the value is a token of its own,
///  and a value holding a byte that is not UTF-8 arrives whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// normaliseInlineShort('-amhello') // ['-a', '-m', 'hello']
/// ```
pub(crate) fn normalise_inline_short(token: &OsStr) -> Vec<OsString> {
    let bytes: &[u8] = argument_bytes(token);
    if !is_a_short_cluster(bytes) || bytes.len() <= 2 {
        return vec![token.to_os_string()];
    }
    let Some(value_index) = find_value_option_index(bytes) else {
        return vec![token.to_os_string()];
    };
    let mut tokens: Vec<OsString> = Vec::new();
    for letter in cluster_letters(bytes).iter().take(value_index - 1) {
        tokens.push(os_from_path_bytes(&[b'-', *letter]));
    }
    tokens.push(os_from_path_bytes(&[b'-', bytes[value_index]]));
    let inline: &[u8] = &bytes[value_index + 1..];
    if !inline.is_empty() {
        tokens.push(os_from_path_bytes(inline));
    }
    return tokens;
}
