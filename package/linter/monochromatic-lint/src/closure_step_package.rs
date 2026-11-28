//! What:
//!  One package's part of the closure check:
//!  the outcome a step collects,
//!  the warning finding of a package that was not checked,
//!  and the words of a run that could not be trusted.
//! Why:
//!  A package is skipped for a reason the machine or its manifest gives,
//!  or checked by running the Dylint library over it,
//!  and each of those exits leaves a finding the run must report;
//!  keeping them together lets the step itself hold only which packages run
//!  and in what order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ClosureOutcome = { findings: Diagnostic[]; skips: Diagnostic[]; notes: string[] };
//! function skipFinding(manifest: string, subject: string, reason: string): Diagnostic;
//! function checkPackage(pkg: ClosurePackage, context): ClosureOutcome;
//! ```

/// Import the processes,
///  the mapping and the failure finding of the check.
use crate::{
    closure_cargo::{
        DylintRun,
        dependencies_missing,
        run_dylint,
    },
    closure_findings::{
        RULE,
        Targets,
        findings_from,
    },
    closure_manifest::{
        Prerequisite,
        read_prerequisites,
    },
    closure_packages::ClosurePackage,
    closure_probe::{
        PackageSetup,
        Readiness,
        prepare,
    },
    closure_report::{
        Parsed,
        parse_records,
    },
    diagnostic::{
        Diagnostic,
        Severity,
    },
    run_failure::{
        file_start,
        processing_failure,
    },
    run_paths::display_name,
};
/// Import the borrowed operating-system string and a native path.
use std::{
    ffi::OsString,
    path::Path,
};

/// What:
///  What the step adds to a run.
/// Why:
///  Findings and skips both join the run's JSONL,
///  notes join the `--debug` notes;
///  skips are kept apart from findings so that a caller can tell a package that was not checked
///  from a package that was checked and reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClosureOutcome = { findings: Diagnostic[]; skips: Diagnostic[]; notes: string[] };
/// ```
#[derive(Debug, Default)]
pub struct ClosureOutcome {
    /// Findings of the check,
    ///  including processing failures.
    pub findings: Vec<Diagnostic>,
    /// One warning finding per package that was not checked,
    ///  each naming the package and the command that fixes the gap.
    pub skips: Vec<Diagnostic>,
    /// Execution notes for `--debug`.
    pub notes: Vec<String>,
}

/// What:
///  Build the warning finding of a package that was not checked.
/// Why:
///  The owner decided on 2026-11-07 that a skip is a warning and never a silent message,
///  because a cold-cache skip once hid five packages' findings behind a green run;
///  the finding sits at the start of the package's `Cargo.toml`,
///  one wording keeps the findings searchable by the rule name,
///  and the severity is fixed because the configured severity belongs to the closure reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skipFinding(manifest: string, subject: string, reason: string): Diagnostic;
/// ```
fn skip_finding(
    manifest: &str,
    subject: &str,
    reason: &str,
) -> Diagnostic {
    return Diagnostic::new(
        RULE,
        Severity::Warn,
        format!(
            "{RULE} skipped {subject}: {reason}. Nothing in this package was checked by this rule, so a run without findings says nothing about its closures."
        ),
        String::from(manifest),
        file_start(),
    );
}

/// What:
///  Skip each of several packages for one reason.
/// Why:
///  A machine-wide gap or a missing driver costs every package of a group,
///  and the warning must name each package so that the report shows how much went unchecked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skipPackages(packages: ClosurePackage[], reason: string, cwd: Path): Diagnostic[];
/// ```
pub(crate) fn skip_packages(
    packages: &[&ClosurePackage],
    reason: &str,
    cwd: &Path,
) -> Vec<Diagnostic> {
    let mut skips: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for package in packages {
        let Some(directory) = package.manifest.parent() else {
            continue;
        };
        skips.push(skip_finding(
            display_name(&package.manifest, cwd).as_str(),
            display_name(directory, cwd).as_str(),
            reason,
        ));
    }
    return skips;
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
fn tail(
    text: &str,
    count: usize,
) -> String {
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
fn failure_message(
    run: &DylintRun,
    parsed: &Parsed,
) -> String {
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
///  Explain a `cargo dylint` run that exited successfully but printed none of cargo's records.
/// Why:
///  `cargo dylint` exits `0` without checking anything when it finds no library package to load,
///  which is what a library whose toolchain is absent ends in,
///  so a package that the compiler never saw would otherwise be reported as a package without closures.
///  The message names the cause the driver check already knew,
///  and quotes the end of cargo's own messages,
///  which is where a cause it did not know is said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uncheckedMessage(run: DylintRun, subject: string, driverNote?: string): string;
/// ```
fn unchecked_message(
    run: &DylintRun,
    subject: &str,
    driver_note: Option<&str>,
) -> String {
    let known: String = match driver_note {
        Some(note) => format!("{note}. "),
        None => String::new(),
    };
    let said: String = tail(&run.stderr, 12);
    let messages: &str = if said.is_empty() {
        "none"
    } else {
        said.as_str()
    };
    return format!(
        "{RULE} could not check {subject}: `cargo dylint` exited with {} and printed none of cargo's records, so nothing in this package was checked by this rule and a run without findings says nothing about its closures. The result is incomplete, so the run cannot be trusted. {known}The end of cargo's messages was:\n{messages}",
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
///  A run that exits successfully without printing one record is a failure too:
///  the compiler never saw the package,
///  and the driver check's own note names the cause when it recorded one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkPackage(package: ClosurePackage, context, driverNote?: string): ClosureOutcome;
/// ```
pub(crate) fn check_package(
    package: &ClosurePackage,
    cargo: &Path,
    search: &OsString,
    targets: &Targets,
    cwd: &Path,
    driver_note: Option<&str>,
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
        outcome
            .skips
            .push(skip_finding(manifest_name.as_str(), &subject, &reason));
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
            outcome
                .skips
                .push(skip_finding(manifest_name.as_str(), &subject, &reason));
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
        return outcome;
    }
    if parsed.records == 0 {
        outcome.findings.push(processing_failure(
            manifest_name.as_str(),
            file_start(),
            unchecked_message(&run, subject.as_str(), driver_note),
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
pub(crate) fn absorb(
    total: &mut ClosureOutcome,
    part: ClosureOutcome,
) {
    total.findings.extend(part.findings);
    total.skips.extend(part.skips);
    total.notes.extend(part.notes);
}
