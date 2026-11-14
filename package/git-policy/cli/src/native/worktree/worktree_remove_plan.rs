//! What:
//!  The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy:
//! the one directory it names,
//!  and whether the person forced it.
//! Why:
//!  Only the plain form (one directory,
//! one optional `-f` or `--force`,
//! a non-empty name) is served;
//! every other option changes what Git checks,
//! so those stay with Git.
//! The name is read the way Git reads it:
//!  first as the last components of the registered worktree paths,
//! then as a path.
//! The force is part of the plan because it decides whether the removal runs Git's
//!  cleanliness proof at all (`remove_worktree` in Git's `builtin/worktree.c` calls
//! `check_clean_worktree` only when
//!  `force` is zero),
//! and a forced removal is the one an agent runs on a worktree it has worked in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseRemovePlan(['worktree', 'remove', '../topic'], 0, cwd, commonDir); // undefined: Git removes it
//! ```

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The worktree a `git worktree remove <path>` names,
/// resolved to a directory that exists.
/// Why:
///   The removal needs the worktree's real path;
/// Git also accepts a unique last component,
/// which is
///  found among the registered paths before the name is read as a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovePlan = { target: string; operand: string; force: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovePlan {
    /// The real path of the worktree directory.
    pub target: PathBuf,
    /// The operand as the person typed it,
    /// which Git also matches against the last components of every registered path.
    pub operand: OsString,
    /// Whether the person passed `-f` or `--force`,
    /// which is what makes Git remove a worktree that holds changes.
    pub force: bool,
}

/// What:
///  What a switch word says about Git's force for this command:
///  whether it forces the removal or turns the force off,
///  or nothing when the word is no spelling of that option.
/// Why:
///   Git's `OPT__FORCE` reads `-f`,
///  a cluster of `f` (`-ff` sets it twice),
///  `--force`,
///  `--no-force`,
///  and any prefix of the two long names that names only one option
///  (`--forc`,
///  `--no-for`),
///  so the guard reads the same set Git reads.
///  A word this does not know is not read as a force either way:
///  the caller refuses rather than guessing whether the person asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forceReading('-ff') // true; forceReading('--no-force') // false; forceReading('--quiet') // undefined
/// ```
fn force_reading(token: &OsString) -> Option<bool> {
    let word: &[u8] = token.as_encoded_bytes();
    let short: &[u8] = word.strip_prefix(b"-")?;
    // A cluster of nothing but `f`, as `-f` and `-ff` are.
    if !short.is_empty() && short.iter().all(|byte: &u8| return *byte == b'f') {
        return Some(true);
    }
    let long: &[u8] = word.strip_prefix(b"--")?;
    // Git's parse_options accepts any prefix of a long option that names only one.
    let of = |name: &[u8]| -> bool { return !long.is_empty() && long.len() <= name.len() && name.starts_with(long) };
    if of(b"force") {
        return Some(true);
    }
    if of(b"no-force") {
        return Some(false);
    }
    return None;
}

/// Whether a word asks Git to print this command's usage rather than remove a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asksForUsage('-h') // true
/// ```
fn asks_for_usage(token: &OsString) -> bool {
    return matches!(token.as_encoded_bytes(), b"-h" | b"--help");
}

/// What:
///  How a `git worktree remove` reads:
///  the one directory it names and whether the person forced it,
///  a command that removes nothing here because Git answers it with its usage,
///  or a form this program cannot read and must therefore refuse.
/// Why:
///   Two callers read the same words:
///  the plan the ladder serves,
///  which serves only the forms it knows,
///  and the guard a typed removal meets before any route,
///  which must not stand down on a form it cannot read:
///  an option this program does not know could be one Git accepts unforced,
///  and a removal handed to Git on that guess destroys the worktree unasked
///  (a review of `9e6c8619a9`,
///  finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removalShape(args, index) // { kind: 'served'; operand; force } | { kind: 'not-a-removal' } | { kind: 'unprovable'; reason }
/// ```
pub(crate) enum RemovalShape {
    /// The plain form:
    /// one directory,
    /// and whether the force is on.
    Served {
        /// The directory as the person typed it.
        operand: OsString,
        /// Whether Git's force is on for this removal.
        force: bool,
    },
    /// A form Git answers itself:
    /// no removal is made, so nothing is asked about a worktree.
    NotARemoval,
    /// A form this program does not read,
    /// with the words saying which word it stopped at.
    Unprovable(String),
}

/// What:
///  Read the words of a `git worktree remove` as Git reads them.
/// Why:
///  Git's parser takes the force switches before the one operand,
///  an optional `--`,
///  and then exactly one argument;
///  anything else makes it print its usage and remove nothing,
///  while a word this program does not know could still be an option a later Git accepts unforced.
///  Only the two answers Git itself gives,
///  a removal and its usage,
///  are read as final here;
///  the rest is the refusal the guard words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removalShape(args, index) // served | not-a-removal | unprovable
/// ```
pub(crate) fn removal_shape(arguments: &[OsString], subcommand_index: usize) -> RemovalShape {
    let word = |at: usize| -> Option<&[u8]> { return arguments.get(at).map(|one| return one.as_encoded_bytes()) };
    if word(subcommand_index) != Some(b"worktree") || word(subcommand_index + 1) != Some(b"remove") {
        return RemovalShape::NotARemoval;
    }
    let mut force: bool = false;
    let mut rest: &[OsString] = &arguments[subcommand_index + 2..];
    // The switches before the operand, as Git's parser takes them.
    while let Some(first) = rest.first() {
        let bytes: &[u8] = first.as_encoded_bytes();
        if asks_for_usage(first) {
            return RemovalShape::NotARemoval;
        }
        if bytes == b"--" {
            rest = &rest[1..];
            break;
        }
        match force_reading(first) {
            Some(on) => {
                force = on;
                rest = &rest[1..];
            }
            None => {
                if !bytes.starts_with(b"-") {
                    break;
                }
                // A wrapper control written where this command does not read it is the one option a person meets and
                // can fix at once, so the words say where it goes; any other option is named as it stands.
                return RemovalShape::Unprovable(match crate::wrapper_controls::control_meaning(bytes).is_some() {
                    true => String::from(
                        "the wrapper's opt-out stands where `git worktree remove` does not read it, and it must stand directly after `worktree`, as it does in the command this refusal prints",
                    ),
                    false => format!("it carries the option {}", String::from_utf8_lossy(bytes)),
                });
            }
        }
    }
    // One argument, which Git may still take after a `--`.
    let [operand] = rest else { return RemovalShape::NotARemoval };
    // Joined to the start directory an empty operand would name that directory, while Git answers that '' is not a working tree.
    if operand.is_empty() {
        return RemovalShape::NotARemoval;
    }
    return RemovalShape::Served { operand: operand.clone(), force };
}

/// What:
///  The one directory a `git worktree remove` names and whether the person forced it,
///  for the forms the ladder serves,
///  or nothing for any other form.
/// Why:
///   The ladder serves only the plain form it can prove,
///  so every other form is Git's;
///  the guard a typed removal meets reads `removal_shape` instead,
///  which is the same reading with the forms it must refuse rather than leave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// operandAndForce(args, index) // { operand, force } | undefined
/// ```
pub(crate) fn operand_and_force(arguments: &[OsString], subcommand_index: usize) -> Option<(OsString, bool)> {
    return match removal_shape(arguments, subcommand_index) {
        RemovalShape::Served { operand, force } => Some((operand, force)),
        RemovalShape::NotARemoval | RemovalShape::Unprovable(_) => None,
    };
}

/// What:
///  Plan one invocation of `git worktree remove` that names exactly one directory and at most one `-f` or `--force`.
/// Why:
///   Every other option changes what Git checks,
/// so those forms stay with Git.
/// The forced form is served because Git's own
///  `--force` removal deletes exactly the worktree the destroy deletes,
/// and because a forced removal is the form
///  the wrapper's own reports tell a person to run.
/// `common_dir` is the invoking repository's common Git directory,
/// where Git's list of worktree paths lives;
/// without it only the path form is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovePlan(args: string[], subcommandIndex: number, start: string, commonDir?: string): RemovePlan | undefined;
/// ```
pub fn parse_remove_plan(arguments: &[OsString], subcommand_index: usize, start: &Path, common_dir: Option<&Path>) -> Option<RemovePlan> {
    let (operand, force): (OsString, bool) = operand_and_force(arguments, subcommand_index)?;
    // Git matches the typed name against the registered paths first, and a name that matches one of them names that worktree.
    if let Some(common) = common_dir
        && let Some(target) = crate::worktree_remove_registry::path_git_picks_for_operand(common, operand.as_os_str())
    {
        return Some(RemovePlan { target, operand, force });
    }
    let target: PathBuf = std::fs::canonicalize(start.join(operand.as_os_str())).ok()?;
    return target.is_dir().then_some(RemovePlan { target, operand, force });
}

/// Removal plan controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_plan_tests.rs"]
mod tests;
