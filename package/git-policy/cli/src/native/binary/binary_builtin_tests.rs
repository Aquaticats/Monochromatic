//! What:
//!  The pre-forward built-in policies and the fixed transforms through the built
//!       executable,
//!  each with the command it stops and the neighbouring command it lets run.
//! Why:
//!  These are the rules a person meets on ordinary commands.
//!  Each must stop exactly
//!      what it names,
//!  leave the repository untouched when it does,
//!  and let Git run
//!      unchanged otherwise.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['clean', '-fd']).status).toBe(1); expect(existsSync(untracked)).toBe(true);
//! ```

/// Import the shared fixtures and the bounded process helpers.
use super::support::{
    Fixture, Observed, fixture, git, observe, porcelain, remove, repository, run_direct,
    run_wrapped, silent_success, stderr_of, wrapped,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The start of a finding line numbered `sequence` for one policy.
fn finding_start(sequence: u64, policy: &str) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"{policy}\",\"severity\":\"error\",\"code\":\"{policy}/"
    );
}

/// Require that a command was stopped with exit status 1 by exactly one finding of `policy`.
fn assert_rejected_by(observed: &Observed, policy: &str, context: &str) {
    let stderr: String = stderr_of(observed);
    assert_eq!(observed.code, Some(1), "{context}: {stderr}");
    assert_eq!(observed.stdout, Vec::<u8>::new(), "{context}");
    assert!(
        stderr.starts_with(finding_start(0, policy).as_str()),
        "{context}: {stderr}"
    );
    assert_eq!(stderr.matches('\n').count(), 1, "{context}: {stderr}");
}

/// Every local branch of a repository,
///  one name per line.
fn branches(fixture: &Fixture, repo: &Path) -> String {
    let output: std::process::Output = git(
        fixture,
        repo,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    );
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// require-root reads the directory Git works in after every `-C`,
///  not the caller's directory.
#[test]
fn require_root_reads_the_directory_git_works_in() {
    let fixture: Fixture = fixture("builtin-root");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::create_dir(repo.join("nested")).expect("nested directory");
    let nested_text: String = repo.join("nested").to_string_lossy().into_owned();
    let native: Observed = run_direct(&fixture, repo.as_path(), &["log", "--oneline"]);
    assert_eq!(native.code, Some(0));
    // From outside the repository, `-C` into the subdirectory is stopped ...
    assert_rejected_by(
        &run_wrapped(
            &fixture,
            fixture.root.as_path(),
            &["-C", nested_text.as_str(), "log", "--oneline"],
        ),
        "require-root",
        "-C nested",
    );
    // ... and a second `-C` back to the top level is not.
    assert_eq!(
        run_wrapped(
            &fixture,
            fixture.root.as_path(),
            &["-C", nested_text.as_str(), "-C", "..", "log", "--oneline"]
        ),
        native
    );
    // Commands that create a repository or need none are exempt below the top level.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.join("nested").as_path(),
            &["init", "--quiet", "inner"]
        ),
        silent_success()
    );
    assert!(repo.join("nested/inner/.git").exists());
    remove(&fixture);
}

/// linked-worktree-only stops destructive commands in the main worktree, and in a linked worktree while it holds
///  uncommitted state.
#[test]
fn destructive_commands_run_only_in_a_linked_worktree() {
    let fixture: Fixture = fixture("builtin-linked");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("linked");
    git(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", "../linked"],
    );
    std::fs::write(repo.join("untracked.txt"), b"keep\n").expect("untracked file");
    std::fs::write(linked.join("untracked.txt"), b"remove\n").expect("untracked file");
    for arguments in [
        vec!["clean", "-f"],
        vec!["clean", "-fd", "--quiet"],
        vec!["stash"],
        vec!["reset", "--hard"],
        vec!["reset", "--keep", "HEAD"],
    ] {
        assert_rejected_by(
            &run_wrapped(&fixture, repo.as_path(), arguments.as_slice()),
            "linked-worktree-only",
            arguments.join(" ").as_str(),
        );
    }
    assert_eq!(porcelain(&fixture, repo.as_path()), "?? untracked.txt\n");
    // Reading the stash list only inspects, so it is forwarded even in the main worktree.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["stash", "list"]),
        silent_success()
    );
    // A dry run only inspects, so it is forwarded even in the main worktree.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["clean", "-n"]),
        Observed {
            code: Some(0),
            stdout: b"Would remove untracked.txt\n".to_vec(),
            stderr: Vec::<u8>::new(),
        }
    );
    // In a linked worktree that holds an untracked file the same command is rejected, since it would delete it.
    assert_rejected_by(
        &run_wrapped(&fixture, linked.as_path(), &["clean", "-f", "--quiet"]),
        "linked-worktree-only",
        "clean in a linked worktree with an untracked file",
    );
    assert!(linked.join("untracked.txt").exists());
    // Once the worktree holds nothing to lose the same command runs.
    std::fs::remove_file(linked.join("untracked.txt")).expect("remove the untracked file");
    assert_eq!(
        run_wrapped(&fixture, linked.as_path(), &["clean", "-f", "--quiet"]),
        silent_success()
    );
    assert!(repo.join("untracked.txt").exists());
    // The hatch runs it in the main worktree and never reaches Git.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["clean", "--no-enforce-worktree", "-f", "--quiet"]
        ),
        silent_success()
    );
    assert!(!repo.join("untracked.txt").exists());
    remove(&fixture);
}

/// A repository inside the caller's tool cache is exempt from linked-worktree-only.
#[test]
fn a_tool_cache_checkout_is_exempt_from_the_linked_worktree_rule() {
    let fixture: Fixture = fixture("builtin-cache");
    // The fixture's home directory is the caller's `HOME`; `uv` keeps Git checkouts below it.
    std::fs::create_dir_all(fixture.root.join("home/.cache/uv")).expect("tool cache");
    let cached: PathBuf = repository(&fixture, OsStr::new("home/.cache/uv/checkout"));
    let ordinary: PathBuf = repository(&fixture, OsStr::new("ordinary"));
    for repo in [&cached, &ordinary] {
        std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
        git(
            &fixture,
            repo.as_path(),
            &["commit", "--quiet", "--message=tracked"],
        );
        std::fs::write(repo.join("tracked.txt"), b"two\n").expect("modify tracked file");
    }
    // In the cache the main worktree may be reset; Git runs and restores the file.
    assert_eq!(
        run_wrapped(&fixture, cached.as_path(), &["reset", "--hard", "--quiet"]),
        silent_success()
    );
    assert_eq!(
        std::fs::read(cached.join("tracked.txt")).expect("tracked file"),
        b"one\n"
    );
    // Positive control: the same command in an ordinary main worktree is rejected.
    assert_rejected_by(
        &run_wrapped(
            &fixture,
            ordinary.as_path(),
            &["reset", "--hard", "--quiet"],
        ),
        "linked-worktree-only",
        "ordinary repository",
    );
    assert_eq!(
        std::fs::read(ordinary.join("tracked.txt")).expect("tracked file"),
        b"two\n"
    );
    // Without a usable home directory there is no cache to exempt.
    std::fs::write(cached.join("tracked.txt"), b"two\n").expect("modify tracked file");
    for home in ["", "/nonexistent-home"] {
        assert_rejected_by(
            &observe(
                wrapped(&fixture)
                    .env("HOME", home)
                    .current_dir(&cached)
                    .args(["reset", "--hard", "--quiet"]),
                b"",
            ),
            "linked-worktree-only",
            home,
        );
    }
    assert_eq!(
        std::fs::read(cached.join("tracked.txt")).expect("tracked file"),
        b"two\n"
    );
    remove(&fixture);
}

/// A plugin clone inside mise's data directory is exempt from linked-worktree-only,
///  so `mise plugins update` can force-checkout its own clone;
///  a sibling directory with a similar name and an ordinary repository are not.
#[test]
fn a_mise_plugin_clone_is_exempt_from_the_linked_worktree_rule() {
    let fixture: Fixture = fixture("builtin-mise");
    // The fixture's home directory is the caller's `HOME`; mise keeps plugin clones below its data directory.
    std::fs::create_dir_all(fixture.root.join("home/.local/share/mise/plugins")).expect("mise data");
    std::fs::create_dir_all(fixture.root.join("home/.local/share/mise-other")).expect("sibling");
    let plugin: PathBuf = repository(&fixture, OsStr::new("home/.local/share/mise/plugins/cmake"));
    let sibling: PathBuf = repository(&fixture, OsStr::new("home/.local/share/mise-other/cmake"));
    let ordinary: PathBuf = repository(&fixture, OsStr::new("ordinary"));
    for repo in [&plugin, &sibling, &ordinary] {
        std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
        git(
            &fixture,
            repo.as_path(),
            &["commit", "--quiet", "--message=tracked"],
        );
        std::fs::write(repo.join("tracked.txt"), b"two\n").expect("modify tracked file");
    }
    // The form mise runs on a plugin: a forced checkout that discards local changes.
    assert_eq!(
        run_wrapped(
            &fixture,
            plugin.as_path(),
            &["checkout", "--force", "--quiet", "HEAD"],
        ),
        silent_success()
    );
    assert_eq!(
        std::fs::read(plugin.join("tracked.txt")).expect("tracked file"),
        b"one\n"
    );
    // Positive controls: the same command elsewhere is rejected and changes nothing.
    for (repo, label) in [(&sibling, "sibling directory"), (&ordinary, "ordinary repository")] {
        assert_rejected_by(
            &run_wrapped(
                &fixture,
                repo.as_path(),
                &["checkout", "--force", "--quiet", "HEAD"],
            ),
            "linked-worktree-only",
            label,
        );
        assert_eq!(
            std::fs::read(repo.join("tracked.txt")).expect("tracked file"),
            b"two\n"
        );
    }
    remove(&fixture);
}

/// branch-worktree-only stops branch creation,
///  including the branch Git would guess from a remote.
#[test]
fn branch_creation_is_rejected_in_the_current_worktree() {
    let fixture: Fixture = fixture("builtin-branch");
    let upstream: PathBuf = repository(&fixture, OsStr::new("upstream"));
    git(&fixture, upstream.as_path(), &["branch", "feature"]);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(
        &fixture,
        repo.as_path(),
        &["remote", "add", "origin", "../upstream"],
    );
    git(&fixture, repo.as_path(), &["fetch", "--quiet", "origin"]);
    for arguments in [
        vec!["switch", "-c", "created"],
        vec!["checkout", "-b", "created"],
        vec!["branch", "created"],
        // Git would create `feature` from the one remote that has it.
        vec!["switch", "feature"],
        vec!["checkout", "feature"],
    ] {
        assert_rejected_by(
            &run_wrapped(&fixture, repo.as_path(), arguments.as_slice()),
            "branch-worktree-only",
            arguments.join(" ").as_str(),
        );
    }
    assert_eq!(branches(&fixture, repo.as_path()), "main\n");
    // Listing and switching to an existing branch create nothing and are forwarded.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["branch", "--list"]),
        run_direct(&fixture, repo.as_path(), &["branch", "--list"])
    );
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["switch", "--quiet", "main"]),
        silent_success()
    );
    // A name no remote has is left for Git to refuse.
    let unknown: Observed = run_direct(&fixture, repo.as_path(), &["switch", "no-such-branch"]);
    assert_eq!(unknown.code, Some(128));
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["switch", "no-such-branch"]),
        unknown
    );
    // The hatch creates the branch and never reaches Git.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &[
                "switch",
                "--no-enforce-worktree-branch",
                "--quiet",
                "-c",
                "created"
            ]
        ),
        silent_success()
    );
    assert_eq!(branches(&fixture, repo.as_path()), "created\nmain\n");
    remove(&fixture);
}

/// add-explicit stops bulk staging and names the tokens the caller wrote.
#[test]
fn bulk_staging_is_rejected() {
    let fixture: Fixture = fixture("builtin-add");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("file.txt"), b"content\n").expect("file");
    for arguments in [
        vec!["add", "."],
        vec!["add", "-A"],
        vec!["add", "--all"],
        vec!["add", "-u"],
        vec!["add", "--no-enforce-final-newline", "."],
    ] {
        assert_rejected_by(
            &run_wrapped(&fixture, repo.as_path(), arguments.as_slice()),
            "add-explicit",
            arguments.join(" ").as_str(),
        );
    }
    assert_eq!(porcelain(&fixture, repo.as_path()), "?? file.txt\n");
    // The hatch stages everything; the content policy is escaped too so the command can run.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &[
                "add",
                "--no-enforce-bulk-add",
                "--no-enforce-final-newline",
                "."
            ]
        ),
        silent_success()
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "A  file.txt\n");
    remove(&fixture);
}

/// The first error stops the pass;
///  keep-going reports every finding,
///  numbered in order,
///  and still blocks.
#[test]
fn keep_going_reports_every_finding_and_still_blocks() {
    let fixture: Fixture = fixture("builtin-keep-going");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    // Below the top level and creating a branch: the first and the third policy object.
    assert_rejected_by(
        &run_wrapped(&fixture, nested.as_path(), &["switch", "-c", "created"]),
        "require-root",
        "without keep-going",
    );
    let both: Observed = run_wrapped(
        &fixture,
        nested.as_path(),
        &["switch", "--cli-git-keep-going", "-c", "created"],
    );
    let stderr: String = stderr_of(&both);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(both.code, Some(1), "{stderr}");
    assert_eq!(lines.len(), 2, "{stderr}");
    assert!(
        lines[0].starts_with(finding_start(0, "require-root").as_str()),
        "{stderr}"
    );
    assert!(
        lines[1].starts_with(finding_start(1, "branch-worktree-only").as_str()),
        "{stderr}"
    );
    assert_eq!(branches(&fixture, repo.as_path()), "main\n");
    remove(&fixture);
}

/// A policy set to `warn` reports on standard error and lets Git run;
///  commit-only rejects on its own.
#[test]
fn warnings_let_git_run_and_commit_only_rejects() {
    let fixture: Fixture = fixture("builtin-warn");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        r#"{ "policies": { "branch-worktree-only": "warn" } }"#,
    )
    .expect("configuration");
    let warned: Observed = run_wrapped(&fixture, repo.as_path(), &["branch", "created"]);
    assert_eq!(warned.code, Some(0));
    assert_eq!(warned.stdout, Vec::<u8>::new());
    assert_eq!(
        stderr_of(&warned),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"branch-worktree-only\",\"severity\":\"warn\",\"code\":\"branch-worktree-only/branch-creation-requires-worktree\",\"message\":\"cli-git: git branch branch creation is rejected in the current worktree. Use `git worktree add -b <branch> <path> [<start-point>]` so new branch work starts in its own checkout, or pass --no-enforce-worktree-branch to bypass for this invocation.\",\"fix\":\"none\"}\n"
    );
    assert_eq!(branches(&fixture, repo.as_path()), "created\nmain\n");
    // The commit-only transform rejects `-a` and a pathless commit, here on a dry run.
    std::fs::write(repo.join("file.txt"), b"content\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "file.txt"]);
    for (arguments, code) in [
        (
            vec!["commit", "--dry-run", "-a", "-m", "x"],
            "commit-only/all-flag",
        ),
        (
            vec!["commit", "--dry-run", "-m", "x"],
            "commit-only/pathspec-required",
        ),
    ] {
        let rejected: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(rejected.code, Some(1), "{arguments:?}");
        assert!(
            stderr_of(&rejected).starts_with(
                format!(
                    "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"core-finding\",\"trigger\":\"pre-forward\",\"coreId\":\"commit-only\",\"code\":\"{code}\","
                )
                .as_str()
            ),
            "{arguments:?}: {}",
            stderr_of(&rejected)
        );
    }
    // Its own hatch skips the transform, and Git never sees the hatch.
    let native: Observed = run_direct(
        &fixture,
        repo.as_path(),
        &["commit", "--dry-run", "-a", "-m", "x"],
    );
    assert_eq!(native.code, Some(0));
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["commit", "--no-enforce-only", "--dry-run", "-a", "-m", "x"]
        ),
        native
    );
    remove(&fixture);
}
