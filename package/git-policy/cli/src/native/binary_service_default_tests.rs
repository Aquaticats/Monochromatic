//! What:
//!  The built wrapper's choice of commit path when the variable is unset, `0` or `1`.
//! Why:
//!  Once the activation is installed, plain scoped commits go through the service without any setting; `0` keeps the
//!  commit native; `1` on a machine without the activation stops with the plain install message and changes nothing
//!  (`doc/planning/cli-git-resident-service.md`, "Default and the uninstalled machine").
//!  The installed state is a unit file in the account's configuration directory and a socket in its runtime directory,
//!  both made by the test; no service manager is involved.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::service_commit::{SERVICE_VARIABLE, SOCKET_RELATIVE};
use git_policy_cli::service_install::SOCKET_UNIT_NAME;
use git_policy_cli::service_journal::DONE_EXTENSION;
use git_policy_cli::service_messages::INSTALL_COMMAND;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const SERVICE: &str = env!("CARGO_BIN_EXE_cli-git-service");
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with one committed file, edited, and a runtime directory the service can listen in.
fn prepared(name: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    // A socket path must fit in a few dozen characters, which the fixture tree's depth does not allow; the agent
    //  scratch directory is short and disposable.
    let scratch: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    let runtime: PathBuf = scratch.join(format!("run-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&runtime).expect("runtime directory");
    return (fixture, repo, runtime);
}

fn install_units(fixture: &Fixture) {
    let units: PathBuf = fixture.root.join("home/.config/systemd/user");
    std::fs::create_dir_all(&units).expect("unit directory");
    std::fs::write(units.join(SOCKET_UNIT_NAME), b"[Socket]\n").expect("socket unit");
}

fn start_at(fixture: &Fixture, runtime: &Path) -> Child {
    let socket: PathBuf = runtime.join(SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let child: Child = Command::new(SERVICE)
        .args(["--journal", fixture.root.join("journal").to_str().expect("UTF-8"), "--path", socket.to_str().expect("UTF-8")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the service starts");
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&socket).is_err() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return child;
}

fn commit(fixture: &Fixture, repo: &Path, runtime: &Path, setting: Option<&str>) -> Output {
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "-m", "subject", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime);
    if let Some(value) = setting {
        command.env(SERVICE_VARIABLE, value);
    }
    return command.output().expect("the wrapped commit runs");
}

fn finished_batches(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

/// Remove the runtime directory a control made, and the fixture.
fn finish(fixture: &Fixture, runtime: &Path) {
    std::fs::remove_dir_all(runtime).expect("remove only the runtime directory the control made");
    remove_settled(fixture);
}

fn subject(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["log", "-1", "--format=%s"]).trim_end().to_string();
}

/// With the activation installed and no setting, the commit goes through the service.
#[test]
fn an_installed_activation_is_used_without_a_setting() {
    let (fixture, repo, runtime) = prepared("service-default-on");
    install_units(&fixture);
    let mut service: Child = start_at(&fixture, runtime.as_path());
    let output: Output = commit(&fixture, &repo, runtime.as_path(), None);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(subject(&fixture, &repo), "subject");
    assert_eq!(finished_batches(&fixture), 1, "the service landed it");
    let _killed = service.kill();
    let _waited = service.wait();
    finish(&fixture, runtime.as_path());
}

/// `0` keeps the commit native even with the activation installed and the service running.
#[test]
fn zero_keeps_the_commit_native() {
    let (fixture, repo, runtime) = prepared("service-default-off");
    install_units(&fixture);
    let mut service: Child = start_at(&fixture, runtime.as_path());
    let output: Output = commit(&fixture, &repo, runtime.as_path(), Some("0"));
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(subject(&fixture, &repo), "subject");
    assert_eq!(finished_batches(&fixture), 0, "the service was not asked");
    let _killed = service.kill();
    let _waited = service.wait();
    finish(&fixture, runtime.as_path());
}

/// With no activation installed and no setting, the commit runs the way it did before the service existed.
#[test]
fn an_uninstalled_machine_keeps_its_commit_path() {
    let (fixture, repo, runtime) = prepared("service-default-uninstalled");
    let output: Output = commit(&fixture, &repo, runtime.as_path(), None);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(subject(&fixture, &repo), "subject");
    finish(&fixture, runtime.as_path());
}

/// `1` without the activation stops with the plain install message, quickly, and changes nothing.
#[test]
fn asking_for_the_service_without_it_stops_plainly() {
    let (fixture, repo, runtime) = prepared("service-asked-uninstalled");
    let before: String = subject(&fixture, &repo);
    let began: Instant = Instant::now();
    let output: Output = commit(&fixture, &repo, runtime.as_path(), Some("1"));
    assert!(!output.status.success(), "the commit must not land behind the person's back");
    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains(INSTALL_COMMAND), "{stderr}");
    assert!(began.elapsed() < PATIENCE / 2, "no waiting for a start that cannot happen");
    assert_eq!(subject(&fixture, &repo), before, "nothing changed");
    finish(&fixture, runtime.as_path());
}
