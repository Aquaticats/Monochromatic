//! What:
//!  Plans what a forwarded Git command does once the wrapper has stripped its own controls:
//!  plain forward,
//! forward with a settled worktree copy,
//! or forward under the index-writer coordination.
//! Why:
//!  `entry.rs` decides the whole invocation;
//! this part decides only the forwarded command,
//! so each file stays small and
//!  neither uses the other in a circle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // forwardAction(realGit, args, overlay, stderr, env, copyBypassed, checks): Action
//! ```

/// Import the sibling modules the decision combines.
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE, failure};
use crate::index_writer_forward::CoordinatedForward;
use crate::index_writer_commands::is_index_writer;
use crate::forwarded_command::{ResolvedCommand, Route, creates_worktrees, resolve_forwarded_command};
use crate::invocation_config::{config_invalid_event, load_identity_config};
use crate::repository_location::RepositoryLocation;
use crate::transaction_git::GitContext;
use crate::worktree_identity::{WorktreeIdentity, common_directory};
/// The linter the executable gives the Markdown policy.
/// The shipped checks and their constructor.
use crate::policy_checks::ShippedChecks;
use crate::repository_facts::{GitFacts, RepositoryFacts};
use crate::worktree_add_plan::{AddPlan, copy_start_directory, parse_add_plan, parse_copy_destination, start_directory};
use crate::worktree_remove_plan::{RemovePlan, parse_remove_plan};
use crate::worktree_copy_forward::{CopyFacts, copy_applies_with};
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments and environment values are never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.argv and process.env, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};
/// What:
///  Choose how a forwarded command runs:
///  plainly,
///  under the landing lock when it writes the real index,
///  and settled by the linked-worktree copy when it runs from a linked worktree or a bare repository
///  (or creates a worktree from a main checkout whose configuration names the rules file).
///  `copy_bypassed` is true for the wrapper's opt-out and for Git's query forms.
/// Why:
///   Only these commands need the wrapper to outlive Git or to hold a lock around it;
///  everything else keeps the exact process replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isIndexWriter(command) && !copyApplies) forward(args); else await forwardToRealGit({ ..., unprovenOwnerTimeoutMs });
/// ```
pub(crate) fn forward_action(
    real_git: PathBuf,
    arguments: Vec<OsString>,
    overlay: Vec<(OsString, OsString)>,
    stderr: String,
    environment: &[(OsString, OsString)],
    copy_bypassed: bool,
    checks: &mut ShippedChecks<GitFacts>,
) -> Action {
    let context: GitContext = GitContext {
        real_git: real_git.clone(),
        overlay: overlay.clone(),
        global_prefix: Vec::new(),
        // These queries send no input, so no scratch directory is needed.
        common_dir: None,
    };
    let command: ResolvedCommand = resolve_forwarded_command(&context, environment, arguments.as_slice());
    // An alias value may carry its own `-c` options, and Git applies those after any option put before the alias word,
    //  so the real Git gets the expansion the wrapper already resolved: the hooks-off options then follow the alias's own.
    let forwarded_arguments: Vec<OsString> = match command.route {
        Route::Alias => command.arguments.clone(),
        Route::Direct | Route::ShellAlias | Route::Unresolved => arguments,
    };
    let writes_index: bool = is_index_writer(&command);
    if !writes_index && copy_bypassed {
        return Action::Forward {
            real_git,
            arguments: forwarded_arguments,
            overlay,
            stderr,
        };
    }
    let location: RepositoryLocation = match checks.facts.location() {
        Ok(found) => found,
        Err(message) => return failure(message.as_str()),
    };
    if location.identity == WorktreeIdentity::OutsideWorktree {
        return Action::Forward {
            real_git,
            arguments: forwarded_arguments,
            overlay,
            stderr,
        };
    }
    // An invalid configuration refuses an index writer; a main checkout's worktree copy reads none.
    if configuration_needed(writes_index)
        && let Err(error) = load_identity_config(&location.identity)
    {
        return Action::Exit {
            code: ENGINE_FAILURE_EXIT_CODE,
            stdout: String::new(),
            stderr: config_invalid_event(&error),
        };
    }
    let creates: bool = creates_worktrees(&command);
    let add_plan: Option<AddPlan> = if creates { plan_worktree_add(&command) } else { None };
    let destination: Option<PathBuf> = if creates { plan_copy_destination(&command) } else { None };
    let remove_plan: Option<RemovePlan> = if command.word() == b"worktree" {
        plan_worktree_remove(&command, common_directory(&location.identity))
    } else {
        None
    };
    let serves_ladder: bool = add_plan.is_some() || remove_plan.is_some();
    let copy: Option<CopyFacts> =
        if !copy_bypassed && copy_applies_with(&location.identity, creates, serves_ladder) {
            Some(CopyFacts {
                identity: location.identity.clone(),
                creates,
                destination,
                add_plan,
                remove_plan,
            })
        } else {
            None
        };
    if !writes_index {
        let Some(facts) = copy else {
            return Action::Forward {
                real_git,
                arguments: forwarded_arguments,
                overlay,
                stderr,
            };
        };
        return Action::ForwardSettled {
            real_git,
            arguments: forwarded_arguments,
            overlay,
            copy: Box::new(facts),
            stderr,
        };
    }
    return Action::ForwardCoordinated {
        forward: Box::new(CoordinatedForward {
            real_git,
            arguments: forwarded_arguments,
            overlay,
            identity: location.identity,
            command,
            environment: environment.to_vec(),
            copy,
        }),
        stderr,
    };
}

/// What:
///  The plan of a `git worktree add` in a form the cheap-worktree ladder serves,
/// read from the resolved command and the
///  process's working directory.
/// Why:
///   The destination is relative to the directory Git starts in,
/// which the global `-C` options move;
/// an unreadable
///  working directory or a form outside the table leaves the command to plain Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planWorktreeAdd(command: ResolvedCommand): AddPlan | undefined;
/// ```
fn plan_worktree_add(command: &ResolvedCommand) -> Option<AddPlan> {
    let current: PathBuf = std::env::current_dir().ok()?;
    let start: PathBuf = start_directory(&command.arguments[..command.subcommand_index], current.as_path())?;
    return parse_add_plan(command.arguments.as_slice(), command.subcommand_index, start.as_path());
}

/// What:
///  The destination of a `git worktree add` in any form of Git's documented options,
/// read from the resolved command and the
///  process's working directory.
/// Why:
///   The ignored-state copy goes to the worktree this command created,
/// which the command line names;
/// a form the reading
///  does not know leaves the destination unknown,
/// and the copy then says it did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planCopyDestination(command: ResolvedCommand): string | undefined;
/// ```
fn plan_copy_destination(command: &ResolvedCommand) -> Option<PathBuf> {
    let current: PathBuf = std::env::current_dir().ok()?;
    let start: PathBuf = copy_start_directory(&command.arguments[..command.subcommand_index], current.as_path())?;
    return parse_copy_destination(command.arguments.as_slice(), command.subcommand_index, start.as_path());
}

/// What:
///  The plan of a plain `git worktree remove <directory>`,
/// read from the resolved command and the process's working
///  directory.
/// Why:
///   The directory is relative to where Git starts,
/// which the global `-C` options move,
/// and Git picks a
///  registered worktree by the last components of its path before it reads the name as a path,
/// so the common
///  directory that holds that list is passed beside the start directory.
/// Anything else is Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planWorktreeRemove(command: ResolvedCommand, commonDir?: string): RemovePlan | undefined;
/// ```
fn plan_worktree_remove(command: &ResolvedCommand, common_dir: Option<&Path>) -> Option<RemovePlan> {
    let current: PathBuf = std::env::current_dir().ok()?;
    let start: PathBuf = start_directory(&command.arguments[..command.subcommand_index], current.as_path())?;
    return parse_remove_plan(command.arguments.as_slice(), command.subcommand_index, start.as_path(), common_dir);
}

/// What:
///  Whether the forwarded command needs the repository configuration.
/// Why:
///   An index writer reads its patience from it;
/// the copy of a main checkout's ignored state reads none,
///  so a command that only creates a worktree loads nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isIndexWriter(command)
/// ```
fn configuration_needed(writes_index: bool) -> bool {
    return writes_index;
}
