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
//!  Where each part of the step lives:
//!  `layout_step_packages` holds the packages the plans select and each rule's severity and allow list,
//!  `layout_step_diagnostics` holds the diagnostics and `--debug` notes a rule's findings become,
//!  and `layout_step_unlistable_names` holds the names no allow-list line could hold.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LayoutOutcome = { findings: Diagnostic[]; notes: string[] };
//! function runLayoutStep(plans: FilePlan[], cwd: Path): LayoutOutcome;
//! ```

/// Import every part of the layout checks,
///  the run's plan,
///  and the finding and path types.
use crate::{
    diagnostic::Diagnostic,
    layout_crate_roots::crate_root_files,
    layout_file_order::check_file_order,
    layout_finding::LayoutFinding,
    layout_folder_index_files::UnlistableName,
    layout_folder_links::check_folder_links,
    layout_line_spans::LineSpans,
    layout_module_tree::{
        ModuleTree,
        build_module_tree,
    },
    layout_module_uses::{
        ModuleUse,
        collect_uses,
    },
    layout_path_resolution::link_glob_targets,
    layout_step_diagnostics::{
        diagnostics,
        list_for,
        note_keys,
    },
    layout_step_packages::{
        Selection,
        select_packages,
    },
    layout_step_unlistable_names::{
        UnlistableFound,
        unlistable_link_name,
        unlistable_name,
    },
    run_failure::{
        file_start,
        processing_failure,
    },
    run_paths::display_name,
    run_plan::FilePlan,
};
/// Import an ordered map and native paths.
use std::{
    collections::BTreeMap,
    path::{
        Path,
        PathBuf,
    },
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
///  Check one package.
/// Why:
///  Root discovery and tree problems end the package early with a processing failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkPackage(manifest: Path, selection: Selection, cwd: Path, outcome: LayoutOutcome): void;
/// ```
fn check_package(
    manifest: &Path,
    selection: &Selection,
    cwd: &Path,
    outcome: &mut LayoutOutcome,
    spans: &mut LineSpans,
) {
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
                spans.span(&problem.file, problem.line),
                format!(
                    "The layout rules could not complete the module tree of {}, so the package was not checked: {}.",
                    display_name(package, cwd),
                    problem.message
                ),
            ));
        }
        return;
    }
    // What: whether a folder name that a `uses` line could not carry stops the run.
    // Why: the file rule reads no `uses` line, so it names such a folder in a key, and a key holds the name as written.
    let arrow_words_are_refused: bool = selection.folder.is_some();
    // What: the first name of the package the list reader could never read back.
    // Why: the rules cannot answer for a tree whose findings no line could hold,
    // and an entry of a `uses` folder is such a name as much as a module file is.
    let unlistable: Option<UnlistableFound> = unlistable_name(&tree, package, arrow_words_are_refused)
        .or_else(|| return unlistable_link_name(&tree, package, arrow_words_are_refused));
    if let Some(found) = unlistable {
        // What: the reason, and what renaming the name must give, in the words of the refusal.
        // Why: a name fails for one cause, and the reader must know which one to fix.
        let (why, fix): (String, &str) = match found.reason {
            UnlistableName::EndsInTrimmedWhitespace => (
                String::from("its name ends in whitespace that an allow-list reader trims away"),
                "its name ends in a visible character",
            ),
            UnlistableName::HoldsLineBreak => (
                String::from("its name holds a line break, which ends a list line"),
                "its name holds no line break",
            ),
            UnlistableName::HoldsBackslash => (
                String::from(
                    "its name holds a backslash, which a `uses` line writes as the folder separator `/` on a machine that separates folders with it, so such a line would name another folder",
                ),
                "its name holds no backslash",
            ),
            // What: the one arrow shape this name has, and what a line naming it would read as.
            // Why: four alternatives in one sentence leave the reader to work out which one is theirs.
            UnlistableName::HoldsArrowWords => (
                format!(
                    "its name {}, so a `uses` line naming it would read as another pair of folders",
                    found.shape.unwrap_or("holds an arrow word")
                ),
                "its name holds no arrow word at its beginning or its end",
            ),
        };
        // What: the name to rename, which is a folder inside the path or the file the path ends in.
        // Why: the file the finding points at is not always the name to change, and a sentence that named it would send its reader to the wrong place.
        let (subject, noun): (String, &str) = if found.is_folder {
            (format!("the folder {}", found.written), "the folder")
        } else {
            (found.written.clone(), "the file")
        };
        outcome.findings.push(processing_failure(
            display_name(&found.file, cwd).as_str(),
            spans.span(&found.file, 1),
            format!(
                "The layout rules stop at {subject}, because {why}. No key could name it and no line could tolerate a finding about it, so rename {noun} so {fix}, then run the rules again."
            ),
        ));
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
    if let Some(severity) = selection.folder
        && let Ok(allow) = list_for(&selection.folder_allow, package, FOLDER_RULE, cwd, outcome)
    {
        let found: Vec<LayoutFinding> =
            check_folder_links(&tree, uses.as_slice(), package, allow.as_ref());
        note_keys(&found, FOLDER_RULE, outcome);
        outcome
            .findings
            .extend(diagnostics(found, FOLDER_RULE, severity, cwd, spans));
    }
    if let Some(severity) = selection.order
        && let Ok(allow) = list_for(&selection.order_allow, package, FILE_RULE, cwd, outcome)
    {
        let found: Vec<LayoutFinding> =
            check_file_order(&tree, uses.as_slice(), package, allow.as_ref());
        note_keys(&found, FILE_RULE, outcome);
        outcome
            .findings
            .extend(diagnostics(found, FILE_RULE, severity, cwd, spans));
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
pub fn run_layout_step(
    plans: &[FilePlan],
    cwd: &Path,
) -> LayoutOutcome {
    let mut outcome: LayoutOutcome = LayoutOutcome::default();
    let mut notes: Vec<String> = Vec::<String>::new();
    let chosen: BTreeMap<PathBuf, Selection> = select_packages(plans, &mut notes);
    outcome.notes.extend(notes);
    let mut spans: LineSpans = LineSpans::default();
    for (manifest, selection) in &chosen {
        check_package(manifest, selection, cwd, &mut outcome, &mut spans);
    }
    return outcome;
}

/// Controls of the layout step stay outside release artifacts.
#[cfg(test)]
#[path = "layout_step_tests.rs"]
mod tests;
