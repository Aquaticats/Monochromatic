//! What:
//!  Turn the compiler's reports into the linter's findings,
//!  with the severity and the exemptions of each file's own configuration.
//! Why:
//!  The check reports through the same JSONL channel as every other rule,
//!  so a finding carries the display path of a file that the run was asked to lint
//!  and the severity that the configuration selects for that file;
//!  a report in a file with the rule off,
//!  or in a file the run was not asked to lint,
//!  is dropped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Target = { display: string; severity: Severity };
//! function targetsOf(plans: FilePlan[]): Map<Path, Target>;
//! function findingsFrom(reports: Report[], packageDirectory: Path, targets: Map<Path, Target>): Diagnostic[];
//! ```

/// Import the finding model,
///  the report shape and the plan data.
use crate::{
    closure_report::Report,
    diagnostic::{Diagnostic, Severity, Span},
    markdown_lfs_target::lexical_normal,
    run_failure::host_span,
    run_paths::Language,
    run_plan::{FilePlan, RootRules},
};
/// Import an ordered map and native paths.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// What:
///  The rule identifier of every finding of the check.
/// Why:
///  The closure lint and the stale-expectation report belong to one suppression scheme,
///  so they share the rule that selects the check in configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RULE = 'rust/avoidable-closure';
/// ```
pub const RULE: &str = "rust/avoidable-closure";

/// What:
///  What the run knows about one file that selects the rule.
/// Why:
///  The compiler names a file by path;
///  the finding needs the name the run reports and the severity configured for that file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { display: string; severity: Severity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    /// The name findings report for the file.
    pub display: String,
    /// The severity the configuration selects for the file.
    pub severity: Severity,
}

/// The files of a run that select the rule,
///  keyed by their resolved path.
pub type Targets = BTreeMap<PathBuf, Target>;

/// What:
///  The key under which a file is looked up:
///  its resolved path when the file system answers,
///  otherwise its lexically normal form.
/// Why:
///  The compiler may print a path through a different symbolic link than the one the run was started with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyOf(path: Path): Path { try { return realpath(path); } catch { return normalize(path); } }
/// ```
pub fn key_of(path: &Path) -> PathBuf {
    return match std::fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(_) => lexical_normal(path),
    };
}

/// What:
///  Collect the Rust files of a run whose own rules select the check.
/// Why:
///  A file with the rule off,
///  or only a Markdown file,
///  has no entry,
///  so reports in it are dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function targetsOf(plans: FilePlan[]): Map<Path, Target>;
/// ```
pub fn targets_of(plans: &[FilePlan]) -> Targets {
    let mut targets: Targets = Targets::new();
    for plan in plans {
        if plan.language != Language::Rust {
            continue;
        }
        let RootRules::Rust { settings } = &plan.root else {
            continue;
        };
        let Some(selected) = &settings.avoidable_closure else {
            continue;
        };
        targets.insert(
            key_of(&plan.absolute),
            Target {
                display: plan.display.clone(),
                severity: selected.severity,
            },
        );
    }
    return targets;
}

/// What:
///  Place a report in its source file.
/// Why:
///  The compiler counts columns in characters,
///  while the linter's Rust findings count bytes,
///  so the position is recomputed from the report's byte offset in the file text;
///  without the text the compiler's own line and column stand in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spanOf(report: Report, source?: string): Span;
/// ```
pub fn span_of(report: &Report, source: Option<&str>) -> Span {
    let length: usize = report.byte_end.saturating_sub(report.byte_start);
    if let Some(text) = source
        && report.byte_start <= text.len()
    {
        let mut span: Span = host_span(Language::Rust, text, report.byte_start);
        span.length = length;
        return span;
    }
    return Span {
        offset: report.byte_start,
        length,
        line: report.line,
        column: report.column,
    };
}

/// What:
///  Map a package's reports to findings.
/// Why:
///  Reports are resolved against the package directory,
///  matched to the files of the run,
///  and sorted by file and position so the output does not depend on the order cargo built its targets in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsFrom(reports: Report[], packageDirectory: Path, targets: Map<Path, Target>): Diagnostic[];
/// ```
pub fn findings_from(
    reports: &[Report],
    package_directory: &Path,
    targets: &Targets,
) -> Vec<Diagnostic> {
    let mut sources: BTreeMap<PathBuf, Option<String>> = BTreeMap::new();
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for report in reports {
        let key: PathBuf = key_of(&package_directory.join(&report.file));
        let Some(target) = targets.get(&key) else {
            continue;
        };
        if !sources.contains_key(&key) {
            let text: Option<String> = std::fs::read_to_string(&key).ok();
            sources.insert(key.clone(), text);
        }
        let source: Option<&str> = sources.get(&key).and_then(Option::as_deref);
        let mut finding: Diagnostic = Diagnostic::new(
            RULE,
            target.severity,
            report.message.clone(),
            target.display.clone(),
            span_of(report, source),
        );
        finding.help = report.help.clone();
        findings.push(finding);
    }
    findings.sort_by(order);
    return findings;
}

/// What:
///  Order two findings by file and position.
/// Why:
///  A named comparison is the argument that `sort_by` takes without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function order(left: Diagnostic, right: Diagnostic): number;
/// ```
fn order(left: &Diagnostic, right: &Diagnostic) -> std::cmp::Ordering {
    let left_offset: usize = left.labels.first().map_or(0, offset_of);
    let right_offset: usize = right.labels.first().map_or(0, offset_of);
    return left
        .filename
        .cmp(&right.filename)
        .then(left_offset.cmp(&right_offset));
}

/// What:
///  The byte offset of a label.
/// Why:
///  A named mapping keeps `order` free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function offsetOf(label: Label): number { return label.span.offset; }
/// ```
fn offset_of(label: &crate::diagnostic::Label) -> usize {
    return label.span.offset;
}

/// Mapping controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_findings_tests.rs"]
mod tests;
