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
use crate::entry_forward_action_planning::forward_action;
use crate::entry_push_stacking_and_lfs_upload::stack_if_push;
use crate::exit_report_written::write_exit;
use crate::entry_run_forwarded_actions::{finish_forwarded, run_announced, run_coordinated_action, write_stream};
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE, failure, with_leading_notice};
use crate::announced_query::{WRAPPER_VERSION, announced_query, announcement};
use crate::push_stacking::{STACKED_PUSH_MARKER, run_stacked_push};
use crate::child_environment::{
    FORWARD_TARGET_VARIABLE, environment_value, invocation_overlay,
};
/// The two answers to "has this process a controlling terminal",
/// which decides whether the overlay of a typed command carries the never-prompt pair.
use crate::control_terminal::Terminal;
use crate::effective_target::default_allowed_worktree_dirs;
use crate::forwarding::replace_process_with_real_git;
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
use crate::repository_facts::{GitFacts, git_facts};
use crate::entry_repository_link_stop::repository_link_stop;
use crate::rule_require_root::{RequireRootDecision, decide_require_root};
/// The variable that names the forbidden-strings rules list.
use crate::scanner_selection::RULES_VARIABLE;
use crate::wrapped_command::{WrappedOutcome, run_wrapped_command};
use crate::wrapper_controls::{Controls, no_controls, strip_global_controls};
use crate::wrapper_names_old_and_new::COMMAND_WORD;
use crate::worktree_copy_forward::run_forwarded_git;
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
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// The management namespace:
///  `git monochromatic-git-policy-cli ...` is handled by the wrapper,
///  never by Git.
pub const MANAGEMENT_COMMAND: &str = COMMAND_WORD.current;

/// The line on standard error that an invocation spelled with the earlier word gets before the same output:
///  the earlier spelling works,
/// and the new one is named.
pub const EARLIER_COMMAND_WORD_NOTICE: &str = "monochromatic-git-policy-cli: `git cli-git` is the earlier spelling of this \
                                               command; `git monochromatic-git-policy-cli` does the same, so use that from now on.\n";

/// What:
///  Explain that another monochromatic-git-policy-cli wrapper selected this executable as real Git.
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
            "another monochromatic-git-policy-cli wrapper selected this monochromatic-git-policy-cli executable ({}) as real Git. \
             Forwarding again would never reach Git. Remove the extra monochromatic-git-policy-cli from PATH or \
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
    // What: Both spellings of the management word are answered by the wrapper; the earlier one adds one notice line.
    // Why:  The owner decided the earlier word "still works, with a one-line notice".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const earlier = words[prefixLen] === COMMAND_WORD.earlier;
    // ```
    if layout.outcome == GlobalOutcome::Command
        && (global_clean[layout.prefix_len] == MANAGEMENT_COMMAND || global_clean[layout.prefix_len] == COMMAND_WORD.earlier)
    {
        let earlier_spelling: bool = global_clean[layout.prefix_len] == COMMAND_WORD.earlier;
        // `&list[a..]` borrows from index `a` on; `&list[..a]` borrows up to it.
        let answer: Action = plan_management_with_section(
            &global_clean[layout.prefix_len + 1..],
            &global_clean[..layout.prefix_len],
            &global_controls,
            environment,
            inputs,
            &host_filesystem_section,
        );
        if earlier_spelling {
            return with_leading_notice(answer, Some(EARLIER_COMMAND_WORD_NOTICE));
        }
        return answer;
    }
    // A command that names the repository through a link to the 860 drive that does not resolve stops here with the plain
    // message, before Git prints its own; with no identity file the cost is one failed open.
    if let Some(stop) = repository_link_stop(global_clean.as_slice(), &layout, environment, inputs.current_directory.as_path()) {
        return stop;
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
        inputs.terminal,
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
/// function planWithRealGit(args: string[], environment: [string, string][], realGit: string, globalClean: string[], layout: GlobalLayout, terminal: Terminal): Action;
/// ```
fn plan_with_real_git(
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
    real_git: PathBuf,
    global_clean: Vec<OsString>,
    layout: &GlobalLayout,
    terminal: Terminal,
) -> Action {
    // What: Every Git and git-lfs child of this invocation gets the overlay, and one whose process has no controlling terminal
    //  is told never to prompt, so a missing login fails at once instead of opening a desktop dialog nobody can answer.
    // Why:  The push itself, its LFS upload and the upload plan's questions all start from this one list, so no start added
    //  later on this route can miss the pair.
    let overlay: Vec<(OsString, OsString)> =
        invocation_overlay(environment, real_git.as_path(), terminal);
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
    // The test-only runtime probe answers before anything else of the wrapper runs (nothing in production sets its variable).
    if crate::runtime_probe::is_requested(environment) {
        return crate::runtime_probe::run();
    }
    // From here on every real-Git start of this process runs with traditional hooks off.
    crate::hooks_off::activate();
    let action: Action = match process_resolution_inputs(environment) {
        // A monochromatic-git-policy-cli executable that a wrapper started as Git stops before it plans anything or starts anything.
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
                    "monochromatic-git-policy-cli: cannot start {}: {error}. Hint: if you moved or upgraded Git, check that the file exists and is executable with `ls -l {}`, \
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
        Action::Exit { code, stdout, stderr } => {
            return write_exit(code, stdout.as_bytes(), stderr.as_bytes());
        }
        Action::ExitBytes { code, stdout, stderr } => {
            return write_exit(code, stdout.as_slice(), stderr.as_slice());
        }
    }
}

/// The two spellings of the management word:
///  the same behaviour,
///  and one notice line for the earlier one.
#[cfg(test)]
#[path = "entry_command_word_tests.rs"]
mod command_word_tests;

/// Plans of the announced queries,
///  `--version`,
/// `-v`,
/// `--help` and `-h`,
/// against the queries that stay exact.
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
