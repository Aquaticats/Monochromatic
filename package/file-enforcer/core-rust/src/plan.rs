//! What:
//!  Plan every rule against the virtual view and validate the whole run.
//! Why:
//!  Rules run in config order and each reads what earlier rules planned.
//! Every rule failure,
//! every pair of writers
//!  that cannot share a path,
//! and every rule that reads a later rule's output is collected before anything is
//!  published,
//! so a failing run writes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function planRun(root: string, rules: Rule[]): PlanReport
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Change`,
/// `Rule` from `crate::rule`.
use crate::rule::{Change, Rule};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// What:
///  Everything a run learned while planning.
///  `(String, String)` is a pair of owned text (TS:
/// `[string, string]`).
/// Why:
///  Drift detection,
/// publishing,
/// `explain` and `watch` all start from this one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanReport = { view: View; owners: Map<string, string>; removedDirectories: [string, string][]; warnings: Diagnostic[]; errors: Diagnostic[]; rules: [string, string][] };
/// ```
#[derive(Debug)]
pub struct PlanReport {
    /// The view,
    /// with the overlay of planned outputs and the record of reads.
    pub view: View,
    /// Path to the id of the last rule that planned it.
    pub owners: BTreeMap<String, String>,
    /// Directories rules want removed,
    /// with the rule id.
    pub removed_directories: Vec<(String, String)>,
    /// Warnings from rules.
    pub warnings: Vec<Diagnostic>,
    /// Failures from rules and from whole-run validation.
    pub errors: Vec<Diagnostic>,
    /// `(id, kind)` of every rule,
    /// in order.
    pub rules: Vec<(String, String)>,
}

/// What:
///  The path a change is about.
/// Why:
///  Conflict checks key on paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function changePath(change: Change): string
/// ```
fn change_path(change: &Change) -> &str {
    match change {
        Change::Write { path, .. } => return path,
        Change::Remove { path } => return path,
        Change::SetMode { path, .. } => return path,
        Change::RemoveDirectory { path } => return path,
    }
}

/// What:
///  Run every rule's `plan` in order and validate the run as a whole.
/// Why:
///  The planning half of every command;
/// publishing happens only when `errors` is empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planRun(root: string, rules: Rule[]): PlanReport
/// ```
pub fn plan_run(root: &Path, rules: &[Box<dyn Rule>]) -> PlanReport {
    let mut report = PlanReport {
        view: View::new(root),
        owners: BTreeMap::new(),
        removed_directories: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
        rules: Vec::new(),
    };
    let mut writers: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, rule) in rules.iter().enumerate() {
        report.rules.push((rule.id().to_string(), rule.kind().to_string()));
        report.view.begin_rule(index);
        let rule_plan = match rule.plan(&mut report.view) {
            Ok(rule_plan) => rule_plan,
            Err(error) => {
                report.errors.push(error.with_rule(rule.id()));
                continue;
            }
        };
        for warning in rule_plan.warnings {
            report.warnings.push(warning);
        }
        for change in &rule_plan.changes {
            let path = change_path(change).to_string();
            let earlier = writers.entry(path.clone()).or_default();
            for previous in earlier.iter() {
                if *previous != index && !(rules[*previous].composes() && rule.composes()) {
                    let message = format!(
                        "the rules {} and {} both write {path}, and they do not compose on one file",
                        rules[*previous].id(),
                        rule.id()
                    );
                    report.errors.push(Diagnostic::new("output-conflict", &message).with_path(&path));
                }
            }
            if !earlier.contains(&index) {
                earlier.push(index);
            }
            if let Change::RemoveDirectory { path: directory } = change {
                report.removed_directories.push((directory.clone(), rule.id().to_string()));
            }
            report.owners.insert(path, rule.id().to_string());
            if let Err(error) = report.view.apply(change) {
                report.errors.push(error.with_rule(rule.id()));
            }
        }
    }
    check_read_order(&mut report, rules, &writers);
    return report;
}

/// What:
///  Report a rule that read a path a later rule writes,
/// unless the reader writes that path itself or both rules compose.
/// Why:
///  Config order is execution order;
/// reading an output that is only planned later would see stale content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkReadOrder(report: PlanReport, rules: Rule[], writers: Map<string, number[]>): void
/// ```
fn check_read_order(report: &mut PlanReport, rules: &[Box<dyn Rule>], writers: &BTreeMap<String, Vec<usize>>) {
    let mut reported: Vec<String> = Vec::new();
    for (reader, path) in report.view.accesses() {
        let Some(path_writers) = writers.get(path) else { continue };
        if path_writers.contains(reader) {
            continue;
        }
        for writer in path_writers {
            // Composing rules are stages of one pipeline over a shared file: a later stage transforms what the
            // earlier stage planned (or left unchanged), so an earlier composing reader sees no stale content.
            if rules[*reader].composes() && rules[*writer].composes() {
                continue;
            }
            if writer > reader {
                let message = format!(
                    "the rule {} reads {path}, which the later rule {} writes; move {} before {}",
                    rules[*reader].id(),
                    rules[*writer].id(),
                    rules[*writer].id(),
                    rules[*reader].id()
                );
                if !reported.contains(&message) {
                    reported.push(message.clone());
                }
            }
        }
    }
    for message in reported {
        report.errors.push(Diagnostic::new("rule-order", &message));
    }
}
