//! What:
//!  The pathspec modes and the pathspec magic that make a conflict-side line unreadable to the guard, and the sentences that
//!  tell a person the same line without them would be saved and then allowed.
//! Why:
//!  A mode (`GIT_ICASE_PATHSPECS`, `--glob-pathspecs`, ...) changes which files a pathspec names, and magic (`:(icase)`, `:!`,
//!  `:/`) does the same for one token, so the guard cannot say which files Git would replace and keeps the rejection. The
//!  rejection alone does not say what to change; when the line would be allowed without the mode or magic, the message does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! modesInEffect(['--icase-pathspecs'], overlay); // ['--icase-pathspecs']
//! modeNote(['--icase-pathspecs'], 'merge'); // 'This merge is in progress ... the same command without `--icase-pathspecs` ...'
//! ```

use std::ffi::OsString;

/// The global options that set a pathspec mode, the ones `git.c` accepts before the subcommand.
pub const MODE_OPTIONS: [&str; 5] = ["--icase-pathspecs", "--glob-pathspecs", "--noglob-pathspecs", "--literal-pathspecs", "--no-literal-pathspecs"];

/// The variables that set a pathspec mode.
pub const MODE_VARIABLES: [&str; 4] = ["GIT_ICASE_PATHSPECS", "GIT_GLOB_PATHSPECS", "GIT_NOGLOB_PATHSPECS", "GIT_LITERAL_PATHSPECS"];

/// What:
///  The spellings of the pathspec modes a command runs under: each global option present, each variable present in the
///  process environment or the child overlay.
/// Why:
///   The note names exactly what the person set, so the way out is to remove exactly that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modesInEffect(globalPrefix: string[], overlay: [string, string][]): string[];
/// ```
pub fn modes_in_effect(global_prefix: &[OsString], overlay: &[(OsString, OsString)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for option in MODE_OPTIONS {
        if global_prefix.iter().any(|each| { return each.as_encoded_bytes() == option.as_bytes(); }) {
            found.push(String::from(option));
        }
    }
    for variable in MODE_VARIABLES {
        let set: bool = std::env::var_os(variable).is_some() || overlay.iter().any(|(name, _)| { return name.as_encoded_bytes() == variable.as_bytes(); });
        if set {
            found.push(String::from(variable));
        }
    }
    return found;
}

/// The word for an operation in a sentence.
fn operation_word(operation: super::conflict_side_state::Operation) -> &'static str {
    use super::conflict_side_state::Operation;
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
///   The person is told which mode to drop, and that the same command without it is saved first and then allowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modeNote(modes: string[], operation: Operation): string;
/// ```
pub fn mode_note(modes: &[String], operation: super::conflict_side_state::Operation) -> String {
    let named: String = modes.iter().map(|each| { return format!("`{each}`"); }).collect::<Vec<String>>().join(", ");
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
///   The person is told which magic to drop and which file to name, and that the line is then saved first and then allowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function magicNote(magic: [string, string][], operation: Operation): string;
/// ```
pub fn magic_note(magic: &[(String, String)], operation: super::conflict_side_state::Operation) -> String {
    let spellings: String = magic.iter().map(|(spelling, _)| { return format!("`{spelling}`"); }).collect::<Vec<String>>().join(", ");
    let names: String = magic.iter().map(|(_, name)| { return format!("`{name}`"); }).collect::<Vec<String>>().join(", ");
    return format!(
        " This {} is in progress and every path named is a conflicted file once the pathspec magic is left off ({spellings}). Pathspec \
         magic changes which files a pathspec names, so name each conflicted file itself ({names}) and the line is saved first and \
         then allowed.",
        operation_word(operation)
    );
}

/// Both notes, and the modes a command runs under.
#[cfg(test)]
#[path = "conflict_side_modes_tests.rs"]
mod tests;
