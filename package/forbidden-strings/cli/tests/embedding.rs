//! What:
//!  Public scanner API exercised from a real library consumer with disposable cache state.
//! Why:
//!  Private matcher fixtures do not prove exported loading,
//!  cache warnings or native rule-file paths work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Start a bounded child test with its own home/cache, then call the exported Scanner interface.
//! ```

/// Import only the published library boundary,
///  not private scanner internals.
use forbidden_strings::{CandidateScan, Scanner, ScanFinding};
/// Import native filesystem and child-process primitives for disposable verification.
use std::path::{Path, PathBuf};
use std::process::Command;

/// Child consumer selected explicitly by the driver;
///  ordinary test enumeration performs no state mutation here.
#[test]
fn embedded_consumer_probe() {
    // The driver controls this variable in a separate process instead of mutating shared process environment.
    let Some(directory) = std::env::var_os("FORBIDDEN_STRINGS_EMBEDDED_FIXTURE") else { return; };
    let root: PathBuf = PathBuf::from(directory);
    let rules: PathBuf = root.join("rules.txt");
    let scanner: Scanner = Scanner::load(rules.as_path(), false, true).expect("public load");
    assert_eq!(scanner.cache_warnings().len(), 1);
    assert_eq!(scanner.cache_warnings()[0].reason(), "missing");
    assert_eq!(scanner.cache_warnings()[0].recovery(), "compile-from-text");
    let report: CandidateScan = scanner.scan(73, Path::new("private/EMBEDDED_NEEDLE_LONG.txt"), b"first\nEMBEDDED_NEEDLE_LONG\n");
    assert_eq!(report.identity, 73);
    assert_eq!(report.display_path, "private/[REDACTED]");
    assert_eq!(report.findings, vec![
        ScanFinding::Name { component: 2, rule: String::from("0") },
        ScanFinding::Content { line: 2, rule: String::from("0") },
    ]);
    let cached: Scanner = Scanner::load(rules.as_path(), false, true).expect("cached public load");
    assert!(cached.cache_warnings().is_empty());
    let missing: PathBuf = root.join("missing.txt");
    assert!(Scanner::load(missing.as_path(), false, true).is_err());
    assert!(Scanner::load(missing.as_path(), true, true).is_err());
    let builtin: Scanner = Scanner::load(missing.as_path(), true, false).expect("implicit missing runtime file uses shipped baseline");
    assert!(builtin.scan(74, Path::new("clean.txt"), b"plain").findings.is_empty());
    let invalid: PathBuf = root.join("invalid.txt");
    std::fs::write(&invalid, "/PRIVATE_PATTERN_LONG/g\n").expect("invalid rule fixture");
    let error = match Scanner::load(invalid.as_path(), false, true) {
        Ok(_) => panic!("invalid rule must fail loading"),
        Err(failure) => failure,
    };
    assert!(!error.to_string().contains("PRIVATE_PATTERN_LONG"));
    native_rule_path(&root);
}

/// Native rule-file paths retain their original bytes through cache repair's source revalidation.
#[cfg(unix)]
fn native_rule_path(root: &Path) {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let filename: OsString = OsString::from_vec(b"rules-\xff.txt".to_vec());
    let path: PathBuf = root.join(filename);
    std::fs::write(&path, "NATIVE_NEEDLE_LONG\n").expect("native rule file");
    let scanner: Scanner = Scanner::load(path.as_path(), false, true).expect("native rules load");
    let report: CandidateScan = scanner.scan(75, Path::new("clean.txt"), b"NATIVE_NEEDLE_LONG\n");
    assert_eq!(report.findings, vec![ScanFinding::Content { line: 1, rule: String::from("0") }]);
    let cached: Scanner = Scanner::load(path.as_path(), false, true).expect("native rules cache reuse");
    assert!(cached.cache_warnings().is_empty());
}

/// This byte-construction control is Unix-specific;
///  other platforms retain the public API controls.
#[cfg(not(unix))]
fn native_rule_path(_root: &Path) {}

/// The actual driver proves the public consumer runs with an isolated cache,
///  not just an early-returning probe.
#[test]
fn public_embedding_runs_without_real_user_state() {
    let root: PathBuf = std::env::temp_dir().join(format!("scanner-embedded-{}", std::process::id()));
    std::fs::create_dir(&root).expect("fresh disposable fixture");
    std::fs::write(root.join("rules.txt"), "EMBEDDED_NEEDLE_LONG\n").expect("runtime rules");
    let executable: PathBuf = std::env::current_exe().expect("test consumer executable");
    let output = Command::new(executable)
        .args(["--exact", "embedded_consumer_probe", "--nocapture"])
        .env("FORBIDDEN_STRINGS_EMBEDDED_FIXTURE", &root)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", root.join("cache"))
        .env("HOME", &root)
        .output()
        .expect("spawn isolated public consumer");
    std::fs::remove_dir_all(&root).expect("remove only this fixture");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}
