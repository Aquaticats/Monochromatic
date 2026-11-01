//! What:
//!  Controls for the catalog-specifier check over real candidates:
//!  every kind of finding the shared rule reports reaches the engine,
//!  files that are not manifests are left alone,
//!  and the three unsupported manifest names are reported by name.
//! Why:
//!  The rule itself is tested in `monochromatic-catalog-specifier`;
//!  these controls prove this policy hands it the right files and the candidate bytes,
//!  and that unreadable candidates fail the policy instead of passing it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await check(['disallowed/package.json'])).toEqual([{ code: 'disallowed-specifier', path: 'disallowed/package.json' }]);
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::check_catalog_specifiers;
use crate::candidate_prediction::CandidateRequest;
use crate::diagnostics::EngineFailureCode;
use crate::event_path::EventPath;
use crate::policy_content::{ContentState, LifecycleContent};
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::test_support::{fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Symlinks,
/// non-UTF-8 names and the shared helpers live in the sibling control file.
#[path = "policy_catalog_specifiers_path_tests.rs"]
mod path_cases;

/// The policy inside the stage:
/// severities,
/// skips and escapes.
#[path = "policy_catalog_specifiers_stage_tests.rs"]
mod stage_cases;

/// The check's outcome for `git add` with these paths in `repo`.
pub(super) fn add_outcome(repo: &Path, values: &[OsString]) -> PolicyOutcome {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.to_path_buf());
    return check_catalog_specifiers(
        &mut ContentState::new(),
        &LifecycleContent::Requested(CandidateRequest::Add(values.to_vec())),
        &mut facts,
    );
}

/// The findings of an outcome that was evaluated;
///  any other outcome fails the test with what it was.
pub(super) fn findings_of(outcome: PolicyOutcome) -> Vec<PolicyFinding> {
    match outcome {
        PolicyOutcome::Findings(found) => return found,
        other => panic!("the check must be evaluated, not {other:?}"),
    }
}

/// The code and the exact pathname of each finding,
/// in order.
pub(super) fn code_and_path(found: &[PolicyFinding]) -> Vec<(&'static str, EventPath)> {
    let mut pairs: Vec<(&'static str, EventPath)> = Vec::new();
    for finding in found {
        pairs.push((finding.code, finding.path.clone().expect("a finding names its file")));
    }
    return pairs;
}

/// Text arguments as operating-system text.
pub(super) fn os(values: &[&str]) -> Vec<OsString> {
    let mut converted: Vec<OsString> = Vec::new();
    for value in values {
        converted.push(OsString::from(value));
    }
    return converted;
}

/// Write a file below the repository,
/// creating its directory.
pub(super) fn write(repo: &Path, path: &str, bytes: &[u8]) {
    let file: PathBuf = repo.join(path);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
    std::fs::write(file, bytes).expect("file");
}

/// The path as the event form of a UTF-8 name.
fn event(path: &str) -> EventPath {
    return EventPath::from_git_bytes(path.as_bytes());
}

/// Every kind of finding the rule reports reaches the engine with its kebab-case code,
/// the file and the rule's own
///  sentence,
/// and a clean manifest reports nothing.
#[test]
fn every_kind_of_finding_reaches_the_engine() {
    let root: PathBuf = fixture("catalog-kinds");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), "disallowed/package.json", br#"{"dependencies": {"zod": "^4.6.7"}}"#);
    write(
        repo.as_path(),
        "duplicate/package.json",
        br#"{"devDependencies": {"a": "catalog:", "a": "catalog:"}}"#,
    );
    write(repo.as_path(), "invalid/package.json", b"[]");
    write(repo.as_path(), "malformed/package.json", br#"{"dependencies": []}"#);
    write(repo.as_path(), "nonstring/package.json", br#"{"optionalDependencies": {"a": 1}}"#);
    write(
        repo.as_path(),
        "ok/package.json",
        br#"{"dependencies": {"a": "catalog:", "b": "catalog:team", "c": "workspace:*"}, "peerDependencies": {"a": "*"}}"#,
    );
    write(repo.as_path(), "unsupported/package.json5", b"{}");
    let found: Vec<PolicyFinding> = findings_of(add_outcome(
        repo.as_path(),
        &os(&[
            "disallowed", "duplicate", "invalid", "malformed", "nonstring", "ok", "unsupported",
        ]),
    ));
    assert_eq!(
        code_and_path(found.as_slice()),
        vec![
            ("disallowed-specifier", event("disallowed/package.json")),
            ("duplicate-key", event("duplicate/package.json")),
            ("invalid-manifest", event("invalid/package.json")),
            ("malformed-section", event("malformed/package.json")),
            ("non-string-specifier", event("nonstring/package.json")),
            ("unsupported-manifest-format", event("unsupported/package.json5")),
        ]
    );
    for finding in &found {
        assert!(!finding.fix_available, "{}", finding.code);
        assert_eq!(finding.location, None);
        let shown: String = String::from(finding.path.as_ref().expect("path").text());
        assert!(finding.message.starts_with(shown.as_str()), "{}", finding.message);
    }
    // The sentence is the rule's: the section, the dependency, the offending value and the remedy.
    assert!(found[0].message.contains("dependencies `zod` is \"^4.6.7\""), "{}", found[0].message);
    assert!(found[0].message.contains("workspace:*"), "{}", found[0].message);
    remove(root.as_path());
}

/// Files that are not manifests are never read,
/// whatever they hold,
/// and a directory named like a manifest is not one.
#[test]
fn files_that_are_not_manifests_are_ignored() {
    let root: PathBuf = fixture("catalog-ignored");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let literal: &[u8] = br#"{"dependencies": {"zod": "^4.6.7"}}"#;
    for path in [
        "package-lock.json",
        "my-package.yaml",
        "package.json.bak",
        "docs/package.jsonc",
        "docs/mypackage.json",
        "package.yaml/readme.txt",
        "package.json5/readme.txt",
        "pnpm-workspace.yaml",
    ] {
        write(repo.as_path(), path, literal);
    }
    let found: Vec<PolicyFinding> = findings_of(add_outcome(
        repo.as_path(),
        &os(&["package-lock.json", "my-package.yaml", "package.json.bak", "docs", "package.yaml", "package.json5", "pnpm-workspace.yaml"]),
    ));
    assert_eq!(found, Vec::<PolicyFinding>::new());
    remove(root.as_path());
}

/// `package.json5`,
/// `package.yaml` and `package.yml` are reported wherever they sit,
/// whatever bytes they hold,
///  at the root and in a nested directory.
#[test]
fn the_three_unsupported_manifest_names_are_reported() {
    let root: PathBuf = fixture("catalog-unsupported");
    let repo: PathBuf = repository(root.as_path(), "repo");
    for path in [
        "package.json5",
        "package.yaml",
        "package.yml",
        "deep/er/package.json5",
        "deep/er/package.yaml",
        "deep/er/package.yml",
    ] {
        write(repo.as_path(), path, b"\xff\xfe not text, never read");
    }
    let found: Vec<PolicyFinding> = findings_of(add_outcome(
        repo.as_path(),
        &os(&["package.json5", "package.yaml", "package.yml", "deep"]),
    ));
    let mut expected: Vec<(&'static str, EventPath)> = Vec::new();
    for path in [
        "deep/er/package.json5",
        "deep/er/package.yaml",
        "deep/er/package.yml",
        "package.json5",
        "package.yaml",
        "package.yml",
    ] {
        expected.push(("unsupported-manifest-format", event(path)));
    }
    assert_eq!(code_and_path(found.as_slice()), expected);
    remove(root.as_path());
}

/// A manifest removed in the index cannot add a literal version,
/// so a deletion is not checked.
///  A manifest that is changed again is.
#[test]
fn a_deleted_manifest_is_not_checked() {
    let root: PathBuf = fixture("catalog-deleted");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), "package.json", br#"{"dependencies": {"zod": "^4.6.7"}}"#);
    write(repo.as_path(), "package.yaml", b"name: x\n");
    git(repo.as_path(), &["add", "package.json", "package.yaml"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=manifests"]);
    std::fs::remove_file(repo.join("package.json")).expect("remove manifest");
    std::fs::remove_file(repo.join("package.yaml")).expect("remove yaml");
    let found: Vec<PolicyFinding> = findings_of(add_outcome(repo.as_path(), &os(&["package.json", "package.yaml"])));
    assert_eq!(found, Vec::<PolicyFinding>::new());
    // Changed again, both are checked.
    write(repo.as_path(), "package.json", br#"{"dependencies": {"zod": "^5"}}"#);
    write(repo.as_path(), "package.yaml", b"name: y\n");
    let again: Vec<PolicyFinding> = findings_of(add_outcome(repo.as_path(), &os(&["package.json", "package.yaml"])));
    assert_eq!(
        code_and_path(again.as_slice()),
        vec![
            ("disallowed-specifier", event("package.json")),
            ("unsupported-manifest-format", event("package.yaml")),
        ]
    );
    remove(root.as_path());
}

/// Without candidates nothing is prepared;
/// candidates that cannot be prepared fail the policy and never pass it.
#[test]
fn no_candidates_check_nothing_and_unreadable_candidates_fail() {
    let mut idle: ScriptedFacts = scripted_facts();
    assert_eq!(
        check_catalog_specifiers(&mut ContentState::new(), &LifecycleContent::None, &mut idle),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(idle.asked, Vec::<String>::new());
    assert_eq!(
        check_catalog_specifiers(
            &mut ContentState::new(),
            &LifecycleContent::Requested(CandidateRequest::Add(Vec::new())),
            &mut scripted_facts()
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from("the scripted facts prepare no candidates"),
            path: None,
        }
    );
}
