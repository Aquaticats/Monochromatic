//! Controls for the landing's target comparison, nonce reflog message and compare-and-swap.

use super::*;
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use crate::transaction_journal::Base;

/// The reflog subject prefix names the transaction nonce.
#[test]
fn reflog_prefix_names_the_nonce() {
    assert_eq!(landing_reflog_prefix("0b6c"), "commit (cli-git 0b6c):");
}

/// The reflog message is the prefix followed by the landed commit's subject, read without trailing newline.
#[test]
fn reflog_message_ends_with_the_landed_subject() {
    let root = fixture("landing-support-message");
    let repo = repository(root.as_path(), "repo");
    let head = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let message = landing_reflog_message(&git_context(), repo.as_path(), "0b6c", &head).expect("subject");
    assert_eq!(message, "commit (cli-git 0b6c): initial");
    remove(&root);
}

/// Equal targets compare equal; an unborn target equals only another unborn target.
#[test]
fn same_target_compares_commits_and_unborn() {
    assert!(same_target(&Base::Unborn, &Base::Unborn));
    assert!(!same_target(&Base::Unborn, &Base::Commit("a".repeat(40))));
    assert!(same_target(&Base::Commit("a".repeat(40)), &Base::Commit("a".repeat(40))));
    assert!(!same_target(&Base::Commit("a".repeat(40)), &Base::Commit("b".repeat(40))));
}

/// A compare-and-swap on a branch advances it and writes the nonce reflog message once.
#[test]
fn compare_and_swap_advances_the_branch_with_the_nonce_reflog() {
    let root = fixture("landing-support-cas");
    let repo = repository(root.as_path(), "repo");
    let initial = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tree = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let next = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", initial.as_str(), "-m", "second"]);
    let output = compare_and_swap_target(
        &git_context(),
        repo.as_path(),
        &CompareAndSwap {
            target_ref: "refs/heads/main",
            detached: false,
            nonce: "0b6c",
            new_oid: &next,
            expected_old: &Base::Commit(initial.clone()),
            hash_length: 40,
        },
    )
    .expect("update-ref ran");
    assert!(output.succeeded(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "refs/heads/main"]), next);
    let reflog = git_text(repo.as_path(), &["reflog", "show", "--format=%gs", "refs/heads/main"]);
    assert_eq!(reflog.matches("commit (cli-git 0b6c):").count(), 1);
    remove(&root);
}

/// A target that no longer holds the expected value refuses the swap and stays unchanged.
#[test]
fn compare_and_swap_refuses_a_moved_target() {
    let root = fixture("landing-support-moved");
    let repo = repository(root.as_path(), "repo");
    let initial = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tree = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let next = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", initial.as_str(), "-m", "second"]);
    let output = compare_and_swap_target(
        &git_context(),
        repo.as_path(),
        &CompareAndSwap {
            target_ref: "refs/heads/main",
            detached: false,
            nonce: "0b6c",
            new_oid: &next,
            expected_old: &Base::Commit("f".repeat(40)),
            hash_length: 40,
        },
    )
    .expect("update-ref ran");
    assert!(!output.succeeded());
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "refs/heads/main"]), initial);
    remove(&root);
}

/// An unborn expected value is the all-zero object ID of the hash length, so a missing ref is created.
#[test]
fn compare_and_swap_creates_a_missing_ref_for_an_unborn_expectation() {
    let root = fixture("landing-support-unborn");
    let repo = repository(root.as_path(), "repo");
    let initial = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let output = compare_and_swap_target(
        &git_context(),
        repo.as_path(),
        &CompareAndSwap {
            target_ref: "refs/heads/created",
            detached: false,
            nonce: "0b6c",
            new_oid: &initial,
            expected_old: &Base::Unborn,
            hash_length: 40,
        },
    )
    .expect("update-ref ran");
    assert!(output.succeeded(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "refs/heads/created"]), initial);
    remove(&root);
}

/// A detached landing moves `HEAD` itself, without dereferencing it, and writes the nonce to the `HEAD` reflog.
#[test]
fn compare_and_swap_detached_moves_head_without_dereferencing() {
    let root = fixture("landing-support-detached");
    let repo = repository(root.as_path(), "repo");
    let initial = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git_text(repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let tree = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let next = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", initial.as_str(), "-m", "second"]);
    let output = compare_and_swap_target(
        &git_context(),
        repo.as_path(),
        &CompareAndSwap {
            target_ref: "HEAD",
            detached: true,
            nonce: "0b6c",
            new_oid: &next,
            expected_old: &Base::Commit(initial.clone()),
            hash_length: 40,
        },
    )
    .expect("update-ref ran");
    assert!(output.succeeded(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), next);
    let reflog = git_text(repo.as_path(), &["reflog", "show", "--format=%gs", "HEAD"]);
    assert!(reflog.contains("commit (cli-git 0b6c):"));
    remove(&root);
}
