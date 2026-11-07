//! What:
//!  Whether a `commit` command line opens an editor: the long options that never decide it, the options that take a value, and
//!  the walk over the arguments.
//! Why:
//!  `service_commit_args.rs` parses the options the service covers; the question whether an editor opens is a large part of it
//!  and depends on nothing in it, so it is a file the other uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! opensEditor(args) // true when Git would start the person's editor
//! ```

use std::ffi::OsString;
/// Long options with no value that never decide whether an editor opens.
pub(crate) const NEUTRAL_LONG: [&str; 15] = [
    "--quiet", "--no-verify", "--only", "--no-status", "--no-post-rewrite", "--no-gpg-sign", "--all", "--signoff", "--allow-empty",
    "--amend", "--reset-author", "--allow-empty-message", "--verbose", "--include", "--pathspec-file-nul",
];

/// Long options with a value that never decide whether an editor opens.
pub(crate) const NEUTRAL_VALUE_LONG: [&str; 7] = ["--author", "--date", "--trailer", "--cleanup", "--gpg-sign", "--template", "--pathspec-from-file"];

/// What:
///  Whether `git commit` with these arguments may open an editor.
/// Why:
///   The answer decides what happens to a commit Git built when the branch moved meanwhile: with no editor it is built
///  again on the new tip, with an editor the typed message is saved for the person instead of being lost. It errs towards
///  "may open": an option this reading does not know, or a spelling Git accepts as an abbreviation, counts as an editor,
///  because a message saved without need costs nothing and a message lost costs the person's typing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// opensEditor(args)
/// ```
pub fn opens_editor(arguments: &[OsString]) -> bool {
    let mut message_source: bool = false;
    let mut edit: Option<bool> = None;
    let mut index: usize = 0;
    while index < arguments.len() {
        let Some(token) = arguments[index].to_str() else { return true };
        index += 1;
        if token == "--" {
            break;
        }
        if !token.starts_with('-') || token == "-" {
            continue;
        }
        if let Some(long) = token.strip_prefix("--") {
            let (name, value): (&str, Option<&str>) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (long, None),
            };
            let dashed: String = format!("--{name}");
            let takes_value: bool = ["--message", "--file", "--reuse-message", "--reedit-message", "--fixup", "--squash"].contains(&dashed.as_str())
                || NEUTRAL_VALUE_LONG.contains(&dashed.as_str());
            // A separate value is the next token, except for the options whose value may be attached only.
            let detached: Option<&str> = if takes_value && value.is_none() && !["--gpg-sign", "--squash"].contains(&dashed.as_str()) {
                index += 1;
                match arguments.get(index - 1) {
                    Some(next) => next.to_str(),
                    None => None,
                }
            } else {
                None
            };
            let given: Option<&str> = value.or(detached);
            match dashed.as_str() {
                "--edit" => edit = Some(true),
                "--no-edit" => edit = Some(false),
                "--message" | "--file" | "--reuse-message" => message_source = true,
                "--reedit-message" | "--squash" => return true,
                "--fixup" => {
                    let names_a_commit: bool = match given {
                        Some(text) => text.contains(':'),
                        None => true,
                    };
                    if names_a_commit {
                        return true;
                    }
                    message_source = true;
                }
                other if NEUTRAL_LONG.contains(&other) || NEUTRAL_VALUE_LONG.contains(&other) => {}
                _ => return true,
            }
        } else {
            let letters: Vec<char> = token.chars().skip(1).collect();
            let mut at: usize = 0;
            while at < letters.len() {
                let letter: char = letters[at];
                at += 1;
                match letter {
                    'e' => edit = Some(true),
                    'c' => return true,
                    'm' | 'F' | 'C' | 't' => {
                        message_source |= letter != 't';
                        if at == letters.len() {
                            index += 1;
                        }
                        at = letters.len();
                    }
                    'S' => at = letters.len(),
                    'q' | 'n' | 'o' | 'a' | 's' | 'v' | 'i' => {}
                    _ => return true,
                }
            }
        }
    }
    return match edit {
        Some(wanted) => wanted,
        None => !message_source,
    };
}
