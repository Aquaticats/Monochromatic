//! Component-level matching,
//!  redaction,
//!  and path selection tests.

/// Imports production name scanner and logical-path selection.
use super::{logical_path, scan_path, scan_normalized_records};
use crate::ScanFinding;

/// Compiles runtime rules in memory without touching any user-owned cache.
fn load_rules(text: &str) -> crate::frx_load::LoadedRules {
    return crate::frx_load::test_rules(text);
}

/// Finds each component independently without exposing the offending spelling.
#[test]
fn matching_directory_and_filename_mask_every_printed_occurrence() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let hit = scan_path("VAULTTOKEN_LONG/VAULTTOKEN_LONG.txt", &loaded);
    assert_eq!(hit.display, "[REDACTED]/[REDACTED]");
    assert_eq!(hit.findings, vec![
        "[REDACTED]/[REDACTED]:name:1 rule=0",
        "[REDACTED]/[REDACTED]:name:2 rule=0",
    ]);
}

/// A rule containing a slash cannot consume bytes from adjacent components.
#[test]
fn slash_boundary_does_not_match() {
    let loaded = load_rules("/ALPHA_LONG.BETA_LONG/\n");
    let hit = scan_path("ALPHA_LONG/BETA_LONG", &loaded);
    assert_eq!(hit.display, "ALPHA_LONG/BETA_LONG");
    assert!(hit.findings.is_empty());
}

/// A clean pathname stays visible and contributes no name findings.
#[test]
fn clean_names_remain_readable() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let hit = scan_path("./src/clean.ts", &loaded);
    assert_eq!(hit.display, "./src/clean.ts");
    assert!(hit.findings.is_empty());
}

/// Short literals keep their existing word-byte boundary behavior in names.
#[test]
fn short_literal_respects_word_boundaries() {
    let loaded = load_rules("cat\n");
    assert!(scan_path("educate.txt", &loaded).findings.is_empty());
    let hit = scan_path("cat.txt", &loaded);
    assert_eq!(hit.findings, vec!["[REDACTED]:name:1 rule=0"]);
}

/// Named anchored regexes match one entire component and retain rule identity.
#[test]
fn named_anchored_rule_matches_one_name() {
    let loaded = load_rules("==> qqq-name <==\n/^VAULTTOKEN_LONG$/\n");
    assert!(scan_path("prefixVAULTTOKEN_LONG", &loaded).findings.is_empty());
    let hit = scan_path("src/VAULTTOKEN_LONG", &loaded);
    assert_eq!(hit.findings, vec!["src/[REDACTED]:name:2 rule=qqq-name"]);
}

/// Duplicate matches in one component yield one finding per rule and segment.
#[test]
fn repeated_name_match_is_deduplicated() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let hit = scan_path("VAULTTOKEN_LONG_VAULTTOKEN_LONG.ts", &loaded);
    assert_eq!(hit.findings, vec!["[REDACTED]:name:1 rule=0"]);
}

/// Embedded line breaks fail closed rather than altering anchored rule semantics.
#[test]
fn line_break_in_name_is_rejected_without_leaking_it() {
    let loaded = load_rules("/^VAULTTOKEN_LONG$/\n");
    for name in ["src/VAULTTOKEN_LONG\nfile.ts", "src/VAULTTOKEN_LONG\r"] {
        let hit = scan_path(name, &loaded);
        assert_eq!(hit.display, "[REDACTED]");
        assert_eq!(hit.findings, vec!["[REDACTED]: unsupported pathname line break"]);
    }
}

/// A literal locator-like suffix in a filename cannot impersonate a name finding.
#[test]
fn colons_in_visible_path_are_encoded() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let hit = scan_path("[REDACTED]:name/VAULTTOKEN_LONG", &loaded);
    assert_eq!(hit.display, "[REDACTED]\\x3aname/[REDACTED]");
    assert_eq!(hit.findings, vec!["[REDACTED]\\x3aname/[REDACTED]:name:2 rule=0"]);
}

/// Navigation markers do not count as directory names in locator positions.
#[test]
fn navigation_markers_are_not_segments() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let hit = scan_path("../VAULTTOKEN_LONG/a.txt", &loaded);
    assert_eq!(hit.display, "../[REDACTED]/a.txt");
    assert_eq!(hit.findings, vec!["../[REDACTED]/a.txt:name:1 rule=0"]);
}

/// A repo-local absolute positional file is named by its repository-relative path in native spelling.
///
/// Each root is paired with a file path built from that root:
///  the canonical root,
/// which Windows spells with a `\\?\` verbatim prefix,
/// and the plain temporary directory,
///  the non-verbatim form Windows `current_dir` reports.
/// The relative name is native (`nested\test.txt` on Windows),
///  and the pathname scan displays it with `/` everywhere.
#[test]
fn absolute_path_under_root_uses_relative_name() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let dir: std::path::PathBuf = std::env::temp_dir().join(format!("name-root-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("nested")).expect("create path fixture");
    std::fs::write(dir.join("nested").join("test.txt"), "clean").expect("write path fixture");
    let canonical: std::path::PathBuf = std::fs::canonicalize(&dir).expect("canonical root");
    // Joining with the host separator gives the expected native relative spelling on every platform.
    let expected: std::path::PathBuf = std::path::Path::new("nested").join("test.txt");
    for root in [canonical.as_path(), dir.as_path()] {
        let file: std::path::PathBuf = root.join("nested").join("test.txt");
        let logical: String = logical_path(file.to_str().expect("utf8 path"), Some(root));
        assert_eq!(logical.as_str(), expected.to_str().expect("utf8 expected name"), "root {root:?}");
        assert_eq!(scan_path(&logical, &loaded).display, "nested/test.txt", "root {root:?}");
    }
    std::fs::remove_dir_all(&dir).expect("remove path fixture");
}

/// A symlink is named by its own entry,
///  not the clean target it points at.
#[test]
#[cfg(unix)]
fn symlink_name_is_not_replaced_by_target() {
    let dir = std::env::temp_dir().join(format!("name-link-{}", std::process::id()));
    std::fs::create_dir_all(dir.join(".git")).expect("create repository marker");
    std::fs::write(dir.join("clean.txt"), "clean").expect("create link target");
    let link = dir.join("VAULTTOKEN_LONG");
    std::os::unix::fs::symlink(dir.join("clean.txt"), &link).expect("create symlink");
    let root = std::fs::canonicalize(&dir).expect("canonical root");
    assert_eq!(logical_path(link.to_str().expect("utf8 link"), Some(&root)), "VAULTTOKEN_LONG");
    std::fs::remove_dir_all(dir).expect("remove link fixture");
}

/// External and standalone positional names retain every supplied segment.
#[test]
fn external_name_keeps_supplied_segments() {
    assert_eq!(logical_path("/external/private/file.txt", None), "/external/private/file.txt");
    let root = std::path::Path::new("/repository");
    assert_eq!(logical_path("/external/private/file.txt", Some(root)), "/external/private/file.txt");
}

/// Explicit native-prefix boundaries exercise shared policy on every verification host.
#[test]
fn supplied_volume_prefix_skips_each_part_before_counting_names() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let drive = scan_normalized_records(b"C:/VAULTTOKEN_LONG/VAULTTOKEN_LONG", 1, &loaded);
    assert_eq!(drive.display, "C\\x3a/[REDACTED]/[REDACTED]");
    assert_eq!(drive.findings, vec![
        ScanFinding::Name { component: 1, rule: String::from("0") },
        ScanFinding::Name { component: 2, rule: String::from("0") },
    ]);
    let network = scan_normalized_records(b"//VAULTTOKEN_LONG/VAULTTOKEN_LONG/VAULTTOKEN_LONG", 2, &loaded);
    assert_eq!(network.display, "//VAULTTOKEN_LONG/VAULTTOKEN_LONG/[REDACTED]");
    assert_eq!(network.findings, vec![ScanFinding::Name { component: 1, rule: String::from("0") }]);
    let navigation = scan_normalized_records(b".//../VAULTTOKEN_LONG", 0, &loaded);
    assert_eq!(navigation.display, ".//../[REDACTED]");
    assert_eq!(navigation.findings, vec![ScanFinding::Name { component: 1, rule: String::from("0") }]);
}

/// Native Windows volume markers are not searchable directory-name segments.
#[test]
#[cfg(windows)]
fn windows_volume_prefix_is_not_name_segment() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let drive = scan_path("C:\\VAULTTOKEN_LONG\\clean.txt", &loaded);
    assert_eq!(drive.findings, vec!["C\\x3a/[REDACTED]/clean.txt:name:1 rule=0"]);
    let network = scan_path("\\\\server\\share\\VAULTTOKEN_LONG\\clean.txt", &loaded);
    assert_eq!(network.findings, vec!["//server/share/[REDACTED]/clean.txt:name:1 rule=0"]);
    let drive_relative = scan_path("C:VAULTTOKEN_LONG.txt", &loaded);
    assert_eq!(drive_relative.findings, vec!["C\\x3a/[REDACTED]:name:1 rule=0"]);
}

/// Native Windows device-namespace prefixes are skipped exactly,
///  so the following name is scanned.
#[test]
#[cfg(windows)]
fn windows_device_namespace_prefix_is_not_name_segment() {
    let loaded = load_rules("VAULTTOKEN_LONG\n");
    let port = scan_path("\\\\.\\COM1\\VAULTTOKEN_LONG\\clean.txt", &loaded);
    assert_eq!(port.findings, vec!["//./COM1/[REDACTED]/clean.txt:name:1 rule=0"]);
    let volume = scan_path("\\\\.\\C:\\VAULTTOKEN_LONG\\clean.txt", &loaded);
    assert_eq!(volume.findings, vec!["//./C\\x3a/[REDACTED]/clean.txt:name:1 rule=0"]);
    let pipe = scan_path("\\\\.\\pipe\\VAULTTOKEN_LONG.with.dots", &loaded);
    assert_eq!(pipe.findings, vec!["//./pipe/[REDACTED]:name:1 rule=0"]);
    let clean = scan_path("\\\\.\\COM1\\clean\\clean.txt", &loaded);
    assert_eq!(clean.display, "//./COM1/clean/clean.txt");
    assert!(clean.findings.is_empty());
}
