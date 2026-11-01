//! What:
//!  The refusal frontier through the built executable:
//!  every command that needs work
//!       this executable does not do stops with exit status 2,
//!  names what is missing and
//!       leaves the repository untouched.
//! Why:
//!  A commit,
//!  a `git add` the unported content policy should check,
//!  a real push,
//!  a worktree copy or a command beside
//!      a landing transaction that reached Git would run without the installed wrapper's
//!      protection,
//!  and nothing would show it.
//!  Each refusal has a positive control:
//!  the
//!      neighbouring command that is forwarded really changes the repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['commit', '-m', 'x']).status).toBe(2); expect(head()).toBe(before);
//! ```

/// Import the shared fixtures,
///  the bounded process helpers and the refusal texts.
use super::support::{
    Fixture, Observed, TRANSACTION_ID, fixture, git, observe, porcelain, registered_transaction,
    remove, repository, run_direct, run_wrapped, silent_success, stderr_of, stopped_with, wrapped,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The commit `HEAD` names,
///  as text.
fn head(fixture: &Fixture, repo: &Path) -> String {
    let output: std::process::Output = git(fixture, repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// Every ref of a repository,
///  one name per line.
fn refs(fixture: &Fixture, repo: &Path) -> String {
    let output: std::process::Output = git(fixture, repo, &["for-each-ref", "--format=%(refname)"]);
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// A repository with a linked worktree `linked` on branch `topic`.
fn repository_with_linked_worktree(fixture: &Fixture) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("linked");
    git(
        fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", "../linked"],
    );
    return (repo, linked);
}

/// A dry run is forwarded, with the commit-only insertion, and commits nothing;
///  the index and `HEAD` stay as they were.
#[test]
fn a_dry_run_commit_is_forwarded_and_commits_nothing() {
    let fixture: Fixture = fixture("frontier-commit");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("file.txt"), b"content\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "file.txt"]);
    let head_before: String = head(&fixture, repo.as_path());
    let native: Observed = run_direct(
        &fixture,
        repo.as_path(),
        &["commit", "-o", "--dry-run", "-m", "dry", "--", "file.txt"],
    );
    assert_eq!(native.code, Some(0));
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["commit", "--dry-run", "-m", "dry", "--", "file.txt"]
        ),
        native
    );
    assert_eq!(head(&fixture, repo.as_path()), head_before);
    // Git itself makes every status-format option a dry run, so those are forwarded too.
    for format in ["--short", "--porcelain", "--long", "-z"] {
        let through_git: Observed = run_direct(
            &fixture,
            repo.as_path(),
            &["commit", "-o", format, "--", "file.txt"],
        );
        assert_eq!(through_git.code, Some(0), "{format}");
        assert_eq!(
            run_wrapped(
                &fixture,
                repo.as_path(),
                &["commit", format, "--", "file.txt"]
            ),
            through_git,
            "{format}"
        );
        assert_eq!(head(&fixture, repo.as_path()), head_before, "{format}");
    }
    assert_eq!(porcelain(&fixture, repo.as_path()), "A  file.txt\n");
    remove(&fixture);
}

/// The final-newline warning of `git add` about `file.txt`.
const FILE_WARNING: &str = "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"final-newline\",\"severity\":\"warn\",\"code\":\"final-newline/noncanonical-final-newline\",\"message\":\"Non-empty text file must end with exactly one LF byte.\",\"path\":\"file.txt\",\"fix\":\"none\"}\n";

/// The forbidden-root-context error of `git add` about the top-level `CONTEXT.md`.
const CONTEXT_ERROR: &str = "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"mono/forbidden-root-context\",\"severity\":\"error\",\"code\":\"mono/forbidden-root-context/root-context-forbidden\",\"message\":\"Root CONTEXT.md is forbidden; read source code directly.\",\"path\":\"CONTEXT.md\",\"fix\":\"none\"}\n";

/// `git add` runs the ported content policies over what it would stage.
///  The built-in policy warns,
/// a listed root-context policy refuses,
///  and a listed Markdown or dependent-version policy refuses nothing:
/// Markdown checks no Markdown file here,
///  and the dependent-version policy plans only for a forwarded commit.
#[test]
fn add_runs_the_content_policies_over_what_it_stages() {
    let fixture: Fixture = fixture("frontier-add");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("file.txt"), b"no final newline").expect("file");
    std::fs::write(repo.join("second.txt"), b"second\n").expect("second file");
    std::fs::write(repo.join("CONTEXT.md"), b"context\n").expect("context file");
    // The built-in policy warns about the bytes the add stages, and Git stages them.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["add", "--", "file.txt"]),
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: FILE_WARNING.as_bytes().to_vec(),
        }
    );
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "A  file.txt\n?? CONTEXT.md\n?? second.txt\n"
    );
    // A listed ported policy at error stops the add, and nothing is staged.
    let configuration: PathBuf = repo.join("cli-git.config.jsonc");
    std::fs::write(
        &configuration,
        "{ \"policies\": { \"mono/forbidden-root-context\": \"error\" } }\n",
    )
    .expect("configuration");
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["add", "--", "CONTEXT.md"]),
        Observed {
            code: Some(1),
            stdout: Vec::<u8>::new(),
            stderr: CONTEXT_ERROR.as_bytes().to_vec(),
        }
    );
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "A  file.txt\n?? CONTEXT.md\n?? cli-git.config.jsonc\n?? second.txt\n"
    );
    // A listed Markdown policy at error checks a file that is not Markdown: the add stages it.
    std::fs::write(
        &configuration,
        "{ \"policies\": { \"markdown/autofix\": \"error\" } }\n",
    )
    .expect("configuration");
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["add", "--", "second.txt"]),
        silent_success()
    );
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "A  file.txt\nA  second.txt\n?? CONTEXT.md\n?? cli-git.config.jsonc\n"
    );
    // A listed dependent-version policy at error plans only for a forwarded commit: the add stages.
    std::fs::write(repo.join("third.txt"), b"third\n").expect("third file");
    std::fs::write(
        &configuration,
        "{ \"policies\": { \"mono/dependent-version-bump\": \"error\" } }\n",
    )
    .expect("configuration");
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["add", "--", "third.txt"]),
        silent_success()
    );
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "A  file.txt\nA  second.txt\nA  third.txt\n?? CONTEXT.md\n?? cli-git.config.jsonc\n"
    );
    remove(&fixture);
}

/// A real push is scanned at the manual-push gate and forwarded when clean;
///  a dry run is forwarded with `--atomic`.
#[test]
fn a_real_push_is_scanned_and_forwarded() {
    let fixture: Fixture = fixture("frontier-push");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(
        &fixture,
        fixture.root.as_path(),
        &["init", "--quiet", "--bare", "remote.git"],
    );
    let remote: PathBuf = fixture.root.join("remote.git");
    git(
        &fixture,
        repo.as_path(),
        &["remote", "add", "origin", "../remote.git"],
    );
    // A clean push is scanned, forwarded and publishes the branch.
    let pushed: Observed = run_wrapped(&fixture, repo.as_path(), &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{pushed:?}");
    assert!(
        !refs(&fixture, remote.as_path()).is_empty(),
        "the clean push must publish main"
    );
    // A dry run publishes nothing and is forwarded with the atomic-push insertion.
    let native: Observed = run_direct(
        &fixture,
        repo.as_path(),
        &["push", "--atomic", "--dry-run", "origin", "main"],
    );
    assert_eq!(native.code, Some(0));
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["push", "--dry-run", "origin", "main"]
        ),
        native
    );
    assert_eq!(
        refs(&fixture, remote.as_path()),
        "refs/heads/main
"
    );
    // Positive control: with the only manual-push policy escaped, the same push publishes.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &[
                "push",
                "--quiet",
                "--no-enforce-final-newline",
                "origin",
                "main"
            ]
        ),
        silent_success()
    );
    assert_eq!(refs(&fixture, remote.as_path()), "refs/heads/main\n");
    remove(&fixture);
}

/// Live transactions no longer stop a guarded command and a broken transaction record does;
/// dead transactions are recovered first.
/// Worktree-copy leases and journals no longer stop a guarded command: the forwarding settles them.
#[test]
fn leftover_transaction_state_refuses_guarded_commands() {
    let fixture: Fixture = fixture("frontier-state");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_linked_worktree(&fixture);
    let guarded: [&str; 3] = ["reset", "--soft", "HEAD"];
    // Positive control: in a quiet repository the guarded command runs.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &guarded),
        silent_success()
    );
    let registry: PathBuf = registered_transaction(repo.as_path(), true);
    // A live transaction no longer refuses a guarded command that writes no index.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &guarded),
        silent_success()
    );
    // The linked worktree has its own registry, so it is not stopped by the main one.
    assert_eq!(
        run_wrapped(&fixture, linked.as_path(), &guarded),
        silent_success()
    );
    // A read-only command is forwarded beside the same live transaction.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["rev-parse", "--is-inside-work-tree"]
        )
        .stdout,
        b"true\n"
    );
    assert!(registry.join(TRANSACTION_ID).exists());
    // A dead owner's transaction is recovered, and the command runs.
    std::fs::remove_dir_all(&registry).expect("remove registry");
    registered_transaction(repo.as_path(), false);
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &guarded),
        silent_success()
    );
    assert!(!registry.join(TRANSACTION_ID).exists());
    // A published directory without a valid owner record stops the command, names the
    // directory and keeps its contents.
    let broken: PathBuf = registry.join(TRANSACTION_ID);
    std::fs::create_dir(&broken).expect("published directory");
    std::fs::write(broken.join("owner.json"), b"{}").expect("malformed owner");
    std::fs::write(broken.join("preparing.json"), b"kept").expect("evidence");
    let stopped: Observed = run_wrapped(&fixture, repo.as_path(), &guarded);
    assert_eq!(stopped.code, Some(2));
    assert_eq!(
        stderr_of(&stopped),
        format!(
            "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"content-unavailable\",\"message\":\"Transaction owner record is malformed: {}\"}}\n",
            broken.display()
        )
    );
    assert_eq!(
        std::fs::read(broken.join("owner.json")).expect("kept"),
        b"{}"
    );
    assert_eq!(
        std::fs::read(broken.join("preparing.json")).expect("kept"),
        b"kept"
    );
    // A read-only command that asked for the location stops the same way.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["log", "--oneline"]).code,
        Some(2)
    );
    std::fs::remove_dir_all(&registry).expect("remove registry");
    // The legacy directory stops with the instructions to recover it.
    let legacy: PathBuf = repo.join(".git/cli-git-transaction");
    std::fs::create_dir(&legacy).expect("legacy transaction directory");
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &guarded),
        stopped_with(
            format!(
                "cli-git: {} holds a commit journal in a format this executable does not recover. \
                 Nothing was changed. Run any git command once with the previous cli-git executable to \
                 recover it, then run this command again.\n",
                legacy.display()
            )
            .as_str()
        )
    );
    assert!(legacy.exists());
    std::fs::remove_dir(&legacy).expect("remove legacy directory");
    // A worktree-copy journal or lease in the environment no longer stops a guarded command;
    // the forwarding recovers or honors it.
    assert_eq!(
        observe(
            wrapped(&fixture)
                .current_dir(&repo)
                .env("CLI_GIT_WORKTREE_COPY_LEASE", "inherited")
                .args(guarded),
            b""
        ),
        silent_success()
    );
    // The landing lease does not stop a guarded command that writes no index: coordination honours it.
    assert_eq!(
        observe(
            wrapped(&fixture)
                .current_dir(&repo)
                .env("CLI_GIT_LANDING_LEASE", "inherited")
                .args(guarded),
            b""
        ),
        silent_success()
    );
    assert_eq!(
        stderr_of(&run_wrapped(&fixture, repo.as_path(), &guarded)),
        ""
    );
    remove(&fixture);
}

/// A real push whose new commit breaks a manual-push policy is refused with exit 1 and publishes nothing.
#[test]
fn a_real_push_of_a_nonconforming_commit_is_refused_and_publishes_nothing() {
    let fixture: Fixture = fixture("frontier-push-refused");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(
        &fixture,
        fixture.root.as_path(),
        &["init", "--quiet", "--bare", "remote.git"],
    );
    let remote: PathBuf = fixture.root.join("remote.git");
    git(
        &fixture,
        repo.as_path(),
        &["remote", "add", "origin", "../remote.git"],
    );
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        "{ \"policies\": { \"final-newline\": \"error\" } }\n",
    )
    .expect("configuration");
    git(&fixture, repo.as_path(), &["add", "cli-git.config.jsonc"]);
    git(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "-m", "configure"],
    );
    git(
        &fixture,
        repo.as_path(),
        &["push", "--quiet", "origin", "main"],
    );
    std::fs::write(repo.join("bad.txt"), "no final newline").expect("nonconforming file");
    git(&fixture, repo.as_path(), &["add", "bad.txt"]);
    git(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "-m", "nonconforming"],
    );
    let before: String = refs(&fixture, remote.as_path());
    let pushed: Observed = run_wrapped(&fixture, repo.as_path(), &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(1), "{pushed:?}");
    assert!(
        String::from_utf8_lossy(&pushed.stderr)
            .contains("final-newline/noncanonical-final-newline"),
        "{pushed:?}"
    );
    assert_eq!(refs(&fixture, remote.as_path()), before);
    remove(&fixture);
}
