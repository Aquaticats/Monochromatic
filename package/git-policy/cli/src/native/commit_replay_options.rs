//! What:
//!  The commit options a replay repeats outside native `git commit`: the key a signed commit is re-signed with.
//! Why:
//!  A replay rebuilds the prepared commit on a moved parent, so it must sign the same way
//!  (`src/policy-engine/commit-replay-options.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! replayOptions(['--gpg-sign=abc', '-m', 'x']); // { signingKey: 'abc' }
//! ```

use super::commit_args::normalise_commit_args;

/// The token that ends the options and starts the pathspecs.
const PATHSPEC_SEPARATOR: &str = "--";

/// Short commit options that consume the next token.
const VALUE_LETTERS: [char; 5] = ['m', 'F', 'C', 'c', 't'];

/// Short commit options whose value, when present, is attached (`-S<key>`, `-u<mode>`).
const ATTACHED_VALUE_LETTERS: [char; 2] = ['S', 'u'];

/// Long commit options that consume the next token when written without `=`.
const LONG_VALUE_OPTIONS: [&str; 12] = [
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
    "--pathspec-from-file",
];

/// What:
///  The replay-relevant options of one commit invocation.
/// Why:
///   They are the only commit options a replay repeats itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReplayOptions = { signingKey?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReplayOptions {
    /// The key ID given to `-S` or `--gpg-sign`, absent for the configured key.
    pub signing_key: Option<String>,
}

/// What:
///  Option state while scanning.
/// Why:
///   Each token either changes the decisions, expects a value next, or ends the options at `--`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanState = { signingKey; valueNext; done };
/// ```
#[derive(Clone, Debug, Default)]
struct ScanState {
    /// The latest signing key, empty for the configured key.
    signing_key: String,
    /// Whether the next token is an option value.
    value_next: bool,
    /// Whether the pathspec separator was passed.
    done: bool,
}

/// What:
///  Apply one short-option cluster such as `-aS` or `-Skey`.
/// Why:
///   An attached value after `S` is the key, and a value letter consumes the next token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanCluster({ state, token });
/// ```
fn scan_cluster(state: &ScanState, token: &str) -> ScanState {
    let letters: Vec<char> = token.chars().skip(1).collect();
    let value_index: Option<usize> = letters.iter().position(|letter| {
        return VALUE_LETTERS.contains(letter) || ATTACHED_VALUE_LETTERS.contains(letter);
    });
    let value_letter: Option<char> = value_index.map(|index| { return letters[index]; });
    let attached: String = match value_index {
        Some(index) => letters[index + 1..].iter().collect(),
        None => String::new(),
    };
    return ScanState {
        signing_key: if value_letter == Some('S') { attached.clone() } else { state.signing_key.clone() },
        value_next: value_letter.is_some_and(|letter| { return VALUE_LETTERS.contains(&letter); }) && attached.is_empty(),
        done: state.done,
    };
}

/// What:
///  Apply one token.
/// Why:
///   Long flags set the decisions directly, value options consume the next token, and clusters scan letter by letter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanToken({ state, token });
/// ```
fn scan_token(state: ScanState, token: &str) -> ScanState {
    if state.done {
        return state;
    }
    if state.value_next {
        return ScanState {
            value_next: false,
            ..state
        };
    }
    if token == PATHSPEC_SEPARATOR {
        return ScanState { done: true, ..state };
    }
    if token == "--gpg-sign" || token.starts_with("--gpg-sign=") {
        return ScanState {
            signing_key: String::from(token.get("--gpg-sign=".len()..).unwrap_or("")),
            ..state
        };
    }
    if LONG_VALUE_OPTIONS.contains(&token) {
        return ScanState {
            value_next: true,
            ..state
        };
    }
    // A lone dash is a cluster without letters, which changes nothing.
    if token.starts_with('-') && !token.starts_with("--") {
        return scan_cluster(&state, token);
    }
    return state;
}

/// What:
///  Read the replay-relevant options from private commit arguments.
/// Why:
///   Native preparation ran with these arguments, so they decide what a replay repeats.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// replayOptions(commitArgs);
/// ```
pub fn replay_options(commit_args: &[String]) -> ReplayOptions {
    let mut state: ScanState = ScanState::default();
    for token in normalise_commit_args(commit_args) {
        state = scan_token(state, token.as_str());
    }
    return ReplayOptions {
        signing_key: if state.signing_key.is_empty() { None } else { Some(state.signing_key) },
    };
}

/// Replay option controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_replay_options_tests.rs"]
mod tests;
