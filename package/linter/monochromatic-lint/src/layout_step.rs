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
    },
    layout_allow_list::{
        AllowList,
        read_allow_list,
    },
    layout_crate_roots::crate_root_files,
    layout_file_order::check_file_order,
    layout_finding::{
        LayoutFinding,
        key_names,
    },
    layout_folder_index_files::{
        UnlistableName,
        unlistable_folder_name_part,
        unlistable_name_part,
    },
    layout_folder_links::check_folder_links,
    layout_line_spans::LineSpans,
    layout_link_entries::USES_FOLDER,
    layout_module_tree::{
        ModuleTree,
        build_module_tree,
    },
    layout_module_uses::{
        ModuleUse,
        collect_uses,
        folder_of,
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
    collections::{
        BTreeMap,
        BTreeSet,
    },
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
///  The first path whose name no list line could hold,
///  with that path and the name written with the character escaped.
/// Why:
///  Every key about such a path holds the name,
///  and a line that ends in a trimmed character reads as if the character were not there,
///  while a line ends at a newline,
///  a `uses` line writes a folder as a path with `/` between its segments,
///  so a folder name that holds a backslash would name another folder on a machine that separates folders with it,
///  and a folder name that ends or begins with the words a `uses` line sets its arrow off with,
///  or that is the arrow alone,
///  leaves that line unable to say which of its two folders is the one used,
///  so no finding about it could be written down;
///  a run over such a tree would report findings no list can hold.
///  The caller passes the paths the keys are built from,
///  so the parts checked are exactly those parts,
///  and the last part of each is a name rather than a folder,
///  because a file name and an entry of a `uses` folder are no paths:
///  their keys write them as the source does, backslash and arrow words included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistablePath(paths: Path[], base: Path): [Path, string, UnlistableName] | undefined;
/// ```
fn unlistable_path(
    paths: &[PathBuf],
    base: &Path,
) -> Option<(PathBuf, String, UnlistableName)> {
    for path in paths {
        let mut parts: Vec<String> = Vec::<String>::new();
        let mut unlistable: Option<UnlistableName> = None;
        // What: each name a key writes, as the file system spells it.
        // Why: a key writes a name's own backslash as written, so the parts of the written path cannot show one.
        let names: Vec<String> = key_names(path, base);
        let last: usize = names.len().saturating_sub(1);
        for (index, part) in names.iter().enumerate() {
            // What: whether this part is a folder, which a `uses` line must name as a path, or the name the path ends in.
            // Why: only a folder name can leave a `uses` line unable to say where one folder ends.
            let folder: bool = index != last;
            let checked: Option<(String, UnlistableName)> = if folder {
                unlistable_folder_name_part(part.as_str())
            } else {
                unlistable_name_part(part.as_str())
            };
            match checked {
                Some((escaped, reason)) => {
                    if unlistable.is_none() {
                        unlistable = Some(reason);
                    }
                    parts.push(escaped);
                }
                None => parts.push(part.clone()),
            }
        }
        if let Some(reason) = unlistable {
            return Some((path.clone(), parts.join("/"), reason));
        }
    }
    return None;
}

/// What:
///  The first module file of a package whose name no list line could hold.
/// Why:
///  The walk of the module tree names the files and folders every key is built from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistableName(tree: ModuleTree, base: Path): [Path, string, UnlistableName] | undefined;
/// ```
fn unlistable_name(
    tree: &ModuleTree,
    base: &Path,
) -> Option<(PathBuf, String, UnlistableName)> {
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for module in &tree.modules {
        paths.push(module.file.clone());
    }
    return unlistable_path(paths.as_slice(), base);
}

/// What:
///  The first entry of a `uses` folder of the package whose name no list line could hold.
/// Why:
///  A link's key holds the entry's own name,
///  so an entry named with a trailing space or a line break keys a finding no line could hold,
///  although no module file of the package carries that name.
///  An entry is no path and never appears on a `uses` line,
///  so its key writes its name as the source does:
///  a backslash or the arrow words are no reason to refuse it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistableLinkName(tree: ModuleTree, base: Path): [Path, string, UnlistableName] | undefined;
/// ```
fn unlistable_link_name(
    tree: &ModuleTree,
    base: &Path,
) -> Option<(PathBuf, String, UnlistableName)> {
    let mut folders: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for index in 0..tree.modules.len() {
        folders.insert(folder_of(tree, index));
    }
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for folder in &folders {
        let uses: PathBuf = folder.join(USES_FOLDER);
        let Ok(listing) = std::fs::read_dir(&uses) else {
            continue;
        };
        for entry in listing.flatten() {
            paths.push(entry.path());
        }
    }
    paths.sort();
    // What: an entry of a `uses` folder is a name and no path, so only the character the reader trims and a line break forbid it.
    // Why: its key writes the entry as written, and no `uses` line names an entry.
    return unlistable_path(paths.as_slice(), base);
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
    // What: the first name of the package the list reader could never read back.
    // Why: the rules cannot answer for a tree whose findings no line could hold,
    // and an entry of a `uses` folder is such a name as much as a module file is.
    let unlistable: Option<(PathBuf, String, UnlistableName)> =
        unlistable_name(&tree, package).or_else(|| return unlistable_link_name(&tree, package));
    if let Some((file, written, reason)) = unlistable {
        // What: the reason, and what renaming the file must give, in the words of the refusal.
        // Why: a name fails for one of four causes, and the reader must know which one to fix.
        let (why, fix): (&str, &str) = match reason {
            UnlistableName::EndsInTrimmedWhitespace => (
                "its name ends in whitespace that an allow-list reader trims away",
                "its name ends in a visible character",
            ),
            UnlistableName::HoldsLineBreak => (
                "its name holds a line break, which ends a list line",
                "its name holds no line break",
            ),
            UnlistableName::HoldsBackslash => (
                "its name holds a backslash, which a `uses` line writes as the folder separator `/` on a machine that separates folders with it, so such a line would name another folder",
                "its name holds no backslash",
            ),
            UnlistableName::HoldsArrowWords => (
                "its name holds ` -> `, begins with `-> `, ends with ` ->`, or is `->`, so a `uses` line naming it would read as another pair of folders",
                "its name holds no arrow word at its beginning or its end",
            ),
        };
        outcome.findings.push(processing_failure(
            display_name(&file, cwd).as_str(),
            spans.span(&file, 1),
            format!(
                "The layout rules stop at {written}, because {why}, so no key could name it and no line could tolerate a finding about it. Rename it so {fix}, then run the rules again."
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
