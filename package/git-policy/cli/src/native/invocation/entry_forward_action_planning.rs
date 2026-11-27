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
use crate::action::{
    Action,
    ENGINE_FAILURE_EXIT_CODE,
    failure,
};
use crate::forwarded_command::{
    ResolvedCommand,
    Route,
    creates_worktrees,
    resolve_forwarded_command,
    the_command_git_runs,
    the_directory_a_commands_operands_are_read_from,
};
use crate::index_writer_commands::is_index_writer;
use crate::index_writer_forward::CoordinatedForward;
use crate::invocation_config::{
    config_invalid_event,
    load_identity_config,
};
/// The linter the executable gives the Markdown policy.
/// The shipped checks and their constructor.
use crate::policy_checks::ShippedChecks;
use crate::repository_facts::{
    GitFacts,
    RepositoryFacts,
};
use crate::repository_location::RepositoryLocation;
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::{
    add_is_quiet,
    copy_start_directory,
    parse_add_plan,
    parse_copy_destination,
    start_directory,
};
/// The facts a forward carries,
/// and whether the copy applies to its words.
use crate::worktree_command_facts::{
    AddPlan,
    CopyFacts,
    RemovePlan,
};
use crate::worktree_copy_forward::copy_applies_with;
use crate::worktree_identity::{
    WorktreeIdentity,
    common_directory,
};
use crate::worktree_remove_plan::{
    parse_removal_target,
    parse_remove_plan,
};
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
use std::path::{
    Path,
    PathBuf,
};
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
    mut stderr: String,
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
    let command: ResolvedCommand = resolve_forwarded_command(
        &context,
        environment,
        arguments.as_slice(),
        checks.start_directory.path(),
    );
    // What: The command the slot refusals read. A `!` alias the resolver could follow runs one plain `git` command in
    //  its shell, and that is the command Git will run; the destructive-command guard reads the same value by the same
    //  call (`the_command_git_runs`). A shell alias the wrapper cannot read stays unread here, exactly as the guard
    //  leaves it to Git (`SPEC.md`, "git by the word Git will run").
    // Why: Reading the alias's own name as the command word let `git -c 'alias.x=!git worktree remove' x -f -f <slot>`
    //  delete a bound slot's registration while the plain spelling was refused.
    let slot_command: Option<ResolvedCommand> = the_command_git_runs(&command);
    // What: The directory a slot command's operands are read from. Git runs a `!` alias's shell at the top of the
    //  working tree (`git.c:388` to `git.c:407`), so the command inside it reads a relative operand from there, never
    //  from the directory this wrapper was started in; every other spelling runs where the caller's `-C` chain reaches,
    //  which its own reader applies to the directory this process was started in.
    // Why: `git -c 'alias.rmwt=!git worktree remove --force' rmwt ./wt/two` from a subdirectory removed a bound slot's
    //  registration with no refusal, because the operand was looked for under the subdirectory while Git looked for it
    //  under the top.
    let started_in: Option<&Path> = checks.start_directory.path();
    let slot_start: Option<PathBuf> =
        the_directory_a_commands_operands_are_read_from(&context, &command, started_in);
    // An alias value may carry its own `-c` options, and Git applies those after any option put before the alias word,
    //  so the real Git gets the expansion the wrapper already resolved: the hooks-off options then follow the alias's own.
    let forwarded_arguments: Vec<OsString> = match command.route {
        Route::Alias => command.arguments.clone(),
        Route::Direct | Route::ShellAlias | Route::Unresolved | Route::Unread => arguments,
    };
    let writes_index: bool = is_index_writer(&command);
    // A `git worktree remove` or `git worktree move` behind a global option this build does not read is refused in a
    //  repository that holds slot records, so it keeps the copy bypassed only when no record stands
    //  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
    let slot_command_the_global_hides: bool = slot_command
        .as_ref()
        .is_some_and(crate::entry_zfs_slot_commands::is_a_slot_command_behind_an_unread_global);
    if !writes_index && copy_bypassed && !slot_command_the_global_hides {
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
    // The Git common directory this command's repository has: the one its identity names, or, for a command that stands
    // inside a Git directory and so in no worktree of it, the one Git itself named
    // (`git -C <repository>/.git worktree remove <slot>`).
    let common_dir: Option<PathBuf> = match common_directory(&location.identity) {
        Some(common) => Some(common.to_path_buf()),
        None => location.common_directory_outside_a_worktree.clone(),
    };
    #[cfg(target_os = "linux")]
    if let Some(read) = slot_command.as_ref()
        && let Some(ending) =
            crate::entry_zfs_slot_commands::unread_global_refusal_of(read, common_dir.as_deref())
    {
        return ending;
    }
    if location.identity == WorktreeIdentity::OutsideWorktree {
        // A command started inside a Git directory stands in no worktree, and Git still names the repository it belongs
        //  to: the slot refusals run over that directory, whose records are the ones the command would endanger
        //  (`SPEC.md`, "Removal of a ZFS worktree clone").
        #[cfg(target_os = "linux")]
        if let Some(read) = slot_command.as_ref()
            && let Some(common) = common_dir.as_deref()
            && let Some(ending) = crate::entry_zfs_slot_commands::the_slot_endings(
                read,
                common,
                &mut stderr,
                slot_start.as_deref(),
            )
        {
            return ending;
        }
        // The removal's refusal reads the pool, which only the run that would have carried the removal out may do, so a
        //  removal the command names is settled by the copy run; the run forwards it once the refusal answered nothing.
        //  A repository with no record folder can refuse nothing, so it keeps its plain forward and the cost of a run.
        #[cfg(target_os = "linux")]
        if let Some(common) = common_dir.as_deref()
            && crate::worktree_clone_binding_record::registration_home::home_folder(common).is_dir()
            && let Some(read) = slot_command.as_ref()
            && let Some(target) = plan_removal_target(read, Some(common), slot_start.as_deref())
        {
            return Action::ForwardSettled {
                real_git,
                arguments: forwarded_arguments,
                overlay,
                copy: Box::new(CopyFacts {
                    identity: location.identity,
                    common_dir,
                    creates: false,
                    destination: None,
                    add_plan: None,
                    // The ladder serves no command that stands in no worktree: its steps read a worktree and a checkout.
                    remove_plan: None,
                    removal_target: Some(target),
                    quiet: false,
                }),
                stderr,
            };
        }
        return Action::Forward {
            real_git,
            arguments: forwarded_arguments,
            overlay,
            stderr,
        };
    }
    // The wrapper refuses `git worktree move` for a slot a managed clone is bound to, mounted or not, restores the
    // pointer a prune would otherwise drop, and reports what a repair can restore
    // (`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Unmounted, pruned or moved worktrees"; plan step 12).
    #[cfg(target_os = "linux")]
    if let Some(common) = common_directory(&location.identity)
        && let Some(read) = slot_command.as_ref()
        && let Some(ending) = crate::entry_zfs_slot_commands::the_slot_endings(
            read,
            common,
            &mut stderr,
            slot_start.as_deref(),
        )
    {
        return ending;
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
    let add_plan: Option<AddPlan> = if creates {
        plan_worktree_add(&command, checks.start_directory.path())
    } else {
        None
    };
    // The person's `--quiet` comes from the same reading of the words, whether or not the ladder serves
    // the form, so an add it hands to plain Git still prints no line a quiet add would suppress.
    let quiet: bool =
        creates && add_is_quiet(command.arguments.as_slice(), command.subcommand_index);
    let destination: Option<PathBuf> = if creates {
        plan_copy_destination(&command, checks.start_directory.path())
    } else {
        None
    };
    // The ladder's own removal plan reads the caller's command, because a shell alias's removal is the shell's to
    //  carry out: the wrapper cannot destroy that subvolume in place of the `git` the shell starts.
    let remove_plan: Option<RemovePlan> = if command.word() == b"worktree" {
        plan_worktree_remove(
            &command,
            common_dir.as_deref(),
            checks.start_directory.path(),
        )
    } else {
        None
    };
    // The refusal of a removal of a bound slot reads the worktree in every spelling Git accepts, the double force included, and
    // a form it does not serve still has to reach that refusal (`SPEC.md`, "Removal of a ZFS worktree clone").
    //  The command it reads is the one Git will run, so a `!` alias the resolver could follow reaches the same refusal.
    let removal_target: Option<PathBuf> = match slot_command.as_ref() {
        Some(read) if read.word() == b"worktree" => {
            plan_removal_target(read, common_dir.as_deref(), slot_start.as_deref())
        }
        _ => None,
    };
    let serves_ladder: bool = add_plan.is_some() || removal_target.is_some();
    let copy: Option<CopyFacts> =
        if !copy_bypassed && copy_applies_with(&location.identity, creates, serves_ladder) {
            Some(CopyFacts {
                identity: location.identity.clone(),
                common_dir,
                creates,
                destination,
                add_plan,
                remove_plan,
                removal_target,
                quiet,
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
///  directory this command was started in.
/// Why:
///   The destination is relative to the directory Git starts in,
/// which the global `-C` options move;
/// a directory the caller could
///  not name or a form outside the table leaves the command to plain Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planWorktreeAdd(command: ResolvedCommand, current?: string): AddPlan | undefined;
/// ```
fn plan_worktree_add(
    command: &ResolvedCommand,
    current: Option<&Path>,
) -> Option<AddPlan> {
    let start: PathBuf = start_directory(&command.arguments[..command.subcommand_index], current?)?;
    return parse_add_plan(
        command.arguments.as_slice(),
        command.subcommand_index,
        start.as_path(),
    );
}

/// What:
///  The destination of a `git worktree add` in any form of Git's documented options,
/// read from the resolved command and the
///  directory this command was started in.
/// Why:
///   The ignored-state copy goes to the worktree this command created,
/// which the command line names;
/// a form the reading
///  does not know leaves the destination unknown,
/// and the copy then says it did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planCopyDestination(command: ResolvedCommand, current?: string): string | undefined;
/// ```
fn plan_copy_destination(
    command: &ResolvedCommand,
    current: Option<&Path>,
) -> Option<PathBuf> {
    let start: PathBuf =
        copy_start_directory(&command.arguments[..command.subcommand_index], current?)?;
    return parse_copy_destination(
        command.arguments.as_slice(),
        command.subcommand_index,
        start.as_path(),
    );
}

/// What:
///  The plan of a plain `git worktree remove <directory>`,
/// read from the resolved command and the directory this
///  command was started in.
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
/// function planWorktreeRemove(command: ResolvedCommand, commonDir?: string, current?: string): RemovePlan | undefined;
/// ```
fn plan_worktree_remove(
    command: &ResolvedCommand,
    common_dir: Option<&Path>,
    current: Option<&Path>,
) -> Option<RemovePlan> {
    let start: PathBuf = start_directory(&command.arguments[..command.subcommand_index], current?)?;
    return parse_remove_plan(
        command.arguments.as_slice(),
        command.subcommand_index,
        start.as_path(),
        common_dir,
    );
}

/// What:
///  The worktree a `git worktree remove` names in any form Git accepts,
///  read from the resolved command and the directory this
///  command was started in.
/// Why:
///   The refusal of a removal of a bound slot reads the command whatever its
///  options,
///  and the operand's relation to the directory Git starts in is the same one
///  the plan reads.
///  The start directory is read over Git's whole global table rather than the
///  ladder's short list,
///  because `-c`,
///  `--no-optional-locks` and a `--git-dir` change nothing about
///  where the operand lies and must not hide it from the refusal;
///  a spelling that table does not know still names a registered worktree,
///  which is what Git matches first (`parse_removal_target`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planRemovalTarget(command, commonDir?, current?): string | undefined;
/// ```
fn plan_removal_target(
    command: &ResolvedCommand,
    common_dir: Option<&Path>,
    current: Option<&Path>,
) -> Option<PathBuf> {
    let start: Option<PathBuf> =
        copy_start_directory(&command.arguments[..command.subcommand_index], current?);
    return parse_removal_target(
        command.arguments.as_slice(),
        command.subcommand_index,
        start.as_deref(),
        common_dir,
    );
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
