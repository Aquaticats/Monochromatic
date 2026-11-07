//! The unit tests of the report text:
//! silence when clean,
//! one prefixed line per finding,
//! the closing rule.

/// Imports the function and constants under test.
use crate::report::{LINE_PREFIX, RULE_SUMMARY, render_report};
/// Imports the scan result type.
use crate::scan::ScanReport;

/// Imports the rule,
/// to make real findings.
use monochromatic_catalog_specifier::check_manifest;
/// Imports the path type.
use std::path::Path;

/// A manifest with one literal version.
const LITERAL: &[u8] = br#"{ "name": "x", "dependencies": { "left-pad": "1.3.0" } }"#;

fn report_with(findings_from: &[&str], notes: &[&str]) -> ScanReport {
    let mut report: ScanReport = ScanReport { manifests_checked: 0, findings: Vec::new(), notes: Vec::new() };
    for path in findings_from {
        report.findings.extend(check_manifest(path, LITERAL));
    }
    for note in notes {
        report.notes.push(note.to_string());
    }
    return report;
}

#[test]
fn a_scan_with_nothing_to_say_is_silent() {
    assert_eq!(render_report(Path::new("/work"), &report_with(&[], &[])), None);
}

#[test]
fn every_line_carries_the_prefix_and_names_each_finding_and_note() {
    let report: ScanReport = report_with(&["a/package.json", "b/package.json"], &["could not read c/package.json: denied"]);

    let text: String = render_report(Path::new("/work"), &report).expect("a report");

    for line in text.lines() {
        assert!(line.starts_with(LINE_PREFIX), "line without the prefix: {line}");
    }
    for finding in &report.findings {
        assert!(text.contains(&finding.to_string()));
    }
    assert!(text.contains("could not read c/package.json: denied"));
    assert!(text.contains("/work"));
    assert!(text.ends_with('\n'));
}

#[test]
fn the_report_never_names_the_wrapper() {
    let report: ScanReport = report_with(&["a/package.json"], &["could not read b/package.json: denied"]);

    let text: String = render_report(Path::new("/work"), &report).expect("a report").to_lowercase();

    assert!(!text.contains("shadow"));
}

#[test]
fn the_report_ends_with_the_rule_and_says_pnpm_still_runs() {
    let text: String = render_report(Path::new("/work"), &report_with(&["a/package.json"], &[])).expect("a report");

    assert!(text.trim_end().ends_with(RULE_SUMMARY));
    assert!(text.contains("pnpm runs anyway"));
}

#[test]
fn the_count_in_the_first_line_matches_findings_plus_notes_and_agrees_in_number() {
    let one: String = render_report(Path::new("/work"), &report_with(&["a/package.json"], &[])).expect("a report");
    let report: ScanReport = report_with(&["a/package.json", "b/package.json"], &["a note"]);
    let total: usize = report.findings.len() + report.notes.len();
    let many: String = render_report(Path::new("/work"), &report).expect("a report");

    assert!(one.lines().next().expect("a first line").contains("1 problem in"));
    assert!(many.lines().next().expect("a first line").contains(&format!("{total} problems in")));
}
