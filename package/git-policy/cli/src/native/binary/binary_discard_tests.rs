//! What:
//!  Every command form that can discard uncommitted state,
//! through the built executable:
//! rejected in the main
//!  worktree,
//! rejected in a dirty linked worktree,
//! allowed in a clean one,
//! and allowed anywhere with the hatch.
//! Why:
//!  The policy exists because such a command once removed staged renames without any record.
//! The controls also
//!  prove that each documented way forward (the hatch,
//! a clean linked worktree) really runs the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // spawnSync(wrapper, ['restore', '--', 'tracked.txt'], { cwd: repo }).status === 1
//! ```

use super::commit::run_commit;
use super::support::{
    Fixture, Observed, fixture, git, observe_raw, porcelain, remove, repository, run_wrapped, stderr_of,
    stub_manual_viewer, wrapped,
};
use git_policy_cli::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use git_policy_cli::rule_linked_worktree::GuardedCommand;
use std::ffi::OsStr;
use std::path::PathBuf;

/// A repository with one tracked file edited,
/// and a second branch `topic`.
fn dirty_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=tracked"],
    );
    git(fixture, repo.as_path(), &["branch", "topic"]);
    std::fs::write(repo.join("tracked.txt"), b"two\n").expect("edit");
    return repo;
}

/// Whether the wrapper itself rejected the command.
fn rejected_by_policy(observed: &Observed) -> bool {
    return observed.code == Some(1)
        && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// Every discarding form is rejected in the main worktree and changes nothing.
#[test]
fn every_discarding_form_is_rejected_in_the_main_worktree() {
    let fixture: Fixture = fixture("discard-main");
    let repo: PathBuf = dirty_repository(&fixture);
    let before: String = porcelain(&fixture, repo.as_path());
    for arguments in [
        vec!["restore", "--", "tracked.txt"],
        vec!["restore", "--staged", "tracked.txt"],
        vec!["checkout", "--", "tracked.txt"],
        vec!["checkout", "-f"],
        vec!["checkout", "topic"],
        vec!["switch", "--discard-changes", "topic"],
        vec!["switch", "topic"],
        vec!["reset"],
        vec!["reset", "--mixed", "HEAD"],
        vec!["rm", "-f", "tracked.txt"],
        vec!["mv", "--force", "tracked.txt", "moved.txt"],
        vec!["checkout-index", "-f", "-a"],
        vec!["read-tree", "--reset", "HEAD"],
        vec!["merge", "--abort"],
        vec!["rebase", "--abort"],
        vec!["cherry-pick", "--abort"],
        vec!["revert", "--abort"],
        vec!["am", "--abort"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?}: {}",
            stderr_of(&observed)
        );
    }
    assert_eq!(porcelain(&fixture, repo.as_path()), before);
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"two\n"
    );
    remove(&fixture);
}

/// The invocations that carry the hatch for one guarded form,
/// each written as the caller types it after `git`.
/// A `match` must name every variant,
/// so a new form stops compilation here until it has a hatch invocation.
fn hatch_invocations(command: GuardedCommand) -> Vec<Vec<&'static str>> {
    return match command {
        GuardedCommand::Stash => vec![vec!["stash", WORKTREE_ENFORCEMENT_ESCAPE_HATCH]],
        GuardedCommand::Clean => vec![vec!["clean", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "-f", "-d"]],
        GuardedCommand::ResetHard => vec![vec!["reset", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--hard"]],
        GuardedCommand::ResetMixed => vec![vec!["reset", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--quiet"]],
        GuardedCommand::RestoreStaged => vec![vec!["restore", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--staged", "tracked.txt"]],
        GuardedCommand::RestoreWorktree => vec![vec!["restore", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--", "tracked.txt"]],
        GuardedCommand::CheckoutPaths => vec![vec!["checkout", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--", "tracked.txt"]],
        GuardedCommand::CheckoutMove => vec![vec!["checkout", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "topic"]],
        GuardedCommand::SwitchForce => vec![vec!["switch", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--discard-changes", "topic"]],
        GuardedCommand::SwitchMove => vec![vec!["switch", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "topic"]],
        GuardedCommand::Remove => vec![vec!["rm", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "-f", "--quiet", "tracked.txt"]],
        GuardedCommand::Move => vec![vec!["mv", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "-f", "missing.txt", "other.txt"]],
        GuardedCommand::CheckoutIndex => vec![vec!["checkout-index", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "-f", "-a"]],
        GuardedCommand::ReadTree => vec![vec!["read-tree", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--reset", "HEAD"]],
        GuardedCommand::AbortMerge => vec![vec!["merge", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--abort"]],
        GuardedCommand::AbortRebase => vec![vec!["rebase", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--abort"]],
        GuardedCommand::AbortPick => vec![vec!["cherry-pick", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--abort"], vec!["revert", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--abort"]],
        GuardedCommand::AbortAm => vec![vec!["am", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--abort"]],
    };
}

/// The hatch directly after the subcommand runs every guarded form the policy knows:
/// the wrapper's own rejection is what each of them loses,
/// and no form's hatch token reaches Git,
/// which then answers for itself.
#[test]
fn the_hatch_after_the_subcommand_runs_each_form() {
    let fixture: Fixture = fixture("discard-hatch");
    let repo: PathBuf = dirty_repository(&fixture);
    for command in GuardedCommand::ALL {
        for arguments in hatch_invocations(command) {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
            assert!(
                !rejected_by_policy(&observed),
                "{command:?} {arguments:?}: {}",
                stderr_of(&observed)
            );
            assert!(
                !stderr_of(&observed).contains(WORKTREE_ENFORCEMENT_ESCAPE_HATCH),
                "{command:?} {arguments:?} leaked the hatch to Git: {}",
                stderr_of(&observed)
            );
        }
    }
    remove(&fixture);
}

/// A clean linked worktree runs the forms;
/// a dirty one rejects them;
/// a lone branch switch needs a clean tree.
#[test]
fn a_linked_worktree_runs_the_forms_only_while_clean() {
    let fixture: Fixture = fixture("discard-linked");
    let repo: PathBuf = dirty_repository(&fixture);
    git(&fixture, repo.as_path(), &["checkout", "--", "tracked.txt"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "side", "../linked"],
    );
    for arguments in [
        vec!["restore", "--staged", "tracked.txt"],
        vec!["switch", "topic"],
        vec!["stash"],
        vec!["reset"],
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(
            !rejected_by_policy(&observed),
            "{arguments:?} in a clean linked worktree: {}",
            stderr_of(&observed)
        );
    }
    std::fs::write(linked.join("tracked.txt"), b"three\n").expect("edit in the linked worktree");
    for arguments in [
        vec!["restore", "--", "tracked.txt"],
        vec!["checkout", "--", "tracked.txt"],
        vec!["switch", "side"],
        vec!["checkout", "side"],
        vec!["stash"],
        vec!["reset", "--hard"],
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?} in a dirty linked worktree: {}",
            stderr_of(&observed)
        );
    }
    assert_eq!(
        std::fs::read(linked.join("tracked.txt")).expect("tracked file"),
        b"three\n"
    );
    // The documented way forward: commit the work with explicit paths, after which the worktree is clean again.
    // A commit is the one command here that is not a single Git command: the wrapper lands it through its
    // transaction (40 Git process starts and 36 sync calls, 0.15 to 0.45 s with no extra load, against 4 ms for
    // direct Git), so it runs under the commit bound; the other 11 wrapped commands of this test took at most
    // 0.05 s each.
    let committed: Observed = run_commit(
        &fixture,
        linked.as_path(),
        &["commit", "--quiet", "--message=work", "--", "tracked.txt"],
    );
    assert_eq!(committed.code, Some(0), "{}", stderr_of(&committed));
    assert_eq!(porcelain(&fixture, linked.as_path()), "");
    let switched: Observed = run_wrapped(&fixture, linked.as_path(), &["switch", "topic"]);
    assert!(!rejected_by_policy(&switched), "{}", stderr_of(&switched));
    remove(&fixture);
}

/// Reading forms stay available everywhere.
#[test]
fn reading_forms_are_never_rejected() {
    let fixture: Fixture = fixture("discard-reading");
    stub_manual_viewer(&fixture);
    let repo: PathBuf = dirty_repository(&fixture);
    // The manual page is Git's work, not the wrapper's, so a stub stands in for `man`; the marker proves Git
    // received the `--help` the wrapper forwarded and took its manual-page path.
    let help: Observed = run_wrapped(&fixture, repo.as_path(), &["restore", "--help"]);
    assert!(!rejected_by_policy(&help), "{}", stderr_of(&help));
    assert_eq!(help.code, Some(0), "{}", stderr_of(&help));
    assert_eq!(String::from_utf8_lossy(&help.stdout), "manual page requested: git-restore\n");
    for arguments in [
        vec!["stash", "list"],
        vec!["status"],
        vec!["diff"],
        vec!["reset", "--soft", "HEAD"],
        vec!["checkout"],
        vec!["clean", "-n"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(
            !rejected_by_policy(&observed),
            "{arguments:?}: {}",
            stderr_of(&observed)
        );
    }
    remove(&fixture);
}

/// Run one wrapped command in the main worktree and require that it succeeded.
fn runs(fixture: &Fixture, repo: &std::path::Path, arguments: &[&str]) -> Observed {
    let observed: Observed = run_wrapped(fixture, repo, arguments);
    assert_eq!(observed.code, Some(0), "{arguments:?}: {}", stderr_of(&observed));
    return observed;
}

/// Run one wrapped commit in the main worktree under the commit bound and require that it succeeded.
fn runs_commit(fixture: &Fixture, repo: &std::path::Path, arguments: &[&str]) -> Observed {
    let observed: Observed = run_commit(fixture, repo, arguments);
    assert_eq!(observed.code, Some(0), "{arguments:?}: {}", stderr_of(&observed));
    return observed;
}

/// Every alternative a rejection text names runs in the main worktree,
/// so no normal workflow lacks a way forward.
#[test]
fn every_named_alternative_runs_in_the_main_worktree() {
    let fixture: Fixture = fixture("discard-alternatives");
    let repo: PathBuf = dirty_repository(&fixture);
    // Looking instead of reverting.
    assert_eq!(
        runs(&fixture, repo.as_path(), &["show", "HEAD:tracked.txt"]).stdout,
        b"one\n"
    );
    let patch: Observed = runs(&fixture, repo.as_path(), &["diff", "--", "tracked.txt"]);
    assert!(String::from_utf8_lossy(&patch.stdout).contains("+two"));
    runs(&fixture, repo.as_path(), &["diff", "--cached", "--name-only"]);
    runs(&fixture, repo.as_path(), &["stash", "list"]);
    runs(&fixture, repo.as_path(), &["clean", "-n"]);
    // Moving the branch pointer without touching files.
    runs(&fixture, repo.as_path(), &["reset", "--soft", "HEAD"]);
    // Copies of staged files elsewhere, instead of overwriting the worktree.
    git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    runs(
        &fixture,
        repo.as_path(),
        &["checkout-index", "--prefix=copy/", "--all"],
    );
    assert_eq!(
        std::fs::read(repo.join("copy/tracked.txt")).expect("copied staged file"),
        b"two\n"
    );
    // Stopping an operation while keeping the work: these have no operation to stop, and none is rejected.
    for arguments in [
        vec!["merge", "--quit"],
        vec!["rebase", "--quit"],
        vec!["cherry-pick", "--quit"],
        vec!["revert", "--quit"],
        vec!["am", "--quit"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(!rejected_by_policy(&observed), "{arguments:?}: {}", stderr_of(&observed));
    }
    std::fs::remove_dir_all(repo.join("copy")).expect("remove the copied files");
    // Unstage-by-commit: only the named path lands, the other staged entry stays staged. This is the one
    // transaction commit of this test (40 Git process starts and 36 sync calls, 0.17 to 0.35 s with no extra
    // load); the other 14 wrapped commands took at most 0.12 s each, so only the commit runs under the commit bound.
    std::fs::write(repo.join("other.txt"), b"other\n").expect("other file");
    git(&fixture, repo.as_path(), &["add", "--", "other.txt"]);
    runs_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=only tracked", "--", "tracked.txt"],
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "A  other.txt\n");
    // A clean linked worktree for anything else.
    runs(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "wip", "../wip"],
    );
    // Stop tracking a file and keep it on disk.
    runs(&fixture, repo.as_path(), &["rm", "--cached", "--quiet", "other.txt"]);
    assert!(repo.join("other.txt").exists());
    remove(&fixture);
}

/// Saving a patch and then using the hatch reverts a file,
/// and the patch brings the edits back.
#[test]
fn a_saved_patch_makes_the_hatch_revert_reversible() {
    let fixture: Fixture = fixture("discard-patch");
    let repo: PathBuf = dirty_repository(&fixture);
    let patch: Observed = runs(&fixture, repo.as_path(), &["diff", "--", "tracked.txt"]);
    let patch_file: PathBuf = fixture.root.join("backup.patch");
    std::fs::write(patch_file.as_path(), patch.stdout.as_slice()).expect("backup patch");
    runs(
        &fixture,
        repo.as_path(),
        &["restore", "--no-enforce-worktree", "--", "tracked.txt"],
    );
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"one\n"
    );
    runs(
        &fixture,
        repo.as_path(),
        &["apply", patch_file.to_str().expect("utf-8 path")],
    );
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"two\n"
    );
    remove(&fixture);
}

/// Every decision about a discarding form leaves one line in the audit file of the common Git directory:
/// the
///  rejection in the main worktree,
/// the run in a clean linked worktree,
/// and the run under the hatch.
#[test]
fn each_decision_leaves_one_line_in_the_audit_file() {
    let fixture: Fixture = fixture("discard-audit");
    let repo: PathBuf = dirty_repository(&fixture);
    let audit: PathBuf = repo.join(".git/monochromatic-git-policy-cli-audit.jsonl");
    let rejected: Observed = run_wrapped(&fixture, repo.as_path(), &["restore", "--", "tracked.txt"]);
    assert!(rejected_by_policy(&rejected), "{}", stderr_of(&rejected));
    let text: String = std::fs::read_to_string(&audit).expect("audit file after a rejection");
    assert_eq!(text.lines().count(), 1, "{text}");
    for field in [
        "\"worktree\":\"main\"",
        "\"form\":\"RestoreWorktree\"",
        "\"outcome\":\"rejected\"",
        "\"arguments\":[\"restore\",\"--\",\"tracked.txt\"]",
    ] {
        assert!(text.contains(field), "{field} missing from {text}");
    }
    let hatch: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["stash", "--no-enforce-worktree", "--include-untracked"],
    );
    assert_eq!(hatch.code, Some(0), "{}", stderr_of(&hatch));
    let after_hatch: String = std::fs::read_to_string(&audit).expect("audit file after the hatch");
    assert_eq!(after_hatch.lines().count(), 2, "{after_hatch}");
    assert!(
        after_hatch.lines().nth(1).is_some_and(|line| { return line.contains("\"outcome\":\"hatch\""); }),
        "{after_hatch}"
    );
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    let allowed: Observed = run_wrapped(&fixture, linked.as_path(), &["switch", "topic"]);
    assert_eq!(allowed.code, Some(0), "{}", stderr_of(&allowed));
    let after_clean: String = std::fs::read_to_string(&audit).expect("audit file after a clean run");
    let last: &str = after_clean.lines().last().expect("a last line");
    for field in ["\"worktree\":\"linked\"", "\"form\":\"SwitchMove\"", "\"outcome\":\"allowed\"", "\"dirty\":false"] {
        assert!(last.contains(field), "{field} missing from {last}");
    }
    let read_only: Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--short"]);
    assert_eq!(read_only.code, Some(0), "{}", stderr_of(&read_only));
    assert_eq!(
        std::fs::read_to_string(&audit).expect("audit file").lines().count(),
        after_clean.lines().count()
    );
    remove(&fixture);
}

/// A finding reaches the caller with its hint as a separate field,
/// and the message stays the incumbent's text.
#[test]
fn a_finding_carries_its_hint_beside_an_unchanged_message() {
    let fixture: Fixture = fixture("hint-field");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"no final newline").expect("file");
    let raw: Observed = observe_raw(
        wrapped(&fixture).current_dir(repo.as_path()).args(["monochromatic-git-policy-cli", "check", "--all"]),
        b"",
    );
    let text: String = String::from_utf8_lossy(raw.stdout.as_slice()).into_owned();
    assert!(
        text.contains("\"message\":\"Non-empty text file must end with exactly one LF byte.\",\"hint\":\"Hint: if you are committing or checking this file"),
        "{text}"
    );
    assert!(text.contains("git monochromatic-git-policy-cli fix -- <path>") || text.contains("exactly one newline"), "{text}");
    remove(&fixture);
}
