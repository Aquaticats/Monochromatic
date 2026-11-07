//! Unit tests of the post-run cleanup text:
//! zero,
//! one and many removals,
//! the verbose list,
//! and failures.

use std::path::{Path, PathBuf};

use crate::cleanup_report::{cleanup_lines, format_size};
use crate::prune::PruneReport;

const KIBIBYTE: u64 = 1024;
const MEBIBYTE: u64 = KIBIBYTE * 1024;

fn removed(count: usize, bytes: u64) -> PruneReport {
    let deleted: Vec<PathBuf> = (0..count).map(|index| return PathBuf::from(format!("/cargo/registry/cache/x/monochromatic-lint-0.{index}.0.crate"))).collect();
    return PruneReport { deleted, bytes, failed: Vec::new() };
}

#[test]
fn sizes_use_the_largest_binary_unit_above_one_with_one_decimal() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(KIBIBYTE - 1), "1023 B");
    assert_eq!(format_size(KIBIBYTE), "1.0 KiB");
    assert_eq!(format_size(KIBIBYTE + KIBIBYTE / 2), "1.5 KiB");
    assert_eq!(format_size(12 * MEBIBYTE + MEBIBYTE * 3 / 10), "12.3 MiB");
    assert_eq!(format_size(3 * MEBIBYTE * KIBIBYTE), "3.0 GiB");
}

#[test]
fn a_cleanup_that_removed_and_failed_nothing_prints_nothing() {
    assert!(cleanup_lines(&PruneReport::default(), Path::new("/cargo"), false).is_empty());
    assert!(cleanup_lines(&PruneReport::default(), Path::new("/cargo"), true).is_empty());
}

#[test]
fn one_removed_path_is_one_summary_line_that_says_it_is_post_run_cleanup_with_count_size_and_directory() {
    let lines = cleanup_lines(&removed(1, MEBIBYTE), Path::new("/cargo"), false);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line = &lines[0];
    assert!(line.starts_with("mpm-shadow: "), "{line}");
    for expected in ["post-run cleanup", "after the mpm command", "1 ", "1.0 MiB", "/cargo/registry"] {
        assert!(line.contains(expected), "{expected}: {line}");
    }
    assert!(!line.contains("deleted stale"), "{line}");
}

#[test]
fn many_removed_paths_are_still_one_line_by_default() {
    let lines = cleanup_lines(&removed(18, 5 * MEBIBYTE), Path::new("/cargo"), false);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("18 "), "{}", lines[0]);
    assert!(lines[0].contains("5.0 MiB"), "{}", lines[0]);
    assert!(lines[0].contains(MPM_SHADOW_VERBOSE), "the line names how to list them: {}", lines[0]);
}

/// The variable the summary line tells the reader about.
const MPM_SHADOW_VERBOSE: &str = crate::cleanup_report::VERBOSE_ENV;

#[test]
fn verbose_adds_one_line_per_removed_path_after_the_summary() {
    let report = removed(3, MEBIBYTE);
    let lines = cleanup_lines(&report, Path::new("/cargo"), true);
    assert_eq!(lines.len(), 1 + report.deleted.len(), "{lines:?}");
    assert!(lines[0].contains("post-run cleanup"), "{}", lines[0]);
    for (line, path) in lines[1..].iter().zip(&report.deleted) {
        assert_eq!(line, &format!("mpm-shadow: deleted stale {}", path.display()));
    }
}

#[test]
fn a_path_that_could_not_be_removed_is_always_listed_with_its_reason() {
    let mut report = removed(2, KIBIBYTE);
    report.failed.push((PathBuf::from("/cargo/registry/cache/x/monochromatic-lint-0.9.0.crate"), "Permission denied (os error 13)".to_string()));
    for verbose in [false, true] {
        let lines = cleanup_lines(&report, Path::new("/cargo"), verbose);
        let text = lines.join("\n");
        assert!(text.contains("monochromatic-lint-0.9.0.crate"), "{verbose}: {text}");
        assert!(text.contains("Permission denied (os error 13)"), "{verbose}: {text}");
        assert!(lines[0].contains("1 could not be removed"), "{verbose}: {}", lines[0]);
    }
}

#[test]
fn a_cleanup_that_only_failed_still_says_so() {
    let report = PruneReport {
        deleted: Vec::new(),
        bytes: 0,
        failed: vec![(PathBuf::from("/cargo/registry/cache/x/a.crate"), "Read-only file system (os error 30)".to_string())],
    };
    let lines = cleanup_lines(&report, Path::new("/cargo"), false);
    assert!(lines.len() >= 2, "{lines:?}");
    assert!(lines[0].contains("post-run cleanup") && lines[0].contains("0 "), "{}", lines[0]);
    assert!(lines.join("\n").contains("Read-only file system"), "{lines:?}");
}
