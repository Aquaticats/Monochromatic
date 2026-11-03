//! What:
//!  Client-side behavior of a commit that lands through the resident service, seen from the person's terminal.
//! Why:
//!  The wrapper process prints what the person reads (warnings, the summary line, the wait notices), while the service
//!  lands the commit; each control starts a service of its own at a throwaway socket and runs the built wrapper against it.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::{Running, runtime_for, start};
use super::support::{Fixture, fixture, git, repository};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// A repository with `a.txt` committed and edited in the worktree, ready for one commit.
fn edited_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    return repo;
}

/// The wrapped command with the runtime directory of the service named `name`.
fn served(fixture: &Fixture, repo: &Path, name: &str, arguments: &[&str]) -> Command {
    let mut command: Command = wrapped_commit(fixture, repo, arguments);
    command.env("XDG_RUNTIME_DIR", runtime_for(name));
    return command;
}

/// The number of batches the service of `fixture` finished.
fn finished_batches(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal"))
        .expect("journal")
        .flatten()
        .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == git_policy_cli::service_journal::DONE_EXTENSION; }); })
        .count();
}

/// A commit that lands through the service in a repository that configures an external hook prints the same
///  `external-hooks-unsupported` warning the transaction route prints, and the hook never runs.
#[test]
fn a_service_commit_warns_that_configured_hooks_never_run() {
    let fixture: Fixture = fixture("client-hooks-warning");
    let repo: PathBuf = edited_repository(&fixture);
    let marker: PathBuf = fixture.root.join("marker");
    let script: PathBuf = fixture.root.join("hook-program");
    super::support::executable(script.as_path(), format!("#!/bin/sh\necho ran >> '{}'\nexit 9\n", marker.display()).as_bytes());
    git(&fixture, repo.as_path(), &["config", "hook.on-commit.event", "pre-commit"]);
    git(&fixture, repo.as_path(), &["config", "hook.on-commit.command", script.to_str().expect("UTF-8")]);
    let name: String = format!("client-hooks-{}", std::process::id());
    let _service: Running = start(&fixture, name.as_str());
    let output: Output = served(&fixture, &repo, name.as_str(), &["commit", "--quiet", "-m", "second", "--", "a.txt"]).output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    assert_eq!(finished_batches(&fixture), 1, "the service landed it: {stderr}");
    assert!(!marker.exists(), "the hook ran");
    assert!(stderr.contains("external-hooks-unsupported") && stderr.contains("hook.on-commit"), "{stderr}");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), "second");
    remove_settled(&fixture);
}

/// The summary lines on standard output, each with the name it carries: the first word that ends the bracket.
fn summaries(output: &Output) -> Vec<(String, String)> {
    return String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| { return line.starts_with('['); })
        .map(|line| {
            let name: &str = line.split_whitespace().find(|word| { return word.ends_with(']'); }).unwrap_or("").trim_end_matches(']');
            return (line.to_string(), name.to_string());
        })
        .collect();
}

/// The name Git itself gives `HEAD`.
fn git_short_head(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["rev-parse", "--short", "HEAD"]).trim().to_string();
}

/// A commit the service builds from the command line prints the name Git abbreviates to: the length `core.abbrev` asks for,
///  and the automatic length when it is unset.
#[test]
fn the_summary_line_of_a_service_commit_has_the_name_length_git_gives() {
    for (label, abbrev) in [("unset", None), ("twelve", Some("12")), ("whole", Some("no"))] {
        let fixture: Fixture = fixture(format!("client-abbrev-{label}").as_str());
        let repo: PathBuf = edited_repository(&fixture);
        if let Some(value) = abbrev {
            git(&fixture, repo.as_path(), &["config", "core.abbrev", value]);
        }
        let name: String = format!("client-abbrev-{label}-{}", std::process::id());
        let _service: Running = start(&fixture, name.as_str());
        let output: Output = served(&fixture, &repo, name.as_str(), &["commit", "-m", "second", "--", "a.txt"]).output().expect("the wrapped commit runs");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(finished_batches(&fixture), 1, "the service landed it");
        let lines: Vec<(String, String)> = summaries(&output);
        assert_eq!(lines.len(), 1, "{label}: {:?}", String::from_utf8_lossy(&output.stdout));
        assert_eq!(lines[0].1, git_short_head(&fixture, &repo), "{label}: {}", lines[0].0);
        assert!(lines[0].0.starts_with("[main "), "{label}: {}", lines[0].0);
        remove_settled(&fixture);
    }
}

/// A commit that Git builds (an option the service does not read) prints one summary line with the same name, and `--quiet`
///  prints none, whoever builds the commit.
#[test]
fn a_commit_git_builds_prints_one_summary_line_and_quiet_prints_none() {
    let fixture: Fixture = fixture("client-built-summary");
    let repo: PathBuf = edited_repository(&fixture);
    git(&fixture, repo.as_path(), &["config", "core.abbrev", "11"]);
    let name: String = format!("client-built-{}", std::process::id());
    let _service: Running = start(&fixture, name.as_str());
    let built: Output = served(&fixture, &repo, name.as_str(), &["commit", "--cleanup=scissors", "-m", "built by git", "--", "a.txt"]).output().expect("the wrapped commit runs");
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    assert_eq!(finished_batches(&fixture), 1, "the service landed it");
    let lines: Vec<(String, String)> = summaries(&built);
    assert_eq!(lines.len(), 1, "{:?}", String::from_utf8_lossy(&built.stdout));
    assert_eq!(lines[0].1, git_short_head(&fixture, &repo));
    assert!(lines[0].0.ends_with("built by git"), "{}", lines[0].0);
    // Git's own summary of a commit it builds also tells what changed, as native `git commit` prints it.
    assert!(String::from_utf8_lossy(&built.stdout).contains(" 1 file changed, 1 insertion(+)"), "{:?}", String::from_utf8_lossy(&built.stdout));
    for options in [&["--quiet"][..], &["-q"], &["-qn"]] {
        std::fs::write(repo.join("a.txt"), format!("quiet {options:?}\n")).expect("edit");
        let mut arguments: Vec<&str> = vec!["commit"];
        arguments.extend_from_slice(options);
        arguments.extend_from_slice(&["--cleanup=scissors", "-m", "silent", "--", "a.txt"]);
        let quiet: Output = served(&fixture, &repo, name.as_str(), arguments.as_slice()).output().expect("the wrapped commit runs");
        assert!(quiet.status.success(), "{options:?}: {}", String::from_utf8_lossy(&quiet.stderr));
        assert_eq!(String::from_utf8_lossy(&quiet.stdout), "", "{options:?}");
    }
    remove_settled(&fixture);
}

/// The first commit of a branch is worded `(root-commit)` and a detached head `detached HEAD`, as Git words them.
#[test]
fn a_root_commit_and_a_detached_head_are_worded_as_git_words_them() {
    let fixture: Fixture = fixture("client-summary-states");
    let unborn: PathBuf = fixture.root.join("unborn");
    std::fs::create_dir(&unborn).expect("directory");
    git(&fixture, unborn.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(unborn.join("a.txt"), b"first\n").expect("file");
    git(&fixture, unborn.as_path(), &["add", "a.txt"]);
    git(&fixture, unborn.as_path(), &["config", "maintenance.autoDetach", "false"]);
    let name: String = format!("client-states-{}", std::process::id());
    let _service: Running = start(&fixture, name.as_str());
    let first: Output = served(&fixture, &unborn, name.as_str(), &["commit", "-m", "the root", "--", "a.txt"]).output().expect("the wrapped commit runs");
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let root_lines: Vec<(String, String)> = summaries(&first);
    assert_eq!(root_lines.len(), 1, "{:?}", String::from_utf8_lossy(&first.stdout));
    assert!(root_lines[0].0.starts_with("[main (root-commit) "), "{}", root_lines[0].0);
    assert_eq!(root_lines[0].1, git_short_head(&fixture, &unborn));
    git(&fixture, unborn.as_path(), &["checkout", "--quiet", "--detach"]);
    std::fs::write(unborn.join("a.txt"), b"first\nsecond\n").expect("edit");
    let detached: Output = served(&fixture, &unborn, name.as_str(), &["commit", "-m", "adrift", "--", "a.txt"]).output().expect("the wrapped commit runs");
    assert!(detached.status.success(), "{}", String::from_utf8_lossy(&detached.stderr));
    let detached_lines: Vec<(String, String)> = summaries(&detached);
    assert_eq!(detached_lines.len(), 1, "{:?}", String::from_utf8_lossy(&detached.stdout));
    assert!(detached_lines[0].0.starts_with("[detached HEAD "), "{}", detached_lines[0].0);
    remove_settled(&fixture);
}

/// A service process that is stopped, with its runtime directory removed, when the control ends.
struct Held {
    child: Child,
    runtime: PathBuf,
}

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.child.kill();
        let _waited = self.child.wait();
        let _removed = std::fs::remove_dir_all(&self.runtime);
    }
}

/// A service whose Git is the program at `git_program`, listening where the wrapper looks for it under the runtime directory
///  of `name`.
fn service_with_git(fixture: &Fixture, name: &str, git_dir: &Path, git_program: &Path) -> Held {
    let runtime: PathBuf = runtime_for(name);
    let socket: PathBuf = git_policy_cli::service_instance::socket_for(runtime.as_path(), git_dir);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let child: Child = Command::new(env!("CARGO_BIN_EXE_cli-git-service"))
        .args(["--journal"])
        .arg(fixture.root.join("journal"))
        .arg("--path")
        .arg(&socket)
        .arg("--instance")
        .arg(git_policy_cli::service_instance::instance_of(git_dir))
        .arg("--git")
        .arg(git_program)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the service starts");
    let held: Held = Held { child, runtime };
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&socket).is_err() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return held;
}

/// How long a control waits for a state it started, far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(60);

/// A Git for the service that stops the landing at its branch move, with the index held, until the control releases it:
///  it creates `entered` when the move starts and goes on once `release` exists.
fn stalling_git(fixture: &Fixture) -> (PathBuf, PathBuf, PathBuf) {
    let entered: PathBuf = fixture.root.join("landing-entered");
    let release: PathBuf = fixture.root.join("landing-release");
    let program: PathBuf = fixture.root.join("stalling-git");
    super::support::executable(
        program.as_path(),
        format!(
            "#!/bin/sh\nfor argument in \"$@\"; do\n  if [ \"$argument\" = update-ref ]; then\n    : > '{}'\n    while [ ! -e '{}' ] && {}; do sleep 0.02; done\n    break\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
            entered.display(),
            release.display(),
            super::support::while_test_lives(fixture.root.as_path())
        )
        .as_bytes(),
    );
    return (program, entered, release);
}

/// While the service lands a commit it holds the repository's index; a wrapped command that writes the index in the same
///  worktree waits for it, says what it waits for, and then succeeds, as it would behind a commit that returned.
#[test]
fn an_index_writer_waits_for_the_service_to_finish_landing_and_says_so() {
    let fixture: Fixture = fixture("client-index-wait");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("b.txt"), b"staged by the second command\n").expect("b.txt");
    let (program, entered, release) = stalling_git(&fixture);
    let name: String = format!("client-index-wait-{}", std::process::id());
    let _service: Held = service_with_git(&fixture, name.as_str(), repo.join(".git").as_path(), program.as_path());
    let mut landing: Command = served(&fixture, &repo, name.as_str(), &["commit", "--quiet", "-m", "slow landing", "--", "a.txt"]);
    let landing_child: Child = landing.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the commit starts");
    let began: Instant = Instant::now();
    while !entered.exists() {
        assert!(began.elapsed() < PATIENCE, "the landing never reached the branch move");
        std::thread::sleep(Duration::from_millis(10));
    }
    let index_lock: PathBuf = repo.join(".git/index.lock");
    assert!(std::fs::read_to_string(&index_lock).is_ok_and(|text| { return text.starts_with("cli-git-service"); }), "the service holds the index while it lands");
    let mut adding: Command = served(&fixture, &repo, name.as_str(), &["add", "b.txt"]);
    adding.env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
    let mut adding_child: Child = adding.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().expect("the add starts");
    let adding_stderr = adding_child.stderr.take().expect("piped standard error");
    let (noticed, notice) = std::sync::mpsc::channel::<()>();
    let reader = std::thread::spawn(move || {
        let mut all: String = String::new();
        for line in std::io::BufRead::lines(std::io::BufReader::new(adding_stderr)).map_while(Result::ok) {
            if line.contains("commit service") {
                let _told = noticed.send(());
            }
            all.push_str(&line);
            all.push('\n');
        }
        return all;
    });
    // The landing is released once the add has said that it waits for the service: the notice is the add's own report that it
    //  met the held index and did not give up, so no clock in this control decides whether the add has reached the wait.
    notice.recv_timeout(PATIENCE).expect("the add said that it waits for the service");
    let early: Option<std::process::ExitStatus> = adding_child.try_wait().expect("the add can be watched");
    std::fs::write(&release, b"").expect("release");
    let added_status: std::process::ExitStatus = adding_child.wait().expect("the add ends");
    let landed: Output = landing_child.wait_with_output().expect("the commit ends");
    let stderr: String = reader.join().expect("the add's standard error is read");
    assert_eq!(early, None, "the add did not give up while the service held the index: {stderr}");
    assert!(added_status.success(), "the add succeeded after the wait: {stderr}");
    assert!(landed.status.success(), "{}", String::from_utf8_lossy(&landed.stderr));
    assert!(stderr.contains("commit service") && stderr.contains("git add"), "the wait is named: {stderr}");
    assert!(!stderr.contains("index.lock"), "no internal file name reaches the person: {stderr}");
    assert_eq!(git_text(&fixture, &repo, &["diff", "--cached", "--name-only"]).trim(), "b.txt", "the add staged its file after the landing");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), "slow landing");
    remove_settled(&fixture);
}

/// A hold of the service that is far older than any landing is a dead service's leftover: the wrapped command does not wait for
///  it as if a commit were in progress, and keeps the behavior it has with a lock nobody can prove alive.
#[test]
fn a_stale_mark_of_the_service_is_not_waited_for() {
    let fixture: Fixture = fixture("client-stale-mark");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("b.txt"), b"blocked\n").expect("b.txt");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock, b"cli-git-service 41\n").expect("a leftover hold");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - Duration::from_secs(3600);
    std::fs::File::options().write(true).open(&lock).expect("the lock opens").set_modified(long_ago).expect("the age is set");
    let mut adding: Command = wrapped_commit(&fixture, &repo, &["add", "b.txt"]);
    adding.env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
    let added: Output = adding.output().expect("the add runs");
    let stderr: String = String::from_utf8_lossy(&added.stderr).into_owned();
    assert!(!added.status.success(), "{stderr}");
    assert!(!stderr.contains("waiting for the commit service"), "{stderr}");
    assert!(lock.exists(), "the leftover is not the command's to remove");
    remove_settled(&fixture);
}

/// An index lock that is not the service's is a foreign program's, and the wrapped command keeps native Git's behavior with it:
///  it does not claim to wait for the service.
#[test]
fn a_foreign_index_lock_is_not_called_the_commit_service() {
    let fixture: Fixture = fixture("client-foreign-lock");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("b.txt"), b"blocked\n").expect("b.txt");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("a foreign lock");
    let mut adding: Command = wrapped_commit(&fixture, &repo, &["add", "b.txt"]);
    adding.env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
    let added: Output = adding.output().expect("the add runs");
    let stderr: String = String::from_utf8_lossy(&added.stderr).into_owned();
    assert!(!added.status.success(), "{stderr}");
    assert!(!stderr.contains("commit service"), "{stderr}");
    remove_settled(&fixture);
}
