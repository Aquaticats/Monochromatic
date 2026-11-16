//! What:
//!  The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy:
//! the one directory it names,
//!  and whether the person forced it.
//! Why:
//!  Only the plain form (one directory,
//! the force switches Git's own table reads,
//! wherever in the words they stand,
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
    // A word that stops at the `no-` with nothing after it names no option of Git's table at all: `--no-` is Git's usage
    // answer, and reading it as a force turned off would judge a removal Git never makes (measured by the walk in
    // `worktree_remove_plan_tests.rs`, which holds every spelling against real Git).
    if long == b"no-" {
        return None;
    }
    if of(b"no-force") {
        return Some(false);
    }
    return None;
}

/// What:
///  Whether a word asks Git to print this command's usage,
///  which is an answer that removes nothing.
/// Why:
///   `-h` and `--help` are Git's own, at every length of the long name that still names it alone,
///  and a cluster of this command's own short switches that holds the `h` reaches it too (`-fh`).
///  A word that Git answers with its usage must not be read as a removal,
///  or the guard would judge a worktree the command was never going to touch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asksForUsage('-h') // true; asksForUsage('--hel') // true; asksForUsage('--quiet') // false
/// ```
fn asks_for_usage(token: &OsString) -> bool {
    const NAMED: &str = "help";
    // The short switches of this command, in the table Git declares for it: `f` for the force and `h` for the usage.
    const SHORT: &[u8] = b"fh";
    let word: &[u8] = token.as_encoded_bytes();
    if let Some(cluster) = word.strip_prefix(b"-") {
        let reads_it: bool = !cluster.is_empty() && cluster.iter().all(|byte: &u8| return SHORT.contains(byte)) && cluster.contains(&b'h');
        return reads_it;
    }
    let Some(long) = word.strip_prefix(b"--") else { return false };
    return !long.is_empty() && long.len() <= NAMED.len() && NAMED.as_bytes().starts_with(long);
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
        /// Where that directory stands among the words Git receives,
        /// so the caller can hand Git the worktree's own path in its place.
        operand_at: usize,
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
///   Git's `parse_options` reads the whole argument list:
///  an option stands anywhere before a `--`,
///  the `--` ends the options and is dropped,
///  and the words that are not options are the arguments the command then counts.
///  A command of `remove` with exactly one of those removes that worktree;
///  any other count,
///  or a word that asks for the usage,
///  makes Git print its usage and remove nothing,
///  while a word this program does not know could still be an option a later Git accepts unforced.
///  Only the two answers Git itself gives,
///  a removal and its usage,
///  are read as final here;
///  the rest is the refusal the guard words.
///  An option written after the operand was read as "nothing is removed" until a review of `d351b65746` showed
///  that Git accepts it and deletes the worktree with no look at what it holds.
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
    // Where each word that is not an option stands, in the order the person wrote them.
    let mut operands: Vec<usize> = Vec::new();
    let mut options_ended: bool = false;
    for (at, argument) in arguments.iter().enumerate().skip(subcommand_index + 2) {
        let bytes: &[u8] = argument.as_encoded_bytes();
        // The `--`, the single `-`, and every word that does not start with a `-` are arguments, in any position.
        if options_ended || !bytes.starts_with(b"-") || bytes == b"-" {
            operands.push(at);
            continue;
        }
        if bytes == b"--" {
            options_ended = true;
            continue;
        }
        if asks_for_usage(argument) {
            return RemovalShape::NotARemoval;
        }
        match force_reading(argument) {
            Some(on) => force = on,
            None => {
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
    // Exactly one argument, wherever among the options it stood.
    let [operand_at] = operands[..] else { return RemovalShape::NotARemoval };
    let operand: &OsString = &arguments[operand_at];
    // Joined to the start directory an empty operand would name that directory, while Git answers that '' is not a working tree.
    if operand.is_empty() {
        return RemovalShape::NotARemoval;
    }
    return RemovalShape::Served { operand: operand.clone(), operand_at, force };
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
        RemovalShape::Served { operand, force, .. } => Some((operand, force)),
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
/// `picked` is the worktree Git's own answer names, where the guard a typed removal meets read it:
/// the plan then names that worktree and resolves nothing of its own,
/// so the worktree the ladder destroys is the one that was judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovePlan(args: string[], subcommandIndex: number, start: string, commonDir?: string, picked?: string): RemovePlan | undefined;
/// ```
pub fn parse_remove_plan(arguments: &[OsString], subcommand_index: usize, start: &Path, common_dir: Option<&Path>, picked: Option<&Path>) -> Option<RemovePlan> {
    let (operand, force): (OsString, bool) = operand_and_force(arguments, subcommand_index)?;
    // The worktree Git's own answer names, where the guard read it: the ladder then destroys that worktree and no other,
    // which is what makes the worktree judged and the worktree removed one thing rather than two resolutions agreeing
    // (a review of `e615fb599b`, finding 1).
    if let Some(known) = picked {
        return Some(RemovePlan { target: known.to_path_buf(), operand, force });
    }
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
