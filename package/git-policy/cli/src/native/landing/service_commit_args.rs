//! What:
//!  Which `git commit` command lines the resident service lands,
//! and the message such a line stands for.
//! Why:
//!  The service lands the scoped commit an agent or a person types most (`git commit -m <message> -- <paths>`),
//! with the
//!  options that only shape the message or the identity:
//! `-F <file>`,
//! `--author`,
//! `--date`,
//! `--trailer`,
//! `--cleanup`,
//!  `--allow-empty-message`,
//! `-s`,
//! `--amend` and `--reset-author`.
//! Every other option (editor,
//! signing,
//! templates) still
//!  goes through the transaction until the service covers it,
//! so this module answers "is this line one the service covers" before anything
//!  is prepared,
//! and builds the message with the clean-up Git applies when no editor opens.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parsed = parseServiceCommit(args); // undefined when the line needs native git commit
//! ```

pub use crate::service_commit_args_editor_opening::opens_editor;
use crate::commit_args_token_bytes::{is_an_equals_sign, is_one_of};
use crate::path_bytes::{argument_bytes, os_from_path_bytes};
use std::ffi::OsString;

/// What a covered command line asked for.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServiceCommitArgs {
    /// The `-m` texts,
    /// in order;
    /// each is a paragraph.
    pub messages: Vec<String>,
    /// The `-F` file,
    /// read by the caller relative to the working directory;
    /// never standard input.
    pub file: Option<OsString>,
    /// Whether `--quiet` was given.
    pub quiet: bool,
    /// The `--author` text,
    /// always of the form `Name <email>`.
    pub author: Option<String>,
    /// The `--date` text,
    /// left to Git to parse.
    pub date: Option<String>,
    /// The `--trailer` texts,
    /// in order.
    pub trailers: Vec<String>,
    /// Whether `--cleanup=verbatim` keeps the message exactly as given.
    pub verbatim: bool,
    /// Whether an empty message is allowed.
    pub allow_empty_message: bool,
    /// Whether `-s` asks for a `Signed-off-by` trailer of the committer.
    pub signoff: bool,
    /// The commit whose message `-C` reuses,
    /// with its authorship.
    pub reuse: Option<String>,
    /// The commit `--fixup` or `--squash` names,
    /// and which of the two.
    pub fixup: Option<(FixupKind, String)>,
    /// Whether `--cleanup=strip` also removes comment lines.
    pub strip: bool,
    /// Whether `--no-edit` was given.
    pub no_edit: bool,
    /// Whether `--amend` replaces the last commit:
    /// its parents,
    /// its message (without a message option) and its author.
    pub amend: bool,
    /// Whether `--reset-author` makes the committer the author of an amend,
    /// with a new author date.
    pub reset_author: bool,
}

/// Which message prefix `--fixup` and `--squash` make.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixupKind {
    /// `--fixup=<commit>`:
    /// the message is `fixup! <subject>`.
    /// (`--squash` always opens an editor,
    /// so it is not covered.)
    Fixup,
}

/// Short option letters that take no value and change nothing the service must do.
const FLAG_LETTERS: [char; 7] = ['q', 'n', 'o', 'a', 's', 'i', 'p'];

/// Long options that take no value and change nothing the service must do.
const FLAG_WORDS: [&str; 16] = [
    "--quiet", "--no-verify", "--only", "--no-status", "--no-post-rewrite", "--no-edit", "--no-gpg-sign", "--all", "--signoff", "--allow-empty",
    "--amend", "--reset-author", "--include", "--patch", "--interactive", "--pathspec-file-nul",
];

/// Long options that take one value,
/// with the field they fill.
const VALUE_WORDS: [&str; 9] = ["--message", "--file", "--author", "--date", "--trailer", "--cleanup", "--reuse-message", "--fixup", "--pathspec-from-file"];

/// The value of a long option given as `--name=value` or `--name value`;
/// `index` moves past a separate value.
fn long_value<'a>(token: &'a str, name: &str, arguments: &'a [OsString], index: &mut usize) -> Option<Option<&'a str>> {
    if token == name {
        let value: &str = arguments.get(*index)?.to_str()?;
        *index += 1;
        return Some(Some(value));
    }
    let rest: &str = token.strip_prefix(name)?;
    let value: &str = rest.strip_prefix('=')?;
    return Some(Some(value));
}

/// What:
///  Reads the arguments after `commit` and says whether the service covers them.
/// Why:
///   Anything not listed here is refused,
/// so a new Git option is never silently dropped.
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
                parsed.signoff |= token == "--signoff";
                parsed.no_edit |= token == "--no-edit";
                parsed.amend |= token == "--amend";
                parsed.reset_author |= token == "--reset-author";
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
                        "--reuse-message" => parsed.reuse = Some(value.to_string()),
                        // The private index already holds the paths the file names; the message does not depend on them.
                        "--pathspec-from-file" => {}
                        "--fixup" => {
                            // `amend:` and `reword:` open an editor or change parents; only a plain commit name is covered.
                            if value.contains(':') || value.is_empty() {
                                return None;
                            }
                            parsed.fixup = Some((FixupKind::Fixup, value.to_string()));
                        }
                        _ => match value {
                            "verbatim" => parsed.verbatim = true,
                            "strip" => parsed.strip = true,
                            "whitespace" | "default" => {
                                parsed.verbatim = false;
                                parsed.strip = false;
                            }
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
                if letter == 'm' || letter == 'F' || letter == 'C' {
                    let inline: String = letters[at..].iter().collect();
                    let value: String = if inline.is_empty() {
                        index += 1;
                        arguments.get(index - 1)?.to_str()?.to_string()
                    } else {
                        inline
                    };
                    match letter {
                        'm' => parsed.messages.push(value),
                        'F' => parsed.file = Some(OsString::from(value)),
                        _ => parsed.reuse = Some(value),
                    }
                    at = letters.len();
                } else if FLAG_LETTERS.contains(&letter) {
                    parsed.quiet |= letter == 'q';
                    parsed.signoff |= letter == 's';
                } else {
                    return None;
                }
            }
        }
    }
    // Standard input is read by Git itself; a message from both places is Git's to refuse.
    let sources: usize = usize::from(!parsed.messages.is_empty())
        + usize::from(parsed.file.is_some())
        + usize::from(parsed.reuse.is_some())
        + usize::from(parsed.fixup.is_some());
    // An amend with no message option and `--no-edit` keeps the message of the commit it replaces; with neither, an editor
    //  opens, which is not covered.
    let has_text: bool = sources == 1 || (parsed.amend && parsed.no_edit && sources == 0);
    // An author, when given, is `Name <email>`; none is fine.
    let author_ok: bool = match parsed.author.as_deref() {
        Some(author) => author.contains('<') && author.trim_end().ends_with('>'),
        None => true,
    };
    // A message file named `-` reads standard input, which is not covered.
    let file_ok: bool = match parsed.file.as_deref() {
        Some(file) => file != "-",
        None => true,
    };
    // `-C` takes the author and its date from the reused commit, and `--reset-author` is not covered.
    let reuse_ok: bool = parsed.reuse.is_none() || (parsed.author.is_none() && parsed.date.is_none());
    // `--reset-author` names its own author, so it is covered only for an amend that sets no author, no date and no reused commit.
    let reset_ok: bool = !parsed.reset_author || (parsed.amend && parsed.reuse.is_none() && parsed.author.is_none() && parsed.date.is_none());
    return if has_text && reuse_ok && reset_ok && author_ok && file_ok { Some(parsed) } else { None };
}

/// What:
///  The commit message Git makes of the given texts when no editor opens.
/// Why:
///   Each text is a paragraph;
/// Git strips trailing whitespace from every line,
/// drops leading and trailing blank lines,
///  squeezes runs of blank lines into one and ends the message with a newline.
/// An empty result is no message.
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
    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        return None;
    }
    return Some(format!("{}\n", lines.join("\n")));
}

/// Long options that name where the message comes from,
/// or edit it,
/// or add to it.
const MESSAGE_VALUE_LONG: [&str; 5] = ["--message", "--file", "--template", "--trailer", "--cleanup"];

/// Long options without a value that open the editor or add a sign-off.
const MESSAGE_FLAG_LONG: [&str; 4] = ["--edit", "--no-edit", "--signoff", "--no-signoff"];

/// Long options that take the author,
/// or the parents,
/// or the whole commit from another commit.
const BORROWING_LONG: [&str; 5] = ["--reedit-message", "--reuse-message", "--squash", "--fixup", "--amend"];

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
fn abbreviates_a_known_option(known: &[&str], name: &[u8]) -> bool {
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
pub fn with_fixed_message(arguments: &[OsString], subcommand_index: usize, file: &std::path::Path) -> Option<Vec<OsString>> {
    let region: Vec<OsString> = crate::commit_args::normalise_commit_args(arguments.get(subcommand_index + 1..)?);
    let split: usize = crate::commit_args::separator_index(region.as_slice()).unwrap_or(region.len());
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
            let all: Vec<&str> = MESSAGE_VALUE_LONG.iter().chain(&MESSAGE_FLAG_LONG).chain(&BORROWING_LONG).copied().collect();
            if is_one_of(rest, &BORROWING_LONG) || (!is_one_of(rest, all.as_slice()) && abbreviates_a_known_option(all.as_slice(), rest)) {
                return None;
            }
            if is_one_of(rest, &MESSAGE_VALUE_LONG) {
                if !attached {
                    index += 1;
                }
                continue;
            }
            if is_one_of(rest, &MESSAGE_FLAG_LONG) {
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

/// Argument controls stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_args_tests.rs"]
mod tests;
