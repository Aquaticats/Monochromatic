//! What:
//!  Controls for the linked scanner:
//!  one load,
//!  typed redacted findings,
//!  cache warnings,
//!  fail-closed loading.
//! Why:
//!  The adapter adds nothing to the scanner's verdicts,
//!  so these controls observe the
//!      scanner's own values through it,
//!  in a child process with a disposable home and cache.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const scanner = CandidateScanner.load({ path: rules, explicit: true }, false);
//! // expect(scanner.scan(candidate, bytes).findings).toEqual([{ kind: 'content', line: 2, rule: '0' }]);
//! ```
#![cfg(unix)]

/// Import the adapter under test and shared fixtures.
use super::{CandidateScanner, RulesSource, ScannerError, ScannerFailure};
use crate::candidate_object::{CandidateMode, parse_object_id};
use crate::candidate_record::CandidateChange;
use crate::candidate_version::{Candidate, CandidateIdentity};
use crate::scanner_test_support::{needle, rules_file, run_isolated};
use forbidden_strings::{CandidateScan, RulesEntry, ScanFinding};
use std::path::{Path, PathBuf};

/// One added regular-file candidate at a list position.
fn candidate(index: usize, path: &[u8]) -> Candidate {
    return Candidate {
        identity: CandidateIdentity {
            generation: 0,
            index,
        },
        path: path.to_vec(),
        mode: CandidateMode::Regular,
        change: CandidateChange::Added,
        object: parse_object_id(&[b'a'; 40]),
    };
}

/// Load the planted single-rule file explicitly,
///  without the built-in baseline.
fn loaded(directory: &Path) -> CandidateScanner {
    let rules: RulesSource = RulesSource {
        entries: vec![RulesEntry { path: rules_file(directory), optional: false }],
        explicit: true,
    };
    match CandidateScanner::load(&rules, false) {
        Ok(scanner) => return scanner,
        Err(error) => panic!("planted rules must load: {error}"),
    }
}

/// The failure of a load that must be refused.
fn load_failure(rules: &RulesSource, builtin_rules: bool) -> ScannerError {
    match CandidateScanner::load(rules, builtin_rules) {
        Ok(_) => panic!("the load must be refused"),
        Err(error) => return error,
    }
}

/// Body:
///  after one load the rules file and cache are deleted,
///  and every later scan still uses the loaded rules.
fn rules_are_loaded_once_body(directory: &Path) {
    let scanner: CandidateScanner = loaded(directory);
    std::fs::remove_file(directory.join("rules.txt")).expect("remove rules after loading");
    let cache: PathBuf = PathBuf::from(
        std::env::var_os("FORBIDDEN_STRINGS_CACHE_DIR").expect("isolated cache directory"),
    );
    std::fs::remove_dir_all(&cache).expect("remove cache after loading");
    let token: String = needle();
    let leak: Vec<u8> = format!("first line\n{token}\nthird line\n").into_bytes();
    let first: CandidateScan = scanner
        .scan(&candidate(4, b"src/a.txt"), leak.as_slice())
        .expect("scan");
    let second: CandidateScan = scanner
        .scan(&candidate(9, b"src/b.txt"), leak.as_slice())
        .expect("scan");
    let clean: CandidateScan = scanner
        .scan(&candidate(2, b"src/c.txt"), b"nothing here\n")
        .expect("scan");
    // The scan's identity is the candidate's list position, and the finding is the scanner's own value.
    assert_eq!(first.identity, 4);
    assert_eq!(second.identity, 9);
    assert_eq!(clean.identity, 2);
    for scan in [&first, &second] {
        assert_eq!(
            scan.findings,
            vec![ScanFinding::Content {
                line: 2,
                rule: String::from("0")
            }]
        );
        assert_eq!(scan.scanned_bytes, leak.len());
    }
    assert_eq!(first.display_path, "src/a.txt");
    assert!(clean.findings.is_empty());
    // Nothing in a result repeats the matched bytes.
    assert!(!format!("{first:?}{second:?}{clean:?}").contains(token.as_str()));
}

/// One load serves every scan of an invocation.
#[test]
fn rules_are_loaded_once() {
    run_isolated(
        "scanner_adapter::tests::rules_are_loaded_once",
        "scanner-once",
        rules_are_loaded_once_body,
    );
}

/// Body:
///  a matching pathname component is masked,
///  non-UTF-8 names are matched as bytes,
///  and a line break fails closed.
fn pathnames_are_scanned_and_masked_body(directory: &Path) {
    let scanner: CandidateScanner = loaded(directory);
    let token: String = needle();
    let named: Vec<u8> = format!("private/{token}/notes.txt").into_bytes();
    let by_name: CandidateScan = scanner
        .scan(&candidate(0, named.as_slice()), b"clean\n")
        .expect("scan");
    assert_eq!(
        by_name.findings,
        vec![ScanFinding::Name {
            component: 2,
            rule: String::from("0")
        }]
    );
    assert_eq!(by_name.display_path, "private/[REDACTED]/notes.txt");
    let mut bytes_name: Vec<u8> = b"dir\xff/".to_vec();
    bytes_name.extend_from_slice(token.as_bytes());
    let by_bytes: CandidateScan = scanner
        .scan(&candidate(1, bytes_name.as_slice()), b"")
        .expect("scan");
    assert_eq!(
        by_bytes.findings,
        vec![ScanFinding::Name {
            component: 2,
            rule: String::from("0")
        }]
    );
    assert_eq!(by_bytes.display_path, "dir\\xff/[REDACTED]");
    // A pathname with a line break cannot be matched line by line; the scanner reports that instead of passing it.
    let broken: CandidateScan = scanner
        .scan(&candidate(2, b"two\nlines.txt"), b"clean\n")
        .expect("scan");
    assert_eq!(broken.findings, vec![ScanFinding::PathnameLineBreak]);
    assert!(!format!("{by_name:?}{by_bytes:?}{broken:?}").contains(token.as_str()));
}

/// Pathnames go to the scanner as native bytes,
///  and its fail-closed findings pass through.
#[test]
fn pathnames_are_scanned_and_masked() {
    run_isolated(
        "scanner_adapter::tests::pathnames_are_scanned_and_masked",
        "scanner-names",
        pathnames_are_scanned_and_masked_body,
    );
}

/// Body:
///  the first load of new rules reports the scanner's cache recovery;
///  a second load finds the cache.
fn cache_warnings_are_the_scanners_own_body(directory: &Path) {
    let first: CandidateScanner = loaded(directory);
    assert_eq!(first.cache_warnings().len(), 1);
    assert_eq!(first.cache_warnings()[0].reason(), "missing");
    assert_eq!(first.cache_warnings()[0].recovery(), "compile-from-text");
    let second: CandidateScanner = loaded(directory);
    assert!(second.cache_warnings().is_empty());
}

/// Cache warnings are returned as the scanner's fixed-token values.
#[test]
fn cache_warnings_are_the_scanners_own() {
    run_isolated(
        "scanner_adapter::tests::cache_warnings_are_the_scanners_own",
        "scanner-cache",
        cache_warnings_are_the_scanners_own_body,
    );
}

/// Body:
///  a missing or invalid rules file leaves no scanner,
///  with the scanner's redacted explanation.
fn load_failures_leave_no_scanner_body(directory: &Path) {
    let missing: PathBuf = directory.join("missing.txt");
    let explicit: RulesSource = RulesSource {
        entries: vec![RulesEntry { path: missing.clone(), optional: false }],
        explicit: true,
    };
    let implicit: RulesSource = RulesSource {
        entries: vec![RulesEntry { path: missing, optional: false }],
        explicit: false,
    };
    for (rules, builtin_rules) in [(&explicit, false), (&explicit, true), (&implicit, false)] {
        let error: ScannerError = load_failure(rules, builtin_rules);
        assert_eq!(error.failure, ScannerFailure::RulesNotLoaded);
        assert!(
            error.message.starts_with(
                "cli-git could not load the forbidden-strings rules, so no file was scanned: read rules "
            ),
            "{error}"
        );
        assert_eq!(error.to_string(), error.message);
    }
    // The one tolerated absence: the default rules file is missing and the built-in baseline is on.
    let baseline: CandidateScanner = match CandidateScanner::load(&implicit, true) {
        Ok(scanner) => scanner,
        Err(error) => panic!("the baseline alone must load: {error}"),
    };
    assert!(baseline.cache_warnings().is_empty());
    assert!(
        baseline
            .scan(&candidate(0, b"plain.txt"), b"plain text\n")
            .expect("scan")
            .findings
            .is_empty()
    );
    // An invalid rule fails the whole load, and the explanation does not repeat the rule.
    let pattern: String = ["PRIVATE", "PATTERN", "BODY"].join("_");
    let invalid: PathBuf = directory.join("invalid.txt");
    std::fs::write(&invalid, format!("/{pattern}/g\n")).expect("invalid rules file");
    let refused: ScannerError = load_failure(
        &RulesSource {
            entries: vec![RulesEntry { path: invalid, optional: false }],
            explicit: true,
        },
        false,
    );
    assert_eq!(refused.failure, ScannerFailure::RulesNotLoaded);
    assert!(!refused.message.contains(pattern.as_str()), "{refused}");
}

/// Loading fails closed,
///  and the built-in and explicit settings reach the scanner.
#[test]
fn load_failures_leave_no_scanner() {
    run_isolated(
        "scanner_adapter::tests::load_failures_leave_no_scanner",
        "scanner-load",
        load_failures_leave_no_scanner_body,
    );
}

/// Body:
///  a candidate whose pathname is no native path is refused by position instead of scanned under another name.
fn unrepresentable_pathname_is_refused_body(directory: &Path) {
    let scanner: CandidateScanner = loaded(directory);
    let error: ScannerError = match scanner.scan(&candidate(7, b""), b"content\n") {
        Ok(_) => panic!("an empty pathname must be refused"),
        Err(failure) => failure,
    };
    assert_eq!(error.failure, ScannerFailure::PathnameUnrepresentable);
    assert_eq!(
        error.message,
        "cli-git could not scan candidate 7: its pathname cannot be expressed as a native path on this platform."
    );
}

/// A pathname the platform cannot express is a typed failure.
#[test]
fn unrepresentable_pathname_is_refused() {
    run_isolated(
        "scanner_adapter::tests::unrepresentable_pathname_is_refused",
        "scanner-pathname",
        unrepresentable_pathname_is_refused_body,
    );
}

/// Body:
///  a list of a required shared file and an optional local file loads one scanner,
///  reports the absent optional file,
///  and applies the rules of the files that exist.
fn listed_rules_files_body(directory: &Path) {
    let token: String = needle();
    let shared: PathBuf = directory.join("shared.txt");
    std::fs::write(&shared, format!("==> qqq-shared <==\n{token}\n")).expect("shared rules");
    let local: PathBuf = directory.join("local.txt");
    let rules: RulesSource = RulesSource {
        entries: vec![
            RulesEntry { path: shared.clone(), optional: false },
            RulesEntry { path: local.clone(), optional: true },
        ],
        explicit: true,
    };
    let scanner: CandidateScanner = match CandidateScanner::load(&rules, false) {
        Ok(found) => found,
        Err(error) => panic!("a list with an absent optional file must load: {error}"),
    };
    assert_eq!(scanner.skipped_optional_rules(), [local.clone()].as_slice());
    let leak: Vec<u8> = format!("{token}\n").into_bytes();
    let scan: CandidateScan = scanner.scan(&candidate(0, b"src/a.txt"), leak.as_slice()).expect("scan");
    assert_eq!(
        scan.findings,
        vec![ScanFinding::Content { line: 1, rule: String::from("qqq-shared") }]
    );
    // Once the optional file exists it joins the rule set, in list order after the shared file.
    std::fs::write(&local, "==> qqq-local <==\nLOCAL_LISTED_RULE_LONG_ENOUGH\n").expect("local rules");
    let both: CandidateScanner = match CandidateScanner::load(&rules, false) {
        Ok(found) => found,
        Err(error) => panic!("both files must load: {error}"),
    };
    assert!(both.skipped_optional_rules().is_empty());
    let local_scan: CandidateScan = both
        .scan(&candidate(1, b"src/b.txt"), b"LOCAL_LISTED_RULE_LONG_ENOUGH\n")
        .expect("scan");
    assert_eq!(
        local_scan.findings,
        vec![ScanFinding::Content { line: 1, rule: String::from("qqq-local") }]
    );
    // A required file that is absent refuses the whole load and names its path.
    let missing: PathBuf = directory.join("gone.txt");
    let refused: ScannerError = load_failure(
        &RulesSource {
            entries: vec![
                RulesEntry { path: shared, optional: false },
                RulesEntry { path: missing.clone(), optional: false },
            ],
            explicit: true,
        },
        false,
    );
    assert_eq!(refused.failure, ScannerFailure::RulesNotLoaded);
    assert!(refused.message.contains(missing.to_string_lossy().as_ref()), "{refused}");
}

/// Several rules files load as one scanner.
#[test]
fn listed_rules_files_load_as_one_scanner() {
    run_isolated(
        "scanner_adapter::tests::listed_rules_files_load_as_one_scanner",
        "scanner-listed",
        listed_rules_files_body,
    );
}
