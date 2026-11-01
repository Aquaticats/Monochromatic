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
