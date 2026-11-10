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
    Fixture, Observed, fixture, git, observe, observe_raw, porcelain, remove, repository, run_direct,
    run_wrapped, stderr_of, stub_manual_viewer, wrapped,
};
use git_policy_cli::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use git_policy_cli::rule_linked_worktree::GuardedCommand;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Output;

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
        GuardedCommand::StashDrop => vec![vec!["stash", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "drop"]],
        GuardedCommand::Clean => vec![vec!["clean", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "-f", "-d"]],
        GuardedCommand::CleanIgnored => vec![vec!["clean", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "-f", "-d", "-x"]],
        GuardedCommand::CleanIgnoredOnly => vec![vec!["clean", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "-f", "-X"]],
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

/// What:
///  A repository whose `.gitignore` names the environment file and a directory,
/// with both present and nothing tracked changed.
/// Why:
///  The ignored files are exactly what a `clean -x` deletes and the wrapper's own
///  `--untracked-files=no` measurement cannot see.
fn repository_with_ignored_work(fixture: &Fixture, name: &OsStr) -> PathBuf {
    let repo: PathBuf = repository(fixture, name);
    std::fs::write(repo.join(".gitignore"), b".env\nignored/\n").expect("ignore file");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    keep_ignored_work(repo.as_path());
    return repo;
}

/// Write the ignored environment file and the ignored directory again.
fn keep_ignored_work(repo: &std::path::Path) {
    std::fs::write(repo.join(".env"), b"SECRET=the person value\n").expect("ignored file");
    std::fs::create_dir_all(repo.join("ignored")).expect("ignored directory");
    std::fs::write(repo.join("ignored/data.bin"), b"ignored work\n").expect("ignored work");
}

/// Whether the ignored work is still on disk.
fn ignored_work_survives(repo: &std::path::Path) -> bool {
    return repo.join(".env").exists() && repo.join("ignored/data.bin").exists();
}

/// What:
///  The audit lines a clean leaves after the ones already in the file,
/// with the last one last.
/// Why:
///   A clean that runs must leave exactly one line naming it and the worktree it ran in,
///  and nothing else about the repository changes the count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [line] = auditLinesSince(repo, before)
/// ```
fn audit_lines_since(repo: &std::path::Path, before: usize) -> Vec<String> {
    return audit_lines_of(repo).split_off(before);
}

/// A `clean` that deletes ignored files runs in a linked worktree,
/// exits with Git's own status,
/// and leaves one audit line naming the command and the worktree,
/// because the owner answered the question about it on 2026-11-10 with
/// "All four are deleted,
///  exit 0,
///  and the audit log records the command."
#[test]
fn a_clean_that_deletes_ignored_files_runs_and_is_logged_in_a_linked_worktree() {
    let fixture: Fixture = fixture("discard-clean-ignored");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    for (arguments, form) in [
        (vec!["clean", "-fdx"], "CleanIgnored"),
        (vec!["clean", "-fdX"], "CleanIgnoredOnly"),
        (vec!["clean", "--force", "-d", "-x"], "CleanIgnored"),
    ] {
        keep_ignored_work(linked.as_path());
        let before: usize = audit_lines_of(repo.as_path()).len();
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(!rejected_by_policy(&observed), "{arguments:?}: {}", stderr_of(&observed));
        assert_eq!(observed.code, Some(0), "{arguments:?}: {}", stderr_of(&observed));
        assert!(!ignored_work_survives(linked.as_path()), "{arguments:?} kept the ignored work");
        let lines: Vec<String> = audit_lines_since(repo.as_path(), before);
        assert_eq!(lines.len(), 1, "{arguments:?} left {lines:?}");
        let last: &str = lines.first().expect("one line");
        for field in [
            format!("\"form\":\"{form}\""),
            String::from("\"outcome\":\"allowed\""),
            String::from("\"worktree\":\"linked\""),
        ] {
            assert!(last.contains(field.as_str()), "{field} missing from {last}");
        }
        // The line names the command the person typed.
        for word in &arguments {
            assert!(last.contains(format!("\"{word}\"").as_str()), "{word} missing from {last}");
        }
    }
    // The main worktree still refuses, and nothing of it is measured or deleted.
    keep_ignored_work(repo.as_path());
    let main: Observed = run_wrapped(&fixture, repo.as_path(), &["clean", "-fdx"]);
    assert!(rejected_by_policy(&main), "{}", stderr_of(&main));
    assert!(ignored_work_survives(repo.as_path()), "the main worktree lost its ignored work");
    // An untracked file is still a reason to refuse the same clean in the linked worktree.
    keep_ignored_work(linked.as_path());
    std::fs::write(linked.join("new.txt"), b"the person wrote this\n").expect("untracked file");
    let untracked: Observed = run_wrapped(&fixture, linked.as_path(), &["clean", "-fdx"]);
    assert!(rejected_by_policy(&untracked), "{}", stderr_of(&untracked));
    assert!(ignored_work_survives(linked.as_path()), "the refused clean deleted the ignored work");
    assert!(linked.join("new.txt").exists(), "the refused clean deleted the untracked file");
    std::fs::remove_file(linked.join("new.txt")).expect("drop the untracked file");
    // The dry runs list what each would delete, in every worktree.
    let dry: Observed = run_wrapped(&fixture, linked.as_path(), &["clean", "-ndx"]);
    assert!(!rejected_by_policy(&dry), "{}", stderr_of(&dry));
    assert_eq!(dry.code, Some(0), "{}", stderr_of(&dry));
    assert!(String::from_utf8_lossy(dry.stdout.as_slice()).contains(".env"), "{}", String::from_utf8_lossy(dry.stdout.as_slice()));
    assert!(ignored_work_survives(linked.as_path()), "a dry run deleted the ignored work");
    // A plain clean, which keeps the ignored files, is unaffected.
    let without_ignored: Observed = run_wrapped(&fixture, linked.as_path(), &["clean", "-fd"]);
    assert!(!rejected_by_policy(&without_ignored), "{}", stderr_of(&without_ignored));
    assert!(ignored_work_survives(linked.as_path()), "a plain clean deleted ignored work");
    remove(&fixture);
}

/// Current branch name of a fixture repository.
fn branch_of(fixture: &Fixture, repo: &std::path::Path) -> String {
    let output: std::process::Output = git(fixture, repo, &["rev-parse", "--abbrev-ref", "HEAD"]);
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

/// A `HEAD` move no longer replaces an ignored file that the branch it moves to tracks.
/// Git refuses the switch itself,
/// so nothing is replaced and nothing has to be measured.
#[test]
fn a_head_move_keeps_an_ignored_file_the_target_branch_tracks() {
    let fixture: Fixture = fixture("discard-ignored-head-move");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    // A branch that tracks the environment file the person keeps locally.
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join(".env"), b"the tracked version\n").expect("tracked environment file");
    git(&fixture, repo.as_path(), &["add", "--force", "--", ".env"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    keep_ignored_work(repo.as_path());
    let person_value: &[u8] = b"SECRET=the person value\n";
    for arguments in [
        vec!["checkout", "tracks"],
        vec!["switch", "tracks"],
        vec!["checkout", "--no-overwrite-ignore", "tracks"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(
            !rejected_by_policy(&observed),
            "{arguments:?} was rejected by the policy instead of by Git: {}",
            stderr_of(&observed)
        );
        assert_ne!(observed.code, Some(0), "{arguments:?} replaced the ignored file");
        assert_eq!(
            std::fs::read(repo.join(".env")).expect("ignored file"),
            person_value,
            "{arguments:?} replaced the ignored file"
        );
        assert_eq!(branch_of(&fixture, repo.as_path()), "main", "{arguments:?} switched anyway");
        assert!(stderr_of(&observed).contains(".env"), "{arguments:?}: {}", stderr_of(&observed));
    }
    // A branch that does not track the ignored file switches as it always did.
    git(&fixture, repo.as_path(), &["branch", "free"]);
    let free: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "free"]);
    assert_eq!(free.code, Some(0), "{}", stderr_of(&free));
    assert_eq!(branch_of(&fixture, repo.as_path()), "free");
    assert_eq!(std::fs::read(repo.join(".env")).expect("ignored file"), person_value);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    // Nothing would be lost, so the person's own spelling keeps working.
    std::fs::remove_file(repo.join(".env")).expect("drop the ignored file");
    for arguments in [vec!["checkout", "tracks"], vec!["switch", "tracks"]] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(observed.code, Some(0), "{arguments:?}: {}", stderr_of(&observed));
        assert_eq!(branch_of(&fixture, repo.as_path()), "tracks", "{arguments:?}");
        git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    }
    // A person who says out loud that the ignored file may be replaced still replaces it.
    keep_ignored_work(repo.as_path());
    let explicit: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--overwrite-ignore", "tracks"]);
    assert_eq!(explicit.code, Some(0), "{}", stderr_of(&explicit));
    assert_eq!(branch_of(&fixture, repo.as_path()), "tracks");
    assert_eq!(
        std::fs::read(repo.join(".env")).expect("ignored file"),
        b"the tracked version\n"
    );
    remove(&fixture);
}

/// A repository whose `tracks` branch holds the environment file the person also keeps locally,
/// with a linked worktree on its own branch,
/// where the local content is written again.
fn repository_that_tracks_an_ignored_file(fixture: &Fixture) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository_with_ignored_work(fixture, OsStr::new("repo"));
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join(".env"), b"the tracked version\n").expect("tracked environment file");
    git(fixture, repo.as_path(), &["add", "--force", "--", ".env"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    keep_ignored_work(linked.as_path());
    return (repo, linked);
}

/// A forced `checkout`,
/// a forced `switch` and a hard `reset` replace an ignored file the target tree tracks,
/// and the ignored file is where a person keeps their environment and their local secrets,
/// so the guard reads the target tree and the ignored paths and refuses while they share a path.
#[test]
fn a_forced_head_move_keeps_an_ignored_file_the_target_branch_tracks() {
    let fixture: Fixture = fixture("discard-forced-ignored");
    let (repo, linked): (PathBuf, PathBuf) = repository_that_tracks_an_ignored_file(&fixture);
    let person_value: &[u8] = b"SECRET=the person value\n";
    // What real Git does with the same command, so the loss the guard prevents is measured and not presumed.
    let replaced: Observed = run_direct(&fixture, linked.as_path(), &["checkout", "--force", "tracks"]);
    assert_eq!(replaced.code, Some(0), "{}", stderr_of(&replaced));
    assert_eq!(
        std::fs::read(linked.join(".env")).expect("ignored file"),
        b"the tracked version\n",
        "real Git kept the ignored file, so this control proves nothing"
    );
    git(&fixture, linked.as_path(), &["checkout", "--force", "side"]);
    keep_ignored_work(linked.as_path());
    for arguments in [
        vec!["checkout", "--force", "tracks"],
        vec!["switch", "--force", "tracks"],
        vec!["reset", "--hard", "tracks"],
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?} replaced the ignored file: {}",
            stderr_of(&observed)
        );
        assert_eq!(
            std::fs::read(linked.join(".env")).expect("ignored file"),
            person_value,
            "{arguments:?} replaced the ignored file"
        );
        assert!(stderr_of(&observed).contains(".env"), "{arguments:?}: {}", stderr_of(&observed));
    }
    // The line names what the refusal saw, so a reader knows which paths were at stake.
    let audit: PathBuf = repo.join(".git/monochromatic-git-policy-cli-audit.jsonl");
    let text: String = std::fs::read_to_string(&audit).expect("audit file");
    let last: &str = text.lines().last().expect("a last line");
    assert!(last.contains("\"form\":\"ResetHard\""), "{last}");
    assert!(last.contains("\"ignoredAndTracked\":\".env\""), "{last}");
    // A target tree that tracks no ignored path leaves the worktree nothing to lose, so the form runs.
    std::fs::remove_file(linked.join(".env")).expect("drop the ignored file");
    for arguments in [
        vec!["reset", "--hard", "side"],
        vec!["checkout", "--force", "tracks"],
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert_eq!(observed.code, Some(0), "{arguments:?}: {}", stderr_of(&observed));
    }
    remove(&fixture);
}

/// A stash entry the repository holds,
/// as the person reads it.
fn stash_entries(fixture: &Fixture, repo: &std::path::Path) -> String {
    let output: std::process::Output = git(fixture, repo, &["stash", "list"]);
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// `stash drop` and `stash clear` from a clean linked worktree discard a stash the whole repository shares,
/// so they are rejected while the repository holds one,
/// and run when it holds none.
#[test]
fn a_stash_drop_that_would_discard_the_repository_stash_is_rejected() {
    let fixture: Fixture = fixture("discard-stash-drop");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    // The person parks work in progress in the main worktree; the entry belongs to the repository.
    std::fs::write(repo.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    git(&fixture, repo.as_path(), &["stash", "push", "--quiet", "--message", "the person work"]);
    assert_eq!(stash_entries(&fixture, repo.as_path()).lines().count(), 1);
    for arguments in [vec!["stash", "drop"], vec!["stash", "clear"]] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?} in a clean linked worktree: {}",
            stderr_of(&observed)
        );
        assert_eq!(
            stash_entries(&fixture, repo.as_path()).lines().count(),
            1,
            "{arguments:?} emptied the repository stash anyway"
        );
    }
    // The deliberate override still drops it.
    let hatch: Observed = run_wrapped(&fixture, linked.as_path(), &["stash", "--no-enforce-worktree", "drop"]);
    assert_eq!(hatch.code, Some(0), "{}", stderr_of(&hatch));
    assert_eq!(stash_entries(&fixture, repo.as_path()), "");
    // With no entry left, the same command runs: there is nothing to lose.
    let empty: Observed = run_wrapped(&fixture, linked.as_path(), &["stash", "drop"]);
    assert!(!rejected_by_policy(&empty), "{}", stderr_of(&empty));
    // Reading the stash stays available everywhere.
    for arguments in [vec!["stash", "list"], vec!["stash", "show"]] {
        let read: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(!rejected_by_policy(&read), "{arguments:?}: {}", stderr_of(&read));
    }
    remove(&fixture);
}

/// An alias reaches the guard of the command it expands to,
/// so the word on the command line cannot hide a form the guard rejects.
#[test]
fn an_alias_reaches_the_guard_of_the_command_it_expands_to() {
    let fixture: Fixture = fixture("discard-alias");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    // An alias given on this command line, and one the repository configures.
    for arguments in [
        vec!["-c", "alias.wipe=reset --hard", "wipe"],
        vec!["-c", "alias.sweep=clean -fdx", "sweep"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?} reached no guard: {}",
            stderr_of(&observed)
        );
        assert_eq!(
            std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
            b"the person wrote this\n",
            "{arguments:?} discarded the edit"
        );
        assert!(ignored_work_survives(repo.as_path()), "{arguments:?} deleted the ignored work");
    }
    git(&fixture, repo.as_path(), &["config", "alias.wipe", "reset --hard"]);
    let configured: Observed = run_wrapped(&fixture, repo.as_path(), &["wipe"]);
    assert!(rejected_by_policy(&configured), "{}", stderr_of(&configured));
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"the person wrote this\n"
    );
    // The audit line names what Git will run, not the alias the person typed.
    let audit: PathBuf = repo.join(".git/monochromatic-git-policy-cli-audit.jsonl");
    let text: String = std::fs::read_to_string(&audit).expect("audit file");
    let last: &str = text.lines().last().expect("a last line");
    assert!(last.contains("\"arguments\":[\"reset\",\"--hard\"]"), "{last}");
    // An alias to a command the guard allows still runs, and an unknown name is still Git's own error.
    git(&fixture, repo.as_path(), &["config", "alias.short", "status --short"]);
    let harmless: Observed = run_wrapped(&fixture, repo.as_path(), &["short"]);
    assert_eq!(harmless.code, Some(0), "{}", stderr_of(&harmless));
    assert!(String::from_utf8_lossy(harmless.stdout.as_slice()).contains("tracked.txt"));
    let unknown: Observed = run_wrapped(&fixture, repo.as_path(), &["no-such-subcommand"]);
    assert_ne!(unknown.code, Some(0));
    assert!(!rejected_by_policy(&unknown), "{}", stderr_of(&unknown));
    // A shell alias holds a shell command, which no reading of its text can judge. Git runs it with its own
    //  directory first on `PATH`, so the `git` inside it is real Git and not this wrapper: the guard cannot see it,
    //  and the wrapper forwards the alias as it always did. The control proves which `git` the shell finds, so the
    //  limit is measured and not guessed.
    git(&fixture, repo.as_path(), &["config", "alias.fixup", "!command -v git"]);
    let shell: Observed = run_wrapped(&fixture, repo.as_path(), &["fixup"]);
    assert_eq!(shell.code, Some(0), "{}", stderr_of(&shell));
    assert!(!rejected_by_policy(&shell), "{}", stderr_of(&shell));
    let found: String = String::from_utf8_lossy(shell.stdout.as_slice()).trim().to_owned();
    assert!(found.ends_with("/git-core/git"), "the shell found {found}, not Git's own directory");
    assert!(found != fixture.root.join("bin/git").display().to_string(), "the shell found this wrapper");
    remove(&fixture);
}

/// The tool-cache exemption is chosen by the invocation's own environment,
/// so a person sees the sentence that says which variable exempted the repository.
#[test]
fn an_exempted_repository_says_which_variable_exempted_it() {
    let fixture: Fixture = fixture("discard-exemption-notice");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    // The same command without the variable is rejected and changes nothing.
    let guarded: Observed = run_wrapped(&fixture, repo.as_path(), &["reset", "--hard"]);
    assert!(rejected_by_policy(&guarded), "{}", stderr_of(&guarded));
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"the person wrote this\n"
    );
    // A variable that names the repository exempts it, and the wrapper says so before Git runs.
    let exempt: Observed = observe(
        wrapped(&fixture)
            .current_dir(repo.as_path())
            .env("MISE_DATA_DIR", repo.as_os_str())
            .args(["reset", "--hard"]),
        b"",
    );
    assert_eq!(exempt.code, Some(0), "{}", stderr_of(&exempt));
    let notice: String = stderr_of(&exempt);
    assert!(notice.contains("exempt from the worktree guard"), "{notice}");
    assert!(notice.contains("MISE_DATA_DIR"), "{notice}");
    assert!(notice.contains(repo.to_str().expect("utf-8 fixture path")), "{notice}");
    assert!(notice.contains("Hint: "), "{notice}");
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"one\n",
        "the exemption still lets the command run"
    );
    // The audit line names why as well.
    let audit: PathBuf = repo.join(".git/monochromatic-git-policy-cli-audit.jsonl");
    let text: String = std::fs::read_to_string(&audit).expect("audit file");
    let last: &str = text.lines().last().expect("a last line");
    assert!(last.contains("\"outcome\":\"exempt\""), "{last}");
    assert!(last.contains("\"exemptedBy\":\"MISE_DATA_DIR\""), "{last}");
    remove(&fixture);
}

/// The audit lines of a repository's common Git directory,
/// newest last.
fn audit_lines_of(repo: &std::path::Path) -> Vec<String> {
    let text: String = std::fs::read_to_string(repo.join(".git/monochromatic-git-policy-cli-audit.jsonl")).unwrap_or_default();
    return text.lines().map(str::to_owned).collect();
}

/// `git worktree remove` runs as it always did,
/// and one audit line says what the removal took.
#[test]
fn a_worktree_removal_records_the_worktree_force_and_what_it_took() {
    let fixture: Fixture = fixture("audit-worktree-remove");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    keep_ignored_work(linked.as_path());
    let removal: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "remove", "--force", linked.to_str().expect("utf-8 fixture path")],
    );
    assert_eq!(removal.code, Some(0), "{}", stderr_of(&removal));
    assert!(!linked.exists(), "Git removed the worktree");
    let lines: Vec<String> = audit_lines_of(repo.as_path());
    let last: &str = lines.last().expect("one audit line for the removal");
    for field in [
        "\"form\":\"WorktreeRemove\"",
        "\"outcome\":\"logged\"",
        "\"worktree\":\"<the path>\"",
        "\"force\":true",
        // The whole ref, so the branch is restored by name without a second lookup.
        "\"head\":\"refs/heads/side\"",
        // A count is text, like every fact, so a reader never has to guess a JSON type per field.
        "\"changed\":\"1\"",
        "\"counted\":\"tracked changes only\"",
    ] {
        let rendered: String = field.replace("<the path>", linked.to_str().expect("utf-8 fixture path"));
        assert!(last.contains(rendered.as_str()), "{rendered} missing from {last}");
    }
    remove(&fixture);
}

/// `git branch -D` runs as it always did,
/// and one audit line names the branch,
/// its commit and whether it was merged.
#[test]
fn a_branch_deletion_records_the_branch_its_commit_and_whether_it_was_merged() {
    let fixture: Fixture = fixture("audit-branch-delete");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    // One branch whose commit nothing else reaches, and one already merged into `main`.
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "gone"]);
    std::fs::write(repo.join("tracked.txt"), b"the lost commit\n").expect("edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=gone", "--", "tracked.txt"]);
    let commit: String = String::from_utf8_lossy(
        git(&fixture, repo.as_path(), &["rev-parse", "gone"]).stdout.as_slice(),
    ).trim().to_owned();
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    git(&fixture, repo.as_path(), &["branch", "kept"]);
    let deletion: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-D", "gone"]);
    assert_eq!(deletion.code, Some(0), "{}", stderr_of(&deletion));
    let last: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line");
    for field in [
        "\"form\":\"BranchDelete\"",
        "\"outcome\":\"logged\"",
        "\"branch\":\"gone\"",
        format!("\"commit\":\"{commit}\"").as_str(),
        "\"merged\":false",
    ] {
        assert!(last.contains(field), "{field} missing from {last}");
    }
    // A branch that `main` already contains says so, so a reader can tell the two apart.
    let merged: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-D", "kept"]);
    assert_eq!(merged.code, Some(0), "{}", stderr_of(&merged));
    let merged_line: String = audit_lines_of(repo.as_path()).last().cloned().expect("a second audit line");
    assert!(merged_line.contains("\"branch\":\"kept\""), "{merged_line}");
    assert!(merged_line.contains("\"merged\":true"), "{merged_line}");
    // Deleting nothing (a name that does not exist) writes no line and stays Git's own error.
    let before: usize = audit_lines_of(repo.as_path()).len();
    let missing: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-D", "no-such-branch"]);
    assert_ne!(missing.code, Some(0));
    assert_eq!(audit_lines_of(repo.as_path()).len(), before);
    remove(&fixture);
}

/// An alias is forwarded as the command it expands to,
/// so the transform that keeps an ignored file sees the
///  same command the guard decided on.
#[test]
fn an_alias_reaches_the_transform_that_keeps_ignored_files() {
    let fixture: Fixture = fixture("discard-alias-transform");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join(".env"), b"the tracked version\n").expect("tracked environment file");
    git(&fixture, repo.as_path(), &["add", "--force", "--", ".env"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    keep_ignored_work(repo.as_path());
    let person_value: &[u8] = b"SECRET=the person value\n";
    // The same command by name, and through an alias configured in the repository.
    for arguments in [vec!["checkout", "tracks"], vec!["go"]] {
        if arguments[0] == "go" {
            git(&fixture, repo.as_path(), &["config", "alias.go", "checkout tracks"]);
        }
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(!rejected_by_policy(&observed), "{arguments:?} was rejected by the policy: {}", stderr_of(&observed));
        assert_ne!(observed.code, Some(0), "{arguments:?} replaced the ignored file");
        assert_eq!(
            std::fs::read(repo.join(".env")).expect("ignored file"),
            person_value,
            "{arguments:?} replaced the ignored file"
        );
        assert_eq!(branch_of(&fixture, repo.as_path()), "main", "{arguments:?} switched anyway");
        assert!(stderr_of(&observed).contains(".env"), "{arguments:?}: {}", stderr_of(&observed));
    }
    // The alias reaches the guard too, so a form the guard rejects is rejected through it.
    git(&fixture, repo.as_path(), &["config", "alias.wipe", "reset --hard"]);
    std::fs::write(repo.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    let wiped: Observed = run_wrapped(&fixture, repo.as_path(), &["wipe"]);
    assert!(rejected_by_policy(&wiped), "{}", stderr_of(&wiped));
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"the person wrote this\n"
    );
    remove(&fixture);
}

/// A clean that deletes ignored files and carries its own exclusion patterns is measured with those patterns,
/// because Git applies them to the set it deletes.
#[test]
fn a_clean_with_an_exclusion_is_measured_with_that_exclusion() {
    let fixture: Fixture = fixture("discard-clean-exclude");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("notes.txt"), b"the person notes\n").expect("untracked file");
    // What real Git does with the same command, so the deletion the guard prevents is measured and not presumed.
    let deleted: Observed = run_direct(&fixture, linked.as_path(), &["clean", "--force", "-X", "--exclude=notes.txt"]);
    assert_eq!(deleted.code, Some(0), "{}", stderr_of(&deleted));
    assert!(
        !linked.join("notes.txt").exists(),
        "real Git kept the file, so this control proves nothing"
    );
    for exclusion in [
        vec!["--exclude=notes.txt"],
        vec!["--exclude", "notes.txt"],
        vec!["-e", "notes.txt"],
        vec!["-enotes.txt"],
    ] {
        std::fs::write(linked.join("notes.txt"), b"the person notes\n").expect("untracked file");
        let mut arguments: Vec<&str> = vec!["clean", "--force", "-X"];
        arguments.extend(exclusion.iter().copied());
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?} deleted the file: {}",
            stderr_of(&observed)
        );
        assert!(
            linked.join("notes.txt").exists(),
            "{arguments:?} deleted the excluded file anyway"
        );
    }
    // Nothing to delete is nothing to refuse: the same spelling runs where the excluded path is absent.
    std::fs::remove_file(linked.join("notes.txt")).expect("drop the untracked file");
    let empty: Observed = run_wrapped(&fixture, linked.as_path(), &["clean", "--force", "-X", "--exclude=notes.txt"]);
    assert_eq!(empty.code, Some(0), "{}", stderr_of(&empty));
    // The ignored work the same spelling deletes is no longer a reason to refuse it, so it runs and goes.
    keep_ignored_work(linked.as_path());
    let deleting: Observed = run_wrapped(&fixture, linked.as_path(), &["clean", "--force", "-X", "--exclude=notes.txt"]);
    assert!(!rejected_by_policy(&deleting), "{}", stderr_of(&deleting));
    assert_eq!(deleting.code, Some(0), "{}", stderr_of(&deleting));
    assert!(!ignored_work_survives(linked.as_path()), "the ignored work stayed");
    remove(&fixture);
}

/// The escape keeps the audit line of a form that is only recorded:
/// the owner asked for the line,
///  and the
///  hatch skips the policy that would otherwise write it.
#[test]
fn an_escaped_recorded_form_still_leaves_its_audit_line() {
    let fixture: Fixture = fixture("audit-escaped-recorded");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    // A branch whose commit nothing else reaches, deleted under the hatch.
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "gone"]);
    std::fs::write(repo.join("tracked.txt"), b"the lost commit\n").expect("edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=gone", "--", "tracked.txt"]);
    let commit: String =
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "gone"]).stdout.as_slice())
            .trim()
            .to_owned();
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let deletion: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "--no-enforce-worktree", "-D", "gone"]);
    assert_eq!(deletion.code, Some(0), "{}", stderr_of(&deletion));
    let lines: Vec<String> = audit_lines_of(repo.as_path());
    let last: &str = lines.last().expect("the deletion left a line");
    for field in [
        "\"form\":\"BranchDelete\"",
        "\"outcome\":\"logged\"",
        "\"branch\":\"gone\"",
        format!("\"commit\":\"{commit}\"").as_str(),
    ] {
        assert!(last.contains(field), "{field} missing from {last}");
    }
    // The same for a removal under the hatch, which would otherwise leave nothing either.
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    let removal: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "--no-enforce-worktree", "remove", "--force", linked.to_str().expect("utf-8 fixture path")],
    );
    assert_eq!(removal.code, Some(0), "{}", stderr_of(&removal));
    assert!(!linked.exists(), "Git removed the worktree");
    let after: Vec<String> = audit_lines_of(repo.as_path());
    assert_eq!(after.len(), lines.len() + 1, "{after:?}");
    let removal_line: &str = after.last().expect("the removal left a line");
    for field in [
        "\"form\":\"WorktreeRemove\"",
        "\"outcome\":\"logged\"",
        "\"head\":\"refs/heads/side\"",
    ] {
        assert!(removal_line.contains(field), "{field} missing from {removal_line}");
    }
    remove(&fixture);
}

/// A record of a recorded form is written before Git runs,
/// so it says that,
///  and a reader cannot take it for the result:
/// a deletion Git refused leaves the same line as one it ran.
#[test]
fn a_recorded_form_says_its_line_was_written_before_git_ran() {
    let fixture: Fixture = fixture("audit-before-git");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    // A branch whose commit nothing else reaches and that no upstream has: `-d` refuses to delete it.
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "gone"]);
    std::fs::write(repo.join("tracked.txt"), b"the lost commit\n").expect("edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=gone", "--", "tracked.txt"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-d", "gone"]);
    assert_ne!(refused.code, Some(0), "Git refuses an unmerged branch");
    assert_eq!(
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["branch", "--list", "gone"]).stdout.as_slice()),
        "  gone\n",
        "the branch is still there"
    );
    let refused_line: String = audit_lines_of(repo.as_path()).last().cloned().expect("the line of a refused deletion");
    for field in [
        "\"form\":\"BranchDelete\"",
        "\"outcome\":\"logged\"",
        "\"branch\":\"gone\"",
        "\"writtenBeforeGitRuns\":true",
    ] {
        assert!(refused_line.contains(field), "{field} missing from {refused_line}");
    }
    // A deletion Git runs leaves the same statement, because the line still cannot know what Git did.
    let deleted: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-D", "gone"]);
    assert_eq!(deleted.code, Some(0), "{}", stderr_of(&deleted));
    let deleted_line: String = audit_lines_of(repo.as_path()).last().cloned().expect("the line of a run deletion");
    assert!(deleted_line.contains("\"writtenBeforeGitRuns\":true"), "{deleted_line}");
    // A removal Git refuses keeps the worktree and says the same about its line.
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    let kept: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "remove", linked.to_str().expect("utf-8 fixture path")],
    );
    assert_ne!(kept.code, Some(0), "Git refuses a dirty worktree without --force");
    assert!(linked.exists(), "the worktree is still there");
    let kept_line: String = audit_lines_of(repo.as_path()).last().cloned().expect("the line of a refused removal");
    for field in [
        "\"form\":\"WorktreeRemove\"",
        "\"writtenBeforeGitRuns\":true",
        "\"head\":\"refs/heads/side\"",
    ] {
        assert!(kept_line.contains(field), "{field} missing from {kept_line}");
    }
    // A decision the policy makes carries no such field: it says what the wrapper did, before Git was asked to run.
    git(&fixture, linked.as_path(), &["checkout", "--quiet", "--", "tracked.txt"]);
    let allowed: Observed = run_wrapped(&fixture, linked.as_path(), &["restore", "--staged", "tracked.txt"]);
    assert_eq!(allowed.code, Some(0), "{}", stderr_of(&allowed));
    let allowed_line: String = audit_lines_of(repo.as_path()).last().cloned().expect("the line of a decision");
    assert!(allowed_line.contains("\"outcome\":\"allowed\""), "{allowed_line}");
    assert!(!allowed_line.contains("writtenBeforeGitRuns"), "{allowed_line}");
    remove(&fixture);
}

/// A deletion of a remote-tracking branch records that reference and its own commit,
/// not the local branch of
///  the same name.
#[test]
fn a_remote_tracking_deletion_records_the_reference_it_deleted() {
    let fixture: Fixture = fixture("audit-branch-delete-remotes");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    // A local branch named like the remote-tracking one, and a remote-tracking ref on another commit.
    git(&fixture, repo.as_path(), &["branch", "origin/gone"]);
    let local: String =
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "origin/gone"]).stdout.as_slice())
            .trim()
            .to_owned();
    std::fs::write(repo.join("tracked.txt"), b"the remote commit\n").expect("edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=remote", "--", "tracked.txt"]);
    let remote: String =
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice())
            .trim()
            .to_owned();
    assert_ne!(local, remote, "the control needs two different commits");
    git(&fixture, repo.as_path(), &["update-ref", "refs/remotes/origin/gone", remote.as_str()]);
    for spelling in [
        vec!["branch", "--remotes", "-D", "origin/gone"],
        vec!["branch", "--remote", "-D", "origin/gone"],
        vec!["branch", "-rD", "origin/gone"],
    ] {
        // The remote-tracking ref is there again before each spelling, and the local branch never moves.
        git(&fixture, repo.as_path(), &["update-ref", "refs/remotes/origin/gone", remote.as_str()]);
        let deletion: Observed = run_wrapped(&fixture, repo.as_path(), spelling.as_slice());
        assert_eq!(deletion.code, Some(0), "{spelling:?}: {}", stderr_of(&deletion));
        let line: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line");
        for field in [
            "\"branch\":\"origin/gone\"",
            "\"reference\":\"refs/remotes/origin/gone\"",
            format!("\"commit\":\"{remote}\"").as_str(),
        ] {
            assert!(line.contains(field), "{spelling:?}: {field} missing from {line}");
        }
        assert!(!line.contains(local.as_str()), "{spelling:?}: the local commit is in {line}");
        // Git deleted the remote-tracking ref and left the local branch where it was.
        let listed: String = String::from_utf8_lossy(
            git(&fixture, repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/remotes"]).stdout.as_slice(),
        )
        .trim()
        .to_owned();
        assert_eq!(listed, "", "{spelling:?} left a remote-tracking ref");
        assert_eq!(
            String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "refs/heads/origin/gone"]).stdout.as_slice())
                .trim()
                .to_owned(),
            local,
            "{spelling:?} moved the local branch"
        );
    }
    // A remote-tracking name the local branch does not have still records the reference it deleted.
    git(&fixture, repo.as_path(), &["update-ref", "refs/remotes/origin/other", remote.as_str()]);
    let other: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-r", "-D", "origin/other"]);
    assert_eq!(other.code, Some(0), "{}", stderr_of(&other));
    let other_line: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line");
    assert!(other_line.contains("\"reference\":\"refs/remotes/origin/other\""), "{other_line}");
    assert!(other_line.contains(remote.as_str()), "{other_line}");
    remove(&fixture);
}

/// A remote-tracking deletion of the upstream mark records the reference Git expanded it to,
/// and a fully
///  qualified operand,
///  which Git refuses,
///  leaves no line.
#[test]
fn a_remote_tracking_deletion_of_the_upstream_mark_records_what_git_deleted() {
    let fixture: Fixture = fixture("audit-branch-delete-upstream");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let commit: String =
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "main"]).stdout.as_slice())
            .trim()
            .to_owned();
    // An upstream stored as a remote-tracking reference, which is what a remote-tracking deletion expands `@{u}` to.
    let place: String = repo.display().to_string();
    git(&fixture, repo.as_path(), &["config", "remote.origin.url", place.as_str()]);
    git(&fixture, repo.as_path(), &["config", "remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"]);
    git(&fixture, repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(&fixture, repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    git(&fixture, repo.as_path(), &["update-ref", "refs/remotes/origin/main", commit.as_str()]);
    // The control: Git itself expands the mark to the reference the line must name.
    assert_eq!(
        String::from_utf8_lossy(
            git(&fixture, repo.as_path(), &["rev-parse", "--symbolic-full-name", "@{u}"]).stdout.as_slice()
        )
        .trim(),
        "refs/remotes/origin/main",
        "the control needs @{{u}} to name the remote-tracking ref"
    );
    let deletion: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-r", "-D", "@{u}"]);
    assert_eq!(deletion.code, Some(0), "{}", stderr_of(&deletion));
    let line: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line");
    for field in [
        "\"branch\":\"origin/main\"",
        "\"reference\":\"refs/remotes/origin/main\"",
        format!("\"commit\":\"{commit}\"").as_str(),
    ] {
        assert!(line.contains(field), "{field} missing from {line}");
    }
    assert_eq!(
        String::from_utf8_lossy(
            git(&fixture, repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/remotes"]).stdout.as_slice()
        )
        .trim(),
        "",
        "Git deleted the remote-tracking ref"
    );
    // A fully qualified operand is no mark: Git prefixes it again, deletes nothing, and no line is written.
    git(&fixture, repo.as_path(), &["update-ref", "refs/remotes/origin/other", commit.as_str()]);
    let before: usize = audit_lines_of(repo.as_path()).len();
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-r", "-D", "refs/remotes/origin/other"]);
    assert_ne!(refused.code, Some(0), "Git refuses a fully qualified remote-tracking operand");
    assert_eq!(
        String::from_utf8_lossy(
            git(&fixture, repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/remotes"]).stdout.as_slice()
        )
        .trim(),
        "refs/remotes/origin/other",
        "the control needs the reference to survive"
    );
    assert_eq!(
        audit_lines_of(repo.as_path()).len(),
        before,
        "a deletion that deleted nothing wrote {:?}",
        audit_lines_of(repo.as_path())
    );
    remove(&fixture);
}

/// A repository whose `tracks` branch holds a file under a directory the ignore rules hide,
/// and a linked
///  worktree that holds the person's file where that directory must go.
fn repository_whose_target_needs_a_directory(fixture: &Fixture) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"private\n").expect("ignore file");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::create_dir(repo.join("private")).expect("directory");
    std::fs::write(repo.join("private/config.txt"), b"the tracked version\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--force", "--", "private/config.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("private"), b"the person file\n").expect("ignored file");
    return (repo, linked);
}

/// A forced form whose target needs a directory where the worktree has an ignored file removes that file,
/// so the guard refuses and names it.
#[test]
fn a_forced_form_keeps_an_ignored_file_where_the_target_needs_a_directory() {
    let fixture: Fixture = fixture("discard-ignored-ancestor");
    let (repo, linked): (PathBuf, PathBuf) = repository_whose_target_needs_a_directory(&fixture);
    for arguments in [
        vec!["reset", "--hard", "tracks"],
        vec!["checkout", "--force", "tracks"],
        vec!["switch", "--force", "tracks"],
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?} removed the ignored file: {}",
            stderr_of(&observed)
        );
        assert_eq!(
            std::fs::read(linked.join("private")).expect("ignored file"),
            b"the person file\n",
            "{arguments:?} removed the ignored file"
        );
        assert!(stderr_of(&observed).contains("private"), "{arguments:?}: {}", stderr_of(&observed));
    }
    // The line names the path the refusal saw.
    let text: String = std::fs::read_to_string(repo.join(".git/monochromatic-git-policy-cli-audit.jsonl"))
        .expect("the audit file of the repository");
    let last: &str = text.lines().last().expect("a last line");
    assert!(last.contains("\"ignoredAndTracked\":\"private\""), "{last}");
    // Real Git removes the file to make the directory, so the loss the guard prevents is measured.
    let removed: Observed = run_direct(&fixture, linked.as_path(), &["reset", "--hard", "tracks"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    assert!(
        !linked.join("private").is_file(),
        "real Git kept the ignored file, so this control proves nothing"
    );
    assert_eq!(
        std::fs::read(linked.join("private/config.txt")).expect("tracked file"),
        b"the tracked version\n"
    );
    remove(&fixture);
}

/// A forced switch to the previous branch keeps an ignored file that branch tracks,
/// in both shorthands Git
///  reads for it.
#[test]
fn a_forced_switch_to_the_previous_branch_keeps_an_ignored_file() {
    let fixture: Fixture = fixture("discard-previous-branch");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join(".env"), b"the tracked version\n").expect("tracked environment file");
    git(&fixture, repo.as_path(), &["add", "--force", "--", ".env"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    // The worktree visited `tracks` last, so both `-` and `@{-1}` name it.
    git(&fixture, linked.as_path(), &["switch", "--quiet", "tracks"]);
    git(&fixture, linked.as_path(), &["switch", "--quiet", "side"]);
    std::fs::write(linked.join(".env"), b"SECRET=the person value\n").expect("ignored file");
    // The named expression is the previous branch, and `-` is Git's own shorthand for it in a switch.
    let resolved: Output = git(&fixture, linked.as_path(), &["rev-parse", "--symbolic-full-name", "@{-1}"]);
    assert_eq!(
        String::from_utf8_lossy(resolved.stdout.as_slice()).trim(),
        "refs/heads/tracks",
        "the control needs the previous-branch expression to name the branch"
    );
    let replaced: Observed = run_direct(&fixture, linked.as_path(), &["switch", "--force", "-"]);
    assert_eq!(replaced.code, Some(0), "{}", stderr_of(&replaced));
    assert_eq!(branch_of(&fixture, linked.as_path()), "tracks", "Git reads - as the previous branch");
    assert_eq!(
        std::fs::read(linked.join(".env")).expect("ignored file"),
        b"the tracked version\n",
        "real Git kept the ignored file, so this control proves nothing"
    );
    // Back to where the person was, with the person's content again.
    git(&fixture, linked.as_path(), &["switch", "--force", "side"]);
    std::fs::write(linked.join(".env"), b"SECRET=the person value\n").expect("ignored file");
    for expression in ["-", "@{-1}"] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["switch", "--force", expression]);
        assert!(
            rejected_by_policy(&observed),
            "switch --force {expression} replaced the ignored file: {}",
            stderr_of(&observed)
        );
        assert_eq!(
            std::fs::read(linked.join(".env")).expect("ignored file"),
            b"SECRET=the person value\n",
            "switch --force {expression} replaced the ignored file"
        );
        assert!(stderr_of(&observed).contains(".env"), "{expression}: {}", stderr_of(&observed));
        assert_eq!(branch_of(&fixture, linked.as_path()), "side", "switch --force {expression} moved HEAD");
    }
    // A branch that tracks nothing the worktree ignores still switches, in the same shorthand.
    git(&fixture, linked.as_path(), &["switch", "--quiet", "tracks"]);
    git(&fixture, linked.as_path(), &["switch", "--quiet", "side"]);
    std::fs::write(linked.join(".env"), b"SECRET=the person value\n").expect("ignored file");
    git(&fixture, repo.as_path(), &["branch", "free"]);
    git(&fixture, linked.as_path(), &["switch", "--quiet", "free"]);
    git(&fixture, linked.as_path(), &["switch", "--quiet", "side"]);
    let free: Observed = run_wrapped(&fixture, linked.as_path(), &["switch", "--force", "-"]);
    assert_eq!(free.code, Some(0), "{}", stderr_of(&free));
    assert_eq!(branch_of(&fixture, linked.as_path()), "free");
    remove(&fixture);
}

/// A deletion that names the previous branch resolves the name Git resolves,
/// so the line names the branch and
///  the commit it took.
#[test]
fn a_deletion_of_the_previous_branch_records_the_branch_it_named() {
    let fixture: Fixture = fixture("audit-previous-branch");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "lost"]);
    std::fs::write(repo.join("tracked.txt"), b"the lost commit\n").expect("edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=lost", "--", "tracked.txt"]);
    let commit: String =
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["rev-parse", "lost"]).stdout.as_slice())
            .trim()
            .to_owned();
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    // Git reads `@{-1}` as the branch the worktree visited before `main`, which is `lost`.
    let resolved: String = String::from_utf8_lossy(
        git(&fixture, repo.as_path(), &["rev-parse", "--symbolic-full-name", "@{-1}"]).stdout.as_slice(),
    )
    .trim()
    .to_owned();
    assert_eq!(resolved, "refs/heads/lost", "the control needs the previous-branch expression to name lost");
    let deletion: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-D", "@{-1}"]);
    assert_eq!(deletion.code, Some(0), "{}", stderr_of(&deletion));
    assert_eq!(
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["branch", "--list", "lost"]).stdout.as_slice()),
        "",
        "Git deleted the branch"
    );
    let line: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line for the deletion");
    for field in [
        "\"form\":\"BranchDelete\"",
        "\"branch\":\"lost\"",
        "\"reference\":\"refs/heads/lost\"",
        format!("\"commit\":\"{commit}\"").as_str(),
    ] {
        assert!(line.contains(field), "{field} missing from {line}");
    }
    remove(&fixture);
}

/// A removal that names a worktree by its unique suffix counts that worktree,
/// not a directory of the same
///  name beside the command.
#[test]
fn a_removal_by_suffix_counts_the_worktree_git_resolves() {
    let fixture: Fixture = fixture("audit-worktree-suffix");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let worktrees: PathBuf = fixture.root.join("worktrees");
    std::fs::create_dir(&worktrees).expect("worktrees directory");
    let linked: PathBuf = worktrees.join("topic");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../worktrees/topic"]);
    std::fs::write(linked.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    // A directory of the same name beside the command, which is not a worktree at all.
    std::fs::create_dir(repo.join("topic")).expect("a directory of the same name");
    let removal: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "--force", "topic"]);
    assert_eq!(removal.code, Some(0), "{}", stderr_of(&removal));
    assert!(!linked.exists(), "Git removed the worktree the suffix names");
    let line: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line");
    for field in [
        "\"form\":\"WorktreeRemove\"",
        "\"worktree\":\"topic\"",
        "\"head\":\"refs/heads/side\"",
        "\"changed\":\"1\"",
    ] {
        assert!(line.contains(field), "{field} missing from {line}");
    }
    remove(&fixture);
}

/// A repository whose `tracks` branch holds a regular file where the ignore rules hide a directory,
/// and a linked
///  worktree that keeps the person's files inside that directory.
fn repository_whose_target_replaces_a_directory(fixture: &Fixture) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"private\n").expect("ignore file");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join("private"), b"the tracked version\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--force", "--", "private"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::create_dir(linked.join("private")).expect("ignored directory");
    std::fs::write(linked.join("private/secret.txt"), b"the person secret\n").expect("ignored file");
    return (repo, linked);
}

/// A forced form whose target holds a regular file where the worktree keeps an ignored directory deletes the
/// files inside it,
/// so the guard refuses and names the file.
#[test]
fn a_forced_form_keeps_an_ignored_directory_the_target_replaces_with_a_file() {
    let fixture: Fixture = fixture("discard-ignored-inside-replaced");
    let (repo, linked): (PathBuf, PathBuf) = repository_whose_target_replaces_a_directory(&fixture);
    for arguments in [
        vec!["reset", "--hard", "tracks"],
        vec!["checkout", "--force", "tracks"],
        vec!["switch", "--force", "tracks"],
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(
            rejected_by_policy(&observed),
            "{arguments:?} deleted the ignored file: {}",
            stderr_of(&observed)
        );
        assert_eq!(
            std::fs::read(linked.join("private/secret.txt")).expect("ignored file"),
            b"the person secret\n",
            "{arguments:?} deleted the ignored file"
        );
        assert!(
            stderr_of(&observed).contains("private/secret.txt"),
            "{arguments:?}: {}",
            stderr_of(&observed)
        );
    }
    // The line names the file the refusal saw.
    let text: String = std::fs::read_to_string(repo.join(".git/monochromatic-git-policy-cli-audit.jsonl"))
        .expect("the audit file of the repository");
    let last: &str = text.lines().last().expect("a last line");
    assert!(last.contains("\"ignoredAndTracked\":\"private/secret.txt\""), "{last}");
    // Real Git replaces the directory with the tracked file, so the loss the guard prevents is measured.
    let removed: Observed = run_direct(&fixture, linked.as_path(), &["reset", "--hard", "tracks"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    assert!(
        !linked.join("private/secret.txt").exists(),
        "real Git kept the ignored file, so this control proves nothing"
    );
    assert_eq!(
        std::fs::read(linked.join("private")).expect("tracked file"),
        b"the tracked version\n"
    );
    remove(&fixture);
}

/// A branch whose name holds bytes that are not UTF-8 keeps its deletion line,
/// with the reference and the
///  commit a reader restores from.
#[cfg(unix)]
#[test]
fn a_deletion_of_a_branch_name_that_is_not_utf8_records_its_commit() {
    use std::os::unix::ffi::OsStringExt;
    let fixture: Fixture = fixture("audit-non-utf8-branch");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    // A branch name Git accepts and no text decoding can hold: `lost-` and the byte 0xff.
    let name: std::ffi::OsString = std::ffi::OsString::from_vec(b"lost-\xff".to_vec());
    git(&fixture, repo.as_path(), &[OsStr::new("checkout"), OsStr::new("--quiet"), OsStr::new("-b"), name.as_os_str()]);
    std::fs::write(repo.join("tracked.txt"), b"the lost commit\n").expect("edit");
    git(&fixture, repo.as_path(), &[OsStr::new("commit"), OsStr::new("--quiet"), OsStr::new("--message=lost"), OsStr::new("--"), OsStr::new("tracked.txt")]);
    let commit: String = String::from_utf8_lossy(
        git(&fixture, repo.as_path(), &[OsStr::new("rev-parse"), name.as_os_str()]).stdout.as_slice(),
    )
    .trim()
    .to_owned();
    git(&fixture, repo.as_path(), &[OsStr::new("checkout"), OsStr::new("--quiet"), OsStr::new("main")]);
    // The wrapper deletes it under its own spelling.
    let observed: Observed = observe(
        wrapped(&fixture).current_dir(repo.as_path()).args([
            std::ffi::OsString::from("branch"),
            std::ffi::OsString::from("-D"),
            std::ffi::OsString::from("--"),
            name.clone(),
        ]),
        b"",
    );
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    let listed: String = String::from_utf8_lossy(
        git(&fixture, repo.as_path(), &[OsStr::new("for-each-ref"), OsStr::new("--format=%(refname)"), OsStr::new("refs/heads")])
            .stdout
            .as_slice(),
    )
    .trim()
    .to_owned();
    assert!(!listed.contains("lost-"), "Git deleted the branch: {listed}");
    let line: String = audit_lines_of(repo.as_path())
        .last()
        .cloned()
        .expect("one audit line for the deletion");
    assert!(line.contains("\"form\":\"BranchDelete\""), "{line}");
    assert!(line.contains(format!("\"commit\":\"{commit}\"").as_str()), "{line}");
    // The line names the whole reference it deleted, byte for byte, so `git update-ref` restores that branch
    //  and not a second one spelled with replacement characters.
    let mut reference_bytes: Vec<u8> = b"refs/heads/".to_vec();
    reference_bytes.extend_from_slice(name.as_encoded_bytes());
    let exact: String = format!(
        "\"referenceHex\":\"{}\"",
        git_policy_cli::path_bytes_in_records::hex_of_bytes(reference_bytes.as_slice())
    );
    assert!(line.contains(exact.as_str()), "{exact} missing from {line}");
    // The text field beside it is the display name, which holds replacement characters and restores nothing.
    assert!(line.contains("\"reference\":\"refs/heads/lost-\u{fffd}\""), "{line}");
    remove(&fixture);
}

/// A worktree path holding a line break is read whole,
/// so a removal by its unique suffix counts that worktree and
///  not a directory beside the command.
#[test]
fn a_removal_counts_the_worktree_whose_path_holds_a_line_break() {
    let fixture: Fixture = fixture("audit-worktree-line-break");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let worktrees: PathBuf = fixture.root.join("worktrees");
    std::fs::create_dir(&worktrees).expect("worktrees directory");
    let linked: PathBuf = worktrees.join("line\nbreak/topic");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../worktrees/line\nbreak/topic"]);
    assert!(linked.is_dir(), "the fixture needs a worktree whose path holds a line break");
    std::fs::write(linked.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    // A directory of the same name beside the command, which is not a worktree at all.
    std::fs::create_dir(repo.join("topic")).expect("a directory of the same name");
    let removal: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "--force", "topic"]);
    assert_eq!(removal.code, Some(0), "{}", stderr_of(&removal));
    assert!(!linked.exists(), "Git removed the worktree the suffix names");
    let line: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line");
    for field in [
        "\"form\":\"WorktreeRemove\"",
        "\"worktree\":\"topic\"",
        "\"head\":\"refs/heads/side\"",
        "\"changed\":\"1\"",
    ] {
        assert!(line.contains(field), "{field} missing from {line}");
    }
    remove(&fixture);
}

/// A deletion whose name is no branch writes no line:
/// `HEAD` is not a branch name to Git,
///  so the record must not
///  name the branch it resolves to.
#[test]
fn a_deletion_of_a_revision_that_is_no_branch_leaves_no_line() {
    let fixture: Fixture = fixture("audit-branch-delete-head");
    let repo: PathBuf = repository_with_ignored_work(&fixture, OsStr::new("repo"));
    let before: usize = audit_lines_of(repo.as_path()).len();
    // Git refuses this deletion: `HEAD` names no branch, so nothing is deleted.
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-D", "HEAD"]);
    assert_ne!(refused.code, Some(0), "Git refuses branch -D HEAD");
    assert!(
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["branch", "--list", "main"]).stdout.as_slice())
            .contains("main"),
        "the control needs main to survive"
    );
    assert_eq!(
        audit_lines_of(repo.as_path()).len(),
        before,
        "a deletion that deleted nothing wrote {:?}",
        audit_lines_of(repo.as_path())
    );
    // A reference literally named `refs/heads/HEAD` is a branch of its own, and Git deletes that one.
    // `git branch` cannot create the name, so the reference is written by `update-ref`.
    git(&fixture, repo.as_path(), &["update-ref", "refs/heads/HEAD", "main"]);
    let deleted: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "-D", "HEAD"]);
    assert_eq!(deleted.code, Some(0), "{}", stderr_of(&deleted));
    assert_eq!(
        String::from_utf8_lossy(git(&fixture, repo.as_path(), &["branch", "--list", "HEAD"]).stdout.as_slice()),
        "",
        "Git deleted the branch named HEAD"
    );
    let line: String = audit_lines_of(repo.as_path()).last().cloned().expect("one audit line");
    // `refs/heads/HEAD` in bytes: the identity of the branch Git deleted.
    assert!(line.contains("\"reference\":\"refs/heads/HEAD\""), "{line}");
    assert!(line.contains("\"referenceHex\":\"726566732f68656164732f48454144\""), "{line}");
    remove(&fixture);
}
