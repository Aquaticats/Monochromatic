//! The hooks of a wrapped commit through the built executable:
//!  configured hooks run once inside the commit, run in the right place, propagate their status, serialize on the
//!  hook lock unless `hooks.concurrentCommits` allows overlap, and the hooks directory of a transaction holds no
//!  entry that names the wrapper.
//! Why:
//!  Git starts the user's configured hook commands itself inside the native commit, so Git never starts the wrapper
//!  for a hook; these controls prove the placement, the lock and the environment end to end with real Git 2.55.0 or
//!  2.56.0 underneath.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // two concurrent `git commit` runs with a barrier hook: the second hook starts after the first one ends
//! ```

use super::commit::{remove_settled, run_commit, wrapped_commit};
use super::support::{Fixture, executable, fixture, git, observe, repository};
use git_policy_cli::hook_dispatch_plan::PLAN_FILENAME;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Poll interval for barrier files.
const POLL: Duration = Duration::from_millis(20);

/// How long a barrier waits before the control fails.
const BARRIER_LIMIT: Duration = Duration::from_secs(60);

/// A repository with `a.txt` and `b.txt` committed and both edited in the worktree, so two commits can each take one.
fn edited_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    for file in ["a.txt", "b.txt"] {
        std::fs::write(repo.join(file), b"one\n").expect("file");
    }
    std::fs::create_dir(repo.join("sub")).expect("sub");
    std::fs::write(repo.join("sub").join("c.txt"), b"one\n").expect("c");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    for file in ["a.txt", "b.txt"] {
        std::fs::write(repo.join(file), b"one\ntwo\n").expect("edit");
    }
    return repo;
}

/// Define the hook `name` for `event` so it runs the script at `script`.
fn configure_hook(fixture: &Fixture, repo: &Path, name: &str, event: &str, script: &Path) {
    git(fixture, repo, &["config", format!("hook.{name}.event").as_str(), event]);
    git(fixture, repo, &["config", format!("hook.{name}.command").as_str(), script.to_str().expect("UTF-8")]);
}

/// The hook script's shared log, one line per entry.
fn log_lines(path: &Path) -> Vec<String> {
    return std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect();
}

/// A hook script that records its start, waits for the release marker, and records its end.
fn barrier_script(log: &Path, release: &Path) -> String {
    return format!(
        "#!/bin/sh\necho \"start $TAG\" >> {log}\ni=0\nwhile [ ! -e {release} ] && [ $i -lt 2500 ]; do sleep 0.02; i=$((i+1)); done\necho \"end $TAG\" >> {log}\n",
        log = log.display(),
        release = release.display()
    );
}

/// Wait until the log names the line.
fn wait_for_line(log: &Path, line: &str) {
    let started: Instant = Instant::now();
    while !log_lines(log).iter().any(|entry| { return entry == line; }) {
        assert!(started.elapsed() < BARRIER_LIMIT, "timed out waiting for {line}");
        std::thread::sleep(POLL);
    }
}

/// Start one wrapped commit of `file` in the background, tagged for the barrier hook.
fn spawn_commit(fixture: &Fixture, repo: &Path, file: &str, tag: &str) -> Child {
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--quiet", "--message", tag, "--", file]);
    command.env("TAG", tag).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    return command.spawn().expect("start the wrapped commit");
}

/// A configured hook runs once in a wrapped commit, and a traditional hook in the repository's hooks directory does
/// not run at all, because traditional hooks are unsupported.
#[test]
fn a_configured_hook_runs_once_and_a_traditional_hook_does_not_run() {
    let fixture: Fixture = fixture("hook-once");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let log: PathBuf = fixture.root.join("log");
    let script: PathBuf = fixture.root.join("hook-script");
    executable(script.as_path(), format!("#!/bin/sh\necho command >> {}\n", log.display()).as_bytes());
    configure_hook(&fixture, repo.as_path(), "script", "pre-commit", script.as_path());
    executable(
        repo.join(".git/hooks/pre-commit").as_path(),
        format!("#!/bin/sh\necho traditional >> {}\n", log.display()).as_bytes(),
    );
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=hooked", "--", "a.txt"]);
    assert_eq!(outcome.code, Some(0), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert_eq!(log_lines(log.as_path()), vec![String::from("command")]);
    assert!(!repo.join(".git/cli-git/hook.lock").exists(), "the lock is released");
    remove_settled(&fixture);
}

/// A hook that changes into a subdirectory still finds the top level of the worktree and the transaction's private
/// index, and the hooks directory it can see holds the plan record only.
#[test]
fn a_hook_that_changes_directory_finds_the_top_level_and_no_entry_in_the_hooks_directory() {
    let fixture: Fixture = fixture("hook-subdirectory");
    let repo: PathBuf = std::fs::canonicalize(edited_repository(&fixture, "repo")).expect("canonical");
    let log: PathBuf = fixture.root.join("log");
    let script: PathBuf = fixture.root.join("hook-script");
    executable(
        script.as_path(),
        format!(
            "#!/bin/sh\n{{\necho \"cwd=$(pwd -P)\"\necho \"work=[$GIT_WORK_TREE]\"\n(cd sub && echo \"top=$(git rev-parse --show-toplevel)\")\n\
             echo \"entries=$(ls \"$(dirname \"$GIT_INDEX_FILE\")/hooks\" | tr '\\n' ' ')\"\n}} >> {}\n",
            log.display()
        )
        .as_bytes(),
    );
    configure_hook(&fixture, repo.as_path(), "script", "pre-commit", script.as_path());
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=hooked", "--", "a.txt"]);
    assert_eq!(outcome.code, Some(0), "{}", String::from_utf8_lossy(&outcome.stderr));
    let lines: Vec<String> = log_lines(log.as_path());
    assert!(lines.contains(&format!("cwd={}", repo.display())), "{lines:?}");
    assert!(lines.contains(&String::from("work=[]")), "Git leaves GIT_WORK_TREE unset for the hook: {lines:?}");
    assert!(lines.contains(&format!("top={}", repo.display())), "{lines:?}");
    assert!(lines.contains(&format!("entries={PLAN_FILENAME} ")), "{lines:?}");
    remove_settled(&fixture);
}

/// A failing hook stops the commit with Git's status and releases the hook lock.
#[test]
fn a_failing_hook_stops_the_commit_and_releases_the_lock() {
    let fixture: Fixture = fixture("hook-status");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let script: PathBuf = fixture.root.join("hook-script");
    executable(script.as_path(), b"#!/bin/sh\nexit 3\n");
    configure_hook(&fixture, repo.as_path(), "script", "pre-commit", script.as_path());
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=hooked", "--", "a.txt"]);
    assert_eq!(outcome.code, Some(1), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert!(!repo.join(".git/cli-git/hook.lock").exists());
    remove_settled(&fixture);
}

/// Two commits serialize their hooks on the hook lock: the second commit's hook starts only after the first one ends.
#[test]
fn two_commits_serialize_their_hooks_by_default() {
    let fixture: Fixture = fixture("hook-serialize");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let log: PathBuf = fixture.root.join("log");
    let release: PathBuf = fixture.root.join("release");
    let script: PathBuf = fixture.root.join("hook-script");
    executable(script.as_path(), barrier_script(log.as_path(), release.as_path()).as_bytes());
    configure_hook(&fixture, repo.as_path(), "script", "pre-commit", script.as_path());
    let mut first: Child = spawn_commit(&fixture, repo.as_path(), "a.txt", "a");
    wait_for_line(log.as_path(), "start a");
    let mut second: Child = spawn_commit(&fixture, repo.as_path(), "b.txt", "b");
    std::thread::sleep(Duration::from_millis(1500));
    assert!(!log_lines(log.as_path()).contains(&String::from("start b")), "second hook started early");
    std::fs::write(&release, b"").expect("release");
    assert!(first.wait().expect("first").success());
    assert!(second.wait().expect("second").success());
    // The second commit may replay onto the first one's landing and re-run `pre-commit`, which adds another pair.
    let lines: Vec<String> = log_lines(log.as_path());
    assert_eq!(lines[..4].to_vec(), vec!["start a", "end a", "start b", "end b"], "{lines:?}");
    assert!(lines[4..].chunks(2).all(|pair| { return pair == ["start b", "end b"]; }), "{lines:?}");
    remove_settled(&fixture);
}

/// With `hooks.concurrentCommits` both commits run their hooks at once.
#[test]
fn two_commits_overlap_their_hooks_with_concurrent_commits() {
    let fixture: Fixture = fixture("hook-overlap");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("cli-git.config.jsonc"), b"{ \"hooks\": { \"concurrentCommits\": true } }\n").expect("config");
    let log: PathBuf = fixture.root.join("log");
    let release: PathBuf = fixture.root.join("release");
    let script: PathBuf = fixture.root.join("hook-script");
    executable(script.as_path(), barrier_script(log.as_path(), release.as_path()).as_bytes());
    configure_hook(&fixture, repo.as_path(), "script", "pre-commit", script.as_path());
    let mut first: Child = spawn_commit(&fixture, repo.as_path(), "a.txt", "a");
    wait_for_line(log.as_path(), "start a");
    let mut second: Child = spawn_commit(&fixture, repo.as_path(), "b.txt", "b");
    wait_for_line(log.as_path(), "start b");
    std::fs::write(&release, b"").expect("release");
    assert!(first.wait().expect("first").success());
    assert!(second.wait().expect("second").success());
    remove_settled(&fixture);
}

/// A start name that used to be a hook entry's name behaves as `git`: nothing dispatches on the name the wrapper was
/// started under, inside a transaction's hooks directory or anywhere else.
#[test]
fn a_start_name_that_used_to_be_an_entry_behaves_as_git() {
    let fixture: Fixture = fixture("hook-other-names");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let hooks: PathBuf = repo.join(".git/cli-git-transactions/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10/hooks");
    std::fs::create_dir_all(&hooks).expect("hooks directory");
    std::fs::write(hooks.join("plan.json"), b"{}\n").expect("plan");
    let elsewhere: PathBuf = fixture.root.join("elsewhere");
    std::fs::create_dir(&elsewhere).expect("elsewhere");
    let mut starts: Vec<PathBuf> = Vec::new();
    for event in ["pre-commit", "prepare-commit-msg", "commit-msg", "post-commit", "pre-push"] {
        for directory in [hooks.as_path(), elsewhere.as_path()] {
            let start: PathBuf = directory.join(event);
            std::os::unix::fs::symlink(&fixture.wrapper, &start).expect("link");
            starts.push(start);
        }
    }
    for start in starts {
        let mut command: Command = super::support::bounded(
            &fixture,
            start.as_path(),
            super::support::wrapper_first_path(&fixture).as_os_str(),
        );
        command.arg("--version").current_dir(&repo);
        let outcome = observe(&mut command, b"");
        assert_eq!(outcome.code, Some(0), "{}", String::from_utf8_lossy(&outcome.stderr));
        assert!(String::from_utf8_lossy(&outcome.stdout).starts_with("git version "), "{start:?}");
    }
    remove_settled(&fixture);
}
