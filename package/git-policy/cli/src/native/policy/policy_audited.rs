//! What:
//!  The two commands the wrapper records and never refuses:
//!  `git worktree remove`,
//!  and `git branch -d`/`-D` from the main worktree.
//! Why:
//!  The owner decided both run exactly as Git runs them,
//!  and both leave one audit line,
//!  so a loss is
//!       attributable and,
//!  for a branch,
//!  reversible:
//!  the line names the worktree and what it took,
//!  or the branch,
//!  its commit and
//!       whether it was merged.
//!  A refusal would only move the same loss to `rm -rf` or `git update-ref -d`,
//!  which is the
//!       owner's own reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // recordAudited({ facts, arguments: ['worktree', 'remove', '--force', '../topic'] })
//! ```

use crate::branch_delete_audit::deletion_facts;
use crate::command_branch_delete::branch_deletion;
use crate::command_worktree::{WorktreeRemoval, worktree_removal};
use crate::discard_audit::{AuditOutcome, record_audit_with_facts};
use crate::discard_audit_facts::AuditFact;
use crate::global_arguments::command_tokens;
use crate::repository_facts::RepositoryFacts;
use crate::repository_location::{RepositoryLocation, effective_directory};
use crate::rule_linked_worktree::AuditedCommand;
use crate::transaction_git::GitContext;
use crate::worktree_audit::{removal_facts, removal_target};
use crate::worktree_identity::worktree_root;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  Record one audited command and change nothing else.
///  `&[OsString]` borrows the arguments Git will receive.
/// Why:
///   The line is written before Git runs,
///  so a reader sees what was about to be taken even when the command is
///       killed half-way.
///  The target worktree's facts are read before the removal,
///  because afterwards nothing is there
///  to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordAudited({ facts, arguments, audited, location })
/// ```
pub fn record_audited(
    facts: &mut dyn RepositoryFacts,
    arguments: &[OsString],
    audited: AuditedCommand,
    location: &RepositoryLocation,
) {
    let context: GitContext = facts.transaction_context();
    match audited {
        AuditedCommand::WorktreeRemove => {
            let Some(removal) = removal_of(arguments) else {
                return;
            };
            // The path is relative to the directory Git works in, not to wherever the wrapper was started: `-C dir`
            //  and a subdirectory of the worktree both change that.
            let directory: PathBuf = match worktree_root(&location.identity) {
                Some(root) => effective_directory(root, location.prefix.as_slice()),
                None => std::env::current_dir().unwrap_or_default(),
            };
            let target: PathBuf = removal_target(&context, removal.target.as_os_str(), directory.as_path());
            let read: Vec<AuditFact> =
                removal_facts(&context, target.as_path(), removal.target.as_os_str(), removal.force);
            record_audit_with_facts(&location.identity, audited, AuditOutcome::Logged, arguments, None, read.as_slice());
        }
        AuditedCommand::BranchDelete => {
            let root: Option<PathBuf> = worktree_root(&location.identity).map(Path::to_path_buf);
            let Some(deletion) = branch_deletion(region_of(arguments)) else {
                return;
            };
            for name in &deletion.names {
                // A name with no reference in that namespace deletes nothing, and Git's own error is the answer: no line.
                let Some(read) = deletion_facts(&context, root.as_deref(), name.as_os_str(), deletion.remotes) else {
                    continue;
                };
                record_audit_with_facts(&location.identity, audited, AuditOutcome::Logged, arguments, None, read.as_slice());
            }
        }
    }
}

/// What:
///  The directory a removal names,
/// and whether `--force` was given.
/// Why:
///   The decision already read the same region,
/// so a region that is not a removal answers nothing and
///  writes no line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { target, force } = worktreeRemoval(regionOf(args)) ?? {};
/// ```
fn removal_of(arguments: &[OsString]) -> Option<WorktreeRemoval> {
    return worktree_removal(region_of(arguments));
}

/// What:
///  The tokens after the subcommand word,
/// or nothing when Git runs no subcommand.
///  `&[OsString]` is a borrowed list;
///  an empty one reads as no
///       deletion.
/// Why:
///   Both readers were written against that region.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const region = commandTokens(args)?.[1] ?? [];
/// ```
fn region_of(arguments: &[OsString]) -> &[OsString] {
    return match command_tokens(arguments) {
        Some((_word, region)) => region,
        None => &[],
    };
}

/// The audited endings and their facts stay out of the release executable.
#[cfg(test)]
#[path = "policy_audited_tests.rs"]
mod tests;
