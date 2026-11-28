//! What:
//!  The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy:
//! the one directory it names,
//!  and whether the person forced it;
//!  and the directory one such command names in every other form,
//!  which is what the clone route's refusal reads.
//! Why:
//!  Only the plain form (one directory,
//! one optional `-f` or `--force`,
//! a non-empty name) is served;
//! every other option changes what Git checks,
//! so those stay with Git.
//!  A slot a whole clone is bound to is refused whatever the command's options,
//!  though,
//!  so the refusal reads the operand of every spelling Git accepts
//!  (`parse_removal_target`),
//!  and both readings take their operands from `worktree_operands`.
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

use crate::worktree_command_facts::RemovePlan;
use crate::worktree_operands::operands_after_options;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// Whether a token is one of the two spellings of Git's force option for this command.
/// Why:
///   Every other spelling (`-ff`,
/// `--forc`,
/// a cluster)
/// counts the force twice or takes an abbreviation the wrapper does not
///  read,
/// so each of those is left to Git rather than read as one force.
fn is_force(token: &OsString) -> bool {
    return matches!(token.as_encoded_bytes(), b"-f" | b"--force");
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
pub fn parse_remove_plan(
    arguments: &[OsString],
    subcommand_index: usize,
    start: &Path,
    common_dir: Option<&Path>,
) -> Option<RemovePlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree"
        || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"remove"
    {
        return None;
    }
    let rest: &[OsString] = &arguments[subcommand_index + 2..];
    let (force, operand): (bool, &OsString) = match rest {
        [only] if !only.as_encoded_bytes().starts_with(b"-") => (false, only),
        [separator, only] if separator.as_encoded_bytes() == b"--" => (false, only),
        [flag, only] if is_force(flag) && !only.as_encoded_bytes().starts_with(b"-") => {
            (true, only)
        }
        [flag, separator, only] if is_force(flag) && separator.as_encoded_bytes() == b"--" => {
            (true, only)
        }
        _ => return None,
    };
    // Joined to the start directory an empty operand would name that directory, while Git answers that '' is not a working tree.
    if operand.is_empty() {
        return None;
    }
    // Git matches the typed name against the registered paths first, and a name that matches one of them names that worktree.
    if let Some(common) = common_dir
        && let Some(target) =
            crate::worktree_remove_registry::path_git_picks_for_operand(common, operand.as_os_str())
    {
        return Some(RemovePlan {
            target,
            operand: operand.clone(),
            force,
        });
    }
    let target: PathBuf = std::fs::canonicalize(start.join(operand)).ok()?;
    return target.is_dir().then_some(RemovePlan {
        target,
        operand: operand.clone(),
        force,
    });
}

/// What:
///  The worktree one `git worktree remove` names,
///  whatever options the command carries,
///  resolved to a directory that exists.
/// Why:
///  A slot a whole managed clone is bound to is refused for every spelling of
///  the command,
///  because `SPEC.md`'s sentence exempts none and Git's own hint steers a
///  person to the double force
///  (`SPEC.md`,
///  "Removal of a ZFS worktree clone";
///  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "refuse instead of falling through to Git's recursive removal").
///  The plan above serves only the plain form,
///  so the refusal needs the operand without the force decision,
///  and it reads the same operands the move refusal does.
///  A command naming two operands names no one worktree,
///  and Git answers it with its own usage;
///  the refusal leaves that form to Git.
///  The folder a registration names may be gone while the registration and the
///  binding record stand,
///  so the reading is Git's own list first and a directory that stands second:
///  a slot a person moved aside or deleted is still the slot the removal would
///  orphan a clone at
///  (`worktree_remove_registry::the_worktree_path_git_would_remove`).
///  The start directory is optional because Git's own match against the
///  registered paths needs none:
///  a spelling whose global options the caller does not serve can still name a
///  worktree by a name the registry knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovalTarget(args: string[], subcommandIndex: number, start?: string, commonDir?: string): string | undefined;
/// ```
pub fn parse_removal_target(
    arguments: &[OsString],
    subcommand_index: usize,
    start: Option<&Path>,
    common_dir: Option<&Path>,
) -> Option<PathBuf> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree"
        || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"remove"
    {
        return None;
    }
    let operands: Vec<&OsString> = operands_after_options(&arguments[subcommand_index + 2..]);
    // Git's removal takes exactly one worktree, and `remove_worktree` answers every other count with its usage.
    let [typed] = operands.as_slice() else {
        return None;
    };
    let operand: &OsString = typed;
    if operand.is_empty() {
        return None;
    }
    // What: The worktree Git would remove for this name, in every spelling that names one.
    // Why: Git's own list holds a registration's path after the slot's folder is moved aside or deleted, and Git's
    //      removal would then delete that registration while the binding record and the clone it names stand, so the
    //      record's refusal has to be reached for that spelling too (`SPEC.md`, "Removal of a ZFS worktree clone").
    if let Some(common) = common_dir
        && let Some(target) = crate::worktree_remove_registry::the_worktree_path_git_would_remove(
            common,
            start,
            operand.as_os_str(),
        )
    {
        return Some(target);
    }
    // Without a list to read, only a directory that stands names a worktree, which is the reading this route always had.
    let target: PathBuf = std::fs::canonicalize(start?.join(operand)).ok()?;
    return target.is_dir().then_some(target);
}

/// What:
///  Which of Git's own readings one long option carries.
/// Why:
///  A countup option has two readings and not one:
///  the set form adds one,
///  and the negated form sets the count back to zero
///  (`parse-options.c:179` to `:196` at Git `v2.56.1`),
///  so a reader that knew only "counts" would count `--force --no-force`
///  as one force where Git counts none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ALongForce = 'not-this-option' | 'one' | 'reset';
/// ```
enum ALongForce {
    /// The word is no spelling of this command's force.
    NotThisOption,
    /// The word adds one force.
    One,
    /// The word sets the count back to zero.
    Reset,
}

/// Whether a word is one Git accepts as a prefix of `force`.
/// Why:
///  Git's long-option reader takes any prefix of a long name that no other
///  option of the command also begins
///  (`parse-options.c:563` to `:566` at Git `v2.56.1`),
///  and `worktree remove` declares one long option,
///  `force`,
///  so `--f`,
/// `--fo`,
/// `--for` and `--forc` are the same reading as `--force`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAForcePrefix = (word: string) => word.length >= 1 && 'force'.startsWith(word);
/// ```
fn is_a_prefix_of_force(word: &[u8]) -> bool {
    return !word.is_empty() && b"force".starts_with(word);
}

/// What:
///  Which force `--<word>` carries.
/// Why:
///  The two readings differ in what they do to the count,
///  and Git's parser strips one `no-` before it compares the word
///  (`parse-options.c:530` to `:537` at Git `v2.56.1`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const longForce = (word: string) => isAForcePrefix(word) ? 'one' : isAForcePrefix(word.replace(/^no-/, '')) ? 'reset' : 'not-this-option';
/// ```
fn long_force_of(word: &[u8]) -> ALongForce {
    if is_a_prefix_of_force(word) {
        return ALongForce::One;
    }
    let Some(negated) = word.strip_prefix(b"no-") else {
        return ALongForce::NotThisOption;
    };
    return if is_a_prefix_of_force(negated) {
        ALongForce::Reset
    } else {
        ALongForce::NotThisOption
    };
}

/// What:
///  The number of forces one `git worktree remove` carried,
///  in every spelling Git accepts.
/// Why:
///  The project decided on 2026-11-28 that this removal counts force the way Git
///  does,
///  so the reading is Git's own and not the wrapper's invention:
///  a short cluster counts one force per `f` (`parse-options.c:426` to `:440`),
///  `--force` and each of its prefixes count one,
///  `--no-force` and each of its prefixes set the count back to zero,
///  and a word after `--` is no option at all
///  (`parse-options.c:120` to `:196` at Git `v2.56.1`).
///  Git permutes its options around the operand,
///  so a force after the path counts as one before it,
///  and every option of the command is read here and not only those before the
///  worktree's name.
///  A word that is an option and not one of this command's force spellings is a
///  spelling Git answers with its usage and no removal at all,
///  and this wrapper answers that spelling before Git would,
///  so the whole count is zero:
///  no command gains force from a word Git refused,
///  and a refused spelling never passes a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const force = countOfTheForce(['-f', '-f', 'one']); // 2
/// ```
pub fn count_of_the_force(tokens: &[OsString]) -> usize {
    let mut count: usize = 0;
    let mut options_end: bool = false;
    for token in tokens {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !options_end && bytes == b"--" {
            options_end = true;
            continue;
        }
        // A word of one `-` is an operand to Git, and everything after `--` is an operand too.
        if options_end || bytes.len() < 2 || !bytes.starts_with(b"-") {
            continue;
        }
        let Some(long) = bytes.strip_prefix(b"--") else {
            let cluster: &[u8] = &bytes[1..];
            if !cluster.iter().all(|letter| return *letter == b'f') {
                return 0;
            }
            count = count.saturating_add(cluster.len());
            continue;
        };
        match long_force_of(long) {
            ALongForce::One => count = count.saturating_add(1),
            ALongForce::Reset => count = 0,
            ALongForce::NotThisOption => return 0,
        }
    }
    return count;
}

/// Removal plan controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_plan_tests.rs"]
mod tests;
