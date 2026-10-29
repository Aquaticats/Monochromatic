//! Concurrency controls for single-flight auto-push across real processes.
//! The TypeScript corpus is `src/auto-push-concurrency.unit.test.ts`; those cases commit through the wrapper,
//! which stays refused until the integration step, so each case here drives `auto_push` from helper processes instead.
//! Helper processes are this test binary re-run with `--ignored`, so their pids and birth identities are real.

use super::*;
use crate::test_support::{fixture, git, remove};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Environment variable naming the repository a helper auto-pushes.
const HELPER_REPO: &str = "AUTO_PUSH_HELPER_REPO";
/// Environment variable naming the landed commit a helper backs up.
const HELPER_LANDED: &str = "AUTO_PUSH_HELPER_LANDED";
/// Environment variable naming the file a helper writes its outcome to.
const HELPER_RESULT: &str = "AUTO_PUSH_HELPER_RESULT";

/// Marker the remote's post-receive hook prints.
const REMOTE_MARKER: &str = "concurrency-remote-marker";

/// Pre-push hook text: append one line to the log for each run; sleep while the gate file is absent,
/// creating the gate first, so that exactly the first run is slow.
fn pre_push_script(log: &Path, gate: &Path, sleep_seconds: &str) -> String {
    return format!(
        "#!/bin/sh\necho pre-push >> '{log}'\nif [ ! -e '{gate}' ]; then\n  touch '{gate}'\n  sleep {sleep_seconds}\nfi\nexit 0\n",
        log = log.display(),
        gate = gate.display(),
    );
}

/// A repository with a bare origin, a post-receive marker and a slow first pre-push.
struct Scenario {
    /// Scratch root of the scenario.
    root: PathBuf,
    /// The repository the helpers push from.
    repo: PathBuf,
    /// The bare remote the helpers push to.
    remote: PathBuf,
    /// Log of every pre-push run, one line each.
    log: PathBuf,
}

/// Build the scenario with `sleep_seconds` as the first pre-push's duration.
fn scenario(name: &str, sleep_seconds: &str) -> Scenario {
    let root = fixture(name);
    let repo = root.join("repo");
    let remote = root.join("remote.git");
    let hooks = root.join("hooks");
    let log = root.join("pre-push.log");
    let gate = root.join("pre-push.gate");
    git(
        &root,
        &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")],
    );
    let post_receive = remote.join("hooks").join("post-receive");
    std::fs::write(
        &post_receive,
        format!("#!/bin/sh\necho \"{REMOTE_MARKER}\"\n"),
    )
    .expect("hook");
    std::fs::set_permissions(&post_receive, std::fs::Permissions::from_mode(0o755)).expect("mode");
    std::fs::create_dir_all(&repo).expect("repo");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    git(&repo, &["config", "user.email", "cli-git@example.invalid"]);
    git(&repo, &["config", "user.name", "cli-git concurrency test"]);
    git(
        &repo,
        &["remote", "add", "origin", remote.to_str().expect("utf-8")],
    );
    std::fs::create_dir_all(&hooks).expect("hooks");
    let pre_push = hooks.join("pre-push");
    std::fs::write(&pre_push, pre_push_script(&log, &gate, sleep_seconds)).expect("pre-push");
    std::fs::set_permissions(&pre_push, std::fs::Permissions::from_mode(0o755)).expect("mode");
    // A configuration hook: traditional hooks never run through the wrapper's real-Git starts.
    git(&repo, &["config", "hook.control.event", "pre-push"]);
    git(&repo, &["config", "hook.control.command", pre_push.to_str().expect("utf-8")]);
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=base"],
    );
    git(
        &repo,
        &["push", "--quiet", "--set-upstream", "origin", "HEAD"],
    );
    // The setup push ran the pre-push hook once; forget that run and reset the gate so each case counts only its own pushes.
    let _log_gone = std::fs::remove_file(&log);
    let _gate_gone = std::fs::remove_file(&gate);
    return Scenario {
        root,
        repo,
        remote,
        log,
    };
}

/// Land one empty commit and return its OID.
fn land(repo: &Path, message: &str) -> String {
    let argument = format!("--message={message}");
    git(
        repo,
        &["commit", "--quiet", "--allow-empty", argument.as_str()],
    );
    return String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
}

/// Number of pre-push runs so far.
fn pre_push_runs(log: &Path) -> usize {
    return std::fs::read_to_string(log)
        .map(|text| {
            return text.lines().count();
        })
        .unwrap_or(0);
}

/// The OID of a ref in the bare remote.
fn remote_ref(remote: &Path, branch: &str) -> String {
    return String::from_utf8_lossy(&git(remote, &["rev-parse", branch]).stdout)
        .trim()
        .to_owned();
}

/// Whether `ancestor` is in the history of `tip` in the remote.
fn remote_contains(remote: &Path, ancestor: &str, tip: &str) -> bool {
    return git_output_status(remote, &["merge-base", "--is-ancestor", ancestor, tip]);
}

/// Whether a Git query in a repository exits zero.
fn git_output_status(directory: &Path, arguments: &[&str]) -> bool {
    return Command::new("/usr/bin/git")
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(arguments)
        .output()
        .map(|output| {
            return output.status.success();
        })
        .unwrap_or(false);
}

/// What a helper process reported, read back from its result file.
#[derive(Debug, Eq, PartialEq)]
struct Reported {
    /// Outcome name: `skipped`, `pushed` or `failed`.
    outcome: String,
    /// Whether another invocation's push decided the outcome.
    joined: bool,
    /// Text the helper showed on its terminal.
    shown: String,
    /// Exit code of the decided push.
    exit_code: i64,
}

/// Start a helper process that auto-pushes `landed` from the scenario repository.
///
/// The helper's stderr is captured so each case reads what the terminal showed.
fn start_helper(scenario: &Scenario, landed: &str, result: &Path) -> Child {
    return Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            "auto_push::concurrency_tests::helper_auto_pushes",
            "--ignored",
            "--test-threads=1",
            "--quiet",
        ])
        .env(HELPER_REPO, &scenario.repo)
        .env(HELPER_LANDED, landed)
        .env(HELPER_RESULT, result)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .expect("start the helper");
}

/// Wait for a helper and read its report.
fn finish(child: Child, result: &Path) -> (Reported, String) {
    let output = child.wait_with_output().expect("helper finished");
    assert!(
        output.status.success(),
        "helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::fs::read_to_string(result).expect("helper wrote its result");
    let mut lines = text.splitn(4, '\n');
    let outcome = lines.next().unwrap_or_default().to_owned();
    let joined = lines.next().unwrap_or_default() == "joined";
    let exit_code: i64 = lines.next().unwrap_or_default().parse().expect("exit code");
    let shown = lines.next().unwrap_or_default().replace("\\n", "\n");
    return (
        Reported {
            outcome,
            joined,
            shown,
            exit_code,
        },
        String::from_utf8_lossy(&output.stderr).into_owned(),
    );
}

/// Helper process body, started by the cases below: auto-push one landed commit and write the outcome.
///
/// The outcome file has four lines: the outcome name, `joined` or `owner`, the exit code, and the shown text
/// with line breaks written as `\n`.
#[test]
#[ignore = "started as a helper process by the auto-push concurrency tests"]
fn helper_auto_pushes() {
    let repo = PathBuf::from(std::env::var_os(HELPER_REPO).expect("repo"));
    let landed = std::env::var(HELPER_LANDED).expect("landed");
    let result = PathBuf::from(std::env::var_os(HELPER_RESULT).expect("result"));
    let git_context = PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: repo,
        environment: [
            ("PATH", "/usr/bin:/bin"),
            ("GIT_CONFIG_NOSYSTEM", "1"),
            ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
            ("GIT_AUTHOR_NAME", "Fixture"),
            ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
            ("GIT_COMMITTER_NAME", "Fixture"),
            ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
        ]
        .iter()
        .map(|(name, value)| {
            return (OsString::from(name), OsString::from(value));
        })
        .collect(),
        lfs_pre_push: None,
    };
    std::fs::write(result.with_extension("started"), "").expect("announce start");
    let mut err: Vec<u8> = Vec::new();
    let reported = auto_push(&git_context, &landed, &mut err).expect("auto-push ran");
    let outcome = match reported.outcome {
        AutoPushOutcome::Skipped => "skipped",
        AutoPushOutcome::Pushed => "pushed",
        AutoPushOutcome::Failed => "failed",
    };
    let body = format!(
        "{outcome}\n{}\n{}\n{}\n",
        if reported.joined { "joined" } else { "owner" },
        reported.exit_code,
        reported.shown.replace('\n', "\\n"),
    );
    std::fs::write(&result, body).expect("write outcome");
    std::io::Write::write_all(&mut std::io::stderr(), &err).expect("terminal");
}

/// Racing processes on one branch: fewer pushes than commits, and every commit reaches the remote.
#[test]
fn racing_processes_coalesce_into_fewer_pushes_than_commits() {
    let scenario = scenario("ap-concurrency-coalesce", "1");
    let landed: Vec<String> = (1..=5)
        .map(|index| {
            return land(&scenario.repo, &format!("c{index}"));
        })
        .collect();
    let results: Vec<PathBuf> = (0..landed.len())
        .map(|index| {
            return scenario.root.join(format!("result-{index}"));
        })
        .collect();
    let children: Vec<Child> = landed
        .iter()
        .zip(&results)
        .map(|(oid, result)| {
            return start_helper(&scenario, oid, result);
        })
        .collect();
    let reports: Vec<(Reported, String)> = children
        .into_iter()
        .zip(&results)
        .map(|(child, result)| {
            return finish(child, result);
        })
        .collect();
    let runs = pre_push_runs(&scenario.log);
    assert!(
        runs < landed.len(),
        "{runs} pushes for {} commits",
        landed.len()
    );
    assert!(runs >= 1);
    let tip = landed.last().expect("landed").clone();
    assert_eq!(remote_ref(&scenario.remote, "main"), tip);
    for oid in &landed {
        assert!(
            remote_contains(&scenario.remote, oid, &tip),
            "{oid} missing on remote"
        );
    }
    for (report, _terminal) in &reports {
        assert_eq!(report.outcome, "pushed", "{report:?}");
        assert_eq!(report.exit_code, 0);
    }
    remove(&scenario.root);
}

/// A push failure is reported by every affected commit, each exits its auto-push cleanly and keeps its commit.
///
/// The first helper's pre-push holds the lock until the test releases it, so the second helper is certain to
/// wait behind the failing push and to join it.
#[test]
fn a_joined_push_failure_is_reported_by_every_commit_it_covered() {
    let scenario = scenario("ap-concurrency-failure", "0");
    let release = scenario.root.join("pre-push.release");
    let pre_push = scenario.root.join("hooks").join("pre-push");
    std::fs::write(
        &pre_push,
        format!(
            "#!/bin/sh\necho pre-push >> '{}'\nwhile [ ! -e '{}' ]; do sleep 0.05; done\necho 'refused-by-concurrency-test' >&2\nexit 1\n",
            scenario.log.display(),
            release.display()
        ),
    )
    .expect("failing hook");
    let landed = land(&scenario.repo, "failing");
    let result_a = scenario.root.join("result-a");
    let result_b = scenario.root.join("result-b");
    let first = start_helper(&scenario, &landed, &result_a);
    let deadline = Instant::now() + Duration::from_secs(20);
    while pre_push_runs(&scenario.log) == 0 {
        assert!(
            Instant::now() < deadline,
            "the first push never reached pre-push"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    let second = start_helper(&scenario, &landed, &result_b);
    let started = result_b.with_extension("started");
    while !started.exists() {
        assert!(Instant::now() < deadline, "the second helper never started");
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_secs(1));
    std::fs::write(&release, "").expect("release the failing push");
    let (report_a, _terminal_a) = finish(first, &result_a);
    let (report_b, _terminal_b) = finish(second, &result_b);
    for report in [&report_a, &report_b] {
        assert_eq!(report.outcome, "failed", "{report:?}");
        assert_ne!(report.exit_code, 0);
        assert!(
            report.shown.contains("refused-by-concurrency-test"),
            "{report:?}"
        );
    }
    assert!(report_b.joined, "{report_b:?}");
    assert_eq!(pre_push_runs(&scenario.log), 1);
    assert_eq!(head_of(&scenario.repo), landed);
    remove(&scenario.root);
}

/// A pusher killed mid-push leaves a lock whose owner is dead; a waiting commit takes it over and pushes both commits.
#[test]
fn takes_over_from_a_pusher_killed_mid_push() {
    let scenario = scenario("ap-concurrency-takeover", "30");
    let first_landed = land(&scenario.repo, "first");
    let result_first = scenario.root.join("result-first");
    let mut killed = start_helper(&scenario, &first_landed, &result_first);
    let group = i32::try_from(killed.id()).expect("pid fits");
    let deadline = Instant::now() + Duration::from_secs(20);
    while pre_push_runs(&scenario.log) == 0 {
        assert!(
            Instant::now() < deadline,
            "the first push never reached pre-push"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    // SAFETY: the group is the helper process group that start_helper created, so only the helper and its Git child receive the signal.
    let signalled = unsafe { libc::kill(-group, libc::SIGKILL) };
    assert_eq!(signalled, 0, "kill the pusher group");
    let _reaped = killed.wait();
    let second_landed = land(&scenario.repo, "second");
    let result_second = scenario.root.join("result-second");
    let second = start_helper(&scenario, &second_landed, &result_second);
    let (report, _terminal) = finish(second, &result_second);
    assert_eq!(report.outcome, "pushed", "{report:?}");
    assert_eq!(remote_ref(&scenario.remote, "main"), second_landed);
    assert!(remote_contains(
        &scenario.remote,
        &first_landed,
        &second_landed
    ));
    remove(&scenario.root);
}

/// A branch without an upstream is pushed with `--set-upstream origin HEAD`, which records the upstream.
#[test]
fn a_branch_without_an_upstream_is_pushed_with_set_upstream() {
    let scenario = scenario("ap-concurrency-set-upstream", "0");
    git(
        &scenario.repo,
        &["checkout", "--quiet", "-b", "feature/new"],
    );
    let landed = land(&scenario.repo, "on feature");
    let result = scenario.root.join("result");
    let (report, _terminal) = finish(start_helper(&scenario, &landed, &result), &result);
    assert_eq!(report.outcome, "pushed", "{report:?}");
    assert_eq!(remote_ref(&scenario.remote, "feature/new"), landed);
    let tracked = String::from_utf8_lossy(
        &git(&scenario.repo, &["config", "branch.feature/new.remote"]).stdout,
    )
    .trim()
    .to_owned();
    assert_eq!(tracked, "origin");
    remove(&scenario.root);
}

/// A detached HEAD is skipped with a note and the remote is left untouched.
#[test]
fn a_detached_head_is_skipped_with_a_note_and_leaves_the_remote_untouched() {
    let scenario = scenario("ap-concurrency-detached", "0");
    let before = remote_ref(&scenario.remote, "main");
    git(&scenario.repo, &["checkout", "--quiet", "--detach", "HEAD"]);
    let landed = land(&scenario.repo, "detached");
    let result = scenario.root.join("result");
    let (report, _terminal) = finish(start_helper(&scenario, &landed, &result), &result);
    assert_eq!(report.outcome, "skipped", "{report:?}");
    assert!(report.shown.contains("HEAD is detached"), "{report:?}");
    assert_eq!(remote_ref(&scenario.remote, "main"), before);
    remove(&scenario.root);
}

/// Without any upstream or origin the commit is skipped silently.
#[test]
fn skips_silently_without_an_upstream_or_an_origin_remote() {
    let root = fixture("ap-concurrency-no-remote");
    git(&root, &["init", "--quiet", "--initial-branch=main"]);
    git(&root, &["config", "user.email", "cli-git@example.invalid"]);
    git(&root, &["config", "user.name", "cli-git concurrency test"]);
    let landed = land(&root, "local only");
    let mut err: Vec<u8> = Vec::new();
    let result = auto_push(
        &PushGit {
            real_git: PathBuf::from("/usr/bin/git"),
            cwd: root.clone(),
            environment: vec![
                (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
                (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
                (
                    OsString::from("GIT_CONFIG_GLOBAL"),
                    OsString::from("/nonexistent-global-config"),
                ),
            ],
            lfs_pre_push: None,
        },
        &landed,
        &mut err,
    )
    .expect("skips");
    assert_eq!(result.outcome, AutoPushOutcome::Skipped);
    assert_eq!(result.shown, "");
    assert!(err.is_empty());
    remove(&root);
}

/// The head of a fixture repository.
fn head_of(repo: &Path) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
}
