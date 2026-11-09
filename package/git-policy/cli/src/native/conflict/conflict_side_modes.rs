//! What:
//!  The pathspec modes and the pathspec magic that make a conflict-side line unreadable to the guard,
//! and the sentences that
//!  tell a person the same line without them would be saved and then allowed.
//! Why:
//!  A mode (`GIT_ICASE_PATHSPECS`,
//! `--glob-pathspecs`,
//! ...) changes which files a pathspec names,
//! and magic (`:(icase)`,
//! `:!`,
//!  `:/`) does the same for one token,
//! so the guard cannot say which files Git would replace and keeps the rejection.
//! The
//!  rejection alone does not say what to change;
//! when the line would be allowed without the mode or magic,
//! the message does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! modesInEffect(['--icase-pathspecs'], overlay); // ['--icase-pathspecs']
//! modeNote(['--icase-pathspecs'], 'merge'); // 'This merge is in progress ... the same command without `--icase-pathspecs` ...'
//! ```

use std::ffi::OsString;

/// The global options that set a pathspec mode,
/// the ones `git.c` accepts before the subcommand.
pub const MODE_OPTIONS: [&str; 5] = ["--icase-pathspecs", "--glob-pathspecs", "--noglob-pathspecs", "--literal-pathspecs", "--no-literal-pathspecs"];

/// The variables that set a pathspec mode.
pub const MODE_VARIABLES: [&str; 4] = ["GIT_ICASE_PATHSPECS", "GIT_GLOB_PATHSPECS", "GIT_NOGLOB_PATHSPECS", "GIT_LITERAL_PATHSPECS"];

/// What:
///  The spellings of the pathspec modes a command runs under:
/// each global option present,
/// each variable present in the
///  process environment or the child overlay.
/// Why:
///   The note names exactly what the person set,
/// so the way out is to remove exactly that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modesInEffect(globalPrefix: string[], overlay: [string, string][]): string[];
/// ```
pub fn modes_in_effect(global_prefix: &[OsString], overlay: &[(OsString, OsString)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for option in MODE_OPTIONS {
        for each in global_prefix {
            if each.as_encoded_bytes() == option.as_bytes() {
                found.push(String::from(option));
                break;
            }
        }
    }
    for variable in MODE_VARIABLES {
        let mut set: bool = std::env::var_os(variable).is_some();
        for (name, _) in overlay {
            set |= name.as_encoded_bytes() == variable.as_bytes();
        }
        if set {
            found.push(String::from(variable));
        }
    }
    return found;
}

/// What:
///  Whether the pathspec modes in effect read `*`,
/// `?`,
/// `[` and `\\` as pattern characters,
/// so a token holding one of them names a set and not one file.
/// Why:
///   `--literal-pathspecs` and `--noglob-pathspecs` make those characters ordinary in a pathspec,
/// so `st*r.txt` then names the file of that name;
/// every other mode makes it a pattern.
///  Git refuses two incompatible modes in one command,
/// so a command that reaches here carries one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const globCharactersAreLiteral = (modes: string[]) => modes.some((mode) => /literal|noglob/i.test(mode));
/// ```
pub fn glob_characters_are_literal(modes: &[String]) -> bool {
    for mode in modes {
        if matches!(mode.as_str(), "--literal-pathspecs" | "--noglob-pathspecs" | "GIT_LITERAL_PATHSPECS" | "GIT_NOGLOB_PATHSPECS") {
            return true;
        }
    }
    return false;
}

/// The items in backticks,
/// joined with commas.
fn backticked<'item>(items: impl Iterator<Item = &'item str>) -> String {
    let mut text: String = String::new();
    for (position, item) in items.enumerate() {
        if position > 0 {
            text.push_str(", ");
        }
        text.push('`');
        text.push_str(item);
        text.push('`');
    }
    return text;
}

/// The magic of a magic-and-file pair.
fn first_of(pair: &(String, String)) -> &str {
    return pair.0.as_str();
}

/// The file of a magic-and-file pair.
fn second_of(pair: &(String, String)) -> &str {
    return pair.1.as_str();
}

/// The word for an operation in a sentence.
fn operation_word(operation: crate::conflict_side_state::Operation) -> &'static str {
    use crate::conflict_side_state::Operation;
    return match operation {
        Operation::Merge => "merge",
        Operation::Rebase => "rebase",
        Operation::CherryPick => "cherry-pick",
        Operation::Revert => "revert",
    };
}

/// What:
///  The sentence added to a rejection when the line names only conflicted files and a pathspec mode is the only reason it
///  is not allowed.
/// Why:
///   The person is told which mode to drop,
/// and that the same command without it is saved first and then allowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modeNote(modes: string[], operation: Operation): string;
/// ```
pub fn mode_note(modes: &[String], operation: crate::conflict_side_state::Operation) -> String {
    let named: String = backticked(modes.iter().map(String::as_str));
    return format!(
        " This {} is in progress and every path named is a conflicted file, so the same command without {named} would be saved first \
         and then allowed. {named} changes which files a pathspec names, so the line is rejected as it stands: run it again without \
         {named} (drop the option or unset the variable).",
        operation_word(operation)
    );
}

/// What:
///  The sentence added to a rejection when every token that is not a plain conflicted file is pathspec magic on a conflicted
///  file's name.
/// Why:
///   The person is told which magic to drop and which file to name,
/// and that the line is then saved first and then allowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function magicNote(magic: [string, string][], operation: Operation): string;
/// ```
pub fn magic_note(magic: &[(String, String)], operation: crate::conflict_side_state::Operation) -> String {
    let spellings: String = backticked(magic.iter().map(first_of));
    let names: String = backticked(magic.iter().map(second_of));
    return format!(
        " This {} is in progress and every path named is a conflicted file once the pathspec magic is left off ({spellings}). Pathspec \
         magic changes which files a pathspec names, so name each conflicted file itself ({names}) and the line is saved first and \
         then allowed.",
        operation_word(operation)
    );
}

/// Both notes,
/// and the modes a command runs under.
#[cfg(test)]
#[path = "conflict_side_modes_tests.rs"]
mod tests;
