//! What:
//!  Exact-snapshot and redacted-identity controls for the embedding interface.
//! Why:
//!  The standalone text protocol cannot be used as the source of candidate identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Scan caller buffers directly, then inspect structured fields and verify no matched bytes escaped.
//! ```

/// Import the real scanner and its closed finding catalog.
use super::{CandidateScan, Scanner};
use crate::{BIN_PROBE_SIZE, ScanFinding};
/// Ordinary string fixtures still enter the public API through its native Path boundary.
use std::path::Path;

/// Construct the production hybrid matcher without reading or publishing the user's runtime cache.
fn scanner() -> Scanner {
    return Scanner { loaded: crate::frx_load::test_rules("VAULTTOKEN_LONG\n") };
}

/// Path and content findings retain their independent positions while sharing the same masked display.
#[test]
fn structured_findings_keep_identity_when_paths_collide_after_masking() {
    let scanner: Scanner = scanner();
    let first: CandidateScan = scanner.scan(41, Path::new("VAULTTOKEN_LONG/a.txt"), b"clean\nVAULTTOKEN_LONG\n");
    let second: CandidateScan = scanner.scan(42, Path::new("VAULTTOKEN_LONG/a.txt"), b"clean\n");
    assert_eq!(first.identity, 41);
    assert_eq!(second.identity, 42);
    assert_eq!(first.display_path, "[REDACTED]/a.txt");
    assert_eq!(first.display_path, second.display_path);
    assert_eq!(first.findings, vec![
        ScanFinding::Name { component: 1, rule: String::from("0") },
        ScanFinding::Content { line: 2, rule: String::from("0") },
    ]);
    assert_eq!(second.findings.len(), 1);
    assert!(!format!("{first:?}").contains("VAULTTOKEN_LONG"));
}

/// A clean buffer cannot inherit the content of a live file sharing the caller's display name.
#[test]
fn scanner_uses_only_supplied_content_and_never_mutates_it() {
    let scanner: Scanner = scanner();
    let bytes: Vec<u8> = b"clean\n".to_vec();
    let snapshot: Vec<u8> = bytes.clone();
    let result: CandidateScan = scanner.scan(7, Path::new("not-created-on-disk.txt"), bytes.as_slice());
    assert!(result.findings.is_empty());
    assert_eq!(result.scanned_bytes, bytes.len());
    assert_eq!(bytes, snapshot);
    assert!(scanner.cache_warnings().is_empty());
}

/// The inherited binary policy scans the leading prefix while leaving later binary bytes uninspected.
#[test]
fn binary_prefix_behavior_matches_the_standalone_reader() {
    let scanner: Scanner = scanner();
    let mut binary: Vec<u8> = vec![0_u8; BIN_PROBE_SIZE];
    binary.extend_from_slice(b"\nVAULTTOKEN_LONG\n");
    let result: CandidateScan = scanner.scan(8, Path::new("image.bin"), binary.as_slice());
    assert_eq!(result.scanned_bytes, BIN_PROBE_SIZE);
    assert!(result.findings.is_empty());
    let mut text: Vec<u8> = vec![b'a'; BIN_PROBE_SIZE];
    text.extend_from_slice(b"\nVAULTTOKEN_LONG\n");
    let checked: CandidateScan = scanner.scan(9, Path::new("text.txt"), text.as_slice());
    assert_eq!(checked.scanned_bytes, text.len());
    assert_eq!(checked.findings, vec![ScanFinding::Content { line: 2, rule: String::from("0") }]);
}

/// Native non-UTF-8 names are matched as bytes and encoded only after the redaction decision.
#[cfg(unix)]
#[test]
fn native_path_bytes_are_not_replaced_before_matching() {
    // Unix's native extension constructs a pathname containing a byte UTF-8 cannot represent.
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;
    let scanner: Scanner = scanner();
    let harmless: PathBuf = PathBuf::from(OsString::from_vec(b"folder/\xffname".to_vec()));
    let clean: CandidateScan = scanner.scan(21, harmless.as_path(), b"");
    assert!(clean.findings.is_empty());
    assert_eq!(clean.display_path, "folder/\\xffname");
    let secret: PathBuf = PathBuf::from(OsString::from_vec(b"folder/\xffVAULTTOKEN_LONG".to_vec()));
    let matched: CandidateScan = scanner.scan(22, secret.as_path(), b"");
    assert_eq!(matched.display_path, "folder/[REDACTED]");
    assert_eq!(matched.findings, vec![ScanFinding::Name { component: 2, rule: String::from("0") }]);
    assert!(!format!("{matched:?}").contains("VAULTTOKEN_LONG"));
}

/// Unsupported line-breaking names remain explicit failures with no raw pathname in the report.
#[test]
fn pathname_failures_do_not_masquerade_as_clean_scans() {
    let result: CandidateScan = scanner().scan(11, Path::new("secret\nname"), b"");
    assert_eq!(result.display_path, "[REDACTED]");
    assert_eq!(result.findings, vec![ScanFinding::PathnameLineBreak]);
    assert!(!format!("{result:?}").contains("secret"));
}
