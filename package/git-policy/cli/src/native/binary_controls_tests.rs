//! What:
//!  Wrapper controls and escape hatches through the built executable:
//!  where they are
//!       removed,
//!  where they are left alone,
//!  and what happens to a command Git refuses.
//! Why:
//!  A control Git sees is an "unknown option" failure;
//!  a control that hides the
//!      subcommand from the rules lets a guarded command through;
//!  a file named like a
//!      hatch must stay a file.
//!  All three are promises about the program run as `git`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['--cli-git-keep-going', 'status'], { cwd: nested }).status).toBe(1);
//! ```

/// Import the shared fixtures,
///  the bounded process helpers and the control table.
use super::support::{
    Fixture, Observed, fixture, git, porcelain, remove, repository, run_direct, run_wrapped,
    silent_success, stderr_of,
};
use git_policy_cli::policy_registry::PolicyId;

use git_policy_cli::wrapper_controls::{CONTROL_SPELLINGS, ControlMeaning};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The single require-root finding line for a command run in `nested/` of `repo`.
fn not_at_root(repo: &Path) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"require-root\",\"severity\":\"error\",\"code\":\"require-root/not-at-root\",\"message\":\"cli-git: not at the root of the git repository. Repo root is {root} but effective cwd is {root}/nested. This rule exists because agents and humans frequently forget which directory they are in. Tip: cd to {root} or pass -C {root} before the subcommand.\",\"fix\":\"none\"}}\n",
        root = repo.display()
    );
}

/// What a caller sees when require-root stops a command run in `nested/` of `repo`.
fn stopped_below_root(repo: &Path) -> Observed {
    return Observed {
        code: Some(1),
        stdout: Vec::<u8>::new(),
        stderr: not_at_root(repo).into_bytes(),
    };
}

/// `git --cli-git-keep-going status`:
///  the control is gone before any rule reads the command.
#[test]
fn keep_going_before_the_subcommand_is_removed_before_any_rule_runs() {
    let fixture: Fixture = fixture("controls-keep-going");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    // From a subdirectory require-root sees `status` and stops it; Git never runs.
    assert_eq!(
        run_wrapped(
            &fixture,
            nested.as_path(),
            &["--cli-git-keep-going", "status"]
        ),
        stopped_below_root(repo.as_path())
    );
    // Without the control the same rule gives the same answer.
    assert_eq!(
        run_wrapped(&fixture, nested.as_path(), &["status"]),
        stopped_below_root(repo.as_path())
    );
    // At the top level Git runs and receives no control, only the status-hints transform.
    std::fs::write(repo.join("untracked.txt"), b"content\n").expect("file");
    let native: Observed = run_direct(
        &fixture,
        repo.as_path(),
        &["-c", "advice.statusHints=false", "status"],
    );
    assert_eq!(native.code, Some(0));
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["--cli-git-keep-going", "status"]
        ),
        native
    );
    // Positive control: with hints left on, Git's output differs, so the comparison can fail.
    assert_ne!(run_direct(&fixture, repo.as_path(), &["status"]), native);
    remove(&fixture);
}

/// Every wrapper control is removed before the subcommand,
///  alone and between Git's own options.
#[test]
fn every_control_before_the_subcommand_is_removed() {
    let fixture: Fixture = fixture("controls-global");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    std::fs::write(repo.join("untracked.txt"), b"content\n").expect("file");
    let repo_text: String = repo.to_string_lossy().into_owned();
    let at_root: Observed = run_direct(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(at_root.stdout, b"?? untracked.txt\n");
    let below_root: Observed =
        run_direct(&fixture, nested.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(below_root.code, Some(0));
    assert_eq!(CONTROL_SPELLINGS.len(), 14);
    for spelling in CONTROL_SPELLINGS {
        assert_eq!(
            run_wrapped(
                &fixture,
                repo.as_path(),
                &[spelling.flag, "status", "--porcelain=v1"]
            ),
            at_root,
            "{}",
            spelling.flag
        );
        // Between two of Git's own global options, from outside the repository.
        assert_eq!(
            run_wrapped(
                &fixture,
                fixture.root.as_path(),
                &[
                    "-C",
                    repo_text.as_str(),
                    spelling.flag,
                    "--no-pager",
                    "status",
                    "--porcelain=v1"
                ]
            ),
            at_root,
            "{}",
            spelling.flag
        );
        // Below the top level the rule still sees `status`; only its own escape lets it through.
        let below: Observed = run_wrapped(
            &fixture,
            nested.as_path(),
            &[spelling.flag, "status", "--porcelain=v1"],
        );
        if spelling.meaning == ControlMeaning::Escape(PolicyId::RequireRoot) {
            assert_eq!(below, below_root, "{}", spelling.flag);
        } else {
            assert_eq!(
                below,
                stopped_below_root(repo.as_path()),
                "{}",
                spelling.flag
            );
        }
    }
    // Positive control: a token that is not a control reaches Git, which refuses it.
    for arguments in [
        vec!["--cli-git-keep-goin", "status"],
        vec!["--no-enforce-only", "status"],
        vec!["--no-enforce-no-such-policy", "status"],
    ] {
        let refused: Observed = run_direct(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(refused.code, Some(129), "{arguments:?}");
        assert_eq!(
            run_wrapped(&fixture, repo.as_path(), arguments.as_slice()),
            refused,
            "{arguments:?}"
        );
    }
    remove(&fixture);
}

/// `git reset -- --no-enforce-worktree` names a file;
///  the same token before `--` is a hatch.
#[test]
fn a_path_named_like_a_hatch_is_forwarded_and_a_hatch_is_removed() {
    let fixture: Fixture = fixture("controls-hatch");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=tracked"],
    );
    std::fs::write(repo.join("--no-enforce-worktree"), b"a file\n").expect("hatch-named file");
    std::fs::write(repo.join("other.txt"), b"other\n").expect("other file");
    git(
        &fixture,
        repo.as_path(),
        &["add", "--", "--no-enforce-worktree", "other.txt"],
    );
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "A  --no-enforce-worktree\nA  other.txt\n"
    );
    // A mixed reset is guarded too, so the real hatch lets it run; after `--` the token is a path: only that file is
    // unstaged, so Git received it unchanged.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["reset", "--no-enforce-worktree", "--quiet", "--", "--no-enforce-worktree"]
        ),
        silent_success()
    );
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "A  other.txt\n?? --no-enforce-worktree\n"
    );
    // A hard reset in the main worktree is rejected, and nothing changes.
    std::fs::write(repo.join("tracked.txt"), b"two\n").expect("modify tracked file");
    let rejected: Observed = run_wrapped(&fixture, repo.as_path(), &["reset", "--hard"]);
    assert_eq!(rejected.code, Some(1));
    assert_eq!(rejected.stdout, Vec::<u8>::new());
    assert!(
        stderr_of(&rejected).starts_with(
            "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"linked-worktree-only\",\"severity\":\"error\",\"code\":\"linked-worktree-only/linked-worktree-required\","
        ),
        "{}",
        stderr_of(&rejected)
    );
    // After `--` the token is still a path, so it escapes nothing.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["reset", "--hard", "--", "--no-enforce-worktree"]
        )
        .code,
        Some(1)
    );
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"two\n"
    );
    // Before `--` the token is the hatch: the reset runs and Git never sees the token.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["reset", "--no-enforce-worktree", "--hard", "--quiet"]
        ),
        silent_success()
    );
    assert_eq!(
        std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
        b"one\n"
    );
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "?? --no-enforce-worktree\n"
    );
    remove(&fixture);
}

/// A command whose options Git refuses is forwarded for Git to refuse,
///  unless it must never be forwarded.
#[test]
fn a_region_git_refuses_is_forwarded_unless_the_command_is_never_forwarded() {
    let fixture: Fixture = fixture("controls-refused");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    for (wrapped_arguments, direct_arguments) in [
        (
            vec!["reset", "--hard", "--no-such-option"],
            vec!["reset", "--hard", "--no-such-option"],
        ),
        (
            vec!["clean", "--no-such-option"],
            vec!["clean", "--no-such-option"],
        ),
        (
            vec!["switch", "--no-such-option"],
            vec!["switch", "--no-such-option"],
        ),
        // A leading control is still removed before Git refuses the rest.
        (
            vec!["add", "--no-enforce-final-newline", "--no-such-option"],
            vec!["add", "--no-such-option"],
        ),
        (
            vec!["push", "--no-enforce-final-newline", "--no-such-option"],
            vec!["push", "--no-such-option"],
        ),
        (
            vec!["status", "--no-such-option"],
            vec!["status", "--no-such-option"],
        ),
    ] {
        let refused: Observed = run_direct(&fixture, repo.as_path(), direct_arguments.as_slice());
        assert_eq!(refused.code, Some(129), "{direct_arguments:?}");
        assert_eq!(
            run_wrapped(&fixture, repo.as_path(), wrapped_arguments.as_slice()),
            refused,
            "{wrapped_arguments:?}"
        );
    }
    // A commit whose options Git refuses is never committed, and a push that may publish is refused.
    assert_ne!(
        run_wrapped(&fixture, repo.as_path(), &["commit", "--no-such-option"]).code,
        Some(0)
    );
    let push: Observed = run_wrapped(&fixture, repo.as_path(), &["push", "--no-such-option"]);
    assert_eq!(push.code, Some(2));
    // The probe asks Git, which refuses the unknown option; the push fails closed with a content-unavailable failure.
    assert!(
        stderr_of(&push).contains("\"code\":\"content-unavailable\""),
        "{}",
        stderr_of(&push)
    );
    // `git stash` is guarded in every form, so the main worktree rejects it before Git is asked.
    let stash: Observed = run_wrapped(&fixture, repo.as_path(), &["stash", "--no-such-option"]);
    assert_eq!(stash.code, Some(1));
    assert!(
        stderr_of(&stash).contains("\"policyId\":\"linked-worktree-only\""),
        "{}",
        stderr_of(&stash)
    );
    remove(&fixture);
}
