//! What:
//!  The closure check as a step of a lint run:
//!  run the Dylint library once per selected Cargo package and turn its reports into findings.
//! Why:
//!  `avoidable_closure` needs the types and captures that only the compiler knows,
//!  so the linter asks the compiler through `cargo dylint` after the per-file rules have run.
//!  A package whose prerequisites are missing on the machine is skipped with a message on standard error,
//!  and a package that cannot be checked for any other reason fails the run with status 2.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ClosureOutcome = { findings: Diagnostic[]; notes: string[]; messages: string[] };
//! function runClosureStep(plans: FilePlan[], cwd: Path, fix: boolean): ClosureOutcome;
//! ```

/// Import the processes,
///  the grouping,
///  the mapping and the failure finding of the check.
use crate::{
    closure_cargo::{
        DylintRun, cargo_program, dependencies_missing, dylint_probe, run_dylint, rustc_dev_missing,
    },
    closure_driver::{DriverGaps, DriverSite, driver_gaps, driver_site_from_environment},
    closure_findings::{RULE, Targets, findings_from, targets_of},
    closure_manifest::{Prerequisite, read_prerequisites},
    closure_packages::{ClosurePackage, group_packages},
    closure_probe::{PackageSetup, Readiness, prepare},
    closure_report::{Parsed, parse_records},
    diagnostic::Diagnostic,
    run_failure::{file_start, processing_failure},
    run_paths::display_name,
    run_plan::FilePlan,
};
/// Import the borrowed operating-system string and native paths.
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// What:
///  What the step adds to a run.
/// Why:
///  Findings join the run's JSONL,
///  notes join the `--debug` notes,
///  and messages are always written to standard error because a skipped package must be visible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClosureOutcome = { findings: Diagnostic[]; notes: string[]; messages: string[] };
/// ```
#[derive(Debug, Default)]
pub struct ClosureOutcome {
    /// Findings of the check,
    ///  including processing failures.
    pub findings: Vec<Diagnostic>,
    /// Execution notes for `--debug`.
    pub notes: Vec<String>,
    /// Lines for standard error without the program prefix.
    pub messages: Vec<String>,
}

/// What:
///  Format a skip message.
/// Why:
///  One wording keeps the messages searchable by the rule name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skipMessage(subject: string, reason: string): string;
/// ```
fn skip_message(subject: &str, reason: &str) -> String {
    return format!("{RULE} skipped {subject}: {reason}.");
}

/// What:
///  The last lines of a tool's standard error.
/// Why:
///  Cargo prints build progress first and its explanation last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tail(text: string, count: number): string;
/// ```
fn tail(text: &str, count: usize) -> String {
    let lines: Vec<&str> = text.lines().collect::<Vec<&str>>();
    let start: usize = lines.len().saturating_sub(count);
    return lines[start..].join("\n");
}

/// What:
///  Explain a `cargo dylint` run that exited unsuccessfully.
/// Why:
///  The explanation names the first compiler error when the records hold one,
///  and the end of cargo's own messages otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failureMessage(run: DylintRun, parsed: Parsed): string;
/// ```
fn failure_message(run: &DylintRun, parsed: &Parsed) -> String {
    let detail: String = match &parsed.compiler_error {
        Some(error) => format!("The first compiler error was:\n{error}"),
        None => format!(
            "The end of cargo's messages was:\n{}",
            tail(&run.stderr, 12)
        ),
    };
    return format!(
        "{RULE} could not check this package: `cargo dylint` exited with {}. The result is incomplete, so the run cannot be trusted. {detail}",
        run.status
    );
}

/// What:
///  Check one package that passed the machine-wide checks.
/// Why:
///  The order is manifest,
/// offline dependencies,
/// prerequisites,
/// then the run itself;
///  each can end the package early with a skip or a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkPackage(package: ClosurePackage, context): ClosureOutcome;
/// ```
fn check_package(
    package: &ClosurePackage,
    cargo: &Path,
    search: &OsString,
    targets: &Targets,
    cwd: &Path,
) -> ClosureOutcome {
    let mut outcome: ClosureOutcome = ClosureOutcome::default();
    let Some(directory) = package.manifest.parent() else {
        return outcome;
    };
    let subject: String = display_name(directory, cwd);
    let manifest_name: String = display_name(&package.manifest, cwd);
    let prerequisites: Vec<Prerequisite> = match read_prerequisites(directory) {
        Ok(found) => found,
        Err(reason) => {
            outcome.findings.push(processing_failure(
                manifest_name.as_str(),
                file_start(),
                format!("{RULE} could not read the prerequisites of {subject}: {reason}"),
            ));
            return outcome;
        }
    };
    if let Some(reason) = dependencies_missing(cargo, &package.manifest) {
        outcome.messages.push(skip_message(&subject, &reason));
        return outcome;
    }
    let setup: PackageSetup = match prepare(
        prerequisites.as_slice(),
        directory,
        cargo,
        search.as_os_str(),
    ) {
        Readiness::Ready(ready) => ready,
        Readiness::Skipped(reason) => {
            outcome.messages.push(skip_message(&subject, &reason));
            return outcome;
        }
    };
    let run: DylintRun = match run_dylint(cargo, &package.library, &package.manifest, &setup) {
        Ok(done) => done,
        Err(reason) => {
            outcome.findings.push(processing_failure(
                manifest_name.as_str(),
                file_start(),
                format!("{RULE} could not check this package: {reason}"),
            ));
            return outcome;
        }
    };
    let parsed: Parsed = parse_records(run.stdout.as_slice());
    outcome
        .findings
        .extend(findings_from(parsed.reports.as_slice(), directory, targets));
    outcome.notes.push(format!(
        "{RULE} checked {subject}: {} report(s) from the compiler",
        parsed.reports.len()
    ));
    if !run.success {
        outcome.findings.push(processing_failure(
            manifest_name.as_str(),
            file_start(),
            failure_message(&run, &parsed),
        ));
    }
    return outcome;
}

/// What:
///  Merge one package's outcome into the step's outcome.
/// Why:
///  A package outcome has the same three parts as the step's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function absorb(total: ClosureOutcome, part: ClosureOutcome): void;
/// ```
fn absorb(total: &mut ClosureOutcome, part: ClosureOutcome) {
    total.findings.extend(part.findings);
    total.notes.extend(part.notes);
    total.messages.extend(part.messages);
}

/// What:
///  Report libraries named in configuration that do not exist.
/// Why:
///  A missing library directory is a configuration error of the repository,
/// not a missing prerequisite of the machine,
///  so it fails the run once per library instead of skipping silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingLibraries(packages: ClosurePackage[]): Diagnostic[];
/// ```
fn missing_libraries(packages: &[ClosurePackage]) -> Vec<Diagnostic> {
    let mut reported: Vec<PathBuf> = Vec::<PathBuf>::new();
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for package in packages {
        if package.library.join("Cargo.toml").is_file() || reported.contains(&package.library) {
            continue;
        }
        reported.push(package.library.clone());
        findings.push(processing_failure(
            package.configuration.as_str(),
            file_start(),
            format!(
                "{RULE} option library names {}, which holds no Cargo.toml of a Dylint library; no package was checked with it. Fix the path, which is relative to this configuration file.",
                package.library.display()
            ),
        ));
    }
    return findings;
}

/// What:
///  Run the closure check for the packages that the plans select.
/// Why:
///  The step is skipped under `--fix`,
///  because the lint has no machine fix and a rewritten file would make the compiler's positions stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runClosureStep(plans: FilePlan[], cwd: Path, fix: boolean): ClosureOutcome;
/// ```
pub fn run_closure_step(plans: &[FilePlan], cwd: &Path, fix: bool) -> ClosureOutcome {
    return run_closure_step_with(plans, cwd, fix, &cargo_program());
}

/// What:
///  Run the closure check with a given `cargo` program.
/// Why:
///  The program is chosen by the machine through an environment variable;
///  taking it as an argument lets tests run the whole step against a stub without changing the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runClosureStepWith(plans: FilePlan[], cwd: Path, fix: boolean, cargo: Path): ClosureOutcome;
/// ```
pub fn run_closure_step_with(
    plans: &[FilePlan],
    cwd: &Path,
    fix: bool,
    cargo: &Path,
) -> ClosureOutcome {
    return run_closure_step_at(plans, cwd, fix, cargo, &driver_site_from_environment());
}

/// What:
///  Run the closure check with a given `cargo` program and the machine facts of the driver check.
/// Why:
///  The driver cache and the program that names the active toolchain come from the environment in production;
///  tests pass them explicitly so no test changes process-wide variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runClosureStepAt(plans: FilePlan[], cwd: Path, fix: boolean, cargo: Path, site: DriverSite): ClosureOutcome;
/// ```
pub fn run_closure_step_at(
    plans: &[FilePlan],
    cwd: &Path,
    fix: bool,
    cargo: &Path,
    site: &DriverSite,
) -> ClosureOutcome {
    let mut outcome: ClosureOutcome = ClosureOutcome::default();
    if fix {
        outcome.notes.push(format!("{RULE} is not run under --fix"));
        return outcome;
    }
    let grouping = group_packages(plans, cwd);
    outcome.notes.extend(grouping.notes);
    if grouping.packages.is_empty() {
        return outcome;
    }
    let library_failures: Vec<Diagnostic> = missing_libraries(grouping.packages.as_slice());
    if !library_failures.is_empty() {
        outcome.findings.extend(library_failures);
        return outcome;
    }
    let probe: Result<String, String> = dylint_probe(cargo);
    let machine_gap: Option<String> = match &probe {
        Err(reason) => Some(reason.clone()),
        Ok(_) => rustc_dev_missing(cargo),
    };
    if let Some(reason) = machine_gap {
        outcome.messages.push(skip_message(
            &format!("all {} package(s)", grouping.packages.len()),
            &reason,
        ));
        return outcome;
    }
    let gaps: DriverGaps = driver_gaps(
        grouping.packages.as_slice(),
        cargo,
        site,
        probe.unwrap_or_default().as_str(),
        cwd,
    );
    outcome.notes.extend(gaps.notes);
    for gap in &gaps.gaps {
        outcome
            .messages
            .push(skip_message(gap.subject.as_str(), gap.reason.as_str()));
    }
    let targets: Targets = targets_of(plans);
    let search: OsString = std::env::var_os("PATH").unwrap_or_default();
    for package in &grouping.packages {
        if gaps
            .gaps
            .iter()
            .any(|gap| return gap.library == package.library)
        {
            continue;
        }
        absorb(
            &mut outcome,
            check_package(package, cargo, &search, &targets, cwd),
        );
    }
    return outcome;
}

/// What:
///  Put the step's messages before the rest of a run's standard error.
/// Why:
///  The messages are not debug output,
/// so they appear on every run;
/// each line carries the program prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withMessages(stderr: string, messages: string[], program: string): string;
/// ```
pub fn with_messages(standard_error: &str, messages: &[String], program: &str) -> String {
    let mut text: String = String::new();
    for message in messages {
        text.push_str(format!("{program}: {message}\n").as_str());
    }
    text.push_str(standard_error);
    return text;
}

/// Step controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_step_tests.rs"]
mod tests;
