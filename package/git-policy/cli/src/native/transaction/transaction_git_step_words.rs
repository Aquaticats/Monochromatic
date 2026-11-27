//! What:
//!  The parse of a Git step's words:
//! the subcommand and where it sits past the global options,
//! the words after it,
//! and their split into the options before a `--`,
//! the other words before it,
//! and the words after it.
//! Why:
//!  The rerun and ending rules judge a step by its subcommand and by the exact options it carries,
//! and a rule that read the raw argument list would take the value of an option for a name.
//!  Kept apart from the rules so both stay under the file limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // subcommandOf(args): [index, name] | undefined; wordsAfter(request, index): string[]; Words.of(words)
//! ```

/// The request whose words are split,
/// and the arguments it holds.
use super::transaction_git_request_and_reply_types::GitRequest;
/// `OsString` is owned operating-system text,
/// the type of a command-line argument.
use std::ffi::OsString;

/// What:
///  The subcommand a request runs and where it sits among the arguments,
///  past the global options (`-c <pair>`,
/// `-C <directory>` and the `--option` forms).
/// Why:
///   The rerun allow-list is decided by the subcommand and,
/// for a few,
/// by its options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const index = args.findIndex(...); // first token that is not a global option or its value
/// ```
pub(crate) fn subcommand_of(arguments: &[OsString]) -> Option<(usize, String)> {
    let mut index: usize = 0;
    while index < arguments.len() {
        let text: String = arguments[index].to_string_lossy().into_owned();
        if text == "-c" || text == "-C" {
            index += 2;
            continue;
        }
        if text.starts_with('-') {
            index += 1;
            continue;
        }
        return Some((index, text));
    }
    return None;
}

/// What:
///  The arguments after the subcommand,
/// as text.
/// Why:
///  The rerun and ending rules judge the words of a step,
/// and arguments that are not valid text are read lossily so a
///  odd byte never makes a rule panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// request.args.slice(position + 1).map(String)
/// ```
pub(super) fn words_after(
    request: &GitRequest,
    position: usize,
) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for argument in &request.arguments[position + 1..] {
        words.push(argument.to_string_lossy().into_owned());
    }
    return words;
}

/// The words of a request after its subcommand,
/// sorted into the options before a `--`,
/// the other words before it,
/// and the words
///  after it.
/// Only options that a proven form lists are accepted,
/// so no option here takes a value the next word could be mistaken
///  for.
pub(super) struct Words<'a> {
    /// The words that start with a hyphen,
    /// before any `--`.
    pub(super) options: Vec<&'a str>,
    /// The other words before any `--`.
    pub(super) before: Vec<&'a str>,
    /// The words after a `--`,
    /// when there was one.
    pub(super) after: Option<Vec<&'a str>>,
}

impl<'a> Words<'a> {
    /// Sort `words`.
    pub(super) fn of(words: &'a [String]) -> Self {
        let mut sorted: Words<'a> = Words {
            options: Vec::new(),
            before: Vec::new(),
            after: None,
        };
        for word in words.iter().map(String::as_str) {
            match sorted.after.as_mut() {
                Some(after) => after.push(word),
                None if word == "--" => sorted.after = Some(Vec::new()),
                None if word.starts_with('-') && word.len() > 1 => sorted.options.push(word),
                None => sorted.before.push(word),
            }
        }
        return sorted;
    }

    /// Whether every option is one of `allowed`.
    pub(super) fn only_options(
        &self,
        allowed: &[&str],
    ) -> bool {
        for option in &self.options {
            if !allowed.contains(option) {
                return false;
            }
        }
        return true;
    }

    /// Whether any word came after a `--` and there is at least one of them.
    pub(super) fn names_paths(&self) -> bool {
        return match &self.after {
            Some(paths) => !paths.is_empty(),
            None => false,
        };
    }

    /// How many of `wanted` are among the options.
    pub(super) fn count_present(
        &self,
        wanted: &[&str],
    ) -> usize {
        let mut present: usize = 0;
        for option in wanted {
            if self.has(option) {
                present += 1;
            }
        }
        return present;
    }

    /// Whether the option `wanted` is there.
    pub(super) fn has(
        &self,
        wanted: &str,
    ) -> bool {
        return self.options.contains(&wanted);
    }
}
