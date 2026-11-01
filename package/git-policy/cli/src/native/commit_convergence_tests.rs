//! Controls for the commit path's convergence over a private index:
//!  a final-newline correction lands in the private index,
//!  a second convergence over the settled index changes nothing,
//!  and a correction the index cannot take stops the path as a patch conflict.

use super::*;
use crate::candidate_prediction::CandidateRequest;
use crate::command_test_support::os_arguments;
use crate::config_schema::PolicyConfig;
use crate::policy_checks::shipped_checks;
use crate::policy_content::LifecycleContent;
use crate::policy_engine::StageRequest;
use crate::repository_facts::git_facts;
use crate::test_support::{REAL_GIT, fixture, git, git_context, remove, repository};
use crate::transaction_git::{GitRequest, run_git};
use crate::wrapper_controls::no_controls;
use std::path::{Path, PathBuf};

/// The private index variable name, as the overlay of every private Git command carries it.
const INDEX_VARIABLE: &str = "GIT_INDEX_FILE";

/// The overlay that points every Git command of the facts at the repository and its private index.
fn overlay(repo: &Path, index: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("GIT_DIR"), repo.join(".git").into_os_string()),
        (OsString::from("GIT_WORK_TREE"), repo.as_os_str().to_os_string()),
        (OsString::from(INDEX_VARIABLE), index.as_os_str().to_os_string()),
    ];
}

/// A repository whose `HEAD` holds `a.txt` with `base\n`, and whose private index holds `staged` for it.
fn scene(name: &str, staged: &[u8]) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"base\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let index: PathBuf = root.join("private-index");
    let mut read: GitRequest = GitRequest::new(repo.as_path(), &["read-tree", "HEAD"]);
    read.without_prefix = true;
    read.index = Some(index.clone());
    let output = run_git(&git_context(), &read).expect("read-tree starts");
    assert!(output.succeeded(), "{}", output.error_text());
    let stage: PrivateStage<'_> = PrivateStage {
        context: &git_context(),
        cwd: repo.as_path(),
        index: index.as_path(),
        object_directory: None,
    };
    let entry: Correction = Correction {
        path: b"a.txt".to_vec(),
        mode: crate::candidate_object::CandidateMode::Regular,
        before: std::rc::Rc::from(b"base\n".as_slice()),
        after: std::rc::Rc::from(staged),
    };
    assert_eq!(stage_correction(&stage, &entry), Ok(()), "the staged text is written");
    return (root, repo, index);
}

/// The blob the private index holds for `a.txt`, read through the private index.
fn private_text(repo: &Path, index: &Path) -> String {
    let mut show: GitRequest = GitRequest::new(repo, &["show", ":a.txt"]);
    show.without_prefix = true;
    show.index = Some(index.to_path_buf());
    let output = run_git(&git_context(), &show).expect("show starts");
    assert!(output.succeeded(), "{}", output.error_text());
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// Runs the commit-path convergence over the index with the shipped policies.
fn converge_index(repo: &Path, index: &Path, object_directory: Option<&Path>) -> CommitConvergence {
    let facts = git_facts(Path::new(REAL_GIT), &[], overlay(repo, index).as_slice());
    let mut checks: ShippedChecks<_> = shipped_checks(
        facts,
        os_arguments(&[]),
        LifecycleContent::Requested(CandidateRequest::Staged(vec![OsString::from("a.txt")])),
        Vec::<PathBuf>::new(),
    );
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: PolicyConfig::defaults(),
        controls: no_controls(),
        selected: Vec::new(),
    };
    return converge_commit_policies(CommitConvergenceInput {
        request: &request,
        checks: &mut checks,
        stage: PrivateStage {
            context: &git_context(),
            cwd: repo,
            index,
            object_directory,
        },
    });
}

/// The final-newline correction is written into the private index, and the path is reported once.
#[test]
fn the_final_newline_correction_lands_in_the_private_index() {
    let (root, repo, index): (PathBuf, PathBuf, PathBuf) = scene("commit-convergence-newline", b"changed");
    let outcome: CommitConvergence = converge_index(repo.as_path(), index.as_path(), None);
    assert_eq!(outcome.convergence, Convergence::Settled { changed_passes: 1 }, "{:?}", outcome.pass.events);
    assert_eq!(outcome.changed_paths, vec![b"a.txt".to_vec()]);
    assert_eq!(private_text(repo.as_path(), index.as_path()), "changed\n");
    remove(root.as_path());
}

/// Converging a settled private index changes nothing and reports no paths.
#[test]
fn a_settled_private_index_is_left_unchanged() {
    let (root, repo, index): (PathBuf, PathBuf, PathBuf) = scene("commit-convergence-settled", b"already\n");
    let outcome: CommitConvergence = converge_index(repo.as_path(), index.as_path(), None);
    assert_eq!(outcome.convergence, Convergence::Settled { changed_passes: 0 }, "{:?}", outcome.pass.events);
    assert!(outcome.changed_paths.is_empty());
    assert_eq!(private_text(repo.as_path(), index.as_path()), "already\n");
    remove(root.as_path());
}

/// Whether an engine failure event names a patch conflict on the path.
fn names_conflict(events: &[PolicyEvent], path_name: &str) -> bool {
    for event in events {
        if let PolicyEvent::EngineFailure { code: EngineFailureCode::PatchConflict, path: Some(path), .. } = event && path.text() == path_name {
            return true;
        }
    }
    return false;
}

/// A correction whose blob cannot be written stops the path as `patch-conflict`, naming the path.
#[test]
fn a_correction_the_index_cannot_take_is_a_patch_conflict() {
    let (root, repo, index): (PathBuf, PathBuf, PathBuf) = scene("commit-convergence-conflict", b"changed");
    let blocked: PathBuf = root.join("not-a-directory");
    std::fs::write(&blocked, b"a file").expect("blocker");
    let outcome: CommitConvergence = converge_index(repo.as_path(), index.as_path(), Some(blocked.as_path()));
    assert_eq!(outcome.convergence, Convergence::Blocked);
    let coded: bool = names_conflict(outcome.pass.events.as_slice(), "a.txt");
    assert!(coded, "{:?}", outcome.pass.events);
    remove(root.as_path());
}

/// The loop's cycle check reads the stored states through `snapshots_equal`:
///  a correction and its reversal store a state equal to the uncorrected one,
///  and no other pair of the three states is equal.
#[test]
fn stored_states_compare_equal_exactly_when_the_corrected_bytes_match() {
    let (root, repo, index): (PathBuf, PathBuf, PathBuf) = scene("commit-convergence-states", b"changed");
    let facts = git_facts(Path::new(REAL_GIT), &[], overlay(repo.as_path(), index.as_path()).as_slice());
    let mut checks: ShippedChecks<_> = shipped_checks(
        facts,
        os_arguments(&[]),
        LifecycleContent::Requested(CandidateRequest::Staged(vec![OsString::from("a.txt")])),
        Vec::<PathBuf>::new(),
    );
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: PolicyConfig::defaults(),
        controls: no_controls(),
        selected: Vec::new(),
    };
    checks.can_apply_patches = true;
    let context = git_context();
    let mut passes: CommitPasses<'_, _> = CommitPasses::new(CommitConvergenceInput {
        request: &request,
        checks: &mut checks,
        stage: PrivateStage {
            context: &context,
            cwd: repo.as_path(),
            index: index.as_path(),
            object_directory: None,
        },
    });
    assert_eq!(passes.run_pass(0), ConvergencePass::Proposed);
    assert!(passes.apply(1), "the final-newline correction is written");
    let reversal: Correction = Correction {
        path: b"a.txt".to_vec(),
        mode: crate::candidate_object::CandidateMode::Regular,
        before: std::rc::Rc::from(b"changed\n".as_slice()),
        after: std::rc::Rc::from(b"changed".as_slice()),
    };
    passes.checks.content.propose(reversal);
    assert!(passes.apply(2), "the reversal is written");
    assert!(!passes.snapshots_equal(0, 1), "the corrected state differs from the uncorrected one");
    assert!(!passes.snapshots_equal(1, 2), "the reversed state differs from the corrected one");
    assert!(passes.snapshots_equal(0, 2), "the reversed state repeats the uncorrected one");
    assert!(passes.snapshots_equal(1, 1), "a state equals itself");
    remove(root.as_path());
}

/// A correction names its file by bytes, so a name that is not UTF-8 reaches the private index unchanged
///  and is listed back with the same bytes.
#[test]
fn a_correction_of_a_name_that_is_not_utf8_is_written_by_its_bytes() {
    let (root, repo, index): (PathBuf, PathBuf, PathBuf) = scene("commit-convergence-odd-name", b"changed\n");
    let stage: PrivateStage<'_> = PrivateStage {
        context: &git_context(),
        cwd: repo.as_path(),
        index: index.as_path(),
        object_directory: None,
    };
    let entry: Correction = Correction {
        path: b"caf\xe9.txt".to_vec(),
        mode: crate::candidate_object::CandidateMode::Regular,
        before: std::rc::Rc::from(b"".as_slice()),
        after: std::rc::Rc::from(b"odd\n".as_slice()),
    };
    assert_eq!(stage_correction(&stage, &entry), Ok(()));
    let mut listing: GitRequest = GitRequest::new(repo.as_path(), &["ls-files", "-z"]);
    listing.without_prefix = true;
    listing.index = Some(index.clone());
    let output = run_git(&git_context(), &listing).expect("ls-files starts");
    let names: Vec<&[u8]> = output.stdout.split(|byte| { return *byte == 0; }).filter(|name| { return !name.is_empty(); }).collect();
    assert_eq!(names, vec![b"a.txt".as_slice(), b"caf\xe9.txt".as_slice()]);
    remove(root.as_path());
}

/// The patch-conflict failure names a file whose name is not UTF-8 by its readable text and its exact bytes.
#[test]
fn a_patch_conflict_over_a_name_that_is_not_utf8_carries_the_exact_bytes() {
    let pass: PassResult = patch_conflict_pass(&[], b"caf\xe9.txt", String::from("cause"));
    let [PolicyEvent::EngineFailure { message, path: Some(path), .. }] = pass.events.as_slice() else {
        panic!("expected one engine failure with a path: {:?}", pass.events);
    };
    assert_eq!(message, "Policy patch for caf\u{fffd}.txt does not apply to the private index: cause");
    assert_eq!((path.text(), path.exact()), ("caf\u{fffd}.txt", Some(b"caf\xe9.txt".as_slice())));
}
