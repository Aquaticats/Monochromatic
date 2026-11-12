//! What:
//!  The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy:
//! the one directory it names,
//!  and how many times the person forced it.
//! Why:
//!  Only the plain form (one directory,
//! the `-f`/`--force` option asked once or twice,
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
//!  A second `-f` is what a person types when the first removal failed,
//!  which is the removal this wrapper exists to finish:
//!  Git unlocks a worktree only when the force is given twice
//!  (`if (force > 1) unlock_worktree(...)`),
//!  so a form the wrapper did not read was the whole removal handed to Git's own walk,
//!  and a folder a writer defeated stayed behind with nothing pointing at it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseRemovePlan(['worktree', 'remove', '../topic'], 0, cwd, commonDir); // undefined: Git removes it
//! ```

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  How many times the person asked for the removal to be forced:
///  nothing,
///  once,
///  or twice.
/// Why:
///   Git reads two things from the count (`remove_worktree`):
///  any force at all skips its cleanliness proof,
///  and a force given twice is also the permission to unlock the worktree.
///  The two are one number rather than two flags,
///  so a form the wrapper cannot serve is a form it does not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForceAsked = 'no' | 'once' | 'twice';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForceAsked {
    /// Neither `-f` nor `--force` was given,
    /// so Git proves the worktree clean before it goes.
    No,
    /// The force was given once,
    /// so a worktree holding changes is removed.
    Once,
    /// The force was given twice,
    /// which is also the permission to unlock a worktree.
    ///  The unlock itself stays Git's step (`if (force > 1) unlock_worktree(...)`),
    ///  so such a worktree is still Git's own removal,
    ///  and the leftovers its walk leaves are finished from the record this removal writes before Git runs.
    Twice,
}

impl ForceAsked {
    /// What:
    ///  Whether the person forced the removal at all,
    ///  which is what skips Git's cleanliness proof.
    /// Why:
    ///   The proof is the slow step,
    ///  and the destroy answers for exactly the files Git's own forced removal deletes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// asked = (force) => force !== 'no';
    /// ```
    pub fn asked(self) -> bool {
        return !matches!(self, ForceAsked::No);
    }
}

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
/// type RemovePlan = { target: string; operand: string; force: ForceAsked };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovePlan {
    /// The real path of the worktree directory.
    pub target: PathBuf,
    /// The operand as the person typed it,
    /// which Git also matches against the last components of every registered path.
    pub operand: OsString,
    /// How many times the person forced the removal,
    /// which is what makes Git remove a worktree that holds changes
    ///  and,
    /// asked twice,
    /// one that is locked.
    pub force: ForceAsked,
}

/// What:
///  How many forces one token asks for,
///  and nothing when the token is not a force this command knows.
/// Why:
///   Git reads `-f` in any cluster of short flags and `--force` as any unambiguous abbreviation of it,
///  so the spellings a person types for this command are the counts:
///  `-f`,
/// `-ff` and a cluster holding two `f`s,
///  and `--f`,
/// `--for` and the rest of the long name.
///  A token with an `=` carries a value `--force` does not take,
///  which Git refuses,
///  so it is no force here either.
///  A token holding any other letter asks for an option whose effect this plan would not know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forcesAsked = (token) => token.startsWith('--') ? (/^--f(or(ce)?)?$/.test(token.split('=')[0]) ? 1 : undefined) : (/^-f+$/.test(token) ? token.length - 1 : undefined);
/// ```
fn forces_asked(token: &OsString) -> Option<usize> {
    let bytes: &[u8] = token.as_encoded_bytes();
    if let Some(named) = bytes.strip_prefix(b"--") {
        if named.contains(&b'=') {
            return None;
        }
        // `--force` abbreviates to any prefix from `--f` up: it is the only long option of this command.
        let full: &[u8] = b"force";
        return (!named.is_empty() && full.starts_with(named)).then_some(1);
    }
    let letters: &[u8] = bytes.strip_prefix(b"-")?;
    if letters.is_empty() || !letters.iter().all(is_an_f) {
        return None;
    }
    return Some(letters.len());
}

/// Whether one letter of a short token is the `f` the force is spelled with.
/// Why:
///   A cluster of only `f`s is `-f` asked that many times,
///  and any other letter is an option whose effect this plan would not know,
///  so the loop that reads the cluster names this one question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnF = (letter) => letter === 'f';
/// ```
fn is_an_f(letter: &u8) -> bool {
    return *letter == b'f';
}

/// What:
///  Plan one invocation of `git worktree remove` that names exactly one directory and asks for the force no more than
///  twice.
/// Why:
///   Every other option changes what Git checks,
/// so those forms stay with Git.
/// The forced form is served because Git's own
///  `--force` removal deletes exactly the worktree the destroy deletes,
///  a forced removal is the form the wrapper's own reports tell a person to run,
///  and a second force is what a person types when the first removal failed:
///  a form the wrapper did not read was the whole removal handed to Git,
///  whose walk a writer defeats.
///  A third force is Git's own answer,
///  because Git reads it as no more than "unlock",
///  which this removal does not do.
/// `common_dir` is the invoking repository's common Git directory,
/// where Git's list of worktree paths lives;
/// without it only the path form is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovePlan(args: string[], subcommandIndex: number, start: string, commonDir?: string): RemovePlan | undefined;
/// ```
pub fn parse_remove_plan(arguments: &[OsString], subcommand_index: usize, start: &Path, common_dir: Option<&Path>) -> Option<RemovePlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"remove" {
        return None;
    }
    let rest: &[OsString] = &arguments[subcommand_index + 2..];
    let mut forces: usize = 0;
    let mut operand: Option<&OsString> = None;
    let mut options_end: bool = false;
    for token in rest {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !options_end && bytes == b"--" {
            options_end = true;
            continue;
        }
        if !options_end && bytes.starts_with(b"-") {
            let asked: usize = forces_asked(token)?;
            forces += asked;
            if forces > 2 {
                return None;
            }
            continue;
        }
        // A second word is a form this plan does not read: Git's removal takes one directory.
        if operand.is_some() {
            return None;
        }
        operand = Some(token);
    }
    let named: &OsString = operand?;
    let force: ForceAsked = match forces {
        0 => ForceAsked::No,
        1 => ForceAsked::Once,
        _ => ForceAsked::Twice,
    };
    // Joined to the start directory an empty operand would name that directory, while Git answers that '' is not a working tree.
    if named.is_empty() {
        return None;
    }
    // Git matches the typed name against the registered paths first, and a name that matches one of them names that worktree.
    if let Some(common) = common_dir
        && let Some(target) = crate::worktree_remove_registry::path_git_picks_for_operand(common, named.as_os_str())
    {
        return Some(RemovePlan { target, operand: named.clone(), force });
    }
    let typed: PathBuf = start.join(named);
    let found: PathBuf = std::fs::canonicalize(typed).ok()?;
    return found.is_dir().then_some(RemovePlan { target: found, operand: named.clone(), force });
}

/// Removal plan controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_plan_tests.rs"]
mod tests;
