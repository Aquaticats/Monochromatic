//! What:
//!  The key a signed amend names:
//!  which spelling of `-S` or `--gpg-sign` asks for a signature,
//!  and the `-S[key]` form `git commit-tree` takes.
//! Why:
//!  An amend whose branch moved is signed again by `git commit-tree`,
//!  and the key must be the one the person's line named,
//!  because what the wrapper passes to Git means what the person typed.
//!  The question is a reading of the commit region and nothing else,
//!  so it is a file of its own beside `service_amend_replayed.rs`,
//!  which is the only file that uses it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // service_amend_replayed_signing.ts
//! signingOption(['--gpg-s=KEY']) // Some('-SKEY')
//! ```

/// The reading of the commit region,
/// which says where the options end,
/// which tokens are a value,
/// and whether a short-cluster letter carries its value,
/// and Git's own table,
/// which says what a spelling names.
use crate::commit_args::{boundary_index, short_letter_takes_a_value, value_positions};
use crate::command_commit_table::COMMIT_TABLE;
use crate::command_options_long_resolution::long_option_state;
use std::ffi::OsString;

/// What:
///  One argument as text,
///  with bytes that are not valid text replaced.
/// Why:
///   The commit options are read as text,
///  and `map` takes this function's path where a closure would say nothing a name does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lossyText = (token: Buffer) => token.toString('utf8');
/// ```
fn lossy_text(token: &OsString) -> String {
    return token.to_string_lossy().into_owned();
}

/// What:
///  The signing option the command line named,
///  in the spelling `git commit-tree` takes,
///  and nothing when it named none.
/// Why:
///   A commit made again is signed with the key the person chose,
///  so the reading follows Git:
/// `-S[key]`,
///  `--gpg-sign[=key]` in any unambiguous spelling of it,
///  and the letter inside a short cluster (`-qSKEY`) all ask for one,
///  while the value of another option (`-m -SKEY`) and a path after `--` or
///  `--end-of-options` are neither.
///  `commit-tree` takes `-S` and `-S<key>`,
///  so every spelling is handed on as one of those,
///  and a spelling whose key is empty names the default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// signingOption(['--gpg-s=KEY']) // Some('-SKEY')
/// ```
pub(crate) fn signing_option(arguments: &[OsString]) -> Option<String> {
    let text: Vec<String> = arguments.iter().map(lossy_text).collect();
    let values: Vec<usize> = value_positions(text.as_slice());
    let limit: usize = boundary_index(text.as_slice()).unwrap_or(text.len());
    let mut named: Option<String> = None;
    for (position, token) in text[..limit].iter().enumerate() {
        // The value of an option is the person's text, never the request it spells.
        if values.contains(&position) {
            continue;
        }
        // `true` is the option asked for and `false` its `no-` form, which takes the request back.
        if let Some(requested) = long_option_state(token.as_bytes(), "gpg-sign", COMMIT_TABLE) {
            named = if requested { Some(signing_token(token.as_str())) } else { None };
            continue;
        }
        if let Some(from_letters) = signing_letter_option(token.as_str()) {
            named = Some(from_letters);
        }
    }
    return named;
}

/// What:
///  The `git commit-tree` spelling of a long `--gpg-sign` token,
///  with the key the token attached when it attached one.
/// Why:
///   `--gpg-sign` and `--gpg-s` name the default key,
///  and `--gpg-sign=KEY` names `KEY`,
///  which `commit-tree` takes only as `-S` or `-S<key>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// signingToken('--gpg-sign=KEY') // '-SKEY'
/// ```
fn signing_token(token: &str) -> String {
    return match token.split_once('=') {
        Some((_name, key)) => format!("-S{key}"),
        None => String::from("-S"),
    };
}

/// What:
///  The `git commit-tree` spelling of a short cluster that carries `S`,
///  and nothing when the token carries none before a letter that takes a value.
/// Why:
///   `-SKEY` and `-qSKEY` ask for a signature with the key after the letter,
///  while the `S` after `-m` or `-u` is that option's value (`-mS` is the message `S`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// signingLetterOption('-qSKEY') // Some('-SKEY')
/// ```
fn signing_letter_option(token: &str) -> Option<String> {
    if !token.starts_with('-') || token.starts_with("--") || token.len() < 2 {
        return None;
    }
    let letters: Vec<char> = token.chars().skip(1).collect();
    for (index, letter) in letters.iter().enumerate() {
        if *letter == 'S' {
            let key: String = letters[index + 1..].iter().collect();
            return Some(format!("-S{key}"));
        }
        // A letter that takes a value ends the flags: what follows it is the person's value.
        if short_letter_takes_a_value(*letter) {
            return None;
        }
    }
    return None;
}
