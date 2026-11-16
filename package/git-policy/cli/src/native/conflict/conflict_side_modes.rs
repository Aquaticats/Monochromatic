//! What:
//!  The pathspec modes and the pathspec magic that make a pathspec name a set of files,
//! and the spellings of them a command runs under.
//! Why:
//!  A mode (`GIT_ICASE_PATHSPECS`,
//! `--glob-pathspecs`,
//! ...) changes which files a pathspec names,
//! and magic (`:(icase)`,
//! `:!`,
//!  `:/`) does the same for one token,
//! so a reader cannot say which file a token names;
//! the callers that judge a path keep their refusal and name no path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! modesInEffect(['--icase-pathspecs'], overlay); // ['--icase-pathspecs']
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
///  Whether a pathspec mode reaches a start that removes the variables in `remove`.
/// Why:
///   The in-process readers judge pathspec magic themselves and read no mode at all,
///  so their answer is Git's only while
///  no mode is in effect;
///  a name in `remove` is gone from the child's environment,
///  which outranks the overlay
///  that may set it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aModeReachesTheStart(overlay, remove) // true when the child runs under a mode
/// ```
pub fn a_mode_reaches_the_start(overlay: &[(OsString, OsString)], remove: &[&str]) -> bool {
    for variable in MODE_VARIABLES {
        if remove.contains(&variable) {
            continue;
        }
        if std::env::var_os(variable).is_some() {
            return true;
        }
        for (name, _) in overlay {
            if name.as_encoded_bytes() == variable.as_bytes() {
                return true;
            }
        }
    }
    return false;
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

/// The modes a command runs under.
#[cfg(test)]
#[path = "conflict_side_modes_tests.rs"]
mod tests;
