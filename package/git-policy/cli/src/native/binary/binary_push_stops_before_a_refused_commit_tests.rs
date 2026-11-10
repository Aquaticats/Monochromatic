//! What:
//!  A commit whose post-commit gate refused it is never published by a later commit's automatic push.
//! Why:
//!  `README.md` promises that a landed commit a post-commit policy blocks exits `2` and stays local.
//! The queue pushes the branch and not one commit,
//!  so before this rule the next commit's push published it anyway,
//! and the promise
//!  held only until the next commit.
//! This case runs the real executable against a real bare remote and a real service:
//! a commit lands,
//! the gate
//!  refuses it,
//! a later commit lands,
//! and the remote must hold the commit before the refused one and nothing past it,
//! while every command
//!  names the commit that holds the branch back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refused = await amendThroughTheWrapper(); expect(refused.code).toBe(2);
//! await commitThroughTheWrapper(); expect(remoteMain()).toBe(beforeTheRefused);
//! expect(await statusStderr()).toContain(`is held back by the commit ${refused}`);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, FixtureService, Observed, direct, fixture, git, observe, repository, run_wrapped, start_fixture_service};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The longest a control waits for the service to push.
const PATIENCE: Duration = Duration::from_secs(120);

/// The configuration that makes one commit-time policy refuse content it would only warn about.
const STRICT_FINAL_NEWLINE: &str = "{ \"policies\": { \"final-newline\": \"error\" } }\n";

/// A repository with one tracked file,
/// a bare remote that has the branch,
/// and a service the commits go through.
struct Setup {
    fixture: Fixture,
    repo: PathBuf,
    remote: PathBuf,
    service: FixtureService,
}

/// A bare remote,
/// a repository that tracks it,
/// with `base` committed and pushed and the strict configuration in place.
fn setup(name: &str) -> Setup {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), STRICT_FINAL_NEWLINE).expect("the configuration");
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt", "monochromatic-git-policy-cli.config.jsonc"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, service };
}

/// Commit the worktree's `a.txt` with real Git,
/// as a person's own Git would,
/// answering the commit's name.
fn commit_directly(setup: &Setup, text: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), text).expect("a");
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--message", text.trim(), "a.txt"]);
    return local_tip(setup);
}

/// Commit the worktree's `b.txt` through the wrapper,
/// answering the command's observation.
fn commit_b_through(setup: &Setup, text: &str) -> Observed {
    std::fs::write(setup.repo.join("b.txt"), text).expect("b.txt");
    let mut command: std::process::Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", text.trim(), "--", "b.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    return observe(&mut command, b"");
}

/// The commit `directory` names for `revision`,
/// as real Git prints it.
fn named(setup: &Setup, directory: &std::path::Path, revision: &str) -> String {
    let output: std::process::Output = git(&setup.fixture, directory, &["rev-parse", revision]);
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

/// The commit the worktree is on.
fn local_tip(setup: &Setup) -> String {
    return named(setup, setup.repo.as_path(), "HEAD");
}

/// The commit the bare remote's `main` names.
fn remote_main(setup: &Setup) -> String {
    return named(setup, setup.remote.as_path(), "refs/heads/main");
}

/// Wait until `condition` holds,
/// within the patience.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// What:
///  A commit the post-commit gate refused stays local past the next commit of its branch,
/// and every command says which commit holds the branch back.
/// Why:
///   This is the promise the README makes and the owner's answer asked for:
/// the commits before the refused one reach GitHub,
/// the
///  refused one and everything above it do not,
/// and the person is told why without asking a question.
#[test]
fn a_refused_commit_is_not_published_by_the_next_commit() {
    let setup: Setup = setup("refused-commit-stays-local");
    // Two commits a person's own Git made, so neither went through a gate: the second carries content the strict policy refuses, and
    //  the amend below keeps that content while changing only `a.txt`, so the pre-commit pass sees nothing to refuse.
    let before_the_refused: String = commit_directly(&setup, "before\n");
    std::fs::write(setup.repo.join("bad.txt"), "no final newline").expect("bad.txt");
    git(&setup.fixture, &setup.repo, &["add", "bad.txt"]);
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--message=refused content"]);
    // The amend lands a commit the gate then refuses: its delta against its parent holds both files, and the one a person's own Git
    //  wrote has no final newline.
    std::fs::write(setup.repo.join("a.txt"), "amended\n").expect("a");
    let mut amend: std::process::Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--amend", "--no-edit", "-m", "amended", "--", "a.txt"]);
    amend.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let refused: Observed = observe(&mut amend, b"");
    let refused_commit: String = local_tip(&setup);
    assert_eq!(refused.code, Some(2), "a landed commit the gate refuses exits 2: {}", String::from_utf8_lossy(refused.stderr.as_slice()));
    assert!(
        String::from_utf8_lossy(refused.stderr.as_slice()).contains("remains local"),
        "{}",
        String::from_utf8_lossy(refused.stderr.as_slice())
    );
    // A later commit of the same branch: its push is the one that used to publish the refused commit.
    let later: Observed = commit_b_through(&setup, "later\n");
    assert_eq!(later.code, Some(0), "{}", String::from_utf8_lossy(later.stderr.as_slice()));
    // The push runs after the command returns, so the control waits for it: the remote reaches the commit before the refused one,
    //  which no earlier push had delivered.
    wait_until("the remote reaches the commit before the refused one", &|| return remote_main(&setup) == before_the_refused);
    assert!(!remote_has(&setup, refused_commit.as_str()), "the refused commit stays local");
    let later_commit: String = local_tip(&setup);
    assert!(!remote_has(&setup, later_commit.as_str()), "the commit after it does not drag it along");
    // The verdict is on disk and holds the branch back when the same state is read by the library:
    // a command that says nothing is
    //  either a record that is gone or a question that answered nothing.
    let work: PathBuf = git_policy_cli::service_work::work_directory(setup.repo.join(".git").as_path());
    let verdicts: Vec<git_policy_cli::work_item_holding::BlockedVerdict> = git_policy_cli::work_item_holding::blocked_verdicts(work.as_path());
    assert_eq!(verdicts.len(), 1, "the gate kept one verdict on disk: {verdicts:?}");
    assert_eq!(verdicts.first().map(|verdict| return verdict.commit.as_str()), Some(refused_commit.as_str()), "{verdicts:?}");
    // Every command says which commit holds the branch back and which check refused it, and says it again: the verdict stands while
    //  the commit is on the branch's way out.
    for _attempt in 0..2 {
        let said: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
        let words: String = String::from_utf8_lossy(said.stderr.as_slice()).into_owned();
        assert!(words.contains("main is held back by the commit"), "{words} / {:?} / {:?}", said.code, String::from_utf8_lossy(said.stdout.as_slice()));
        assert!(words.contains(refused_commit.as_str()), "the line names the commit: {words}");
        assert!(words.contains("final-newline"), "the line names the check: {words}");
        assert!(words.contains("a person's `git push` still sends it"), "the line names the way out: {words}");
    }
    remove_settled(&setup.fixture);
}

/// Whether the bare remote holds `commit`.
/// The question is asked of the remote's own object store,
/// and "no" is an answer.
fn remote_has(setup: &Setup, commit: &str) -> bool {
    let asked: std::process::Output = direct(&setup.fixture)
        .current_dir(setup.remote.as_path())
        .args(["cat-file", "-e", format!("{commit}^{{commit}}").as_str()])
        .output()
        .expect("real Git answers");
    return asked.status.success();
}
