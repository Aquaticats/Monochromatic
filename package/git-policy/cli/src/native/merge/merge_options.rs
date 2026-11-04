//! What:
//!  The shape of a `git merge` command line that the wrapper can complete in isolation, or the reason it cannot.
//! Why:
//!  The isolated merge reproduces what native Git writes only for options whose effect is known exactly: the
//!  message, the fast-forward choice, signing and the unrelated-histories switch.
//!  Any other option (a strategy, `--squash`, `--no-commit`, `--ff-only`, `--edit`, an abbreviation Git would
//!  accept) is not guessed: the command is forwarded to native Git unchanged, with the refusal it may give.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseMergeShape(region: Uint8Array[]): { shape: MergeShape } | { unsupported: string };
//! ```

/// Owned operating-system text of raw bytes.
use std::ffi::OsString;

/// Whether the person asked for a merge commit even where a fast-forward is possible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FastForwardChoice = 'default' | 'allowed' | 'never';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FastForwardChoice {
    /// Neither `--ff` nor `--no-ff`; `merge.ff` decides.
    Default,
    /// `--ff`.
    Allowed,
    /// `--no-ff`.
    Never,
}

/// How the merge commit is signed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Signing = 'default' | 'never' | { key?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Signing {
    /// Neither option; `commit.gpgSign` decides.
    Default,
    /// `--no-gpg-sign`.
    Never,
    /// `-S`, `--gpg-sign` or either with a key.
    Sign(Option<Vec<u8>>),
}

/// The options of a merge the wrapper can complete in isolation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeShape = { reference: Uint8Array; message?: Uint8Array; fastForward: FastForwardChoice; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeShape {
    /// The one commit-ish being merged, exactly as typed.
    pub reference: OsString,
    /// The message of every `-m`, joined as Git joins them; absent when none was given.
    pub message: Option<Vec<u8>>,
    /// The fast-forward choice.
    pub fast_forward: FastForwardChoice,
    /// The signing choice.
    pub signing: Signing,
    /// Whether `--no-edit` was given; without it a terminal opens the editor.
    pub no_edit: bool,
    /// Whether `-q` or `--quiet` was given.
    pub quiet: bool,
    /// The diffstat choice (`--stat`, `--no-stat`, `-n`); absent when `merge.stat` decides.
    pub stat: Option<bool>,
    /// Whether `--allow-unrelated-histories` was given.
    pub allow_unrelated_histories: bool,
    /// The shortlog choice (`--log`, `--no-log`); absent when `merge.log` decides.
    pub log: Option<bool>,
}

/// The reading of a merge command line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParsedMerge = { kind: 'supported'; shape: MergeShape } | { kind: 'unsupported'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParsedMerge {
    /// The command line is one the wrapper completes exactly.
    Supported(MergeShape),
    /// The command line is forwarded to native Git; the text names why.
    Unsupported(String),
}

/// What:
///  Add one `-m` value the way Git joins them: a blank line between values, each ending in a line feed.
/// Why:
///   `option_parse_message` in Git's `builtin/merge.c` adds a line feed before every value after the first and
///  completes the last line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function appendMessage(buffer: Uint8Array, value: Uint8Array): Uint8Array;
/// ```
fn append_message(buffer: &mut Vec<u8>, value: &[u8]) {
    if !buffer.is_empty() {
        buffer.push(b'\n');
    }
    buffer.extend_from_slice(value);
    if buffer.last() != Some(&b'\n') {
        buffer.push(b'\n');
    }
}

/// What:
///  Whether an option is accepted and has no effect on the merge commit.
/// Why:
///   These options change only what Git prints or which hooks it runs; the isolated merge runs no hook of the
///  merge commit and prints its own result, so they need no handling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isIgnorable = (text: string) => IGNORED.includes(text);
/// ```
fn is_ignorable(text: &str) -> bool {
    return matches!(
        text,
        "-v" | "--verbose" | "--no-verbose" | "--progress" | "--no-progress" | "--no-verify" | "--verify"
            | "--summary" | "--no-summary" | "--no-signoff" | "--no-autostash" | "--no-verify-signatures"
    );
}

/// What:
///  Read the arguments after `merge` into a supported shape, or say why they are not one.
/// Why:
///   Only exact spellings are read; Git accepts unambiguous abbreviations and attached values in forms this reader
///  does not model, and a form it does not model must reach native Git instead of being guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMergeShape(region: string[]): ParsedMerge;
/// ```
pub fn parse_merge_shape(region: &[OsString]) -> ParsedMerge {
    let mut positional: Vec<OsString> = Vec::new();
    let mut message: Vec<u8> = Vec::new();
    let mut has_message: bool = false;
    let mut shape: MergeShape = MergeShape {
        reference: OsString::new(),
        message: None,
        fast_forward: FastForwardChoice::Default,
        signing: Signing::Default,
        no_edit: false,
        quiet: false,
        stat: None,
        allow_unrelated_histories: false,
        log: None,
    };
    let mut tokens = region.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        let Ok(text) = std::str::from_utf8(bytes) else {
            if bytes.starts_with(b"-") {
                return ParsedMerge::Unsupported(String::from("an option that is not UTF-8 text"));
            }
            positional.push(token.clone());
            continue;
        };
        if !text.starts_with('-') || text == "-" {
            positional.push(token.clone());
            continue;
        }
        match text {
            "--no-edit" => shape.no_edit = true,
            "-q" | "--quiet" => shape.quiet = true,
            "--ff" => shape.fast_forward = FastForwardChoice::Allowed,
            "--no-ff" => shape.fast_forward = FastForwardChoice::Never,
            "--stat" => shape.stat = Some(true),
            "--no-stat" | "-n" => shape.stat = Some(false),
            "--allow-unrelated-histories" => shape.allow_unrelated_histories = true,
            "--log" => shape.log = Some(true),
            "--no-log" => shape.log = Some(false),
            "--no-gpg-sign" => shape.signing = Signing::Never,
            "-S" | "--gpg-sign" => shape.signing = Signing::Sign(None),
            "-m" | "--message" => {
                let Some(value) = tokens.next() else {
                    return ParsedMerge::Unsupported(String::from("-m without a value"));
                };
                append_message(&mut message, value.as_encoded_bytes());
                has_message = true;
            }
            other if is_ignorable(other) => {}
            other if other.starts_with("--message=") => {
                append_message(&mut message, &bytes["--message=".len()..]);
                has_message = true;
            }
            other if other.starts_with("--gpg-sign=") => {
                shape.signing = Signing::Sign(Some(bytes["--gpg-sign=".len()..].to_vec()));
            }
            other if other.starts_with("-S") && !other.starts_with("--") => {
                shape.signing = Signing::Sign(Some(bytes[2..].to_vec()));
            }
            other if other.starts_with("-m") && !other.starts_with("--") => {
                append_message(&mut message, &bytes[2..]);
                has_message = true;
            }
            other => return ParsedMerge::Unsupported(format!("the option {other}")),
        }
    }
    if shape.log.is_some() && has_message {
        return ParsedMerge::Unsupported(String::from("--log or --no-log together with -m"));
    }
    if positional.len() != 1 {
        return ParsedMerge::Unsupported(format!("{} commits to merge, not exactly one", positional.len()));
    }
    shape.reference = positional.remove(0);
    if shape.reference.as_encoded_bytes().is_empty() {
        return ParsedMerge::Unsupported(String::from("an empty commit name"));
    }
    if has_message {
        shape.message = Some(message);
    }
    return ParsedMerge::Supported(shape);
}

/// Option forms accepted, refused and passed through.
#[cfg(test)]
#[path = "merge_options_tests.rs"]
mod tests;
