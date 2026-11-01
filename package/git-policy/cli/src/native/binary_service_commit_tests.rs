//! What:
//!  The built wrapper's commit command through the resident service, compared with native `git commit`.
//! Why:
//!  The service path must make the commit native Git would have made (same tree, message and author) and must leave
//!  everything it does not cover to native Git (`doc/planning/cli-git-resident-service.md`, "Migration plan").
//!  The service is started by the test; the wrapper only reaches it when the person opts in with the variable.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::service_commit::SERVICE_VARIABLE;
use std::ffi::OsStr;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const SERVICE: &str = env!("CARGO_BIN_EXE_cli-git-service");
const PATIENCE: Duration = Duration::from_secs(60);

fn query(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return git_text(fixture, repo, arguments).trim_end().to_string();
}

fn start(fixture: &Fixture, name: &str) -> Child {
    let journal: PathBuf = fixture.root.join("journal");
    let child: Child = Command::new(SERVICE)
        .args(["--journal", journal.to_str().expect("UTF-8"), "--abstract", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the service starts");
    let began: Instant = Instant::now();
    while SocketAddr::from_abstract_name(name.as_bytes()).and_then(|socket| { return UnixStream::connect_addr(&socket); }).is_err() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return child;
}

fn through_service(fixture: &Fixture, repo: &Path, name: &str, arguments: &[&str]) -> Output {
    let mut command: Command = wrapped_commit(fixture, repo, arguments);
    command.env(SERVICE_VARIABLE, format!("abstract:{name}"));
    return command.output().expect("the wrapped commit runs");
}

fn fields(fixture: &Fixture, repo: &Path, revision: &str) -> (String, String, String) {
    return (
        query(fixture, repo, &["rev-parse", format!("{revision}^{{tree}}").as_str()]),
        query(fixture, repo, &["log", "-1", "--format=%B", revision]),
        query(fixture, repo, &["log", "-1", "--format=%an <%ae>|%cn <%ce>", revision]),
    );
}

/// A scoped commit with messages lands through the service and equals the commit native Git makes of the same input.
#[test]
fn a_scoped_commit_through_the_service_equals_the_native_commit() {
    let fixture: Fixture = fixture("service-wired");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    std::fs::write(repo.join("b.txt"), "other\n").expect("b");
    git(&fixture, &repo, &["add", "a.txt", "b.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let start_commit: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    let name: String = format!("wired-{}", std::process::id());
    let mut service: Child = start(&fixture, name.as_str());
    std::fs::write(repo.join("a.txt"), "base\nchanged\n").expect("edit a");
    std::fs::write(repo.join("b.txt"), "other\nedited but not named\n").expect("edit b");
    let served: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-m", "subject", "-m", "body text  ", "--", "a.txt"]);
    assert!(served.status.success(), "{}", String::from_utf8_lossy(&served.stderr));
    assert!(String::from_utf8_lossy(&served.stdout).contains("subject"), "{}", String::from_utf8_lossy(&served.stdout));
    let finished: usize = std::fs::read_dir(fixture.root.join("journal"))
        .expect("journal")
        .flatten()
        .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == git_policy_cli::service_journal::DONE_EXTENSION; }); })
        .count();
    assert_eq!(finished, 1, "the service landed it: one finished batch in its journal");
    let through: (String, String, String) = fields(&fixture, &repo, "HEAD");
    assert_eq!(query(&fixture, &repo, &["status", "--short"]), " M b.txt", "only the named file was committed, the index agrees");
    // The same input through native Git, from the same start.
    git(&fixture, &repo, &["reset", "--hard", "--quiet", start_commit.as_str()]);
    std::fs::write(repo.join("a.txt"), "base\nchanged\n").expect("edit a again");
    let native: Output = wrapped_commit(&fixture, &repo, &["commit", "--quiet", "-m", "subject", "-m", "body text  ", "--", "a.txt"]).output().expect("native");
    assert!(native.status.success(), "{}", String::from_utf8_lossy(&native.stderr));
    assert_eq!(fields(&fixture, &repo, "HEAD"), through);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A command line the service does not cover runs through native Git even with the service on, and with no service
///  running at all it still does not need one.
#[test]
fn a_command_line_the_service_does_not_cover_runs_natively() {
    let fixture: Fixture = fixture("service-declined");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    let name: String = format!("declined-{}", std::process::id());
    let amended: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--quiet", "--amend", "-m", "amended", "--", "a.txt"]);
    assert!(amended.status.success(), "no service is running and none was needed: {}", String::from_utf8_lossy(&amended.stderr));
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "amended");
    remove_settled(&fixture);
}

/// The directory holding a program on the search path.
fn directory_of(program: &str) -> Option<PathBuf> {
    return std::env::var_os("PATH").and_then(|paths| { return std::env::split_paths(&paths).find(|directory| { return directory.join(program).is_file(); }); });
}

/// Real Git with `git-lfs` reachable, in the fixture's hermetic environment.
fn git_with_lfs(fixture: &Fixture, repo: &Path, lfs: &Path, arguments: &[&str]) {
    let output: Output = Command::new("/usr/bin/git")
        .current_dir(repo)
        .env_clear()
        .env("PATH", format!("/usr/bin:/bin:{}", lfs.display()))
        .env("HOME", fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
}

/// Whether a program is on the search path, so a control that needs it can say plainly why it did not run.
fn on_path(program: &str) -> bool {
    return std::env::var_os("PATH").is_some_and(|paths| { return std::env::split_paths(&paths).any(|directory| { return directory.join(program).is_file(); }); });
}

/// A commit from inside a PID namespace reaches the service and lands: nothing about the commit depends on process ids.
#[test]
fn a_commit_from_a_pid_namespace_lands() {
    if !on_path("unshare") {
        eprintln!("skipped: unshare is not installed, so no namespace was entered");
        return;
    }
    let fixture: Fixture = fixture("service-pidns");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "base\nnamespaced\n").expect("edit");
    let name: String = format!("pidns-{}", std::process::id());
    let mut service: Child = start(&fixture, name.as_str());
    let inner: Command = wrapped_commit(&fixture, &repo, &["commit", "--quiet", "-m", "from a namespace", "--", "a.txt"]);
    let mut command: Command = Command::new("unshare");
    command.args(["--user", "--map-root-user", "--pid", "--fork", "--mount-proc", "--"]).arg(fixture.root.join("bin/git")).args(["commit", "--quiet", "-m", "from a namespace", "--", "a.txt"]);
    // The bound's `timeout` program is left out: as the first process of a new namespace it cannot manage its child.
    command.env_clear();
    for (key, value) in inner.get_envs() {
        if let Some(set) = value {
            command.env(key, set);
        }
    }
    command.env(SERVICE_VARIABLE, format!("abstract:{name}")).current_dir(&repo);
    let ran: Output = command.output().expect("unshare runs");
    let stderr: String = String::from_utf8_lossy(&ran.stderr).into_owned();
    if !ran.status.success() && stderr.contains("unshare:") {
        eprintln!("skipped: this system does not allow the namespace: {}", stderr.trim());
    } else {
        assert!(ran.status.success(), "status {:?}, stdout {:?}, stderr {stderr:?}", ran.status, String::from_utf8_lossy(&ran.stdout));
        assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "from a namespace");
        assert_eq!(query(&fixture, &repo, &["status", "--short"]), "");
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// Issue 1770: a scoped commit with an unrelated tracked LFS asset that is stat-dirty lands, and no temporary path
///  is printed. Needs the real `git-lfs`; without it the control says so and stops.
#[test]
fn a_scoped_commit_beside_a_stat_dirty_lfs_asset_lands_through_the_service() {
    let Some(lfs) = directory_of("git-lfs") else {
        eprintln!("skipped: git-lfs is not installed, so no LFS filter ran");
        return;
    };
    let fixture: Fixture = fixture("service-lfs");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git_with_lfs(&fixture, &repo, lfs.as_path(), &["lfs", "install", "--local", "--skip-repo"]);
    std::fs::write(repo.join(".gitattributes"), "*.bin filter=lfs diff=lfs merge=lfs -text\n").expect("attributes");
    std::fs::write(repo.join("asset.bin"), vec![7u8; 4 << 20]).expect("asset");
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git_with_lfs(&fixture, &repo, lfs.as_path(), &["add", ".gitattributes", "asset.bin", "a.txt"]);
    git_with_lfs(&fixture, &repo, lfs.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let later: std::time::SystemTime = std::time::SystemTime::now() + Duration::from_secs(5);
    std::fs::File::options().write(true).open(repo.join("asset.bin")).expect("asset").set_modified(later).expect("touch");
    std::fs::write(repo.join("a.txt"), "base\nscoped\n").expect("edit");
    let name: String = format!("lfs-{}", std::process::id());
    let mut service: Child = start(&fixture, name.as_str());
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "--quiet", "-m", "scoped", "--", "a.txt"]);
    let path: String = command.get_envs().find(|(key, _)| { return *key == "PATH"; }).and_then(|(_, value)| { return value.map(|set| { return set.to_string_lossy().into_owned(); }); }).unwrap_or_default();
    command.env("PATH", format!("{path}:{}", lfs.display())).env(SERVICE_VARIABLE, format!("abstract:{name}"));
    let ran: Output = command.output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&ran.stderr).into_owned();
    assert!(ran.status.success(), "{stderr}");
    assert!(!stderr.contains("lfs/tmp") && !stderr.contains("Error cleaning"), "{stderr}");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "scoped");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// `git cli-git service install` through the built wrapper puts the units next to the account's configuration, enables
///  the socket through the service manager's control program, is repeatable, and `uninstall` takes it all out again.
#[test]
fn the_wrapper_installs_and_removes_the_service_activation() {
    let fixture: Fixture = fixture("service-install-binary");
    let calls: PathBuf = fixture.root.join("systemctl.log");
    super::support::executable(
        fixture.root.join("bin/systemctl").as_path(),
        format!("#!/bin/sh\necho \"$@\" >> '{}'\nexit 0\n", calls.display()).as_bytes(),
    );
    super::support::copy_executable(Path::new(SERVICE), fixture.root.join("wrapper/cli-git-service").as_path());
    let run = |arguments: &[&str]| -> Output {
        return wrapped_commit(&fixture, &fixture.root, arguments).output().expect("the wrapper runs");
    };
    let installed: Output = run(&["cli-git", "service", "install"]);
    assert!(installed.status.success(), "{}", String::from_utf8_lossy(&installed.stderr));
    let units: PathBuf = fixture.root.join("home/.config/systemd/user");
    for name in [git_policy_cli::service_install::SOCKET_UNIT_NAME, git_policy_cli::service_install::SERVICE_UNIT_NAME] {
        assert!(units.join(name).is_file(), "{name} was written");
    }
    let service: String = std::fs::read_to_string(units.join(git_policy_cli::service_install::SERVICE_UNIT_NAME)).expect("unit");
    assert!(service.contains(fixture.root.join("wrapper/cli-git-service").to_string_lossy().as_ref()), "{service}");
    assert!(String::from_utf8_lossy(&installed.stdout).contains("Installed"), "{}", String::from_utf8_lossy(&installed.stdout));
    let again: Output = run(&["cli-git", "service", "install"]);
    assert!(String::from_utf8_lossy(&again.stdout).contains("already installed"), "{}", String::from_utf8_lossy(&again.stdout));
    let removed: Output = run(&["cli-git", "service", "uninstall"]);
    assert!(removed.status.success(), "{}", String::from_utf8_lossy(&removed.stderr));
    assert!(!units.join(git_policy_cli::service_install::SOCKET_UNIT_NAME).exists());
    let log: String = std::fs::read_to_string(&calls).expect("calls");
    assert!(log.contains("--user enable --now cli-git.socket") && log.contains("disable --now"), "{log}");
    let wrong: Output = run(&["cli-git", "service", "start"]);
    assert!(!wrong.status.success(), "an unknown word is a usage error");
    remove_settled(&fixture);
}
