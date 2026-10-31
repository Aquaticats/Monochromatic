//! What:
//!  Candidate-byte isolation controls:
//!  the scanner sees staged or committed bytes,
//!  never the live worktree.
//! Why:
//!  A forbidden string that exists only in the worktree file must not block a commit
//!      that does not contain it,
//!  and one that exists only in the staged blob must be
//!      reported even after the worktree copy was cleaned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stage(clean); writeWorktree(needle); expect(await scanVersion(...)).toHaveNoFindings();
//! ```
#![cfg(unix)]

/// Import the run under test and shared fixtures.
use super::scan_version;
use crate::candidate_object::{ObjectId, parse_object_id};
use crate::candidate_store::CandidateStore;
use crate::candidate_version::{CandidateSource, CandidateVersion};
use crate::git_metadata::strip_git_line;
use crate::scanner_adapter::{CandidateScanner, RulesSource};
use crate::scanner_test_support::{needle, rules_file, run_isolated};
use crate::test_support::{REAL_GIT, git, repository};
use forbidden_strings::{CandidateScan, RulesEntry, ScanFinding};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Load the planted single-rule file explicitly,
///  without the built-in baseline.
pub(super) fn loaded(rules: &Path) -> CandidateScanner {
    let source: RulesSource = RulesSource {
        entries: vec![RulesEntry { path: rules.to_path_buf(), optional: false }],
        explicit: true,
    };
    match CandidateScanner::load(&source, false) {
        Ok(scanner) => return scanner,
        Err(error) => panic!("planted rules must load: {error}"),
    }
}

/// A store over real Git selecting one repository with `-C`.
pub(super) fn store(repository: &Path) -> CandidateStore {
    return CandidateStore::new(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), repository.as_os_str().to_os_string()],
        &[],
    );
}

/// The findings of the scan whose candidate sits at a pathname.
pub(super) fn findings_at(
    version: &CandidateVersion,
    scans: &[CandidateScan],
    path: &[u8],
) -> Vec<ScanFinding> {
    let index: usize = match version.candidate_at_path(path) {
        Some(candidate) => candidate.identity.index,
        None => panic!("no candidate at {:?}", String::from_utf8_lossy(path)),
    };
    for scan in scans {
        if scan.identity == index {
            return scan.findings.clone();
        }
    }
    panic!(
        "candidate at {:?} was not scanned",
        String::from_utf8_lossy(path)
    );
}

/// Whether any scan belongs to the candidate at a pathname.
pub(super) fn was_scanned(
    version: &CandidateVersion,
    scans: &[CandidateScan],
    path: &[u8],
) -> bool {
    let index: usize = match version.candidate_at_path(path) {
        Some(candidate) => candidate.identity.index,
        None => panic!("no candidate at {:?}", String::from_utf8_lossy(path)),
    };
    for scan in scans {
        if scan.identity == index {
            return true;
        }
    }
    return false;
}

/// Body:
///  the same token is planted once only in the worktree and once only in the staged blob.
fn staged_bytes_are_scanned_not_the_worktree_body(directory: &Path) {
    let scanner: CandidateScanner = loaded(rules_file(directory).as_path());
    let token: String = needle();
    let leak: String = format!("safe line\n{token}\n");
    let repo: PathBuf = repository(directory, "repo");
    // Staged clean, then the worktree copy gains the token.
    std::fs::write(repo.join("worktree-only.txt"), "clean\n").expect("clean file");
    // Staged with the token, then the worktree copy is cleaned and another is removed.
    std::fs::write(repo.join("staged-only.txt"), leak.as_str()).expect("leaking file");
    std::fs::write(repo.join("staged-then-removed.txt"), leak.as_str()).expect("leaking file");
    std::fs::write(repo.join("clean.txt"), "clean\n").expect("clean file");
    std::os::unix::fs::symlink(token.as_str(), repo.join("link"))
        .expect("link whose target is the token");
    git(repo.as_path(), &["add", "--all"]);
    std::fs::write(repo.join("worktree-only.txt"), leak.as_str()).expect("worktree edit");
    std::fs::write(repo.join("staged-only.txt"), "clean\n").expect("worktree edit");
    std::fs::remove_file(repo.join("staged-then-removed.txt")).expect("worktree removal");
    std::fs::write(repo.join("never-staged.txt"), leak.as_str()).expect("untracked file");
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    let scans: Vec<CandidateScan> =
        scan_version(&scanner, &mut subject, &version, &[]).expect("scan pass");
    let reported: Vec<ScanFinding> = vec![ScanFinding::Content {
        line: 2,
        rule: String::from("0"),
    }];
    assert_eq!(scans.len(), 5);
    assert_eq!(
        findings_at(&version, &scans, b"worktree-only.txt"),
        Vec::new()
    );
    assert_eq!(findings_at(&version, &scans, b"clean.txt"), Vec::new());
    // The reverse direction, which is also the positive control: the same bytes are reported when staged.
    assert_eq!(findings_at(&version, &scans, b"staged-only.txt"), reported);
    assert_eq!(
        findings_at(&version, &scans, b"staged-then-removed.txt"),
        reported
    );
    // A symbolic link's content is its target text.
    assert_eq!(
        findings_at(&version, &scans, b"link"),
        vec![ScanFinding::Content {
            line: 1,
            rule: String::from("0")
        }]
    );
    assert!(version.candidate_at_path(b"never-staged.txt").is_none());
    assert!(!format!("{scans:?}").contains(token.as_str()));
}

/// A token only in the worktree is not reported;
///  a token only in the staged blob is.
#[test]
fn staged_bytes_are_scanned_not_the_worktree() {
    run_isolated(
        "scanner_run::tests::staged_bytes_are_scanned_not_the_worktree",
        "run-staged",
        staged_bytes_are_scanned_not_the_worktree_body,
    );
}

/// Body:
///  a landed commit is scanned by what it recorded,
///  after the worktree and index moved on.
fn committed_bytes_are_scanned_not_the_worktree_body(directory: &Path) {
    let scanner: CandidateScanner = loaded(rules_file(directory).as_path());
    let token: String = needle();
    let leak: String = format!("{token}\n");
    let repo: PathBuf = repository(directory, "repo");
    std::fs::write(repo.join("committed-leak.txt"), leak.as_str()).expect("leaking file");
    std::fs::write(repo.join("committed-clean.txt"), "clean\n").expect("clean file");
    std::fs::create_dir(repo.join(token.as_str())).expect("directory named by the token");
    std::fs::write(repo.join(token.as_str()).join("inside.txt"), "clean\n").expect("file in it");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let landed: ObjectId = parse_object_id(strip_git_line(
        git(repo.as_path(), &["rev-parse", "--verify", "HEAD"])
            .stdout
            .as_slice(),
    ))
    .expect("commit name");
    // After landing, the worktree swaps which file holds the token, and the swap is staged.
    std::fs::write(repo.join("committed-leak.txt"), "clean\n").expect("worktree edit");
    std::fs::write(repo.join("committed-clean.txt"), leak.as_str()).expect("worktree edit");
    git(repo.as_path(), &["add", "--all"]);
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::Committed(landed))
        .expect("committed version");
    let scans: Vec<CandidateScan> =
        scan_version(&scanner, &mut subject, &version, &[]).expect("scan pass");
    assert_eq!(
        findings_at(&version, &scans, b"committed-leak.txt"),
        vec![ScanFinding::Content {
            line: 1,
            rule: String::from("0")
        }]
    );
    assert_eq!(
        findings_at(&version, &scans, b"committed-clean.txt"),
        Vec::new()
    );
    // A pathname component matching a rule is reported by its position, and masked in the display path.
    let inside: Vec<u8> = format!("{token}/inside.txt").into_bytes();
    assert_eq!(
        findings_at(&version, &scans, inside.as_slice()),
        vec![ScanFinding::Name {
            component: 1,
            rule: String::from("0")
        }]
    );
    assert!(!format!("{scans:?}").contains(token.as_str()));
}

/// A committed version is scanned from the commit,
///  not from the index or worktree that followed it.
#[test]
fn committed_bytes_are_scanned_not_the_worktree() {
    run_isolated(
        "scanner_run::tests::committed_bytes_are_scanned_not_the_worktree",
        "run-committed",
        committed_bytes_are_scanned_not_the_worktree_body,
    );
}
