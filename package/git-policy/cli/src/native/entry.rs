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
use super::action::{Action, ENGINE_FAILURE_EXIT_CODE, PushForward, failure};
use super::command_push::stackable_option_tokens;
use super::push_stacking::{STACKED_PUSH_MARKER, run_stacked_push};
use super::child_environment::{
    FORWARD_TARGET_VARIABLE, child_environment_overlay, environment_value,
};
use super::effective_target::default_allowed_worktree_dirs;
use super::forwarding::replace_process_with_real_git;
use super::index_writer_forward::{CoordinatedForward, CoordinatedRun, run_coordinated_forward};
use super::index_writer_commands::is_index_writer;
use super::forwarded_command::{ResolvedCommand, creates_or_moves_worktrees, resolve_forwarded_command};
use super::invocation_config::{config_invalid_event, load_identity_config};
use super::config_file::LoadedConfig;
use super::repository_location::RepositoryLocation;
use super::transaction_git::GitContext;
use super::worktree_identity::WorktreeIdentity;
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use super::management::plan_management;
/// The linter the executable gives the Markdown policy.
use super::markdown_linter::executable_linter;
/// The shipped checks and their constructor.
use super::policy_checks::{ShippedChecks, shipped_checks};
/// A forwarded command starts with no candidate content.
use super::policy_content::LifecycleContent;
use super::real_git::{ResolutionInputs, process_resolution_inputs, resolve_real_git};
use super::real_git_candidate::same_file;
use super::repository_facts::{GitFacts, RepositoryFacts, git_facts};
use super::rule_require_root::{RequireRootDecision, decide_require_root};
/// The variable that names the forbidden-strings rules file.
use super::scanner_selection::RULES_VARIABLE;
use super::wrapped_command::{WrappedOutcome, run_wrapped_command};
use super::wrapper_controls::{Controls, no_controls, strip_global_controls};
use super::worktree_copy_forward::{CopyFacts, ForwardedRun, copy_applies, run_forwarded_git};
use super::wrapper_invocation::{StrippedInvocation, strip_wrapper_controls};
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
        return plan_management(
            &global_clean[layout.prefix_len + 1..],
            &global_clean[..layout.prefix_len],
            &global_controls,
            environment,
            inputs,
        );
    }
    // What: `match` on the resolver's `Result`: keep the path or stop with its message.
    // Why:  Without real Git there is nothing to forward to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let realGit; try { realGit = resolveRealGit(inputs); } catch (e) { return failure(String(e)); }
    // ```
    let real_git: PathBuf = match resolve_real_git(inputs) {
        Ok(found) => found,
        Err(error) => return failure(error.to_string().as_str()),
    };
    let overlay: Vec<(OsString, OsString)> =
        child_environment_overlay(environment, real_git.as_path());
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
    // The Markdown linter is found on, and runs with, this invocation's environment.
    checks.markdown.linter = Box::new(executable_linter(environment));
    // `match` picks by variant and binds the fields each ending carries.
    match run_wrapped_command(&stripped, environment, &mut checks) {
        // `arguments: forwarded` binds the field under a new name.
        WrappedOutcome::Forward {
            arguments: forwarded,
            stderr,
        } => {
            // The opt-out, Git's help and version forms, and the commands that need no repository
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
    let action: Action = match process_resolution_inputs(environment) {
        Ok(inputs) => plan_invocation(arguments, environment, &inputs),
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
    match run_coordinated_forward(forward) {
        CoordinatedRun::Ran(code) => return code,
        CoordinatedRun::Failed { stderr, code } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return code;
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
            return ENGINE_FAILURE_EXIT_CODE;
        }
    }
}

/// What:
///  Choose how a forwarded command runs:
///  plainly,
///  under the landing lock when it writes the real index,
///  and settled by the linked-worktree copy when it runs from a linked worktree or a bare repository
///  (or creates a worktree from a main checkout whose configuration names the rules file).
///  `copy_bypassed` is true for the wrapper's opt-out and for Git's help and version forms.
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
    let writes_index: bool = is_index_writer(&command);
    if !writes_index && copy_bypassed {
        return Action::Forward {
            real_git,
            arguments,
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
            arguments,
            overlay,
            stderr,
        };
    }
    let loaded: Option<LoadedConfig> = match configuration_needed(&location.identity, &command, writes_index) {
        false => None,
        true => match load_identity_config(&location.identity) {
            Ok(found) => Some(found),
            Err(error) => {
                return Action::Exit {
                    code: ENGINE_FAILURE_EXIT_CODE,
                    stdout: String::new(),
                    stderr: config_invalid_event(&error),
                };
            }
        },
    };
    let creates_or_moves: bool = creates_or_moves_worktrees(&command);
    // Only a main checkout hands over the configured rules file; `copy_applies` limits it to a creating command.
    let main_checkout_paths: Vec<String> = match (&location.identity, &loaded) {
        (WorktreeIdentity::MainWorktree { .. }, Some(found)) => found
            .config
            .policies
            .forbidden_strings
            .rules_file
            .iter()
            .cloned()
            .collect(),
        _ => Vec::new(),
    };
    let copy: Option<CopyFacts> =
        if !copy_bypassed && copy_applies(&location.identity, creates_or_moves, main_checkout_paths.as_slice()) {
            Some(CopyFacts {
                identity: location.identity.clone(),
                creates_or_moves,
                main_checkout_paths,
                global_prefix: checks.facts.transaction_context().global_prefix,
                environment: environment.to_vec(),
            })
        } else {
            None
        };
    if !writes_index {
        let Some(facts) = copy else {
            return Action::Forward {
                real_git,
                arguments,
                overlay,
                stderr,
            };
        };
        return Action::ForwardSettled {
            real_git,
            arguments,
            overlay,
            copy: Box::new(facts),
            stderr,
        };
    }
    let Some(found) = loaded else {
        return failure("the repository configuration was not loaded for an index writer");
    };
    return Action::ForwardCoordinated {
        forward: Box::new(CoordinatedForward {
            real_git,
            arguments,
            overlay,
            identity: location.identity,
            command,
            environment: environment.to_vec(),
            timeout_ms: found.config.concurrency.index_lock.unproven_owner_timeout_ms,
            copy,
        }),
        stderr,
    };
}

/// What:
///  Turn the forward of a plain push that may be integrated into a run that can integrate it.
/// Why:
///   A push that lost a race to another session cannot succeed again, and the wrapper can only look at its status
///  while it is alive;
///  the stacked push itself carries `STACKED_PUSH_MARKER`, so it is forwarded as it is and never stacks again.
///   Every other action is returned unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackIfPush(action: Action, environment): Action;
/// ```
fn stack_if_push(action: Action, environment: &[(OsString, OsString)]) -> Action {
    if environment_value(environment, STACKED_PUSH_MARKER).is_some() {
        return action;
    }
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
    if command.word() == b"push" && stackable_option_tokens(command.region()).is_some() {
        return Action::ForwardPush {
            forward: Box::new(PushForward {
                real_git,
                subcommand_index: command.subcommand_index,
                arguments: command.arguments,
                overlay,
                copy,
                environment: environment.to_vec(),
            }),
            stderr,
        };
    }
    return match copy {
        Some(facts) => Action::ForwardSettled {
            real_git,
            arguments,
            overlay,
            copy: facts,
            stderr,
        },
        None => Action::Forward {
            real_git,
            arguments,
            overlay,
            stderr,
        },
    };
}

/// What:
///  Whether the forwarded command needs the repository configuration.
/// Why:
///   An index writer reads its patience from it, and a main checkout reads the rules file name from it;
///  linked worktrees and bare repositories need no configuration for the copy, so their commands load nothing new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isIndexWriter(command) || (identity.kind === 'main-worktree' && createsOrMovesWorktrees(command))
/// ```
fn configuration_needed(identity: &WorktreeIdentity, command: &ResolvedCommand, writes_index: bool) -> bool {
    return writes_index
        || (matches!(identity, WorktreeIdentity::MainWorktree { .. })
            && creates_or_moves_worktrees(command));
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

/// Stop controls for a looping,
///  missing or uninterpretable real Git.
#[cfg(test)]
#[path = "entry_stop_tests.rs"]
mod stop_tests;

/// Decision controls and the shared fixtures stay out of the release executable.
#[cfg(test)]
#[path = "entry_tests.rs"]
mod tests;
