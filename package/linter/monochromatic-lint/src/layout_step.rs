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
    diagnostic::{
        Diagnostic,
        Severity,
        Span,
    },
    layout_allow_list::{
        AllowList,
        read_allow_list,
    },
    layout_crate_roots::crate_root_files,
    layout_file_order::check_file_order,
    layout_finding::LayoutFinding,
    layout_folder_links::check_folder_links,
    layout_module_tree::{
        ModuleTree,
        build_module_tree,
    },
    layout_module_uses::{
        ModuleUse,
        collect_uses,
    },
    layout_path_resolution::link_glob_targets,
    markdown_lfs_target::lexical_normal,
    run_failure::{
        file_start,
        processing_failure,
    },
    run_paths::{
        Language,
        display_name,
    },
    run_plan::{
        FilePlan,
        RootRules,
    },
    rust_manifest::discover_manifest,
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
///  The severities selected for one package,
///  and the allow-list file each selected rule names.
/// Why:
///  The first selecting file of the package decides each rule's severity,
///  and its configuration decides what `allow` is relative to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selection = { folder?: Severity; folderAllow?: Path; order?: Severity; orderAllow?: Path };
/// ```
#[derive(Clone, Debug, Default)]
struct Selection {
    /// Severity of the folder rule.
    folder: Option<Severity>,
    /// The allow-list file the folder rule tolerates findings with.
    folder_allow: Option<PathBuf>,
    /// Severity of the file rule.
    order: Option<Severity>,
    /// The allow-list file the file rule tolerates findings with.
    order_allow: Option<PathBuf>,
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
fn select_packages(
    plans: &[FilePlan],
    notes: &mut Vec<String>,
) -> BTreeMap<PathBuf, Selection> {
    let mut chosen: BTreeMap<PathBuf, Selection> = BTreeMap::<PathBuf, Selection>::new();
    for plan in plans {
        if plan.language != Language::Rust {
            continue;
        }
        let RootRules::Rust {
            settings,
        } = &plan.root
        else {
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
        if slot.folder.is_none()
            && let Some(rule) = &settings.folder_uses_links
        {
            slot.folder = Some(rule.severity);
            slot.folder_allow = rule.allow.as_ref().map(|path| {
                return lexical_normal(&plan.config.base.join(path));
            });
        }
        if slot.order.is_none()
            && let Some(rule) = &settings.file_order_in_folder
        {
            slot.order = Some(rule.severity);
            slot.order_allow = rule.allow.as_ref().map(|path| {
                return lexical_normal(&plan.config.base.join(path));
            });
        }
    }
    return chosen;
}

/// What:
///  The span of every line of each file this step reports findings in,
///  read once per file.
/// Why:
///  Findings carry a byte offset,
/// a line and a column,
///  and one file can carry many findings:
///  a list near its cap full of dead lines makes one finding per line about that one file,
///  so reading the file again for each finding cost one read per finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LineSpans { span(file: Path, line: number): Span }
/// ```
#[derive(Debug, Default)]
struct LineSpans {
    /// The lines of each file already read,
    ///  or nothing when the file could not be read.
    by_file: BTreeMap<PathBuf, Option<Vec<Span>>>,
}

/// What:
///  The span of each line of a file in file order,
///  or nothing when the file cannot be read.
/// Why:
///  One pass over the file answers every line it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLineSpans(file: Path): Span[] | undefined;
/// ```
fn read_line_spans(file: &Path) -> Option<Vec<Span>> {
    let text: String = std::fs::read_to_string(file).ok()?;
    let mut spans: Vec<Span> = Vec::<Span>::new();
    let mut start: usize = 0;
    for (index, part) in text.split_inclusive('\n').enumerate() {
        let length: usize = part.trim_end_matches(['\n', '\r']).len();
        spans.push(Span {
            offset: start,
            length,
            line: index + 1,
            column: 1,
        });
        start += part.len();
    }
    return Some(spans);
}

impl LineSpans {
    /// What:
    ///  The span of one line of one file,
    ///  or the file start when the line is zero,
    ///  the file does not hold it,
    ///  or the file cannot be read.
    /// Why:
    ///  A finding about a folder or a link has no line,
    ///  and a file that vanished between two findings must not fail the run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// span(file: Path, line: number): Span
    /// ```
    fn span(
        &mut self,
        file: &Path,
        line: usize,
    ) -> Span {
        if line == 0 {
            return file_start();
        }
        if !self.by_file.contains_key(file) {
            self.by_file
                .insert(file.to_path_buf(), read_line_spans(file));
        }
        let Some(Some(spans)) = self.by_file.get(file) else {
            return file_start();
        };
        return match spans.get(line - 1) {
            Some(span) => span.clone(),
            None => file_start(),
        };
    }
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
    spans: &mut LineSpans,
) -> Vec<Diagnostic> {
    let mut result: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for item in found {
        let mut diagnostic: Diagnostic = Diagnostic::new(
            code,
            severity,
            item.message,
            display_name(&item.file, cwd),
            spans.span(&item.file, item.line),
        );
        diagnostic.help = item.help;
        result.push(diagnostic);
    }
    return result;
}

/// What:
///  Read one rule's allow list,
///  or report why the rule cannot run.
/// Why:
///  A list that cannot be read would let every finding through unseen,
///  so the package fails the run instead of being checked without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listFor(path: Path | undefined, rule: string, cwd: Path, outcome: LayoutOutcome): AllowList | undefined;
/// ```
fn list_for(
    path: &Option<PathBuf>,
    rule: &str,
    cwd: &Path,
    outcome: &mut LayoutOutcome,
) -> Result<Option<AllowList>, ()> {
    let Some(named) = path else {
        return Ok(None);
    };
    match read_allow_list(named) {
        Ok(list) => return Ok(Some(list)),
        Err(reason) => {
            outcome.findings.push(processing_failure(
                display_name(named, cwd).as_str(),
                file_start(),
                format!(
                    "{rule} names an allow list that cannot be read, so the rule was not checked: {reason}"
                ),
            ));
            return Err(());
        }
    }
}

/// What:
///  Add one `--debug` note per finding that an allow-list entry could record.
/// Why:
///  A reader writes an entry by copying the key the rule reports,
///  and never by guessing its shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteKeys(found: LayoutFinding[], rule: string, outcome: LayoutOutcome): void;
/// ```
fn note_keys(
    found: &[LayoutFinding],
    rule: &str,
    outcome: &mut LayoutOutcome,
) {
    for item in found {
        if item.key.is_empty() {
            continue;
        }
        outcome
            .notes
            .push(format!("{rule}: allow-list key: {}", item.key));
    }
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
    link_glob_targets(&mut tree);
    let uses: Vec<ModuleUse> = collect_uses(&tree);
    outcome.notes.push(format!(
        "{}: the layout rules read {} module(s) and {} use(s)",
        display_name(package, cwd),
        tree.modules.len(),
        uses.len()
    ));
    if let Some(severity) = selection.folder
        && let Ok(allow) = list_for(&selection.folder_allow, FOLDER_RULE, cwd, outcome)
    {
        let found: Vec<LayoutFinding> =
            check_folder_links(&tree, uses.as_slice(), package, allow.as_ref());
        note_keys(&found, FOLDER_RULE, outcome);
        outcome
            .findings
            .extend(diagnostics(found, FOLDER_RULE, severity, cwd, spans));
    }
    if let Some(severity) = selection.order
        && let Ok(allow) = list_for(&selection.order_allow, FILE_RULE, cwd, outcome)
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
