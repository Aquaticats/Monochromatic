//! What:
//!  The linter the executable gives `markdown/autofix`:
//!  `monochromatic-lint`,
//!  linked into the executable and run in-process once per candidate,
//!  on a guarded thread with a quiet window.
//! Why:
//!  The owner's rule is that the wrapper embeds the repository's own tools
//!  and starts other programs as rarely as possible.
//!  The linter's library entry `monochromatic_lint::run_embedded::fix_source` is the executable's
//!  `--config --stdin --stdin-filename --fix` run without the program start
//!  (`package/linter/monochromatic-lint/src/run_embedded.rs`),
//!  so there is no program to find on a search path,
//!  no configuration file to write and no stream contract to parse.
//!  The child process used to be the wrapper's kill boundary;
//!  `markdown_linter_guard.rs` replaces it with a thread,
//! a quiet window and panic containment,
//!  which is a hang and panic boundary and not a crash boundary:
//!  a stack overflow or an abort in the linter ends the wrapper.
//!  The size of a finished result is checked against the former child's output bounds
//!  (`markdown_linter_output.rs`),
//! after the linter has produced it.
//!  The configuration is rebuilt from the policy's options for each candidate,
//!  which costs a small JSONC document,
//!  so no state outlives the options it was built from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = embeddedLinter().lint({ topLevel, options, path, source });
//! ```

/// Import the guarded thread runner.
use crate::markdown_linter_guard::{GuardFailure, GuardLimits, NoticeSink, run_guarded};
/// Import the one-rule configuration.
use crate::markdown_linter_config::one_rule_configuration;
/// Import the reading of one run.
use crate::markdown_linter_output::{LintRun, explanation, interpret_embedded_run};
/// Import the conversion of Git's pathname bytes to a path.
use crate::git_metadata::path_from_git_bytes;
/// Import the interface this linter provides.
use crate::policy_markdown::{LintRequest, MarkdownLinter};
/// When a long run prints its notices.
use crate::wait_progress::Cadence;
/// The linter library's in-process entry and its types.
use monochromatic_lint::run_embedded::{EmbeddedFailure, EmbeddedRequest, EmbeddedRun, fix_source};
/// The sink of those notices.
use std::io::Write;
/// `PathBuf` is an owned filesystem path.
use std::path::{Path, PathBuf};
/// `Duration` is a span of time.
use std::time::Duration;

/// What:
///  How long the linter of one candidate may show no sign of work before the wrapper stops waiting for it.
/// Why:
///   A linter that works is waited for however long it takes (owner,
/// 2026-11-03:
/// "Stop only on no progress";
/// pick 9 of
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// the coordinator's under the owner's delegation and open to the owner's
///  veto),
/// and one that shows nothing is a hang.
///  Citation:
/// none,
/// and **unmeasured as a quiet window**.
/// Measured on the host with the former child process,
/// one run takes
///  about 11 milliseconds warm and 227 cold,
/// and the host has run at load averages of 40 to 90;
/// those are whole times,
/// and a
///  whole time is an upper bound of a quiet gap,
/// not the gap.
/// The longest quiet gap of every run is now kept in the ledger
///  (`quiet_ledger.rs`) and the number waits for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINT_QUIET_WINDOW_MS = 60_000;
/// ```
pub const LINT_QUIET_WINDOW: Duration = Duration::from_secs(60);

/// What:
///  The signature of what fixes one source:
///  the linter library's entry in production,
///  a planted function in the controls of the guard.
/// Why:
///  A plain function pointer is `Send`,
/// so it moves onto the guarded thread without capturing anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FixFunction = (request: EmbeddedRequest) => EmbeddedRun; // throws EmbeddedFailure
/// ```
pub type FixFunction = fn(&EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure>;

/// What:
///  The linter of one invocation:
///  what fixes a source,
///  its quiet window,
///  and where long-wait notices go.
/// Why:
///   The window and the function are fields so the controls can plant a stalled or panicking fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class EmbeddedLinter implements MarkdownLinter { constructor(fix, window, cadence, notices) {} }
/// ```
pub struct EmbeddedLinter {
    /// What fixes one source.
    fix: FixFunction,
    /// How long one candidate may show no sign of work.
    window: Duration,
    /// The longest one candidate is waited for,
    /// whatever it shows (`wait_ceilings.rs`,
    /// `LINT_CEILING`).
    ceiling: Duration,
    /// When a run that takes long prints its notices.
    cadence: Cadence,
    /// Where those notices go,
    ///  owned so the guard can write one on a thread of its own (`markdown_linter_guard.rs`);
    ///  nothing once a write never came back.
    notices: NoticeSink,
    /// Whether an earlier candidate of this command was abandoned for being quiet and left its thread running.
    abandoned: bool,
}

/// What:
///  The linter of the running executable:
///  the linked linter,
/// `LINT_QUIET_WINDOW`,
/// and notices on standard error.
/// Why:
///   Checks built before an environment is known get one too;
///  making it touches nothing,
///  and a command with no Markdown candidate never runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const embeddedLinter = () => new EmbeddedLinter(fixSource, LINT_QUIET_WINDOW, cadenceFromEnvironment(), stderr);
/// ```
pub fn embedded_linter() -> EmbeddedLinter {
    return EmbeddedLinter {
        fix: fix_source,
        window: LINT_QUIET_WINDOW,
        ceiling: crate::wait_ceilings::LINT_CEILING,
        cadence: Cadence::from_environment(),
        notices: Some(Box::new(std::io::stderr())),
        abandoned: false,
    };
}

/// What:
///  A linter over `fix` with this quiet window.
/// Why:
///   Controls plant a fix that panics or never returns,
/// with a short window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const embeddedLinterWith = (fix, window) => new EmbeddedLinter(fix, window, cadenceFromEnvironment(), stderr);
/// ```
pub fn embedded_linter_with(fix: FixFunction, window: Duration) -> EmbeddedLinter {
    return EmbeddedLinter {
        fix,
        window,
        ceiling: crate::wait_ceilings::LINT_CEILING,
        cadence: Cadence::from_environment(),
        notices: Some(Box::new(std::io::stderr())),
        abandoned: false,
    };
}

/// What:
///  The reason every candidate after an abandoned one gets.
/// Why:
///   An abandoned thread cannot be stopped and may still be working or holding memory;
///  starting another beside it for each later candidate could pile them up,
/// so the rest of the command
///  leaves its Markdown candidates unchecked and says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ABANDONED_REASON = 'an earlier Markdown check did not finish ...';
/// ```
const ABANDONED_REASON: &str = "an earlier Markdown check in this command did not finish and monochromatic-git-policy-cli stopped waiting \
     for it, so monochromatic-lint is not run again in this command; run the command again, or set the policy \
     markdown/autofix to off to skip the check";

/// What:
///  The sentence for a guarded run that produced no result.
/// Why:
///   Each cause tells the person what happened and the way forward in the linter's terms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function guardFailureReason(failure: GuardFailure): string;
/// ```
fn guard_failure_reason(failure: &GuardFailure) -> String {
    match failure {
        GuardFailure::Start(reason) => {
            return format!("monochromatic-git-policy-cli did not run the markdown check: {reason}");
        }
        GuardFailure::Panicked(reason) => {
            return format!(
                "monochromatic-lint stopped with an internal error ({}); the file was not checked, run the \
                 command again, run monochromatic-lint by hand on the file to see the error, or set the policy \
                 markdown/autofix to off to skip the check",
                explanation(reason.as_str())
            );
        }
        GuardFailure::Quiet(window) => {
            return format!(
                "monochromatic-lint showed no sign of work for {} milliseconds and monochromatic-git-policy-cli stopped waiting for it \
                 (its thread stays in this process until the command ends); run the command again, run \
                 monochromatic-lint by hand on the file to see why it stalls, or set the policy markdown/autofix \
                 to off to skip the check",
                window.as_millis()
            );
        }
        GuardFailure::Ceiling(ceiling) => {
            return format!(
                "monochromatic-lint ran for {} seconds, the longest monochromatic-git-policy-cli lets one Markdown check run, and monochromatic-git-policy-cli stopped waiting \
                 for it (its thread stays in this process until the command ends); run the command again, run \
                 monochromatic-lint by hand on the file, or set the policy markdown/autofix to off to skip the check",
                ceiling.as_secs()
            );
        }
        GuardFailure::Vanished => {
            return String::from(
                "the thread that runs monochromatic-lint ended without a result; this is a defect in monochromatic-git-policy-cli.",
            );
        }
    }
}

/// What:
///  `impl MarkdownLinter for EmbeddedLinter` runs the linked linter for each request.
/// Why:
///   See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class EmbeddedLinter implements MarkdownLinter {}
/// ```
impl MarkdownLinter for EmbeddedLinter {
    /// What:
    ///  Lint one candidate in-process.
    /// Why:
    ///   The linter reads `.lfsconfig` and linked images from the worktree,
    ///  so it is given the top level as its working directory
    ///  and the candidate's name relative to it;
    ///  the thread owns copies of everything it uses,
    ///  so abandoning it when it is quiet leaves nothing the caller needs locked.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lint(request: LintRequest): LintRun; // throws string
    /// ```
    fn lint(&mut self, request: &LintRequest<'_>) -> Result<LintRun, String> {
        if self.abandoned {
            return Err(String::from(ABANDONED_REASON));
        }
        let Some(name) = path_from_git_bytes(request.path) else {
            return Err(String::from(
                "monochromatic-git-policy-cli cannot spell this candidate's pathname for monochromatic-lint on this platform",
            ));
        };
        let waiting_for: String = format!("monochromatic-lint to check {}", name.display());
        let checking: String = name.display().to_string();
        let top_level: PathBuf = request.top_level.to_path_buf();
        let configuration: String = one_rule_configuration(
            request.options.rules.as_slice(),
            request.options.exclude.as_slice(),
        );
        let source: Vec<u8> = request.source.to_vec();
        let fix: FixFunction = self.fix;
        let outcome: Result<Result<EmbeddedRun, EmbeddedFailure>, GuardFailure> = run_guarded(
            move || {
                return fix(&EmbeddedRequest {
                    top_level: top_level.as_path(),
                    configuration: configuration.as_str(),
                    name: Path::new(name.as_os_str()),
                    source: source.as_slice(),
                });
            },
            GuardLimits { window: self.window, ceiling: self.ceiling },
            self.cadence,
            waiting_for.as_str(),
            &mut self.notices,
        );
        match outcome {
            Ok(finished) => {
                return interpret_embedded_run(request.source, request.options.rules.as_slice(), finished);
            }
            Err(failure) => {
                // A thread that was abandoned may still be running, so no second one starts beside it.
                self.abandoned = matches!(failure, GuardFailure::Quiet(_) | GuardFailure::Ceiling(_));
                // The ceiling's sentence names the file, because a ceiling is reached by a file that works for long.
                if matches!(failure, GuardFailure::Ceiling(_)) {
                    return Err(format!("monochromatic-lint, checking {checking}: {}", guard_failure_reason(&failure)));
                }
                return Err(guard_failure_reason(&failure));
            }
        }
    }
}

impl EmbeddedLinter {
    /// The same linter with another ceiling;
    /// controls plant a fix that keeps working with a short one.
    pub fn with_ceiling(mut self, ceiling: Duration) -> Self {
        self.ceiling = ceiling;
        return self;
    }

    /// The same linter with another notice cadence and sink;
    ///  tests choose a short cadence and read the notices from a buffer that a thread can hold.
    pub fn with_notices(mut self, cadence: Cadence, notices: Option<Box<dyn Write + Send>>) -> Self {
        self.cadence = cadence;
        self.notices = notices;
        return self;
    }
}

/// Guard,
/// reason and contract controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_tests.rs"]
mod tests;
