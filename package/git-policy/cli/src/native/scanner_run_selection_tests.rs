//! What:
//!  Controls for which candidates a scan pass skips and how a pass ends on failure.
//! Why:
//!  Skipping the wrong candidate leaves a file unchecked,
//!  and a pass that continued
//!      past a failure would return a partial result that reads as clean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await scanVersion(scanner, store, version, 'config/rules.txt')).toHaveLength(1);
//! ```
#![cfg(unix)]

/// Import the run under test,
///  its sibling fixtures and shared fixtures.
use super::tests::{findings_at, loaded, store, was_scanned};
use super::{ScanRunError, scan_version};
use crate::candidate_error::CandidateFailure;
use crate::candidate_object::parse_object_id;
use crate::candidate_store::CandidateStore;
use crate::candidate_version::{CandidateSource, CandidateVersion};
use crate::scanner_adapter::{CandidateScanner, ScannerFailure};
use crate::scanner_selection::rules_candidate_path;
use crate::scanner_test_support::{needle, rules_file, run_isolated};
use crate::test_support::{git, repository};
use forbidden_strings::{CandidateScan, ScanFinding};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Body:
///  deletions,
///  the rule-source files and the tracked rules file are skipped;
///  the exclusion is shown to matter.
fn ineligible_candidates_are_not_scanned_body(directory: &Path) {
    let token: String = needle();
    let leak: String = format!("{token}\n");
    let repo: PathBuf = repository(directory, "repo");
    let named_by_token: String = format!("{token}.txt");
    std::fs::write(repo.join(named_by_token.as_str()), "clean\n").expect("file named by the token");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    // The file named by the token is deleted, so its pathname leaves the tree.
    git(repo.as_path(), &["rm", "--quiet", named_by_token.as_str()]);
    // The rules file is staged inside the repository and holds the token, as a real one would.
    std::fs::create_dir(repo.join("config")).expect("config directory");
    let rules: PathBuf = repo.join("config/rules.txt");
    std::fs::write(&rules, leak.as_str()).expect("tracked rules file");
    let sources: PathBuf = repo.join("package/forbidden-strings/cli/data");
    std::fs::create_dir_all(&sources).expect("rule source directory");
    std::fs::write(sources.join("builtin-rules.txt"), leak.as_str()).expect("rule source");
    std::fs::write(repo.join("ordinary.txt"), leak.as_str()).expect("ordinary file");
    git(repo.as_path(), &["add", "--all"]);
    let scanner: CandidateScanner = loaded(rules.as_path());
    let rules_path: Vec<u8> = rules_candidate_path(rules.as_path(), repo.as_path())
        .expect("rules file is inside the repository");
    assert_eq!(rules_path, b"config/rules.txt");
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(version.candidates().len(), 4);
    let scans: Vec<CandidateScan> = scan_version(
        &scanner,
        &mut subject,
        &version,
        std::slice::from_ref(&rules_path),
    )
    .expect("scan pass");
    // Only the ordinary file is scanned, and it is reported.
    assert_eq!(scans.len(), 1);
    assert_eq!(
        findings_at(&version, &scans, b"ordinary.txt"),
        vec![ScanFinding::Content {
            line: 1,
            rule: String::from("0")
        }]
    );
    assert!(!was_scanned(&version, &scans, named_by_token.as_bytes()));
    assert!(!was_scanned(
        &version,
        &scans,
        b"package/forbidden-strings/cli/data/builtin-rules.txt"
    ));
    assert!(!was_scanned(&version, &scans, b"config/rules.txt"));
    // Positive control: without naming the rules file, its candidate is scanned and reported.
    let unexcluded: Vec<CandidateScan> =
        scan_version(&scanner, &mut subject, &version, &[]).expect("scan pass");
    assert_eq!(unexcluded.len(), 2);
    assert_eq!(
        findings_at(&version, &unexcluded, b"config/rules.txt"),
        vec![ScanFinding::Content {
            line: 1,
            rule: String::from("0")
        }]
    );
}

/// Deleted paths and rule sources are not given to the scanner.
#[test]
fn ineligible_candidates_are_not_scanned() {
    run_isolated(
        "scanner_run::selection_tests::ineligible_candidates_are_not_scanned",
        "run-eligibility",
        ineligible_candidates_are_not_scanned_body,
    );
}

/// Body:
///  a candidate that cannot be read,
///  or cannot be named,
///  ends the pass with its own typed failure.
fn failures_end_the_pass_body(directory: &Path) {
    let scanner: CandidateScanner = loaded(rules_file(directory).as_path());
    let repo: PathBuf = repository(directory, "repo");
    std::fs::write(repo.join("first.txt"), "clean\n").expect("file");
    std::fs::write(repo.join("second.txt"), "clean\n").expect("file");
    git(repo.as_path(), &["add", "--all"]);
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(
        scan_version(&scanner, &mut subject, &version, &[])
            .expect("scan pass")
            .len(),
        2
    );
    // After an invalidation the version's candidates are stale: nothing is scanned and nothing is returned.
    subject.invalidate();
    let stale: ScanRunError = match scan_version(&scanner, &mut subject, &version, &[]) {
        Ok(_) => panic!("a stale version must not be scanned"),
        Err(failure) => failure,
    };
    match &stale {
        ScanRunError::Candidate(error) => {
            assert_eq!(error.failure, CandidateFailure::StaleCandidate);
            assert_eq!(stale.to_string(), error.message);
        }
        ScanRunError::Scanner(_) => panic!("the candidate layer must be named as the cause"),
    }
    // A version holding a pathname the scanner cannot be given ends with the scanner's failure.
    let unnamed: CandidateVersion = crate::candidate_version::build_version(
        1,
        vec![crate::candidate_record::CandidateRecord {
            path: Vec::new(),
            mode: crate::candidate_object::CandidateMode::Regular,
            change: crate::candidate_record::CandidateChange::Deleted,
            object: None,
        }],
    );
    // A deleted candidate is skipped before its pathname matters.
    assert!(
        scan_version(&scanner, &mut subject, &unnamed, &[])
            .expect("skipped")
            .is_empty()
    );
    let gitlink: CandidateVersion = crate::candidate_version::build_version(
        1,
        vec![crate::candidate_record::CandidateRecord {
            path: Vec::new(),
            mode: crate::candidate_object::CandidateMode::Gitlink,
            change: crate::candidate_record::CandidateChange::Added,
            object: parse_object_id(&[b'a'; 40]),
        }],
    );
    let refused: ScanRunError = match scan_version(&scanner, &mut subject, &gitlink, &[]) {
        Ok(_) => panic!("an unnamed candidate must not be scanned"),
        Err(failure) => failure,
    };
    match &refused {
        ScanRunError::Scanner(error) => {
            assert_eq!(error.failure, ScannerFailure::PathnameUnrepresentable);
            assert_eq!(refused.to_string(), error.message);
        }
        ScanRunError::Candidate(_) => panic!("the scanner must be named as the cause"),
    }
}

/// The first failure ends the pass without a partial result.
#[test]
fn failures_end_the_pass() {
    run_isolated(
        "scanner_run::selection_tests::failures_end_the_pass",
        "run-failures",
        failures_end_the_pass_body,
    );
}
