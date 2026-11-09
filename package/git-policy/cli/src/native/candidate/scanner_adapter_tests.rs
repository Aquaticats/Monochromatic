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
/// Import what a load may write into the shared compiled-rules cache.
use forbidden_strings::CacheAccess;
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
        "candidate::scanner_adapter::tests::rules_are_loaded_once",
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
        "candidate::scanner_adapter::tests::pathnames_are_scanned_and_masked",
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
        "candidate::scanner_adapter::tests::cache_warnings_are_the_scanners_own",
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
                "monochromatic-git-policy-cli could not load the forbidden-strings rules, so no file was scanned: read rules "
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
        "candidate::scanner_adapter::tests::load_failures_leave_no_scanner",
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
        "monochromatic-git-policy-cli could not scan candidate 7: its pathname cannot be expressed as a native path on this platform."
    );
}

/// A pathname the platform cannot express is a typed failure.
#[test]
fn unrepresentable_pathname_is_refused() {
    run_isolated(
        "candidate::scanner_adapter::tests::unrepresentable_pathname_is_refused",
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
        "candidate::scanner_adapter::tests::listed_rules_files_load_as_one_scanner",
        "scanner-listed",
        listed_rules_files_body,
    );
}

/// The compiled artifacts of the isolated cache, by path.
fn artifacts() -> Vec<PathBuf> {
    let cache: PathBuf = PathBuf::from(
        std::env::var_os("FORBIDDEN_STRINGS_CACHE_DIR").expect("isolated cache directory"),
    );
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![cache];
    while let Some(next) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(next) else { continue };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name().is_some_and(|name| return name == "rules.bin") {
                found.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// The planted rules as one explicit entry.
fn planted_rules(directory: &Path) -> RulesSource {
    return RulesSource {
        entries: vec![RulesEntry { path: rules_file(directory), optional: false }],
        explicit: true,
    };
}

/// Load `rules` under `access`, naming no control but the fixture.
fn loaded_with(rules: &RulesSource, access: CacheAccess) -> CandidateScanner {
    match CandidateScanner::load_writing_cache(rules, false, access) {
        Ok(scanner) => return scanner,
        Err(error) => panic!("planted rules must load: {error}"),
    }
}

/// The content findings of one scan of a candidate holding `body`.
fn findings_of(scanner: &CandidateScanner, body: &[u8]) -> Vec<ScanFinding> {
    return scanner.scan(&candidate(0, b"src/a.txt"), body).expect("scan").findings;
}

/// Body:
///  a load that may not write publishes nothing and still scans with the rules it compiled,
///  and a load that may write publishes exactly one artifact.
fn a_load_writes_the_cache_only_when_it_may_body(directory: &Path) {
    let rules: RulesSource = planted_rules(directory);
    let leak: String = format!("first line\n{}\nthird line\n", needle());
    let scanner: CandidateScanner = loaded_with(&rules, CacheAccess::ReadOnly);
    assert!(
        artifacts().is_empty(),
        "a load that may not write wrote: {:?}",
        artifacts()
    );
    assert_eq!(
        findings_of(&scanner, leak.as_bytes()),
        vec![ScanFinding::Content { line: 2, rule: String::from("0") }]
    );
    // The same rules, loaded by a command that owns the cache: one artifact, and the cache answers the next load.
    let writing: CandidateScanner = loaded_with(&rules, CacheAccess::ReadWrite);
    assert_eq!(artifacts().len(), 1, "a load that may write writes once");
    assert_eq!(
        writing.cache_warnings().first().map(|warning| return warning.reason()),
        Some("missing"),
        "a command that publishes compiled from the text: {:?}",
        writing.cache_warnings()
    );
    assert_eq!(
        findings_of(&writing, leak.as_bytes()),
        vec![ScanFinding::Content { line: 2, rule: String::from("0") }]
    );
    let again: CandidateScanner = loaded_with(&rules, CacheAccess::ReadWrite);
    assert!(again.cache_warnings().is_empty(), "the published artifact answers the next load: {:?}", again.cache_warnings());
}

/// A compiled artifact is written only by a load that may write.
#[test]
fn a_load_writes_the_cache_only_when_it_may() {
    run_isolated(
        "candidate::scanner_adapter::tests::a_load_writes_the_cache_only_when_it_may",
        "scanner-cache-access",
        a_load_writes_the_cache_only_when_it_may_body,
    );
}

/// Body:
///  an artifact a crash tore in the middle is refused and the rules file speaks instead.
fn a_partial_artifact_is_never_used_body(directory: &Path) {
    let rules: RulesSource = planted_rules(directory);
    let published: CandidateScanner = loaded_with(&rules, CacheAccess::ReadWrite);
    assert_eq!(
        published.cache_warnings().first().map(|warning| return warning.reason()),
        Some("missing"),
        "the first load of an empty cache compiles from the text"
    );
    let artifact: PathBuf = artifacts().into_iter().next().expect("one artifact");
    let whole: Vec<u8> = std::fs::read(&artifact).expect("read artifact");
    let torn: usize = whole.len() / 2;
    std::fs::write(&artifact, &whole[..torn]).expect("tear the artifact");
    let recovered: CandidateScanner = loaded_with(&rules, CacheAccess::ReadOnly);
    assert_eq!(
        recovered.cache_warnings().first().map(|warning| return warning.reason()),
        Some("invalid"),
        "a torn artifact is refused as invalid, not read as absent: {:?}",
        recovered.cache_warnings()
    );
    assert_eq!(
        findings_of(&recovered, format!("first line\n{}\n", needle()).as_bytes()),
        vec![ScanFinding::Content { line: 2, rule: String::from("0") }]
    );
}

/// A torn artifact is refused, so the scan uses the rules file.
#[test]
fn a_partial_artifact_is_never_used() {
    run_isolated(
        "candidate::scanner_adapter::tests::a_partial_artifact_is_never_used",
        "scanner-cache-partial",
        a_partial_artifact_is_never_used_body,
    );
}

/// Body:
///  an artifact compiled from other rules text is never used for this one:
///  a candidate holding the old text's token passes,
///  and one holding the new text's token is reported.
fn a_cache_of_other_rules_is_never_used_body(directory: &Path) {
    let before: String = format!("{}_BEFORE", needle());
    let after: String = format!("{}_AFTER", needle());
    let rules: RulesSource = planted_rules(directory);
    std::fs::write(directory.join("rules.txt"), format!("{before}\n")).expect("rules before");
    let published: CandidateScanner = loaded_with(&rules, CacheAccess::ReadWrite);
    assert_eq!(
        findings_of(&published, format!("x\n{before}\n").as_bytes()),
        vec![ScanFinding::Content { line: 2, rule: String::from("0") }]
    );
    std::fs::write(directory.join("rules.txt"), format!("{after}\n")).expect("rules after");
    // The cache holds the artifact of the old text, and this load must not scan with it.
    let changed: CandidateScanner = loaded_with(&rules, CacheAccess::ReadOnly);
    assert_eq!(
        changed.cache_warnings().first().map(|warning| return warning.reason()),
        Some("missing"),
        "the artifact of other rules text is a miss, not a hit: {:?}",
        changed.cache_warnings()
    );
    assert_eq!(
        findings_of(&changed, format!("x\n{before}\n").as_bytes()),
        Vec::new(),
        "an artifact of other rules text answered for this one"
    );
    assert_eq!(
        findings_of(&changed, format!("x\n{after}\n").as_bytes()),
        vec![ScanFinding::Content { line: 2, rule: String::from("0") }]
    );
}

/// A compiled artifact of other rules text is never used.
#[test]
fn a_cache_of_other_rules_is_never_used() {
    run_isolated(
        "candidate::scanner_adapter::tests::a_cache_of_other_rules_is_never_used",
        "scanner-cache-other-rules",
        a_cache_of_other_rules_is_never_used_body,
    );
}
