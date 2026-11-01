//! What:
//!  Exact binary-prefix boundaries and matcher reuse after a real caught bounds panic.
//! Why:
//!  Prefix truncation and fresh per-call scratch must remain observable at the embedding boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Test bytes on either side of the probe limit, then compare scanner results before and after an injected fault.
//! ```

/// Import the owning scanner and closed findings,
///  with native path rather than lossy pathname text.
use super::{CandidateScan, Scanner};
use crate::{BIN_PROBE_SIZE, ScanFinding};
use std::path::Path;

/// Construct cache-free production hybrid rules so neither tests nor mutations alter the user's cache.
fn scanner() -> Scanner {
    // The fixture contains both literal and regex matching branches.
    return Scanner { loaded: crate::frx_load::test_rules("VAULTTOKEN_LONG\n/RX[0-9]{2}/\n") };
}

/// Empty and shorter-than-probe snapshots still preserve their exact lengths and identity.
#[test]
fn empty_and_short_binary_snapshots_preserve_contract() {
    // Borrow a temporary native path and byte slice only during each synchronous call.
    let empty: CandidateScan = scanner().scan(0, Path::new("safe"), b"");
    assert_eq!(empty.identity, 0);
    assert_eq!(empty.scanned_bytes, 0);
    assert!(empty.findings.is_empty());
    let short: CandidateScan = scanner().scan(usize::MAX, Path::new("safe"), b"\0VAULTTOKEN_LONG");
    assert_eq!(short.identity, usize::MAX);
    assert_eq!(short.scanned_bytes, 16);
    // String::from owns a stable rule token; Vec's literal holds one typed record.
    assert_eq!(short.findings, vec![ScanFinding::Content { line: 1, rule: String::from("0") }]);
}

/// A NUL at the final probe byte truncates;
///  a NUL at the first tail byte does not.
#[test]
fn nul_on_either_side_of_probe_boundary_selects_exact_snapshot() {
    let scanner: Scanner = scanner();
    // Vec<u8> owns growable bytes, unlike borrowed &[u8] or fixed [u8; N], so the tail can be appended.
    let mut bytes: Vec<u8> = vec![b'a'; BIN_PROBE_SIZE];
    bytes[BIN_PROBE_SIZE - 1] = 0;
    // extend_from_slice copies the borrowed tail rather than changing its caller-owned data.
    bytes.extend_from_slice(b"\nVAULTTOKEN_LONG");
    let truncated: CandidateScan = scanner.scan(1, Path::new("safe"), bytes.as_slice());
    assert_eq!(truncated.scanned_bytes, BIN_PROBE_SIZE);
    assert!(truncated.findings.is_empty());
    bytes[BIN_PROBE_SIZE - 1] = b'a';
    bytes[BIN_PROBE_SIZE] = 0;
    let complete: CandidateScan = scanner.scan(2, Path::new("safe"), bytes.as_slice());
    assert_eq!(complete.scanned_bytes, bytes.len());
    assert_eq!(complete.findings, vec![ScanFinding::Content { line: 1, rule: String::from("0") }]);
}

/// An otherwise matching literal crossing a binary cutoff is not fabricated into a full match.
#[test]
fn binary_cutoff_preserves_hits_only_inside_the_prefix() {
    let scanner: Scanner = scanner();
    let mut bytes: Vec<u8> = vec![0; BIN_PROBE_SIZE - 4];
    bytes.extend_from_slice(b"VAULTTOKEN_LONG");
    assert!(scanner.scan(3, Path::new("safe"), bytes.as_slice()).findings.is_empty());
    // The positive control proves the same token is detected when fully inside the inspected window.
    let positive: CandidateScan = scanner.scan(4, Path::new("safe"), b"\0VAULTTOKEN_LONG\0");
    assert_eq!(positive.findings, vec![ScanFinding::Content { line: 1, rule: String::from("0") }]);
}

/// A real matcher panic caused by invalid internal offsets must not persist hit/scratch state.
#[test]
fn hybrid_matcher_reuses_per_call_state_after_a_caught_bounds_panic() {
    let scanner: Scanner = scanner();
    let before: CandidateScan = scanner.scan(5, Path::new("safe"), b"VAULTTOKEN_LONG\nRX42\n");
    assert_eq!(before.findings.len(), 2);
    // The malformed offset is injected only in this test, never through the public scanner API.
    let set = scanner.loaded.iter_sets().next().expect("fixture set");
    // AssertUnwindSafe explicitly allows catching borrowed engine state; the source audit checks fresh scratch separately.
    let operation = std::panic::AssertUnwindSafe(|| return set.matcher.line_matches(b"RX42", &[0, 99]));
    assert!(std::panic::catch_unwind(operation).is_err());
    let after: CandidateScan = scanner.scan(5, Path::new("safe"), b"VAULTTOKEN_LONG\nRX42\n");
    assert_eq!(before, after);
    assert!(scanner.scan(6, Path::new("safe"), b"ordinary").findings.is_empty());
}
