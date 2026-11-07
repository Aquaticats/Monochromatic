//! What:
//!  The two layout rules as a step of a lint run:
//!  find the Cargo packages that the selected files belong to,
//!  build each package's module tree once,
//!  and report the findings of the folder rule and the file rule.
//! Why:
//!  Both rules need the whole package,
//! not one parsed file,
//!  so they run after the per-file rules,
//! once per package,
//!  as the closure check does.
//!  They are off unless configuration turns them on for some file of the package.
//!  A package whose tree cannot be completed (a missing module file,
//! an unreadable file)
//!  fails the run with a processing failure and is not checked,
//!  because a partial tree would give wrong answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LayoutOutcome = { findings: Diagnostic[]; notes: string[] };
//! function runLayoutStep(plans: FilePlan[], cwd: Path): LayoutOutcome;
//! ```

/// Import every part of the layout checks and the run's plan,
/// finding and path types.
use crate::{
    diagnostic::{Diagnostic, Severity, Span},
    layout_crate_roots::crate_root_files,
    layout_file_order::check_file_order,
    layout_finding::LayoutFinding,
    layout_folder_links::check_folder_links,
    layout_module_tree::{ModuleTree, build_module_tree},
    layout_module_uses::{ModuleUse, collect_uses},
    layout_path_resolution::link_glob_targets,
    run_failure::{file_start, processing_failure},
    run_paths::{Language, display_name},
    run_plan::{FilePlan, RootRules},
    rust_manifest::discover_manifest,
};
/// Import an ordered map and native paths.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// The diagnostic code of the folder rule.
pub const FOLDER_RULE: &str = "rust/folder-uses-links";

/// The diagnostic code of the file rule.
pub const FILE_RULE: &str = "rust/file-order-in-folder";

/// What:
///  What the step adds to a run.
/// Why:
///  Findings join the run's JSONL and notes join the `--debug` notes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LayoutOutcome = { findings: Diagnostic[]; notes: string[] };
/// ```
#[derive(Debug, Default)]
pub struct LayoutOutcome {
    /// Findings of both rules and processing failures.
    pub findings: Vec<Diagnostic>,
    /// Execution notes for `--debug`.
    pub notes: Vec<String>,
}

/// What:
///  The severities selected for one package.
/// Why:
///  The first selecting file of the package decides each rule's severity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selection = { folder?: Severity; order?: Severity };
/// ```
#[derive(Clone, Copy, Debug, Default)]
struct Selection {
    /// Severity of the folder rule.
    folder: Option<Severity>,
    /// Severity of the file rule.
    order: Option<Severity>,
}

/// What:
///  Group the selecting plans by owning package.
/// Why:
///  A package is checked once however many of its files select the rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selectPackages(plans: FilePlan[], notes: string[]): Map<Path, Selection>;
/// ```
fn select_packages(plans: &[FilePlan], notes: &mut Vec<String>) -> BTreeMap<PathBuf, Selection> {
    let mut chosen: BTreeMap<PathBuf, Selection> = BTreeMap::<PathBuf, Selection>::new();
    for plan in plans {
        if plan.language != Language::Rust {
            continue;
        }
        let RootRules::Rust { settings } = &plan.root else {
            continue;
        };
        if settings.folder_uses_links.is_none() && settings.file_order_in_folder.is_none() {
            continue;
        }
        let manifest: PathBuf = match discover_manifest(&plan.absolute) {
            Ok(found) => found,
            Err(error) => {
                notes.push(format!(
                    "{}: not checked by the layout rules: {error}",
                    plan.display
                ));
                continue;
            }
        };
        let slot: &mut Selection = chosen.entry(manifest).or_default();
        if slot.folder.is_none() {
            slot.folder = settings.folder_uses_links;
        }
        if slot.order.is_none() {
            slot.order = settings.file_order_in_folder;
        }
    }
    return chosen;
}

/// What:
///  The span of a whole line of a file,
///  or the file start when the line is zero or the file cannot be read.
/// Why:
///  Findings carry a byte offset,
/// a line and a column.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineSpan(file: Path, line: number): Span;
/// ```
fn line_span(file: &Path, line: usize) -> Span {
    if line == 0 {
        return file_start();
    }
    let Ok(text) = std::fs::read_to_string(file) else {
        return file_start();
    };
    let mut start: usize = 0;
    for (index, part) in text.split_inclusive('\n').enumerate() {
        if index + 1 == line {
            let length: usize = part.trim_end_matches(['\n', '\r']).len();
            return Span {
                offset: start,
                length,
                line,
                column: 1,
            };
        }
        start += part.len();
    }
    return file_start();
}

/// What:
///  Turn the findings of one rule into diagnostics.
/// Why:
///  The rule's code and severity come from the step;
///  the help text travels with the finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function diagnostics(found: LayoutFinding[], code: string, severity: Severity, cwd: Path): Diagnostic[];
/// ```
fn diagnostics(
    found: Vec<LayoutFinding>,
    code: &str,
    severity: Severity,
    cwd: &Path,
) -> Vec<Diagnostic> {
    let mut result: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for item in found {
        let mut diagnostic: Diagnostic = Diagnostic::new(
            code,
            severity,
            item.message,
            display_name(&item.file, cwd),
            line_span(&item.file, item.line),
        );
        diagnostic.help = item.help;
        result.push(diagnostic);
    }
    return result;
}

/// What:
///  Check one package.
/// Why:
///  Root discovery and tree problems end the package early with a processing failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkPackage(manifest: Path, selection: Selection, cwd: Path, outcome: LayoutOutcome): void;
/// ```
fn check_package(manifest: &Path, selection: Selection, cwd: &Path, outcome: &mut LayoutOutcome) {
    let Some(package) = manifest.parent() else {
        return;
    };
    let roots: Vec<PathBuf> = match crate_root_files(manifest) {
        Ok(found) => found,
        Err(reason) => {
            outcome.findings.push(processing_failure(
                display_name(manifest, cwd).as_str(),
                file_start(),
                format!("The layout rules could not find the roots of this package: {reason}"),
            ));
            return;
        }
    };
    if roots.is_empty() {
        outcome.notes.push(format!(
            "{}: the layout rules found no lib.rs, main.rs or src/bin root",
            display_name(manifest, cwd)
        ));
        return;
    }
    let mut tree: ModuleTree = build_module_tree(roots.as_slice());
    if !tree.problems.is_empty() {
        for problem in &tree.problems {
            outcome.findings.push(processing_failure(
                display_name(&problem.file, cwd).as_str(),
                line_span(&problem.file, problem.line),
                format!(
                    "The layout rules could not complete the module tree of {}, so the package was not checked: {}.",
                    display_name(package, cwd),
                    problem.message
                ),
            ));
        }
        return;
    }
    link_glob_targets(&mut tree);
    let uses: Vec<ModuleUse> = collect_uses(&tree);
    outcome.notes.push(format!(
        "{}: the layout rules read {} module(s) and {} use(s)",
        display_name(package, cwd),
        tree.modules.len(),
        uses.len()
    ));
    if let Some(severity) = selection.folder {
        let found: Vec<LayoutFinding> = check_folder_links(&tree, uses.as_slice(), package);
        outcome
            .findings
            .extend(diagnostics(found, FOLDER_RULE, severity, cwd));
    }
    if let Some(severity) = selection.order {
        let found: Vec<LayoutFinding> = check_file_order(&tree, uses.as_slice(), package);
        outcome
            .findings
            .extend(diagnostics(found, FILE_RULE, severity, cwd));
    }
}

/// What:
///  Run both layout rules for the packages that the plans select.
/// Why:
///  Nothing runs unless configuration selects a rule for some Rust file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runLayoutStep(plans: FilePlan[], cwd: Path): LayoutOutcome;
/// ```
pub fn run_layout_step(plans: &[FilePlan], cwd: &Path) -> LayoutOutcome {
    let mut outcome: LayoutOutcome = LayoutOutcome::default();
    let mut notes: Vec<String> = Vec::<String>::new();
    let chosen: BTreeMap<PathBuf, Selection> = select_packages(plans, &mut notes);
    outcome.notes.extend(notes);
    for (manifest, selection) in &chosen {
        check_package(manifest, *selection, cwd, &mut outcome);
    }
    return outcome;
}

/// Controls of the layout step stay outside release artifacts.
#[cfg(test)]
#[path = "layout_step_tests.rs"]
mod tests;
