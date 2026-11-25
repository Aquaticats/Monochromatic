//! What:
//!  The Git registration one managed slot's record names:
//!  the complete registration Git lists the worktree through,
//!  the registration a prune or another program removed,
//!  the one that stands but names another path,
//!  the half-made one no `git worktree list` shows,
//!  and the slot whose record names none yet.
//! Why:
//!  The design asks the report to name "damaged or pruned Git registrations"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds"),
//!  and step 16's settle refuses on exactly these states:
//!  a registration that is gone cannot be rebuilt by `git worktree repair`,
//!  and one that names another path is repaired rather than replaced.
//!  The record holds the registration's path,
//!  so the two files that decide the state are the registration's own
//!  `gitdir` and the slot's `.git`;
//!  both are handed in,
//!  so a control writes the state it judges and reads no repository.
//!  What stands at a complete registration's `locked` file is step 13's
//!  classification and not this check's,
//!  because one lock must never have two readings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkRegistration(facts); // pass | finding('registration') | unknown
//! ```

use super::check_outcome::{CheckOutcome, finding, pass, unknown};
use super::handler_checks::PathState;
use std::path::{Path, PathBuf};

/// What:
///  One slot's registration,
///  the path its record names and what stands there.
/// Why:
///  The recorded path is the only source of which registration belongs to the
///  slot,
///  and the two files below it are what tell a complete registration from a
///  damaged or half-made one,
///  so both travel in one value a control can write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegistrationFacts = { slot; recorded?: string; atThePath: PathState; gitdir?: string; slotGit: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistrationFacts {
    /// The slot's name,
    /// which is the record's key.
    pub slot: String,
    /// The registration's path as the record spells it,
    /// and nothing while the record names none.
    pub recorded: Option<PathBuf>,
    /// What the filesystem answered at that path.
    pub at_the_path: PathState,
    /// The path the registration's `gitdir` file names,
    /// when it holds one.
    pub gitdir: Option<PathBuf>,
    /// The `.git` file a complete registration of this slot is named by.
    pub slot_git: PathBuf,
}

/// What:
///  Whether the slot's Git registration is the one a complete add leaves.
/// Why:
///  A missing registration is the state that makes `git worktree` forget the
///  worktree and that nothing in the wrapper rebuilds,
///  a registration naming another path is the one `git worktree repair` fixes,
///  and a directory with no `gitdir` is what a kill inside Git's own add
///  leaves.
///  Each names its own way on,
///  and a path this account may not look at stays unknown rather than passing
///  for a complete registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkRegistration(facts); // pass | finding('registration') | unknown
/// ```
pub fn check_registration(facts: &RegistrationFacts) -> CheckOutcome {
    let Some(recorded) = &facts.recorded else {
        return pass(format!(
            "the record filed under `{}` names no Git registration yet, which is where an add that has not reached its Git step stands",
            facts.slot
        ));
    };
    let path: String = recorded.display().to_string();
    match &facts.at_the_path {
        PathState::Absent => {
            return finding(
                "registration",
                format!(
                    "the record filed under `{}` names the Git registration {path}, and nothing stands there, so `git worktree` no longer lists this worktree: Git's prune removes the entry of a slot that shows no .git once the entry has no index, and another program may have removed it. The clone's files are on its dataset and nothing in this report changed them. `git worktree repair` cannot rebuild a registration that is gone, this release has no command that does, and the clone and its record stay until one does; do not run Git in the folder.",
                    facts.slot
                ),
            );
        }
        PathState::Unreadable { why } => {
            return unknown(format!("the Git registration {path} that the record of `{}` names could not be looked at ({why}), so whether it stands is unproved", facts.slot));
        }
        PathState::File { .. } => {
            return finding("registration", format!("the record filed under `{}` names the Git registration {path}, and a file stands there rather than the registration directory Git writes", facts.slot));
        }
        PathState::WrongKind { found } => {
            return finding("registration", format!("the record filed under `{}` names the Git registration {path}, and {found} stands there rather than the registration directory Git writes", facts.slot));
        }
        PathState::Directory { .. } => {}
    }
    let Some(gitdir) = &facts.gitdir else {
        return finding(
            "registration",
            format!(
                "the Git registration {path} that the record of `{}` names stands with no `gitdir` file, which is what a kill inside Git's own `worktree add` leaves: `git worktree list` does not show it, and the add's own resume clears it with the record's recipe before it runs Git again",
                facts.slot
            ),
        );
    };
    if same_path(gitdir, &facts.slot_git) {
        return pass(format!("the Git registration {path} stands and its `gitdir` names the slot's own .git, so `git worktree` lists this worktree"));
    }
    return finding(
        "registration",
        format!(
            "the Git registration {path} that the record of `{}` names points at {} rather than the slot's own .git ({}), so this worktree is registered under a path no reader of this record expects; `git worktree repair {}` is the command that rewrites the link, and this report changes nothing",
            facts.slot,
            gitdir.display(),
            facts.slot_git.display(),
            facts.slot
        ),
    );
}

/// What:
///  Whether two paths name one file,
///  as far as a reader can tell without following links.
/// Why:
///  The registration's `gitdir` and the slot's `.git` are written by Git from
///  the same destination,
///  so a difference in one trailing separator or a repeated separator is not
///  the damage this check reports,
///  and anything deeper needs a filesystem lookup a report of readings does not
///  make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = samePath('/repo/.claude//worktrees/topic/.git', '/repo/.claude/worktrees/topic/.git'); // true
/// ```
fn same_path(left: &Path, right: &Path) -> bool {
    let normalized = |path: &Path| -> PathBuf {
        let mut kept: Vec<std::ffi::OsString> = Vec::new();
        for component in path.components() {
            let text: std::ffi::OsString = component.as_os_str().to_os_string();
            if text.as_os_str() == std::ffi::OsStr::new(".") {
                continue;
            }
            kept.push(text);
        }
        return kept.iter().collect();
    };
    return normalized(left) == normalized(right);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_checks_tests.rs"]
mod tests;
