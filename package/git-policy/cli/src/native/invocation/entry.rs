//! What:
//!  Decide what one `git` invocation of the native wrapper does,
//!  and perform it.
//! Why:
//!  The executable stays thin:
//!  it gathers process facts,
//!  asks this module for an
//!      action,
//!  and performs it.
//!  The decision itself is testable without replacing
//!      the test process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const action = planInvocation(args, env, inputs); perform(action);
//! ```

/// Import the sibling modules the decision combines.
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE, PushForward, failure, with_leading_notice};
#[cfg(unix)]
use crate::auto_push_trigger::{TipWatch, hand_over_if_ahead, try_watch_tip};
use crate::push_trigger_log::{Decision, Origin, Outcome, record_decision};
use crate::announced_query::{WRAPPER_VERSION, announced_query, announcement, printable_after};
use crate::command_push::stackable_option_tokens;
use crate::external_hooks::external_hooks_warning;
use crate::lfs_pre_push::uses_lfs;
use crate::lfs_upload_plan::settings_from_snapshot;
use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
use crate::push_stacking::{STACKED_PUSH_MARKER, run_stacked_push};
use crate::command_push::parse_push_region;
use crate::child_environment::{
    FORWARD_TARGET_VARIABLE, child_environment_overlay, environment_value,
};
use crate::effective_target::default_allowed_worktree_dirs;
use crate::forwarded_lock_notice::{forwarded_lock_notice, is_covered};
use crate::forwarding::{exit_code, replace_process_with_real_git, run_real_git};
use crate::stale_ref_lock_judge::Judge;
use crate::index_writer_forward::{CoordinatedForward, CoordinatedRun, run_coordinated_forward};
use crate::index_writer_commands::is_index_writer;
use crate::forwarded_command::{ResolvedCommand, Route, creates_worktrees, resolve_forwarded_command};
use crate::invocation_config::{config_invalid_event, load_identity_config};
use crate::repository_location::RepositoryLocation;
use crate::transaction_git::GitContext;
use crate::worktree_identity::WorktreeIdentity;
use crate::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use crate::management::{host_filesystem_section, plan_management_with_section};
/// The linter the executable gives the Markdown policy.
/// The shipped checks and their constructor.
use crate::policy_checks::{ShippedChecks, shipped_checks};
/// A forwarded command starts with no candidate content.
use crate::policy_content::LifecycleContent;
use crate::real_git::{ResolutionInputs, ResolvedGit, process_resolution_inputs, resolve_real_git};
use crate::started_as_git::refusal_when_started_as_git;
use crate::real_git_candidate::same_file;
use crate::repository_facts::{GitFacts, RepositoryFacts, git_facts};
use crate::rule_require_root::{RequireRootDecision, decide_require_root};
/// The variable that names the forbidden-strings rules list.
use crate::scanner_selection::RULES_VARIABLE;
use crate::wrapped_command::{WrappedOutcome, run_wrapped_command};
use crate::wrapper_controls::{Controls, no_controls, strip_global_controls};
use crate::worktree_add_plan::{AddPlan, copy_start_directory, parse_add_plan, parse_copy_destination, start_directory};
use crate::worktree_remove_plan::{RemovePlan, parse_remove_plan};
use crate::worktree_copy_forward::{CopyFacts, ForwardedRun, copy_applies_with, run_forwarded_git};
use crate::wrapper_invocation::{StrippedInvocation, strip_wrapper_controls};
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
/// The trait that gives output streams `.write_all(..)`.
use std::io::Write;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// The management namespace:
///  `git cli-git ...` is handled by the wrapper,
///  never by Git.
pub const MANAGEMENT_COMMAND: &str = "cli-git";

/// What:
///  Explain that another cli-git wrapper selected this executable as real Git.
///       `&Path` borrows this executable's path for the message.
/// Why:
///   Forwarding again would bounce between the two wrappers forever;
///  stopping
///       names this executable so the user can fix PATH.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forwardedToSelf(ownExecutable: string): Action;
/// ```
fn forwarded_to_self(own_executable: &Path) -> Action {
    // `format!` builds owned text; `.as_str()` lends it to the helper.
    return failure(
        format!(
            "another cli-git wrapper selected this cli-git executable ({}) as real Git. \
             Forwarding again would never reach Git. Remove the extra cli-git from PATH or \
             place the real Git executable on PATH.",
            own_executable.display()
        )
        .as_str(),
    );
}

/// What:
///  The caller's home directory from the environment,
///  if it names one.
///       `Option<PathBuf>` is "an owned path or nothing".
/// Why:
///   The tool caches exempt from linked-worktree enforcement live under the home
///       directory;
///  it comes from the injected environment,
///  never from a fixed path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const home = environment.HOME === undefined || environment.HOME === '' ? undefined : environment.HOME;
/// ```
fn home_directory(environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    // `if let Some(home) = ... && cond` runs only when the variable is set and not empty.
    if let Some(home) = environment_value(environment, "HOME")
        && !home.is_empty()
    {
        // `Some(...)` is the "present" case; `PathBuf::from` turns the text into a path.
        return Some(PathBuf::from(home));
    }
    // `None` is the "absent" case.
    return None;
}

/// What:
///  Decide the action for one invocation from injected process facts.
///       `&[OsString]` borrows the arguments after the program name;
///       `&[(OsString, OsString)]` borrows the environment as name/value pairs.
/// Why:
///   Order matters.
///  The recursion check runs before anything else.
///  Wrapper controls
///       written before the subcommand are removed first,
///  because Git's own reading of
///       the arguments stops at them and would hide the subcommand from every later
///       decision.
///  The management namespace is answered by the wrapper.
///  Every other
///       command goes through the wrapped-command lifecycle with real Git behind it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planInvocation(args: string[], environment: [string, string][], inputs: ResolutionInputs): Action;
/// ```
pub fn plan_invocation(
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
    inputs: &ResolutionInputs,
) -> Action {
    // `if let Some(target) = ... && cond` runs only when a forwarding wrapper left its
    // marker and that marker names this executable.
    if let Some(target) = environment_value(environment, FORWARD_TARGET_VARIABLE)
        && same_file(Path::new(&target), inputs.own_executable.as_path())
    {
        return forwarded_to_self(inputs.own_executable.as_path());
    }
    // `mut` allows the scan to record what the removed controls asked for.
    let mut global_controls: Controls = no_controls();
    // `&mut global_controls` lends the record for writing.
    let global_clean: Vec<OsString> = strip_global_controls(arguments, &mut global_controls);
    // `.as_slice()` lends the owned list as a borrowed view.
    let layout: GlobalLayout = global_layout(global_clean.as_slice());
    if layout.outcome == GlobalOutcome::Command
        && global_clean[layout.prefix_len] == MANAGEMENT_COMMAND
    {
        // `&list[a..]` borrows from index `a` on; `&list[..a]` borrows up to it.
        return plan_management_with_section(
            &global_clean[layout.prefix_len + 1..],
            &global_clean[..layout.prefix_len],
            &global_controls,
            environment,
            inputs,
            &host_filesystem_section,
        );
    }
    // What: `match` on the resolver's `Result`: keep the selection or stop with its message.
    // Why:  Without real Git there is nothing to forward to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let resolved; try { resolved = resolveRealGit(inputs); } catch (e) { return failure(String(e)); }
    // ```
    let resolved: ResolvedGit = match resolve_real_git(inputs) {
        Ok(found) => found,
        Err(error) => return failure(error.to_string().as_str()),
    };
    // The merge push of an integrated race is planned again inside this process with the stacking marker set;
    //  the planning of the push it came from has already given the notice.
    let notice: Option<String> = if environment_value(environment, STACKED_PUSH_MARKER).is_some() {
        None
    } else {
        resolved.notice
    };
    let action: Action = plan_with_real_git(
        arguments,
        environment,
        resolved.path,
        global_clean,
        &layout,
    );
    // `.as_deref()` lends the optional owned notice as an optional borrowed one.
    return with_leading_notice(action, notice.as_deref());
}

/// What:
///  Plan an invocation that is not a management command,
///  once the real Git is known.
///  `&GlobalLayout` borrows where the subcommand starts in the cleaned arguments.
/// Why:
///  `plan_invocation` adds the notice of a recorded fallback to whatever action this returns,
///  so every ending of the planning (a forward,
///  a stop,
///  a refusal) carries it without repeating it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planWithRealGit(args: string[], environment: [string, string][], realGit: string, globalClean: string[], layout: GlobalLayout): Action;
/// ```
fn plan_with_real_git(
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
    real_git: PathBuf,
    global_clean: Vec<OsString>,
    layout: &GlobalLayout,
) -> Action {
    let overlay: Vec<(OsString, OsString)> =
        child_environment_overlay(environment, real_git.as_path());
    // What: Top-level `--version`, `-v`, `--help` and `-h` run Git as a child and add the announcement.
    // Why:  The layout is already known here, so every other invocation pays one enum comparison;
    //       the other queries keep the exact process replacement below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const query = announcedQuery(args, layout); if (query) return { kind: 'forward-announced', ... };
    // ```
    if let Some(query) = announced_query(global_clean.as_slice(), layout) {
        return Action::ForwardAnnounced {
            announcement: announcement(query, WRAPPER_VERSION, real_git.as_path()),
            real_git,
            arguments: global_clean,
            overlay,
            stderr: String::new(),
        };
    }
    let stripped: StrippedInvocation = strip_wrapper_controls(arguments);
    // The global options before the subcommand select the repository for every fact query.
    let facts: GitFacts = git_facts(
        real_git.as_path(),
        &stripped.arguments[..stripped.layout.prefix_len],
        overlay.as_slice(),
    );
    // `mut` lets the lifecycle cache facts and state what content policies can read.
    let mut checks: ShippedChecks<GitFacts> = shipped_checks(
        facts,
        // `.clone()` copies the stripped arguments for the rule cores.
        stripped.arguments.clone(),
        LifecycleContent::None,
        default_allowed_worktree_dirs(
            environment,
            // `.as_deref()` lends the optional owned path as an optional borrowed one.
            home_directory(environment).as_deref(),
        ),
    );
    // The forbidden-strings rules variable is read from this invocation's environment.
    checks.scanner_settings.rules_variable = environment_value(environment, RULES_VARIABLE);
    // `match` picks by variant and binds the fields each ending carries.
    match run_wrapped_command(&stripped, environment, &mut checks) {
        // `arguments: forwarded` binds the field under a new name.
        WrappedOutcome::Forward {
            arguments: forwarded,
            stderr,
        } => {
            // The opt-out, Git's other query forms (`--exec-path` and so on), and the commands that need no repository
            // (`version`, `help`, `init`, `clone`, user-level `config`) never settle a copy.
            let copy_bypassed: bool = stripped.controls.skip_worktree_copy
                || stripped.layout.outcome != GlobalOutcome::Command
                || decide_require_root(forwarded.as_slice(), &[])
                    != RequireRootDecision::NeedsRepositoryRoot;
            return stack_if_push(
                forward_action(
                    real_git,
                    forwarded,
                    overlay,
                    stderr,
                    environment,
                    copy_bypassed,
                    &mut checks,
                ),
                environment,
            );
        }
        WrappedOutcome::Exit { code, stderr } => {
            return Action::Exit {
                code,
                // `String::new()` is empty owned text: wrapped commands report on standard error.
                stdout: String::new(),
                stderr,
            };
        }
    }
}

/// What:
///  Run one invocation with the real process facts and return its exit code.
///       `i32` is a signed 32-bit integer,
///  the type of process exit codes.
///       Returns only when Git was not started;
///  a forwarded command never returns.
/// Why:
///   This is everything the executable's `main` does;
///  keeping it in the library
///       lets the same code be measured by tests and mutation runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runProcess(args: string[], environment: [string, string][]): number;
/// ```
pub fn run_process(arguments: &[OsString], environment: &[(OsString, OsString)]) -> i32 {
    // From here on every real-Git start of this process runs with traditional hooks off.
    crate::hooks_off::activate();
    let action: Action = match process_resolution_inputs(environment) {
        // A cli-git executable that a wrapper started as Git stops before it plans anything or starts anything.
        Ok(inputs) => match refusal_when_started_as_git(environment, inputs.own_executable.as_path()) {
            Some(refusal) => failure(refusal.as_str()),
            None => plan_invocation(arguments, environment, &inputs),
        },
        Err(error) => failure(
            format!(
                "cannot determine this executable or the current directory, so real Git \
                 cannot be selected safely: {error}"
            )
            .as_str(),
        ),
    };
    // `match` picks by variant and binds the fields each one carries.
    match action {
        Action::Forward {
            real_git,
            arguments: forwarded,
            overlay,
            stderr,
        } => {
            // Warning events reach the caller before Git's own output.
            write_stream(&mut std::io::stderr(), stderr.as_str());
            let error: std::io::Error = replace_process_with_real_git(
                real_git.as_path(),
                forwarded.as_slice(),
                overlay.as_slice(),
            );
            write_stream(
                &mut std::io::stderr(),
                format!(
                    "cli-git: cannot start {}: {error}. Hint: if you moved or upgraded Git, check that the file exists and is executable with `ls -l {}`, \
                     then reinstall Git or the wrapper; no Git command ran.\n",
                    real_git.display(),
                    real_git.display()
                )
                .as_str(),
            );
            return ENGINE_FAILURE_EXIT_CODE;
        }
        Action::ForwardCoordinated { forward, stderr } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return run_coordinated_action(&forward);
        }
        Action::ForwardAnnounced {
            real_git,
            arguments: forwarded,
            overlay,
            announcement,
            stderr,
        } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return run_announced(
                real_git.as_path(),
                forwarded.as_slice(),
                overlay.as_slice(),
                announcement.as_str(),
            );
        }
        Action::ForwardSettled {
            real_git,
            arguments: forwarded,
            overlay,
            copy,
            stderr,
        } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return finish_forwarded(run_forwarded_git(
                real_git.as_path(),
                forwarded.as_slice(),
                overlay.as_slice(),
                Some(&*copy),
            ));
        }
        Action::ForwardPush { forward, stderr } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return run_stacked_push(&forward);
        }
        Action::Exit {
            code,
            stdout,
            stderr,
        } => {
            // `&mut` lends each stream for writing.
            write_stream(&mut std::io::stdout(), stdout.as_str());
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return code;
        }
    }
}

/// What:
///  Run Git as a child for an announced query,
///  print the announcement when Git exited with status 0,
///  and return Git's status.
/// Why:
///   The child shares the caller's streams, so Git's own answer is untouched and comes first;
///  the announcement follows it only when Git answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await runRealGit(realGit, args, overlay); if (outcome.exited === 0) print(text); return exitCode(outcome);
/// ```
fn run_announced(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    text: &str,
) -> i32 {
    match run_real_git(real_git, arguments, overlay) {
        Ok(outcome) => {
            write_stream(&mut std::io::stdout(), printable_after(outcome, text));
            return exit_code(outcome);
        }
        Err(error) => {
            write_stream(
                &mut std::io::stderr(),
                format!(
                    "cli-git: cannot start {}: {error}. Hint: if you moved or upgraded Git, check that the file exists and is executable with `ls -l {}`, \
                     then reinstall Git or the wrapper; no Git command ran.\n",
                    real_git.display(),
                    real_git.display()
                )
                .as_str(),
            );
            return ENGINE_FAILURE_EXIT_CODE;
        }
    }
}

/// What:
///  The exit code of a forwarded run, after printing the text of a run that failed.
/// Why:
///   A settled run keeps the wrapper alive around Git, so the exit code returns here instead of replacing the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = outcome.exitCode;
/// ```
fn finish_forwarded(run: ForwardedRun) -> i32 {
    // `match` on the ending: a status, or a failure with its own text.
    match run {
        ForwardedRun::Ran(code) => return code,
        ForwardedRun::Failed { stderr, code } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return code;
        }
    }
}

/// What:
///  Run a coordinated forward:
///  print its failure,
///  run Git under the landing lock,
///  or replace the process when no coordination applies after all.
/// Why:
///   The lock must outlive Git,
///  so only a run that takes no lock may replace the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await forwardToRealGit({ ... }); process.exitCode = outcome.exitCode;
/// ```
fn run_coordinated_action(forward: &CoordinatedForward) -> i32 {
    // A command that adds commits keeps the wrapper alive around Git, so the push can be handed over once it succeeded.
    let watch: Option<TipWatch> = if adds_commits(&forward.command) { watch_forward(forward) } else { None };
    let code: i32 = match run_coordinated_forward(forward) {
        CoordinatedRun::Ran(code) => code,
        CoordinatedRun::Failed { stderr, code } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            code
        }
        CoordinatedRun::NotCoordinated if watch.is_some() => {
            finish_forwarded(run_forwarded_git(forward.real_git.as_path(), forward.arguments.as_slice(), forward.overlay.as_slice(), None))
        }
        CoordinatedRun::NotCoordinated => {
            let error: std::io::Error = replace_process_with_real_git(
                forward.real_git.as_path(),
                forward.arguments.as_slice(),
                forward.overlay.as_slice(),
            );
            write_stream(
                &mut std::io::stderr(),
                format!("cli-git: cannot start {}: {error}\n", forward.real_git.display()).as_str(),
            );
            ENGINE_FAILURE_EXIT_CODE
        }
    };
    if let Some(found) = watch.as_ref() {
        if code == 0 {
            hand_over_if_ahead(found, trigger_command(&forward.command).as_str(), &mut std::io::stderr());
        } else {
            // The watch holds the branch and its tip from before the command, which a reader needs to tell which merge failed.
            record_decision(&Decision {
                common_dir: Some(found.common_dir()),
                origin: Some(found.origin()),
                command: trigger_command(&forward.command).as_str(),
                outcome: Outcome::Declined,
                reason: "command-failed",
                detail: format!("Git exited with status {code}; {} was at {} before the command", found.branch_ref(), found.tip_before()).as_str(),
                branch: Some(found.branch_ref()),
            });
        }
    }
    return code;
}

/// Without file system readers for this platform, no push is handed over after a command that adds commits.
#[cfg(not(unix))]
struct TipWatch;

/// Without file system readers for this platform, no branch is watched.
#[cfg(not(unix))]
fn watch_forward(_forward: &CoordinatedForward) -> Option<TipWatch> {
    return None;
}

/// Without file system readers for this platform, nothing is handed over.
#[cfg(not(unix))]
fn hand_over_if_ahead(_watch: &TipWatch, _command: &str, _stderr: &mut dyn Write) {}

/// What:
///  Whether a resolved command adds commits to the branch that the owner wants pushed like a commit: a merge, a pull, a
///  cherry-pick, a revert or an applied patch.
/// Why:
///   A rebase is left out, because a rebased branch would need a forced push; the tip check then also leaves out the
///  rebase a pull runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addsCommits(command: ResolvedGitCommand): boolean;
/// ```
fn adds_commits(command: &ResolvedCommand) -> bool {
    return command.is_builtin() && matches!(command.word(), b"merge" | b"pull" | b"cherry-pick" | b"revert" | b"am");
}

/// The command word a trigger record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = command.word().toString();
/// ```
fn trigger_command(command: &ResolvedCommand) -> String {
    return String::from_utf8_lossy(command.word()).into_owned();
}

/// The branch watch of a coordinated forward, started in the worktree the command runs in.
/// A watch that cannot start leaves a record naming the step,
/// in the common Git directory the wrapper already resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchForward(forward: CoordinatedForward): TipWatch | undefined;
/// ```
#[cfg(unix)]
fn watch_forward(forward: &CoordinatedForward) -> Option<TipWatch> {
    let (root, common): (PathBuf, PathBuf) = match &forward.identity {
        WorktreeIdentity::MainWorktree { worktree_root, common_dir, .. } | WorktreeIdentity::LinkedWorktree { worktree_root, common_dir, .. } => {
            (worktree_root.clone(), common_dir.clone())
        }
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => return None,
    };
    return match try_watch_tip(root.as_path(), forward.real_git.as_path(), forward.overlay.as_slice(), forward.environment.as_slice()) {
        Ok(watch) => Some(watch),
        Err(declined) => {
            // The watch could not start, so no branch is known; the worktree is.
            let git_dir: PathBuf = match &forward.identity {
                WorktreeIdentity::MainWorktree { git_dir, .. } | WorktreeIdentity::LinkedWorktree { git_dir, .. } => git_dir.clone(),
                WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => common.clone(),
            };
            record_decision(&Decision {
                common_dir: Some(common.as_path()),
                origin: Some(Origin { worktree: root.as_path(), git_dir: git_dir.as_path() }),
                command: trigger_command(&forward.command).as_str(),
                outcome: Outcome::Declined,
                reason: declined.reason,
                detail: declined.detail.as_str(),
                branch: None,
            });
            None
        }
    };
}

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
fn forward_action(
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
    let command: ResolvedCommand = resolve_forwarded_command(&context, arguments.as_slice());
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
    let remove_plan: Option<RemovePlan> = if command.word() == b"worktree" { plan_worktree_remove(&command) } else { None };
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
///  The plan of a `git worktree add` in a form the cheap-worktree ladder serves, read from the resolved command and the
///  process's working directory.
/// Why:
///   The destination is relative to the directory Git starts in, which the global `-C` options move; an unreadable
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
///  The destination of a `git worktree add` in any form of Git's documented options, read from the resolved command and the
///  process's working directory.
/// Why:
///   The ignored-state copy goes to the worktree this command created, which the command line names; a form the reading
///  does not know leaves the destination unknown, and the copy then says it did not run.
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
///  The plan of a plain `git worktree remove <directory>`, read from the resolved command and the process's working
///  directory.
/// Why:
///   The directory is relative to where Git starts, which the global `-C` options move; anything else is Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planWorktreeRemove(command: ResolvedCommand): RemovePlan | undefined;
/// ```
fn plan_worktree_remove(command: &ResolvedCommand) -> Option<RemovePlan> {
    let current: PathBuf = std::env::current_dir().ok()?;
    let start: PathBuf = start_directory(&command.arguments[..command.subcommand_index], current.as_path())?;
    return parse_remove_plan(command.arguments.as_slice(), command.subcommand_index, start.as_path());
}

/// What:
///  Whether `command` is a push that must upload LFS objects first, and the configuration warning for configured
///  external hooks, when the same read shows any, as empty text otherwise.
///  A dry-run push reads the configuration too, for the warning, but uploads nothing.
/// Why:
///   A push that sends nothing (`--dry-run`) needs no upload, and a repository without LFS keys needs none either.
///  One configuration read answers it, made with the invocation's own global options; when Git cannot list its
///  configuration the hook is added anyway, because it then decides with Git's own answers.
///  The warning costs no read of its own: a push is the one forwarded command that already holds the snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function needsLfsUpload(command, realGit, overlay): [boolean, string, settings];
/// ```
fn needs_lfs_upload(
    command: &ResolvedCommand,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
) -> (bool, String, Option<Vec<(String, String)>>) {
    if command.word() != b"push" {
        return (false, String::new(), None);
    }
    let dry_run: bool = parse_push_region(command.region(), &[]).is_ok_and(|region| {
        return region.dry_run;
    });
    let context: GitContext = GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        global_prefix: command.arguments[..command.subcommand_index].to_vec(),
        common_dir: None,
    };
    let snapshot: Option<ConfigSnapshot> = match std::env::current_dir() {
        Ok(directory) => read_config_snapshot(&context, directory.as_path()),
        Err(_) => None,
    };
    let warning: String = external_hooks_warning(snapshot.as_ref()).unwrap_or_default();
    if dry_run {
        return (false, warning, None);
    }
    return match snapshot {
        Some(found) => {
            let uses: bool = uses_lfs(&found);
            let settings: Option<Vec<(String, String)>> = uses.then(|| {
                return settings_from_snapshot(&found);
            });
            (uses, warning, settings)
        }
        None => (true, warning, None),
    };
}

/// What:
///  Turn the forward of a plain push that may be integrated into a run that can integrate it,
///  and put the Git LFS upload in front of every push that publishes.
/// Why:
///   A push that lost a race to another session cannot succeed again, and the wrapper can only look at its status
///  while it is alive;
///  the stacked push itself carries `STACKED_PUSH_MARKER`, so it is forwarded as it is and never stacks again.
///   A real push in a repository with LFS keys also runs as a child that the Git LFS upload (`lfs_upload`) precedes,
///  the stacked push too, so the parent can explain a failed upload;
///  the integration step's own dry-run queries never upload.
///   Every other action is returned unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackIfPush(action: Action, environment): Action;
/// ```
fn stack_if_push(action: Action, environment: &[(OsString, OsString)]) -> Action {
    let (real_git, arguments, overlay, copy, stderr) = match action {
        Action::Forward {
            real_git,
            arguments,
            overlay,
            stderr,
        } => (real_git, arguments, overlay, None, stderr),
        Action::ForwardSettled {
            real_git,
            arguments,
            overlay,
            copy,
            stderr,
        } => (real_git, arguments, overlay, Some(copy), stderr),
        other => return other,
    };
    let context: GitContext = GitContext {
        real_git: real_git.clone(),
        overlay: overlay.clone(),
        global_prefix: Vec::new(),
        // These queries send no input, so no scratch directory is needed.
        common_dir: None,
    };
    let command: ResolvedCommand = resolve_forwarded_command(&context, arguments.as_slice());
    let (lfs, hook_warning, lfs_settings): (bool, String, Option<Vec<(String, String)>>) =
        needs_lfs_upload(&command, real_git.as_path(), overlay.as_slice());
    let stderr_with_warning: String = format!("{stderr}{hook_warning}");
    let stacked_already: bool = environment_value(environment, STACKED_PUSH_MARKER).is_some();
    let stack: bool =
        !stacked_already && command.word() == b"push" && stackable_option_tokens(command.region()).is_some();
    // A push that uploads LFS objects runs as a child like a stackable one: the parent that owns the push is the one
    // that explains a failed upload, and no wrapper is started for it.
    if stack || lfs {
        return Action::ForwardPush {
            forward: Box::new(PushForward {
                real_git,
                subcommand_index: command.subcommand_index,
                arguments: command.arguments,
                overlay,
                copy,
                environment: environment.to_vec(),
                lfs,
                lfs_settings,
                stack,
            }),
            stderr: stderr_with_warning,
        };
    }
    // A plain forwarded fetch or push becomes Git itself, so the reference locks of a killed Git are judged here, before.
    let lock_notice: String = if is_covered(command.word()) {
        forwarded_lock_notice(command.word(), &command.arguments[..command.subcommand_index], environment, &Judge::here())
    } else {
        String::new()
    };
    let stderr_with_locks: String = format!("{stderr_with_warning}{lock_notice}");
    let forwarded: Vec<OsString> = arguments;
    return match copy {
        Some(facts) => Action::ForwardSettled {
            real_git,
            arguments: forwarded,
            overlay,
            copy: facts,
            stderr: stderr_with_locks,
        },
        None => Action::Forward {
            real_git,
            arguments: forwarded,
            overlay,
            stderr: stderr_with_locks,
        },
    };
}

/// What:
///  Whether the forwarded command needs the repository configuration.
/// Why:
///   An index writer reads its patience from it; the copy of a main checkout's ignored state reads none,
///  so a command that only creates a worktree loads nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isIndexWriter(command)
/// ```
fn configuration_needed(writes_index: bool) -> bool {
    return writes_index;
}

/// What:
///  Write text to an output stream,
///  ignoring a closed stream.
///       `&mut dyn Write` lends "any writable stream" (`dyn` means the concrete type is
///       chosen at run time,
///  like a TS interface value).
/// Why:
///   The exit code already carries the result;
///  a reader that went away must not
///       turn a diagnostic into a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeStream(stream: Writable, text: string): void { try { stream.write(text); } catch {} }
/// ```
fn write_stream(stream: &mut dyn Write, text: &str) {
    // What: `let _ = ...` deliberately discards the write's `Result`.
    // Why:  There is nowhere left to report a failure to write a diagnostic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { stream.write(text); } catch { /* reader closed */ }
    // ```
    let _ = stream.write_all(text.as_bytes());
}

/// Plans of the announced queries,
///  `--version`, `-v`, `--help` and `-h`, against the queries that stay exact.
#[cfg(test)]
#[path = "entry_announced_tests.rs"]
mod announced_tests;

/// Plans whose real Git is the one the commit service's unit records because `PATH` offers none.
#[cfg(test)]
#[path = "entry_fallback_tests.rs"]
mod fallback_tests;

/// Stop controls for a looping,
///  missing or uninterpretable real Git.
#[cfg(test)]
#[path = "entry_stop_tests.rs"]
mod stop_tests;

/// Decision controls and the shared fixtures stay out of the release executable.
#[cfg(test)]
#[path = "entry_tests.rs"]
mod tests;
