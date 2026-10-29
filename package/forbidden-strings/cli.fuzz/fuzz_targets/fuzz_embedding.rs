//! What:
//!  Native snapshot scanning against an independent fixed-rule byte oracle.
//! Why:
//!  Lossy paths,
//!  prefix truncation,
//!  identity collisions and matcher reuse belong to the public embedding boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Scan arbitrary native names and bytes, then compare content records to a plain byte-search oracle.
//! ```

#![no_main]

/// Import the existing fuzz harness,
///  public findings and cache-free production hybrid construction path.
use libfuzzer_sys::fuzz_target;
use forbidden_strings::{CandidateScan, Scanner, ScanFinding};
use forbidden_strings::fuzz_api::scanner_from_text_for_fuzzing;
/// OnceLock builds a fixture once,
///  unlike Mutex/RwLock which would permit mutation on each scan.
use std::sync::OnceLock;
/// Native Unix path construction preserves every byte,
///  rather than String's UTF-8 restriction.
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// The probe limit is part of the inherited standalone contract,
///  not a fuzzer-specific cap.
const PROBE: usize = 8192;
/// One immutable compiled hybrid set,
///  shared by sequential fuzz iterations to exercise reuse.
static SCANNER: OnceLock<Scanner> = OnceLock::new();

/// Initialize both production matcher subsets with stable non-secret names.
fn scanner() -> Scanner {
    // expect fails only on the fixed fixture, never on arbitrary private bytes.
    return scanner_from_text_for_fuzzing("==> literal <==\nEMBEDDED_NEEDLE_LONG\n==> regex <==\n/RX[0-9]{2}/\n")
        .expect("fixed embedding rules compile");
}

/// Collect expected content records without calling any matcher,
///  splitter or renderer from the scanner.
fn expected(bytes: &[u8]) -> Vec<ScanFinding> {
    // Vec owns a variable number of records, rather than borrowed &[ScanFinding] or a fixed array.
    let mut findings: Vec<ScanFinding> = Vec::new();
    // split lends line slices and preserves interior empty lines for one-based numbering.
    for (index, raw) in bytes.split(|byte| return *byte == b'\n').enumerate() {
        // Strip one CR just like the documented physical-line contract, not arbitrary whitespace.
        let line: &[u8] = raw.strip_suffix(b"\r").unwrap_or(raw);
        // A window comparison is an independent exact-byte literal search.
        if line.windows(20).any(|window| return window == b"EMBEDDED_NEEDLE_LONG") {
            findings.push(ScanFinding::Content { line: index + 1, rule: String::from("literal") });
        }
        // The second rule has exactly two ASCII decimal digits after RX.
        if line.windows(4).any(|window| return window[0..2] == *b"RX" && window[2].is_ascii_digit() && window[3].is_ascii_digit()) {
            findings.push(ScanFinding::Content { line: index + 1, rule: String::from("regex") });
        }
    }
    // Return the independently ordered typed records, never a reconstruction of terminal text.
    return findings;
}

/// Select content records without depending on pathname masking or rendered text.
fn content_findings(report: &CandidateScan) -> Vec<ScanFinding> {
    let mut records: Vec<ScanFinding> = Vec::new();
    // Borrow each enum; clone only the expected location/token record, never the input bytes.
    for finding in &report.findings {
        if let ScanFinding::Content { .. } = finding {
            records.push(finding.clone());
        }
    }
    return records;
}

/// Independently enumerate native Unix pathname findings,
///  rejecting failures for these valid fixed matchers.
fn name_findings(input: &[u8]) -> Vec<ScanFinding> {
    // Line-breaking names intentionally fail closed before any component matching occurs.
    if input.contains(&b'\n') || input.contains(&b'\r') {
        return vec![ScanFinding::PathnameLineBreak];
    }
    let mut findings: Vec<ScanFinding> = Vec::new();
    let mut position: usize = 0;
    for component in input.split(|byte| return *byte == b'/') {
        if component.is_empty() || component == b"." || component == b".." {
            continue;
        }
        position += 1;
        // The independent byte-search oracle supplies rule names, never the scanner's result.
        for finding in expected(component) {
            if let ScanFinding::Content { rule, .. } = finding {
                findings.push(ScanFinding::Name { component: position, rule });
            }
        }
    }
    return findings;
}

/// Exercise public scan,
///  native bytes,
///  redaction,
///  independent content semantics and state reuse.
fn verify(input: &[u8]) {
    let scanner: &Scanner = SCANNER.get_or_init(scanner);
    // Independent positive controls prove both oracle branches can report real matches.
    let positive: CandidateScan = scanner.scan(17, Path::new("EMBEDDED_NEEDLE_LONG/RX42"), b"EMBEDDED_NEEDLE_LONG\nRX42\n");
    assert_eq!(positive.display_path, "[REDACTED]/[REDACTED]");
    assert_eq!(content_findings(&positive), expected(b"EMBEDDED_NEEDLE_LONG\nRX42\n"));
    assert_eq!(positive.findings.len(), 4);
    // Protocol and invalid-UTF-8 positive controls pin preserved safe names independently of arbitrary input.
    let safe_native: PathBuf = PathBuf::from(OsString::from_vec(b"safe/\xffa:b\\c\t".to_vec()));
    assert_eq!(scanner.scan(18, safe_native.as_path(), b"").display_path, "safe/\\xffa\\x3ab\\\\c\\u{9}");
    // Native path bytes remain arbitrary, including invalid UTF-8 and protocol delimiters.
    let path: PathBuf = PathBuf::from(OsString::from_vec(input.to_vec()));
    let report: CandidateScan = scanner.scan(input.len(), path.as_path(), input);
    assert_eq!(report.identity, input.len());
    let prefix: &[u8] = &input[..input.len().min(PROBE)];
    let checked: &[u8] = if prefix.contains(&0) { prefix } else { input };
    assert_eq!(report.scanned_bytes, checked.len());
    let mut complete_expected: Vec<ScanFinding> = name_findings(input);
    complete_expected.extend(expected(checked));
    // Compare every variant: an input-specific EngineError must fail fuzzing, not vanish in a content-only filter.
    assert_eq!(report.findings, complete_expected);
    assert!(!report.display_path.contains('\n'));
    assert!(!report.display_path.contains('\r'));
    assert!(!report.display_path.contains("EMBEDDED_NEEDLE_LONG"));
    if input.contains(&b'\n') || input.contains(&b'\r') {
        assert_eq!(report.display_path, "[REDACTED]");
        assert!(report.findings.contains(&ScanFinding::PathnameLineBreak));
    }
    // Reusing one scanner must not accumulate findings from a previous candidate.
    assert!(scanner.scan(usize::MAX, Path::new("safe"), b"ordinary").findings.is_empty());
    assert!(scanner.cache_warnings().is_empty());
}

// The macro supplies libFuzzer's external ABI; all actual verification lives in the named callback.
fuzz_target!(|input: &[u8]| { verify(input); });
