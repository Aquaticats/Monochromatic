//! What:
//!  Which `git commit` command lines the resident service lands, and the message such a line stands for.
//! Why:
//!  The service lands the plain scoped commit an agent or a person types most (`git commit -m <message> -- <paths>`).
//!  Every other option (amend, editor, signing, templates, trailers, dates) still goes through native `git commit`
//!  until the service covers it, so this module answers "is this line one the service covers" before anything is
//!  prepared, and builds the message with the clean-up Git applies to `-m` text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parsed = parseServiceCommit(args); // undefined when the line needs native git commit
//! ```

use std::ffi::OsString;

/// What a covered command line asked for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceCommitArgs {
    /// The `-m` texts, in order; each is a paragraph.
    pub messages: Vec<String>,
    /// Whether `--quiet` was given.
    pub quiet: bool,
}

/// Short option letters that take no value and change nothing the service must do.
const FLAG_LETTERS: [char; 3] = ['q', 'n', 'o'];

/// Long options that take no value and change nothing the service must do.
const FLAG_WORDS: [&str; 5] = ["--quiet", "--no-verify", "--only", "--no-status", "--no-post-rewrite"];

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
    let mut parsed: ServiceCommitArgs = ServiceCommitArgs { messages: Vec::new(), quiet: false };
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
        if let Some(text) = token.strip_prefix("--message=") {
            parsed.messages.push(text.to_string());
        } else if token == "--message" {
            parsed.messages.push(arguments.get(index)?.to_str()?.to_string());
            index += 1;
        } else if token.starts_with("--") {
            if !FLAG_WORDS.contains(&token) {
                return None;
            }
            parsed.quiet |= token == "--quiet";
        } else {
            let letters: Vec<char> = token.chars().skip(1).collect();
            let mut at: usize = 0;
            while at < letters.len() {
                let letter: char = letters[at];
                at += 1;
                if letter == 'm' {
                    let inline: String = letters[at..].iter().collect();
                    if inline.is_empty() {
                        parsed.messages.push(arguments.get(index)?.to_str()?.to_string());
                        index += 1;
                    } else {
                        parsed.messages.push(inline);
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
    return if parsed.messages.is_empty() { None } else { Some(parsed) };
}

/// What:
///  The commit message Git makes of `-m` texts when no editor opens.
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
