//! What:
//!  Turn parsed command options into output bytes and an exit status.
//! Why:
//!  This is the one place the pieces meet:
//!  discovery,
//!  per-file planning,
//!  bounded workers,
//! output routing and exit accounting.
//!  It takes the working directory and standard input as
//! arguments,
//!  so tests drive the exact production path without touching process state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runCommand(options, cwd, stdin) -> { stdout, stderr, exitCode }
//! ```

/// Import the command grammar and its collaborators.
use crate::{
    cli_options::CliOptions,
    closure_step::{ClosureOutcome, run_closure_step, with_messages},
    config_lookup::CONFIG_NAME,
    diagnostic::Diagnostic,
    file_discovery::DiscoveryOptions,
    path_inputs::collect_inputs,
    run_file::FileOutcome,
    run_finish::{PROGRAM, engine, finish},
    run_lfs::LfsRepos,
    run_modes::{SetupError, init_configuration, print_configuration, rules_listing},
    run_output::RunOutput,
    run_paths::display_name,
    run_plan::{ConfigStore, FilePlan, Planned},
    run_stdin::run_stdin,
    run_workers::process_plans,
    rust_file_engine::RustFileEngine,
};
/// Import the reader trait for standard input and native paths.
use std::{
    io::Read,
    path::{Path, PathBuf},
};

/// What:
///  Plan every discovered file,
///  separating lintable plans from skipped inputs.
/// Why:
///  All configuration errors surface here,
///  before any file is linted or rewritten.
/// The count returned is how many files had no configuration file at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planFiles(store, cwd, files, notes): { plans: FilePlan[]; unconfigured: number };
/// ```
fn plan_files(
    store: &mut ConfigStore,
    cwd: &Path,
    files: &[PathBuf],
    notes: &mut Vec<String>,
) -> Result<(Vec<FilePlan>, usize), SetupError> {
    let mut plans: Vec<FilePlan> = Vec::<FilePlan>::new();
    let mut without_configuration: usize = 0;
    for file in files {
        let planned: Planned = match store.plan(file) {
            Ok(value) => value,
            Err(error) => return Err(SetupError::from_display(&error)),
        };
        match planned {
            Planned::Lint { plan } => {
                // Naming every selected file lets a `--debug` reader see exactly what was linted.
                notes.push(format!("{}: selected for linting", plan.display));
                plans.push(*plan);
            }
            Planned::Ignored => {
                notes.push(format!(
                    "{}: ignored by configuration",
                    display_name(file, cwd)
                ));
            }
            Planned::NoConfiguration => {
                without_configuration += 1;
                notes.push(format!(
                    "{}: no configuration file in its directory or any ancestor",
                    display_name(file, cwd)
                ));
            }
            Planned::Unsupported => {}
        }
    }
    return Ok((plans, without_configuration));
}

/// What:
///  Lint or fix the files named by path arguments,
///  defaulting to the working directory.
/// Why:
///  Discovery,
///  planning,
///  processing and routing run in that order so that setup errors never
/// follow a rewritten file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runPaths(options, cwd, store): RunOutput;
/// ```
fn run_paths(
    options: &CliOptions,
    cwd: &Path,
    store: &mut ConfigStore,
) -> Result<RunOutput, SetupError> {
    let mut ignore_paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for path in &options.ignore_paths {
        // Ignore files resolve against the invocation directory, not the process's own.
        ignore_paths.push(cwd.join(path));
    }
    let discovery: DiscoveryOptions = DiscoveryOptions {
        cwd: cwd.to_path_buf(),
        no_ignore: options.no_ignore,
        ignore_patterns: options.ignore_patterns.clone(),
        ignore_paths,
    };
    let files: Vec<PathBuf> = match collect_inputs(
        options.paths.as_slice(),
        &discovery,
        options.no_error_on_unmatched_pattern,
    ) {
        Ok(found) => found,
        Err(error) => return Err(SetupError::from_display(&error)),
    };
    let mut notes: Vec<String> = vec![format!("discovered {} supported file(s)", files.len())];
    let (plans, without_configuration): (Vec<FilePlan>, usize) =
        plan_files(store, cwd, files.as_slice(), &mut notes)?;
    if !files.is_empty() && without_configuration == files.len() {
        return Err(SetupError {
            message: format!(
                "No {} was found for any of the {} input file(s). Create one with --init or pass --config.",
                CONFIG_NAME,
                files.len()
            ),
        });
    }
    let concurrency: usize = match options.concurrency {
        Some(limit) => limit.get(),
        None => match std::thread::available_parallelism() {
            Ok(available) => available.get(),
            Err(_) => 1,
        },
    };
    notes.push(format!(
        "linting {} file(s) with a concurrency limit of {concurrency}",
        plans.len()
    ));
    let lfs: LfsRepos = LfsRepos::new();
    let mut semantic: RustFileEngine = engine(options.debug);
    let outcomes: Vec<FileOutcome> = process_plans(
        plans.as_slice(),
        options.fix,
        &lfs,
        concurrency,
        &mut semantic,
    );
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    let mut written: usize = 0;
    for outcome in outcomes {
        findings.extend(outcome.findings);
        notes.extend(outcome.notes);
        if outcome.written {
            written += 1;
        }
    }
    notes.push(format!(
        "rewrote {written} file(s); loaded {} Cargo workspace(s)",
        semantic.workspace_count()
    ));
    // The compiler-backed closure check runs last, over the packages that the plans select.
    let closure: ClosureOutcome = run_closure_step(plans.as_slice(), cwd, options.fix);
    findings.extend(closure.findings);
    notes.extend(closure.notes);
    let mut output: RunOutput = finish(options, findings.as_slice(), None, notes.as_slice())?;
    output.stderr = with_messages(output.stderr.as_str(), closure.messages.as_slice(), PROGRAM);
    return Ok(output);
}

/// What:
///  Select and run the requested mode.
/// Why:
///  The command grammar already rejects conflicting modes;
///  each remaining mode returns its
/// own complete output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function execute(options, cwd, stdin): RunOutput; // throws SetupError
/// ```
fn execute(
    options: &CliOptions,
    cwd: &Path,
    stdin: &mut dyn Read,
) -> Result<RunOutput, SetupError> {
    if !cwd.is_absolute() {
        return Err(SetupError {
            message: String::from("The working directory must be an absolute path."),
        });
    }
    let mut output: RunOutput = RunOutput {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: 0,
    };
    if options.rules {
        output.stdout = rules_listing()?;
        return Ok(output);
    }
    if options.init {
        let created: PathBuf = init_configuration(cwd)?;
        output.stdout = format!("{}\n", created.display());
        return Ok(output);
    }
    let mut store: ConfigStore = match ConfigStore::new(cwd, options.config.as_deref()) {
        Ok(value) => value.with_directives(options.directive_options()),
        Err(error) => return Err(SetupError::from_display(&error)),
    };
    if let Some(path) = &options.print_config {
        match print_configuration(&mut store, cwd, path) {
            Ok(text) => output.stdout = text,
            Err(error) => return Err(SetupError::from_display(&error)),
        }
        return Ok(output);
    }
    if options.stdin {
        return run_stdin(options, &mut store, stdin);
    }
    return run_paths(options, cwd, &mut store);
}

/// What:
///  Run one invocation and always return output,
///  converting a setup failure to status 2.
/// Why:
///  A setup or usage error prints one prefixed line on standard error and nothing on standard output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runCommand(options: CliOptions, cwd: Path, stdin: Readable): RunOutput;
/// ```
pub fn run_command(options: &CliOptions, cwd: &Path, stdin: &mut dyn Read) -> RunOutput {
    match execute(options, cwd, stdin) {
        Ok(output) => return output,
        Err(error) => {
            return RunOutput {
                stdout: String::new(),
                stderr: format!("{PROGRAM}: {}\n", error.message),
                exit_code: 2,
            };
        }
    }
}

/// Whole-invocation controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_command_tests.rs"]
mod tests;

/// Fixing,
///  standard-input and LFS invocation controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_command_fix_tests.rs"]
mod fix_tests;
