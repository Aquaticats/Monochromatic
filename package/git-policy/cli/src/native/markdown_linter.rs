//! What:
//!  The linter the executable gives `markdown/autofix`:
//!  `monochromatic-lint`,
//!  linked into the executable and run in-process once per candidate,
//!  on a guarded thread with a deadline.
//! Why:
//!  The owner's rule is that the wrapper embeds the repository's own tools
//!  and starts other programs as rarely as possible.
//!  The linter's library entry `monochromatic_lint::run_embedded::fix_source` is the executable's
//!  `--config --stdin --stdin-filename --fix` run without the program start
//!  (`package/linter/monochromatic-lint/src/run_embedded.rs`),
//!  so there is no program to find on a search path,
//!  no configuration file to write and no stream contract to parse.
//!  The child process used to be the wrapper's kill boundary;
//!  `markdown_linter_guard.rs` rebuilds it with a thread, a deadline and panic containment.
//!  The configuration is rebuilt from the policy's options for each candidate,
//!  which costs a small JSONC document,
//!  so no state outlives the options it was built from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = embeddedLinter().lint({ topLevel, options, path, source });
//! ```

/// Import the guarded thread runner.
use super::markdown_linter_guard::{GuardFailure, run_guarded};
/// Import the one-rule configuration.
use super::markdown_linter_config::one_rule_configuration;
/// Import the reading of one run.
use super::markdown_linter_output::{LintRun, interpret_embedded_run};
/// Import the conversion of Git's pathname bytes to a path.
use super::git_metadata::path_from_git_bytes;
/// Import the interface this linter provides.
use super::policy_markdown::{LintRequest, MarkdownLinter};
/// When a long run prints its notices.
use super::wait_progress::Cadence;
/// The linter library's in-process entry and its types.
use monochromatic_lint::run_embedded::{EmbeddedFailure, EmbeddedRequest, EmbeddedRun, fix_source};
/// The sink of those notices.
use std::io::Write;
/// `PathBuf` is an owned filesystem path.
use std::path::{Path, PathBuf};
/// `Duration` is a span of time.
use std::time::Duration;

/// What:
///  How long one candidate may take before the wrapper stops waiting for it.
/// Why:
///   Measured on the host with the former child process,
///  one run takes about 11 milliseconds warm and 227 cold;
///  the LFS rule also hashes every image a file links,
///  which for a large image takes seconds,
///  and the host has run at load averages of 40 to 90.
///  A minute leaves room for all of that and still ends a hung linter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINT_DEADLINE_MS = 60_000;
/// ```
pub const LINT_DEADLINE: Duration = Duration::from_secs(60);

/// What:
///  The signature of what fixes one source:
///  the linter library's entry in production,
///  a planted function in the controls of the guard.
/// Why:
///  A plain function pointer is `Send`, so it moves onto the guarded thread without capturing anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FixFunction = (request: EmbeddedRequest) => EmbeddedRun; // throws EmbeddedFailure
/// ```
pub type FixFunction = fn(&EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure>;

/// What:
///  The linter of one invocation:
///  what fixes a source,
///  its deadline,
///  and where long-wait notices go.
/// Why:
///   The deadline and the function are fields so the controls can plant a stalled or panicking fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class EmbeddedLinter implements MarkdownLinter { constructor(fix, deadline, cadence, notices) {} }
/// ```
pub struct EmbeddedLinter {
    /// What fixes one source.
    fix: FixFunction,
    /// How long one candidate may take.
    deadline: Duration,
    /// When a run that takes long prints its notices.
    cadence: Cadence,
    /// Where those notices go.
    notices: Box<dyn Write>,
}

/// What:
///  The linter of the running executable:
///  the linked linter, `LINT_DEADLINE`, and notices on standard error.
/// Why:
///   Checks built before an environment is known get one too;
///  making it touches nothing,
///  and a command with no Markdown candidate never runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const embeddedLinter = () => new EmbeddedLinter(fixSource, LINT_DEADLINE, cadenceFromEnvironment(), stderr);
/// ```
pub fn embedded_linter() -> EmbeddedLinter {
    return EmbeddedLinter {
        fix: fix_source,
        deadline: LINT_DEADLINE,
        cadence: Cadence::from_environment(),
        notices: Box::new(std::io::stderr()),
    };
}

/// What:
///  A linter over `fix` with this deadline.
/// Why:
///   Controls plant a fix that panics or never returns, with a short deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const embeddedLinterWith = (fix, deadline) => new EmbeddedLinter(fix, deadline, cadenceFromEnvironment(), stderr);
/// ```
pub fn embedded_linter_with(fix: FixFunction, deadline: Duration) -> EmbeddedLinter {
    return EmbeddedLinter {
        fix,
        deadline,
        cadence: Cadence::from_environment(),
        notices: Box::new(std::io::stderr()),
    };
}

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
            return format!("cli-git could not start the thread that runs monochromatic-lint: {reason}");
        }
        GuardFailure::Panicked(reason) => {
            return format!(
                "monochromatic-lint stopped with an internal error ({reason}); the file was not checked, run the \
                 command again, run monochromatic-lint by hand on the file to see the error, or set the policy \
                 markdown/autofix to off to skip the check"
            );
        }
        GuardFailure::TimedOut(deadline) => {
            return format!(
                "monochromatic-lint did not finish within {} milliseconds and cli-git stopped waiting for it \
                 (its thread stays in this process until the command ends); run the command again, run \
                 monochromatic-lint by hand on the file to see why it stalls, or set the policy markdown/autofix \
                 to off to skip the check",
                deadline.as_millis()
            );
        }
        GuardFailure::Vanished => {
            return String::from(
                "the thread that runs monochromatic-lint ended without a result; this is a defect in cli-git.",
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
    ///  so abandoning it after the deadline leaves nothing the caller needs locked.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lint(request: LintRequest): LintRun; // throws string
    /// ```
    fn lint(&mut self, request: &LintRequest<'_>) -> Result<LintRun, String> {
        let Some(name) = path_from_git_bytes(request.path) else {
            return Err(String::from(
                "cli-git cannot spell this candidate's pathname for monochromatic-lint on this platform",
            ));
        };
        let waiting_for: String = format!("monochromatic-lint to check {}", name.display());
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
            self.deadline,
            self.cadence,
            waiting_for.as_str(),
            &mut self.notices,
        );
        match outcome {
            Ok(finished) => {
                return interpret_embedded_run(request.source, request.options.rules.as_slice(), finished);
            }
            Err(failure) => return Err(guard_failure_reason(&failure)),
        }
    }
}

impl EmbeddedLinter {
    /// The same linter with another notice cadence and sink;
    ///  tests choose a short cadence and read the notices from a buffer.
    pub fn with_notices(mut self, cadence: Cadence, notices: Box<dyn Write>) -> Self {
        self.cadence = cadence;
        self.notices = notices;
        return self;
    }
}

/// Guard, reason and contract controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_tests.rs"]
mod tests;
