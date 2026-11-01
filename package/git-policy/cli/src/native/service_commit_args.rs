//! What:
//!  Which `git commit` command lines the resident service lands, and the message such a line stands for.
//! Why:
//!  The service lands the scoped commit an agent or a person types most (`git commit -m <message> -- <paths>`), with the
//!  options that only shape the message or the identity: `-F <file>`, `--author`, `--date`, `--trailer`, `--cleanup`,
//!  `--allow-empty-message`. Every other option (amend, editor, signing, templates, sign-off) still goes through the
//!  transaction until the service covers it, so this module answers "is this line one the service covers" before anything
//!  is prepared, and builds the message with the clean-up Git applies when no editor opens.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parsed = parseServiceCommit(args); // undefined when the line needs native git commit
//! ```

use std::ffi::OsString;

/// What a covered command line asked for.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServiceCommitArgs {
    /// The `-m` texts, in order; each is a paragraph.
    pub messages: Vec<String>,
    /// The `-F` file, read by the caller relative to the working directory; never standard input.
    pub file: Option<OsString>,
    /// Whether `--quiet` was given.
    pub quiet: bool,
    /// The `--author` text, always of the form `Name <email>`.
    pub author: Option<String>,
    /// The `--date` text, left to Git to parse.
    pub date: Option<String>,
    /// The `--trailer` texts, in order.
    pub trailers: Vec<String>,
    /// Whether `--cleanup=verbatim` keeps the message exactly as given.
    pub verbatim: bool,
    /// Whether an empty message is allowed.
    pub allow_empty_message: bool,
}

/// Short option letters that take no value and change nothing the service must do.
const FLAG_LETTERS: [char; 4] = ['q', 'n', 'o', 'a'];

/// Long options that take no value and change nothing the service must do.
const FLAG_WORDS: [&str; 8] =
    ["--quiet", "--no-verify", "--only", "--no-status", "--no-post-rewrite", "--no-edit", "--no-gpg-sign", "--all"];

/// Long options that take one value, with the field they fill.
const VALUE_WORDS: [&str; 6] = ["--message", "--file", "--author", "--date", "--trailer", "--cleanup"];

/// The value of a long option given as `--name=value` or `--name value`; `index` moves past a separate value.
fn long_value<'a>(token: &'a str, name: &str, arguments: &'a [OsString], index: &mut usize) -> Option<Option<&'a str>> {
    if token == name {
        let value: &str = arguments.get(*index)?.to_str()?;
        *index += 1;
        return Some(Some(value));
    }
    return token.strip_prefix(name).and_then(|rest| { return rest.strip_prefix('='); }).map(Some);
}

/// What:
///  Reads the arguments after `commit` and says whether the service covers them.
/// Why:
///   Anything not listed here is refused, so a new Git option is never silently dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseServiceCommit(args)
/// ```
pub fn parse_service_commit(arguments: &[OsString]) -> Option<ServiceCommitArgs> {
    let mut parsed: ServiceCommitArgs = ServiceCommitArgs::default();
    let mut index: usize = 0;
    while index < arguments.len() {
        let token: &str = arguments[index].to_str()?;
        index += 1;
        if token == "--" {
            break;
        }
        if !token.starts_with('-') || token == "-" {
            // A pathspec given without the separator; the region already holds it.
            continue;
        }
        if token.starts_with("--") {
            if FLAG_WORDS.contains(&token) {
                parsed.quiet |= token == "--quiet";
            } else if token == "--allow-empty-message" {
                parsed.allow_empty_message = true;
            } else {
                let mut matched: bool = false;
                for name in VALUE_WORDS {
                    let Some(Some(value)) = long_value(token, name, arguments, &mut index) else { continue };
                    matched = true;
                    match name {
                        "--message" => parsed.messages.push(value.to_string()),
                        "--file" => parsed.file = Some(OsString::from(value)),
                        "--author" => parsed.author = Some(value.to_string()),
                        "--date" => parsed.date = Some(value.to_string()),
                        "--trailer" => parsed.trailers.push(value.to_string()),
                        _ => match value {
                            "verbatim" => parsed.verbatim = true,
                            "whitespace" | "default" => parsed.verbatim = false,
                            _ => return None,
                        },
                    }
                    break;
                }
                if !matched {
                    return None;
                }
            }
        } else {
            let letters: Vec<char> = token.chars().skip(1).collect();
            let mut at: usize = 0;
            while at < letters.len() {
                let letter: char = letters[at];
                at += 1;
                if letter == 'm' || letter == 'F' {
                    let inline: String = letters[at..].iter().collect();
                    let value: String = if inline.is_empty() {
                        index += 1;
                        arguments.get(index - 1)?.to_str()?.to_string()
                    } else {
                        inline
                    };
                    if letter == 'm' {
                        parsed.messages.push(value);
                    } else {
                        parsed.file = Some(OsString::from(value));
                    }
                    at = letters.len();
                } else if FLAG_LETTERS.contains(&letter) {
                    parsed.quiet |= letter == 'q';
                } else {
                    return None;
                }
            }
        }
    }
    // Standard input is read by Git itself; a message from both places is Git's to refuse.
    let has_text: bool = !parsed.messages.is_empty() != parsed.file.is_some();
    let author_ok: bool = parsed.author.as_deref().is_none_or(|author| { return author.contains('<') && author.trim_end().ends_with('>'); });
    return if has_text && author_ok && parsed.file.as_deref().is_none_or(|file| { return file != "-"; }) { Some(parsed) } else { None };
}

/// What:
///  The commit message Git makes of the given texts when no editor opens.
/// Why:
///   Each text is a paragraph; Git strips trailing whitespace from every line, drops leading and trailing blank lines,
///  squeezes runs of blank lines into one and ends the message with a newline. An empty result is no message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanupMessage(['subject', 'body'])
/// ```
pub fn cleanup_message(messages: &[String]) -> Option<String> {
    let joined: String = messages.join("\n\n");
    let mut lines: Vec<&str> = Vec::new();
    let mut previous_blank: bool = true;
    for line in joined.split('\n') {
        let trimmed: &str = line.trim_end();
        let blank: bool = trimmed.is_empty();
        if blank && previous_blank {
            continue;
        }
        lines.push(trimmed);
        previous_blank = blank;
    }
    while lines.last().is_some_and(|line| { return line.is_empty(); }) {
        lines.pop();
    }
    if lines.is_empty() {
        return None;
    }
    return Some(format!("{}\n", lines.join("\n")));
}

/// Argument controls stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_args_tests.rs"]
mod tests;
