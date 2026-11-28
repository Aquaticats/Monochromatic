//! What:
//!  The closure check as a step of a lint run:
//!  run the Dylint library once per selected Cargo package and turn its reports into findings.
//! Why:
//!  `avoidable_closure` needs the types and captures that only the compiler knows,
//!  so the linter asks the compiler through `cargo dylint` after the per-file rules have run.
//!  A package whose prerequisites are missing on the machine is skipped with a warning finding that names the package
//!  and the command that fixes it,
//!  so a skip is counted like any warning and never hides behind a green run,
//!  and a package that cannot be checked for any other reason fails the run with status 2.
//!  A check that exited `0` without printing one record of a compile is one of those: the package was never seen by the compiler.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ClosureOutcome = { findings: Diagnostic[]; skips: Diagnostic[]; notes: string[] };
//! function runClosureStep(plans: FilePlan[], cwd: Path, fix: boolean): ClosureOutcome;
//! ```

/// Re-export the outcome a step collects,
///  so the caller of the step asks this module for it.
pub use crate::closure_step_package::ClosureOutcome;
/// Import the per-package steps the step runs and merges.
use crate::closure_step_package::{
    absorb,
    check_package,
    skip_packages,
};
/// Import the processes,
///  the grouping and the failure finding of the check.
use crate::{
    closure_cargo::{
        ProbeFailure,
        cargo_program,
        dylint_probe,
        rustc_dev_missing,
    },
    closure_driver::{
        DriverGaps,
        DriverSite,
        driver_gaps,
        driver_site_from_environment,
        gaps_hold_library,
        unknown_reason,
    },
    closure_findings::{
        RULE,
        Targets,
        targets_of,
    },
    closure_packages::{
        ClosurePackage,
        group_packages,
    },
    diagnostic::Diagnostic,
    run_failure::{
        file_start,
        processing_failure,
    },
    run_plan::FilePlan,
};
/// Import the borrowed operating-system string and native paths.
use std::{
    ffi::OsString,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The packages that are checked with one Dylint library.
/// Why:
///  A missing driver belongs to a library,
///  and a named loop is the selection that `filter` would make with a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packagesOfLibrary(packages: ClosurePackage[], library: Path): ClosurePackage[];
/// ```
fn packages_of_library<'packages>(
    packages: &'packages [ClosurePackage],
    library: &Path,
) -> Vec<&'packages ClosurePackage> {
    let mut chosen: Vec<&ClosurePackage> = Vec::<&ClosurePackage>::new();
    for package in packages {
        if package.library.as_path() == library {
            chosen.push(package);
        }
    }
    return chosen;
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
                "{RULE} option library names {}, which holds no Cargo.toml of a Dylint library; no package was checked with it. Fix the path, which is resolved against this configuration's base folder: the working directory for `--config`, and the file's own folder for a discovered configuration.",
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
pub fn run_closure_step(
    plans: &[FilePlan],
    cwd: &Path,
    fix: bool,
) -> ClosureOutcome {
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
    let probe: Result<String, ProbeFailure> = dylint_probe(cargo);
    if let Err(ProbeFailure::Broken(reason)) = &probe {
        outcome.findings.push(processing_failure(
            grouping.packages[0].configuration.as_str(),
            file_start(),
            format!(
                "{RULE} could not check any package, and fails instead of skipping because nothing was checked: {reason}"
            ),
        ));
        return outcome;
    }
    let machine_gap: Option<String> = match &probe {
        Err(failure) => Some(failure.reason().to_string()),
        Ok(_) => rustc_dev_missing(cargo),
    };
    if let Some(reason) = machine_gap {
        let everything: Vec<&ClosurePackage> =
            grouping.packages.iter().collect::<Vec<&ClosurePackage>>();
        outcome
            .skips
            .extend(skip_packages(everything.as_slice(), reason.as_str(), cwd));
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
        let affected: Vec<&ClosurePackage> =
            packages_of_library(grouping.packages.as_slice(), &gap.library);
        outcome
            .skips
            .extend(skip_packages(affected.as_slice(), gap.reason.as_str(), cwd));
    }
    let targets: Targets = targets_of(plans);
    let search: OsString = std::env::var_os("PATH").unwrap_or_default();
    for package in &grouping.packages {
        if gaps_hold_library(gaps.gaps.as_slice(), &package.library) {
            continue;
        }
        let unchecked: Option<String> = unknown_reason(gaps.unknown.as_slice(), &package.library)
            .map(|reason| format!("The Dylint driver check could not be made: {reason}. "));
        absorb(
            &mut outcome,
            check_package(package, cargo, &search, &targets, cwd, unchecked.as_deref()),
        );
    }
    return outcome;
}

/// Step controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_step_tests.rs"]
mod tests;
