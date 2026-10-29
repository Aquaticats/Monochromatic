//! What:
//!  The linter the executable gives `markdown/autofix`:
//!  `monochromatic-lint`,
//!  found
//!       on the invocation's PATH and started once per candidate as
//!       `monochromatic-lint --config=<file> --stdin --stdin-filename=<path> --fix`
//!       from the repository's top level,
//!  with a time bound and output caps.
//! Why:
//!  This is the plan's stream contract (`doc/planning/unified-linter.md`,
//!  section
//!      "Distribution and consumers";
//!  `doc/planning/cli-git-rust-implementation.md`,
//!      lines 181 to 189).
//!  The program and its one-rule configuration are found and
//!      written once per invocation,
//!  when the first candidate needs them,
//!  so a command
//!      with no Markdown candidate never needs the linter.
//!  Every option is written as one
//!      `--name=value` argument,
//!  so a pathname that starts with `-` stays a value.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = await processLinter(env).lint({ topLevel, options, path, source });
//! ```

/// Import the bounded child runner.
use super::bounded_child::{
    ChildFailure, ChildLimits, ChildRequest, FinishedChild, run_bounded_child_noticing,
};
/// When a long run prints its notices.
use super::wait_progress::Cadence;
/// The sink of those notices.
use std::io::Write;
/// Import the reading of the environment as name and value pairs.
use super::child_environment::environment_value;
/// Import the conversion of Git's pathname bytes to a path.
use super::git_metadata::path_from_git_bytes;
/// Import the one-rule configuration and its file.
use super::markdown_linter_config::{one_rule_configuration, write_configuration};
/// Import the PATH lookup.
use super::markdown_linter_lookup::find_linter;
/// Import the reading of one run.
use super::markdown_linter_output::{LintRun, interpret_linter_run};
/// Import the interface this linter provides.
use super::policy_markdown::{LintRequest, MarkdownLinter};
use super::scratch_location::prepare_user_scratch;
/// `OsString` is owned operating-system text.
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path.
use std::path::{Path, PathBuf};
/// `Duration` is a span of time.
use std::time::Duration;
/// The owner-only temporary file that holds the configuration while the invocation runs.
use tempfile::NamedTempFile;

/// What:
///  The bounds of one linter run in the executable.
/// Why:
///   Measured on the host,
///  one run takes about 11 milliseconds warm and 227 cold;
///  the
///       LFS rule also hashes every image a file links,
///  which for a large image takes
///       seconds,
///  and the host has run at load averages of 40 to 90.
///  A minute leaves
///       room for all of that and still ends a hung linter.
///  Standard output is the fixed
///       source:
///  the cap allows a large file whose every link grows into an object URL.
///       Standard error carries only the findings the fixes leave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINTER_LIMITS = { timeoutMs: 60_000, maxStdout: 64 * 2 ** 20, maxStderr: 4 * 2 ** 20 };
/// ```
pub const LINTER_LIMITS: ChildLimits = ChildLimits {
    timeout: Duration::from_secs(60),
    max_stdout: 64 << 20,
    max_stderr: 4 << 20,
};

/// What:
///  The linter of one invocation:
///  the environment it runs with,
///  its bounds,
///  where
///       its files go,
///  and,
///  once known,
///  its program and configuration file.
/// Why:
///   The lookup and the file are made at most once;
///  their failures are remembered and
///       reported to every later candidate the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProcessLinter implements MarkdownLinter { executable?: string | null; configuration?: TemporaryFile | string }
/// ```
pub struct ProcessLinter {
    /// The invocation's whole environment,
    ///  which the linter runs with.
    environment: Vec<(OsString, OsString)>,
    /// The bounds of each run.
    limits: ChildLimits,
    /// The directory for the configuration file and the stream files when one is given;
    ///  `None` means the per-user scratch directory of the environment,
    ///  made on first use.
    scratch: Option<PathBuf>,
    /// The directory in use,
    ///  once decided,
    ///  or the reason it could not be.
    prepared_scratch: Option<Result<PathBuf, String>>,
    /// The program,
    ///  once looked up;
    ///  `Some(None)` remembers that it was not found.
    executable: Option<Option<PathBuf>>,
    /// The configuration file,
    ///  once written,
    ///  or the reason it could not be.
    configuration: Option<Result<NamedTempFile, String>>,
    /// When a run that takes long prints its notices.
    cadence: Cadence,
    /// Where those notices go.
    notices: Box<dyn Write>,
}

/// What:
///  The linter for an invocation with this environment,
///  bounds and scratch directory.
///       `&[(OsString, OsString)]` borrows the environment.
/// Why:
///   Tests pass small bounds and a fixture directory;
///  `executable_linter` leaves the directory to the environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const processLinter = (environment, limits, scratch) => new ProcessLinter(environment, limits, scratch);
/// ```
pub fn process_linter(
    environment: &[(OsString, OsString)],
    limits: ChildLimits,
    scratch: PathBuf,
) -> ProcessLinter {
    return ProcessLinter {
        environment: environment.to_vec(),
        limits,
        scratch: Some(scratch),
        prepared_scratch: None,
        executable: None,
        configuration: None,
        cadence: Cadence::from_environment(),
        notices: Box::new(std::io::stderr()),
    };
}

/// What:
///  The linter of the running executable:
///  `LINTER_LIMITS` and the per-user scratch directory of the environment,
///  made on the first lint (`<state>/cli-git/scratch`, mode `0700`, never the system temporary directory).
/// Why:
///   The placeholder linters of checks built before an environment is known are made with an empty environment,
///  and a command with no Markdown candidate must not touch the disk,
///  so a missing state directory or `HOME` is reported when a candidate needs the linter,
///  not when it is made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const executableLinter = (environment) => new ProcessLinter(environment, LINTER_LIMITS, undefined);
/// ```
pub fn executable_linter(environment: &[(OsString, OsString)]) -> ProcessLinter {
    return ProcessLinter {
        environment: environment.to_vec(),
        limits: LINTER_LIMITS,
        scratch: None,
        prepared_scratch: None,
        executable: None,
        configuration: None,
    };
}

/// What:
///  The sentence for a child that produced no usable run.
/// Why:
///   Each cause tells the person what happened in the linter's terms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function childFailureReason(failure: ChildFailure, program: string): string;
/// ```
fn child_failure_reason(failure: &ChildFailure, program: &std::path::Path) -> String {
    match failure {
        ChildFailure::Files(reason) => {
            return format!(
                "cli-git could not create the temporary files for monochromatic-lint's input and output: {reason}"
            );
        }
        ChildFailure::Start(reason) => {
            return format!(
                "cli-git could not start monochromatic-lint at {}: {reason}",
                program.display()
            );
        }
        ChildFailure::Wait(reason) => {
            return format!("cli-git could not wait for monochromatic-lint: {reason}");
        }
        ChildFailure::TimedOut(bound) => {
            return format!(
                "monochromatic-lint did not finish within {} milliseconds and was stopped; run the command again, \
                 run monochromatic-lint by hand on the file to see why it stalls, or set the policy markdown/autofix \
                 to off to skip the check",
                bound.as_millis()
            );
        }
        ChildFailure::TooLarge { stream, limit } => {
            return format!(
                "monochromatic-lint wrote more than {limit} bytes to its {stream} and was stopped"
            );
        }
    }
}

/// What:
///  `impl ProcessLinter { ... }` attaches the two remembered preparations.
/// Why:
///   They are made on first use and never twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProcessLinter { program(): string; configurationPath(options): string }
/// ```
impl ProcessLinter {
    /// The same linter with another notice cadence and sink;
    ///  tests choose a short cadence and read the notices from a buffer.
    pub fn with_notices(mut self, cadence: Cadence, notices: Box<dyn Write>) -> Self {
        self.cadence = cadence;
        self.notices = notices;
        return self;
    }

    /// What:
    ///  The program,
    ///  looked up on the environment's PATH on first use.
    /// Why:
    ///   A missing linter is a broken installation,
    ///  reported as the reason every
    ///       Markdown candidate stays unchecked (`doc/planning/cli-git-rust-open-decisions.md`,
    ///       "Markdown policy without the linter").
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// program(): string; // throws string
    /// ```
    fn program(&mut self) -> Result<PathBuf, String> {
        if self.executable.is_none() {
            self.executable = Some(find_linter(
                environment_value(self.environment.as_slice(), "PATH").as_deref(),
            ));
        }
        match &self.executable {
            Some(Some(found)) => return Ok(found.clone()),
            Some(None) | None => {
                return Err(String::from(
                    "cli-git could not find monochromatic-lint in any absolute directory on PATH, and the policy markdown/autofix runs it for every Markdown candidate; install it so that it is on PATH",
                ));
            }
        }
    }

    /// What:
    ///  The directory for the configuration file and the stream files,
    ///  made private and swept of stale entries on first use.
    /// Why:
    ///   A command with no Markdown candidate never reaches here,
    ///  so it never touches the disk;
    ///  the per-user location is the environment's,
    ///  never the system temporary directory.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scratchDirectory(): string; // throws string
    /// ```
    fn scratch_directory(&mut self) -> Result<PathBuf, String> {
        if self.prepared_scratch.is_none() {
            self.prepared_scratch = Some(match &self.scratch {
                Some(given) => Ok(given.clone()),
                None => prepare_user_scratch(self.environment.as_slice()),
            });
        }
        match &self.prepared_scratch {
            Some(decided) => return decided.clone(),
            None => {
                return Err(String::from(
                    "cli-git did not decide the linter's scratch directory; this is a defect in cli-git.",
                ));
            }
        }
    }

    /// What:
    ///  The configuration file's path,
    ///  written for the request's options on first use.
    /// Why:
    ///   The options are the invocation's,
    ///  the same for every candidate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// configurationPath(request: LintRequest): string; // throws string
    /// ```
    fn configuration_path(&mut self, request: &LintRequest<'_>, scratch: &Path) -> Result<PathBuf, String> {
        if self.configuration.is_none() {
            let document: String = one_rule_configuration(
                request.options.rules.as_slice(),
                request.options.exclude.as_slice(),
            );
            self.configuration = Some(write_configuration(document.as_str(), scratch));
        }
        match &self.configuration {
            Some(Ok(file)) => return Ok(file.path().to_path_buf()),
            Some(Err(reason)) => return Err(reason.clone()),
            None => {
                return Err(String::from(
                    "cli-git did not write the linter configuration; this is a defect in cli-git.",
                ));
            }
        }
    }
}

/// What:
///  One `--name=value` argument.
///  `&std::ffi::OsStr` borrows the value's raw bytes.
/// Why:
///   Joined to its option,
///  a value that starts with `-` can never read as an option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const option = (name: string, value: string) => `${name}=${value}`;
/// ```
fn option(name: &str, value: &std::ffi::OsStr) -> OsString {
    let mut argument: OsString = OsString::from(name);
    argument.push("=");
    argument.push(value);
    return argument;
}

/// What:
///  `impl MarkdownLinter for ProcessLinter` starts the real linter for each request.
/// Why:
///   See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProcessLinter implements MarkdownLinter {}
/// ```
impl MarkdownLinter for ProcessLinter {
    /// What:
    ///  Lint one candidate through the linter's standard input.
    /// Why:
    ///   The linter reads `.lfsconfig` and linked images from the worktree,
    ///  so it runs
    ///       from the top level and is told the candidate's name relative to it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lint(request: LintRequest): LintRun; // throws string
    /// ```
    fn lint(&mut self, request: &LintRequest<'_>) -> Result<LintRun, String> {
        let program: PathBuf = self.program()?;
        let scratch: PathBuf = self.scratch_directory()?;
        let configuration: PathBuf = self.configuration_path(request, scratch.as_path())?;
        let Some(name) = path_from_git_bytes(request.path) else {
            return Err(String::from(
                "cli-git cannot spell this candidate's pathname for monochromatic-lint on this platform",
            ));
        };
        let arguments: Vec<OsString> = vec![
            option("--config", configuration.as_os_str()),
            OsString::from("--stdin"),
            option("--stdin-filename", name.as_os_str()),
            OsString::from("--fix"),
        ];
        let waiting_for: String = format!("monochromatic-lint to check {}", name.display());
        let finished: FinishedChild = match run_bounded_child_noticing(
            &ChildRequest {
                executable: program.as_path(),
                arguments: arguments.as_slice(),
                directory: request.top_level,
                environment: self.environment.as_slice(),
                input: request.source,
                scratch: scratch.as_path(),
                limits: self.limits,
                waiting_for: waiting_for.as_str(),
                cadence: self.cadence,
            },
            &mut self.notices,
        ) {
            Ok(done) => done,
            Err(failure) => return Err(child_failure_reason(&failure, program.as_path())),
        };
        return interpret_linter_run(request.source, request.options.rules.as_slice(), &finished);
    }
}

/// Lookup,
///  configuration,
///  argument and failure controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_tests.rs"]
mod tests;
