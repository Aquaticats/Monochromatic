//! What:
//!  A consumer of the candidate and scanner layers that sees only the library's public interface.
//! Why:
//!  The policy engine will call these layers from other modules.
//!  This control makes
//!      one whole policy pass the way that caller must:
//!  list,
//!  load rules once,
//!  scan,
//!      attribute findings by identity,
//!  then invalidate after a fix and pass again.
//!      Unit controls inside the modules can reach private items;
//!  this one cannot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // import { CandidateStore, CandidateScanner, scanVersion } from 'git-policy-cli';
//! ```
#![cfg(unix)]

/// The public interface under control,
///  imported the way another crate would.
use forbidden_strings::{CandidateScan, ScanFinding};
use git_policy_cli::candidate_error::CandidateFailure;
use git_policy_cli::candidate_record::CandidateChange;
use git_policy_cli::candidate_store::CandidateStore;
use git_policy_cli::candidate_version::{Candidate, CandidateSource, CandidateVersion};
use git_policy_cli::scanner_adapter::{CandidateScanner, RulesSource};
use git_policy_cli::scanner_run::{ScanRunError, scan_version};
use git_policy_cli::scanner_selection::{rules_candidate_path, rules_source};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::rc::Rc;

/// The real Git 2.56.0 executable installed in the verification image.
const REAL_GIT: &str = "/usr/bin/git";

/// Set only in the re-executed child:
///  names the directory the control may use.
const ISOLATED_FIXTURE_VARIABLE: &str = "CLI_GIT_NATIVE_CONSUMER_FIXTURE";

/// Run real Git in a fixture with no system or global configuration and a fixed identity,
///  and require success.
fn git(directory: &Path, arguments: &[&str]) {
    let output: Output = Command::new(REAL_GIT)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .output()
        .expect("fixture Git command");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The pathnames of the candidates whose scans hold at least one finding,
///  sorted.
fn reported_paths(version: &CandidateVersion, scans: &[CandidateScan]) -> Vec<Vec<u8>> {
    let mut reported: Vec<Vec<u8>> = Vec::new();
    for scan in scans {
        if scan.findings.is_empty() {
            continue;
        }
        // A scan's identity is its candidate's position in the version.
        let candidate: &Candidate = &version.candidates()[scan.identity];
        assert_eq!(candidate.identity.index, scan.identity);
        reported.push(candidate.path.clone());
    }
    reported.sort();
    return reported;
}

/// Body:
///  one policy pass,
///  a fix,
///  an invalidation and a second pass,
///  all through public items.
fn policy_pass(directory: &Path) {
    // The token is assembled at run time so this repository's own commit policy cannot report it here.
    let token: String = ["PLANTED", "CONSUMER", "NEEDLE"].join("_");
    let repo: PathBuf = directory.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(
        repo.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    git(
        repo.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=initial"],
    );
    // The rules file is tracked, as this repository's own is not; either way it is never scanned.
    std::fs::write(repo.join("rules.txt"), format!("{token}\n")).expect("rules file");
    std::fs::write(repo.join("staged-leak.txt"), format!("line\n{token}\n")).expect("leak");
    std::fs::write(repo.join("worktree-leak.txt"), "clean\n").expect("clean");
    git(repo.as_path(), &["add", "--all"]);
    std::fs::write(repo.join("worktree-leak.txt"), format!("{token}\n")).expect("worktree edit");
    std::fs::write(repo.join("staged-leak.txt"), "clean\n").expect("worktree edit");
    // The rules file is selected the way the standalone scanner selects it, relative to the root.
    let rules: RulesSource = rules_source(None, Some(OsStr::new("rules.txt")), repo.as_path()).expect("rules source");
    assert!(rules.explicit);
    let scanner: CandidateScanner = match CandidateScanner::load(&rules, false) {
        Ok(loaded) => loaded,
        Err(error) => panic!("planted rules must load: {error}"),
    };
    assert_eq!(scanner.cache_warnings().len(), 1);
    assert_eq!(scanner.cache_warnings()[0].reason(), "missing");
    let rules_path: Vec<u8> =
        rules_candidate_path(rules.entries[0].path.as_path(), repo.as_path()).expect("rules file is tracked");
    let mut store: CandidateStore = CandidateStore::new(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), repo.clone().into_os_string()],
        &[],
    );
    let version: Rc<CandidateVersion> = store
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(version.candidates().len(), 3);
    let scans: Vec<CandidateScan> =
        scan_version(&scanner, &mut store, &version, std::slice::from_ref(&rules_path)).expect("pass");
    // The rules file is skipped; the staged leak is reported; the worktree-only leak is not.
    assert_eq!(scans.len(), 2);
    assert_eq!(
        reported_paths(&version, scans.as_slice()),
        [b"staged-leak.txt".to_vec()]
    );
    let leak: &Candidate = version
        .candidate_at_path(b"staged-leak.txt")
        .expect("candidate");
    assert_eq!(leak.change, CandidateChange::Added);
    // The finding is found by the candidate's identity, and it is the scanner's own typed value.
    let mut leak_findings: Vec<ScanFinding> = Vec::new();
    for scan in &scans {
        if scan.identity == leak.identity.index {
            leak_findings.extend(scan.findings.iter().cloned());
        }
    }
    assert_eq!(
        leak_findings,
        vec![ScanFinding::Content {
            line: 2,
            rule: String::from("0")
        }]
    );
    assert!(!format!("{scans:?}").contains(token.as_str()));
    // A fix stages the cleaned content; the earlier version must not be read again.
    git(repo.as_path(), &["add", "staged-leak.txt"]);
    store.invalidate();
    match scan_version(&scanner, &mut store, &version, std::slice::from_ref(&rules_path)) {
        Ok(_) => panic!("a version from before the fix must be refused"),
        Err(ScanRunError::Candidate(error)) => {
            assert_eq!(error.failure, CandidateFailure::StaleCandidate);
        }
        Err(ScanRunError::Scanner(error)) => panic!("unexpected scanner failure: {error}"),
    }
    let fixed: Rc<CandidateVersion> = store
        .version(&CandidateSource::StagedAgainstHead)
        .expect("version after the fix");
    let after: Vec<CandidateScan> =
        scan_version(&scanner, &mut store, &fixed, std::slice::from_ref(&rules_path))
            .expect("second pass");
    assert_eq!(after.len(), 2);
    assert!(reported_paths(&fixed, after.as_slice()).is_empty());
}

/// One whole policy pass through the public interface reports staged bytes only,
///  before and after a fix.
#[test]
fn public_interface_runs_a_policy_pass() {
    if let Some(directory) = std::env::var_os(ISOLATED_FIXTURE_VARIABLE) {
        policy_pass(Path::new(&directory));
        return;
    }
    // Cargo's per-target scratch directory, never the system temporary directory (issue 1809).
    let root: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("native-consumer-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    let work: PathBuf = root.join("work");
    std::fs::create_dir_all(&work).expect("fresh fixture");
    std::fs::create_dir(root.join("home")).expect("disposable home");
    // The scanner's rule cache and home directory are disposable for the child, never the real ones.
    let output: Output = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "public_interface_runs_a_policy_pass",
            "--nocapture",
        ])
        .env(
            ISOLATED_FIXTURE_VARIABLE,
            std::fs::canonicalize(&work).expect("canonical fixture"),
        )
        .env("HOME", root.join("home"))
        .env("FORBIDDEN_STRINGS_CACHE_DIR", root.join("cache"))
        .env_remove("XDG_CACHE_HOME")
        .env_remove("FORBIDDEN_STRINGS_RULES")
        .output()
        .expect("start the isolated control");
    let stdout: String = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "isolated control failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed"),
        "the isolated control did not run exactly one test:\n{stdout}"
    );
    std::fs::remove_dir_all(&root).expect("remove only the fixture");
}
