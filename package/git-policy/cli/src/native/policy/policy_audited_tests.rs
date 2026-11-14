//! What:
//!  The two audited endings:
//!  which forms reach them,
//!  that neither is refused for the reason the linked-worktree rule exists,
//!  that a removal is refused only by the removal guard,
//!  and that a form which deletes nothing writes nothing.
//! Why:
//!  The owner decided these run as Git runs them,
//!  so the dirty and ignored-file questions that rule asks are never put to them,
//!  and a line for a
//!       form that deletes nothing would be noise a reader has to filter.
//!  The removal guard is the one refusal the same owner decided:
//!  a worktree a process works in,
//!  or one whose nested worktree holds Git work no other copy has,
//!  is refused before Git starts (`worktree_removal_refusal.rs`).

use super::record_audited;
use crate::command_test_support::{audit_records, os_arguments};
use crate::policy_checks::{ShippedChecks, shipped_checks};
use crate::policy_engine::{PolicyChecks, PolicyOutcome};
use crate::policy_trigger::Trigger;
use crate::policy_registry::PolicyId;
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::repository_location::RepositoryLocation;
use crate::rule_linked_worktree::{
    AuditedCommand, LinkedWorktreeDecision, decide_linked_worktree,
};
use crate::test_support::{fixture, git, remove, repository};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_removal_refusal::WORKTREE_REMOVAL_REFUSED_CODE;
use std::path::PathBuf;

/// The decision for one invocation written as text.
fn decision(values: &[&str]) -> LinkedWorktreeDecision {
    return decide_linked_worktree(os_arguments(values).as_slice());
}

/// Only a removal and a deletion reach the audited ending;
///  every other form is decided as it was.
#[test]
fn only_a_removal_and_a_deletion_are_audited() {
    for (values, expected) in [
        (
            vec!["worktree", "remove", "../topic"],
            AuditedCommand::WorktreeRemove,
        ),
        (
            vec!["worktree", "remove", "--force", "../topic"],
            AuditedCommand::WorktreeRemove,
        ),
        (vec!["branch", "-D", "gone"], AuditedCommand::BranchDelete),
        (vec!["branch", "-d", "gone"], AuditedCommand::BranchDelete),
        (vec!["branch", "--delete", "gone"], AuditedCommand::BranchDelete),
    ] {
        assert_eq!(
            decision(values.as_slice()),
            LinkedWorktreeDecision::Audited(expected),
            "{values:?}"
        );
    }
    for values in [
        vec!["worktree", "add", "../topic"],
        vec!["worktree", "list"],
        vec!["worktree", "move", "a", "b"],
        vec!["worktree", "prune"],
        vec!["worktree"],
        vec!["branch"],
        vec!["branch", "topic"],
        vec!["branch", "-a"],
        vec!["branch", "-m", "old", "new"],
        vec!["status"],
    ] {
        assert_eq!(decision(values.as_slice()), LinkedWorktreeDecision::Pass, "{values:?}");
    }
}

/// The check refuses an audited removal only through the removal guard,
///  and a repository that does not exist writes nothing.
#[test]
fn an_audited_removal_is_refused_only_by_the_removal_guard() {
    let mut checks: ShippedChecks<ScriptedFacts> = shipped_checks(
        scripted_facts(),
        os_arguments(&["worktree", "remove", "../topic"]),
        crate::policy_content::LifecycleContent::None,
        Vec::<crate::effective_target::ToolCacheDirectory>::new(),
    );
    let PolicyOutcome::Findings(findings) =
        checks.check(PolicyId::LinkedWorktreeOnly, Trigger::PreForward)
    else {
        unreachable!("the linked-worktree check answers with the findings it made");
    };
    assert!(
        findings.iter().all(|finding| return finding.code == WORKTREE_REMOVAL_REFUSED_CODE),
        "{findings:?}",
    );
    assert!(checks.facts.asked.contains(&String::from("location")));
}

/// A removal the person forced is never refused,
///  because `--force` is that person's own decision that the worktree goes.
#[test]
fn a_forced_removal_is_never_refused() {
    let mut checks: ShippedChecks<ScriptedFacts> = shipped_checks(
        scripted_facts(),
        os_arguments(&["worktree", "remove", "--force", "../topic"]),
        crate::policy_content::LifecycleContent::None,
        Vec::<crate::effective_target::ToolCacheDirectory>::new(),
    );
    assert_eq!(
        checks.check(PolicyId::LinkedWorktreeOnly, Trigger::PreForward),
        PolicyOutcome::Findings(Vec::<crate::policy_engine::PolicyFinding>::new())
    );
    assert!(checks.facts.asked.contains(&String::from("location")));
}

/// A branch with no local branch of that name writes no line,
///  and a branch that exists writes one with its commit.
#[test]
fn only_a_branch_that_exists_leaves_a_line() {
    let root: PathBuf = fixture("policy-audited");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=base"]);
    git(repo.as_path(), &["branch", "gone"]);
    let location: RepositoryLocation = RepositoryLocation {
        identity: WorktreeIdentity::MainWorktree {
            common_dir: repo.join(".git"),
            git_dir: repo.join(".git"),
            worktree_root: repo.clone(),
        },
        prefix: Vec::<u8>::new(),
    };
    let mut facts: ScriptedFacts = scripted_facts();
    record_audited(
        &mut facts,
        os_arguments(&["branch", "-D", "no-such-branch"]).as_slice(),
        AuditedCommand::BranchDelete,
        &location,
    );
    assert_eq!(audit_records(repo.join(".git").as_path()).len(), 0);
    record_audited(
        &mut facts,
        os_arguments(&["branch", "-D", "gone"]).as_slice(),
        AuditedCommand::BranchDelete,
        &location,
    );
    let records: Vec<serde_json::Value> = audit_records(repo.join(".git").as_path());
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["form"].as_str(), Some("BranchDelete"));
    assert_eq!(records[0]["outcome"].as_str(), Some("logged"));
    assert_eq!(records[0]["facts"]["branch"].as_str(), Some("gone"));
    assert!(records[0]["facts"]["commit"].as_str().is_some_and(|commit| return !commit.is_empty()));
    remove(root.as_path());
}
