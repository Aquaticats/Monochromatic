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
pub use crate::service_commit_args_fixed_message::with_fixed_message;
pub use crate::service_commit_args_message::cleanup_message;
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
const FLAG_LETTERS: &[char] = &['q', 'n', 'o', 'a', 's', 'i', 'p'];

/// Long options that take no value and change nothing the service must do.
const FLAG_WORDS: &[&str] = &[
    "--quiet",
    "--no-verify",
    "--only",
    "--no-status",
    "--no-post-rewrite",
    "--no-edit",
    "--no-gpg-sign",
    "--all",
    "--signoff",
    "--allow-empty",
    "--amend",
    "--reset-author",
    "--include",
    "--patch",
    "--interactive",
    "--pathspec-file-nul",
];

/// Long options that take one value,
/// with the field they fill.
const VALUE_WORDS: &[&str] = &[
    "--message",
    "--file",
    "--author",
    "--date",
    "--trailer",
    "--cleanup",
    "--reuse-message",
    "--fixup",
    "--pathspec-from-file",
];

/// The value of a long option given as `--name=value` or `--name value`;
/// `index` moves past a separate value.
fn long_value<'a>(
    token: &'a str,
    name: &str,
    arguments: &'a [OsString],
    index: &mut usize,
) -> Option<Option<&'a str>> {
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
                for &name in VALUE_WORDS {
                    let Some(Some(value)) = long_value(token, name, arguments, &mut index) else {
                        continue;
                    };
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
    let reuse_ok: bool =
        parsed.reuse.is_none() || (parsed.author.is_none() && parsed.date.is_none());
    // `--reset-author` names its own author, so it is covered only for an amend that sets no author, no date and no reused commit.
    let reset_ok: bool = !parsed.reset_author
        || (parsed.amend
            && parsed.reuse.is_none()
            && parsed.author.is_none()
            && parsed.date.is_none());
    return if has_text && reuse_ok && reset_ok && author_ok && file_ok {
        Some(parsed)
    } else {
        None
    };
}

/// Argument controls stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_args_tests.rs"]
mod tests;
