//! What:
//!  What one invocation does from arguments to Git:
//!  the decision,
//!  real Git resolution,
//!  forwarding,
//!  startup and management words.
//! Why:
//!  The thin executables call into this folder first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // invocation/index.ts: one `export * from './file'` per module below.
//! ```

/// The decision one invocation ends in:
///  become Git,
///  or print and exit.
pub mod action;

/// The value of a Git alias,
///  split as Git splits it,
///  and read from `git config --null` without losing a byte.
pub mod alias_value;

/// The four top-level spellings after which the wrapper announces itself,
///  and the text it prints once Git has answered.
pub mod announced_query;

/// Variables added to every real-Git child:
///  lock PID injection and the forward-target marker.
pub mod child_environment;

/// Worktree target classification for worktree-enforcing policies.
pub mod effective_target;

/// The per-invocation decision the thin executable performs.
pub mod entry;

/// Plans what a forwarded Git command does:
/// plain forward,
/// settled worktree copy,
/// or index-writer coordination.
pub mod entry_forward_action_planning;

/// The stop of an invocation that names the repository through a link to the 860 drive that does not resolve.
pub mod entry_repository_link_stop;

/// Turns a plain forward of a push into a stacked or LFS-uploading push,
/// with the notices a forward owes.
pub mod entry_push_stacking_and_lfs_upload;

/// Runs coordinated and settled forwards and writes to the standard streams.
/// The end of a command that stops without running the caller's request,
/// with its report written as bytes.
pub mod exit_report_written;

pub mod entry_run_forwarded_actions;

/// The ask to compile changed rules text,
/// after a command that moves the worktree.
pub mod entry_rules_text_change_warm;

/// Spellings of the wrapper-only escape hatches shared by several modules.
pub mod escape_hatch;

/// The command a Git alias really runs:
///  the expansion of ordinary aliases,
///  and the executable Git selects first.
pub mod external_git_command;

/// Starting real Git with unchanged arguments,
///  streams and exit status.
pub mod forwarding;

/// Git 2.56.0 global-argument boundaries over unchanged operating-system strings.
pub mod global_arguments;

/// The `git monochromatic-git-policy-cli` decision:
///  help,
///  retired commands,
///  and direct `check`/`fix`.
pub mod management;

/// The direct `monochromatic-git-policy-cli` commands that run in this process,
/// with their refusals.
pub mod management_direct_commands;

/// Which repository the service commands mean.
pub mod management_service_repository_selection;

/// `service settle-earlier-batches`,
/// and the note `doctor` adds about the batches it would settle.
pub mod management_service_settle_earlier_batches;

/// `service stop-units-of-removed-worktrees`,
/// and the note `doctor` adds about the units it would stop.
pub mod management_service_stop_units_of_removed_worktrees;

/// The `git monochromatic-git-policy-cli` argument grammar.
pub mod management_arguments;

/// The refusal of a direct command whose `--policy` selects a policy that is off.
pub mod management_selection;

/// PATH resolution of the real Git executable.
pub mod real_git;

/// Wrapper self-exclusion for one PATH candidate.
pub mod real_git_candidate;

/// The real Git that the commit service's unit file records,
/// read as the fallback when `PATH` offers none.
pub mod real_git_record;

/// The one resolution of the real Git every start uses:
/// `PATH`,
/// then the Git the commit service's unit records.
pub mod real_git_resolution;

/// The marker every start of real Git carries:
///  its format,
///  and the one lookup of it in an environment.
pub mod start_marker;

/// The chain record carried in an inherited descriptor:
///  the start that a launcher which clears the environment cannot hide.
pub mod chain_starts_record;

/// The marker every real-Git start carries and the check that stops a wrapper which finds it was started as Git.
pub mod started_as_git;

/// Wrapper-only control spellings,
///  their meanings,
///  and removal before the subcommand.
pub mod wrapper_controls;

/// Removal of every wrapper control from one invocation,
///  by position.
pub mod wrapper_invocation;

/// The executable's panic hook:
///  where an internal error happened,
///  never its message.
pub mod panic_notice;

/// The names of the commands built into Git 2.56.0.
pub mod git_builtins;

/// The lifecycle of one wrapped Git command,
///  from refusals to forwarding.
pub mod wrapped_command;

/// The steps of a wrapped command beside its guarded stages:
/// escape-hatch record,
/// push failure notices,
/// snapshots,
/// manual push.
pub mod wrapped_command_notices_snapshots_and_manual_push;

/// Every real-Git start of the wrapper carries `core.hooksPath` pointing at a directory that never exists.
pub mod hooks_off;

/// The startup step of a command:
/// isolated merges whose process died,
/// and what an earlier release left behind.
pub mod startup_gate;

/// The Git version check run once before work that depends on Git's behavior.
pub mod git_version;

/// The private per-user cache of the version text,
/// keyed by the Git executable's identity.
pub mod git_version_cache;

/// The Git command a forwarded invocation runs,
///  after ordinary alias expansion.
pub mod forwarded_command;

/// The guarded decision about the command Git will really run,
/// alias expansion included.
pub mod guarded_forwarded_command;
