//! What:
//!  The built wrapper's commit command through the resident service,
//! compared with native `git commit`.
//! Why:
//!  The service path must make the commit native Git would have made (same tree,
//! message and author) and must leave
//!  everything it does not cover to native Git (`doc/planning/cli-git-resident-service.md`,
//! "Migration plan").
//!  The service is started by the test at the socket path under the runtime directory the wrapper looks in,
//! as the
//!  operating system's socket activation would leave it;
//! the wrapper finds the socket and uses it.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::service_instance::{instance_of, socket_for};
use std::ffi::OsStr;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const PATIENCE: Duration = Duration::from_secs(60);

pub(super) fn query(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return git_text(fixture, repo, arguments).trim_end().to_string();
}

/// The runtime directory a service under test uses.
/// A socket path must fit in a few dozen characters,
/// which the
///  fixture tree's depth does not allow,
/// so it lives in the short agent scratch directory.
pub(super) fn runtime_for(name: &str) -> PathBuf {
    let scratch: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    return scratch.join(format!("rn-{}-{}", std::process::id(), runtime_number(name)));
}

/// The names given to `runtime_for` so far in this process,
/// in the order they were first given.
///  A mutex guards the list because tests of one process may name runtimes from several threads;
/// a test-only list,
/// so no
///  production code waits on it.
static RUNTIME_NAMES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// A small number for a runtime name:
///  the socket path is the runtime directory,
/// the program's folder name and a 32-digit instance name,
/// and the whole must
///  fit in a socket address (107 bytes),
/// so the directory name is a number and not the test's name.
fn runtime_number(name: &str) -> usize {
    let mut names = RUNTIME_NAMES.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(position) = names.iter().position(|known| { return known == name; }) {
        return position;
    }
    names.push(name.to_owned());
    return names.len() - 1;
}

/// The services under test,
/// one per worktree as in production,
/// stopped (and their runtime directory removed) when they go out
///  of scope.
pub(super) struct Running {
    children: Vec<Child>,
    runtime: PathBuf,
}

/// Methods over running services.
impl Running {
    pub(super) fn kill(&mut self) -> std::io::Result<()> {
        let mut result: std::io::Result<()> = Ok(());
        for child in &mut self.children {
            let killed = child.kill();
            if result.is_ok() {
                result = killed;
            }
        }
        return result;
    }

    pub(super) fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        let mut last: Option<std::io::Result<std::process::ExitStatus>> = None;
        for child in &mut self.children {
            last = Some(child.wait());
        }
        // No service was started: nothing ended badly.
        return last.unwrap_or_else(|| { return Ok(std::os::unix::process::ExitStatusExt::from_raw(0)); });
    }

    /// Waits for the first service to exit by itself for at most `limit`;
    /// nothing when it is still running then.
    pub(super) fn exited_within(&mut self, limit: Duration) -> Option<std::process::ExitStatus> {
        let began: Instant = Instant::now();
        let child: &mut Child = self.children.first_mut()?;
        loop {
            if let Ok(Some(status)) = child.try_wait() {
                return Some(status);
            }
            if began.elapsed() >= limit {
                return None;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _killed = child.kill();
            let _waited = child.wait();
        }
        let _removed = std::fs::remove_dir_all(&self.runtime);
    }
}

/// Starts the service of every worktree under `root` on the socket the wrapper looks for in `runtime`,
/// with the settings a
///  service manager would give each instance.
/// A worktree made later gets its service from `start_worktree`.
pub(super) fn start_in(root: &Path, runtime: &Path) -> Running {
    return start_in_release(root, runtime, None);
}

/// Starts the service as `start_in` does,
/// announcing `release` to its clients instead of its own,
/// so a test can stand a
///  service of another release where the manager would have left one from before an upgrade.
pub(super) fn start_in_release(root: &Path, runtime: &Path, release: Option<&str>) -> Running {
    let git_dirs: Vec<PathBuf> = super::support::worktree_git_dirs(root, 4);
    let mut running: Running = Running { children: Vec::new(), runtime: runtime.to_path_buf() };
    serve_into(&mut running, root, runtime, git_dirs.as_slice(), release);
    return running;
}

/// Starts the service of every worktree in `git_dirs` and adds it to `running`,
/// waiting until each listens.
fn serve_into(running: &mut Running, root: &Path, runtime: &Path, git_dirs: &[PathBuf], release: Option<&str>) {
    std::fs::create_dir_all(runtime.join("monochromatic-git-policy-cli")).expect("socket directory");
    let journal: PathBuf = root.join("journal");
    let mut sockets: Vec<PathBuf> = Vec::new();
    for git_dir in git_dirs {
        let socket: PathBuf = socket_for(runtime, git_dir.as_path());
        let child: Child = super::support::service_command()
            .args(["--journal", journal.to_str().expect("UTF-8"), "--path", socket.to_str().expect("UTF-8")])
            .args(["--instance", instance_of(git_dir.as_path()).as_str()])
            .args(release.map(|text| { return ["--release", text]; }).into_iter().flatten())
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", root.join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the service starts");
        running.children.push(child);
        sockets.push(socket);
    }
    let began: Instant = Instant::now();
    for socket in sockets {
        while UnixStream::connect(&socket).is_err() {
            assert!(
                began.elapsed() < PATIENCE,
                "the service never listened on {} for the worktrees under {} within {PATIENCE:?}",
                socket.display(),
                root.display()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// Starts the service of one worktree made after the others were,
/// in the runtime directory another handle owns:
/// the handle
///  returned stops this service and leaves the runtime directory to its owner.
pub(super) fn start_worktree(root: &Path, runtime: &Path, repo: &Path) -> Running {
    let mut running: Running = Running { children: Vec::new(), runtime: PathBuf::new() };
    serve_into(&mut running, root, runtime, &[repo.join(".git")], None);
    return running;
}

pub(super) fn start(fixture: &Fixture, name: &str) -> Running {
    return start_in(fixture.root.as_path(), runtime_for(name).as_path());
}

pub(super) fn through_service(fixture: &Fixture, repo: &Path, name: &str, arguments: &[&str]) -> Output {
    let mut command: Command = wrapped_commit(fixture, repo, arguments);
    command.env("XDG_RUNTIME_DIR", runtime_for(name));
    return command.output().expect("the wrapped commit runs");
}

/// `through_service` with environment variables set for the wrapped command.
pub(super) fn through_service_with(fixture: &Fixture, repo: &Path, name: &str, arguments: &[&str], environment: &[(&str, &str)]) -> Output {
    let mut command: Command = wrapped_commit(fixture, repo, arguments);
    command.env("XDG_RUNTIME_DIR", runtime_for(name));
    for (variable, value) in environment {
        command.env(variable, value);
    }
    return command.output().expect("the wrapped commit runs");
}

fn fields(fixture: &Fixture, repo: &Path, revision: &str) -> (String, String, String) {
    return (
        query(fixture, repo, &["rev-parse", format!("{revision}^{{tree}}").as_str()]),
        query(fixture, repo, &["log", "-1", "--format=%B", revision]),
        query(fixture, repo, &["log", "-1", "--format=%an <%ae>|%cn <%ce>", revision]),
    );
}

/// What:
///  A commit whose changed paths would not fit one protocol line is built by Git and still lands.
/// Why:
///   "Commits should never fail."
/// A name that is not text carries its bytes beside the readable spelling,
/// so a request of the most changes at the path length the line ceiling was measured for is longer than the line the
///  service reads;
/// before a changed path carried its bytes such a commit took the route where the wrapper builds it,
/// and it must again,
/// with no word the person did not get from that route before.
#[test]
fn a_commit_whose_changed_paths_would_not_fit_one_line_lands_anyway() {
    use git_policy_cli::service_protocol::{MAX_CHANGES, P99_CHANGED_PATH_CHARACTERS};
    use std::os::unix::ffi::OsStrExt;
    let fixture: Fixture = fixture("service-line-ceiling");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    // The most changes a request may carry, each at the path length the line ceiling was measured for, and each holding a
    //  byte that is not text: the readable spelling and the bytes together are more than one line holds.
    std::fs::create_dir(repo.join("many")).expect("a directory of the control");
    let stem: String = "p".repeat(P99_CHANGED_PATH_CHARACTERS - 5);
    // The pathspec file names every one of them, so the command line stays short and the commit names its paths.
    let mut pathspec: Vec<u8> = Vec::new();
    for number in 0..MAX_CHANGES {
        let mut name: Vec<u8> = format!("{number:04}-{stem}").into_bytes();
        name.push(0xff);
        std::fs::write(repo.join("many").join(OsStr::from_bytes(name.as_slice())), "x\n").expect("a file of the control");
        pathspec.extend(b"many/");
        pathspec.extend(name);
        pathspec.push(0);
    }
    let pathspec_file: PathBuf = fixture.root.join("names.pathspec");
    std::fs::write(pathspec_file.as_path(), pathspec.as_slice()).expect("the pathspec file");
    git(&fixture, &repo, &["add", "--all"]);
    let name: String = format!("ceiling-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let served: Output = through_service(
        &fixture,
        &repo,
        name.as_str(),
        &[
            "commit",
            "--quiet",
            "--message=many names",
            "--pathspec-from-file",
            pathspec_file.to_str().expect("a text path"),
            "--pathspec-file-nul",
        ],
    );
    assert!(served.status.success(), "{}", String::from_utf8_lossy(&served.stderr));
    // The commit Git built is sent whole: the batch the service landed names the commit and no list of changes.
    let journal: PathBuf = fixture.root.join("journal");
    let done: Vec<PathBuf> = std::fs::read_dir(journal.as_path())
        .expect("journal")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.extension().is_some_and(|extension| { return extension == git_policy_cli::service_journal::DONE_EXTENSION; }); })
        .collect();
    assert_eq!(done.len(), 1, "the service landed the commit the wrapper built: {done:?}");
    let intent: String = std::fs::read_to_string(&done[0]).expect("the intent");
    assert!(intent.contains("derived=1"), "the request carried no list of changes: {intent}");
    let listed: String = query(&fixture, &repo, &["ls-tree", "-r", "--name-only", "HEAD"]);
    assert_eq!(listed.lines().count(), MAX_CHANGES, "every staged name is in the commit");
    service.kill().expect("the service ends");
    remove_settled(&fixture);
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
    let mut service: Running = start(&fixture, name.as_str());
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

/// The harness's per-fixture service takes the wrapped commit the way a person's commit is taken,
/// and goes when its owner
///  goes,
/// with its runtime directory.
#[test]
fn the_fixture_service_takes_a_wrapped_commit_and_goes_with_its_owner() {
    let fixture: Fixture = fixture("service-harness");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    let service = super::support::start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    let runtime: PathBuf = service.runtime.clone();
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "-m", "through the harness", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let finished: usize = std::fs::read_dir(fixture.root.join("journal"))
        .expect("journal")
        .flatten()
        .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == git_policy_cli::service_journal::DONE_EXTENSION; }); })
        .count();
    assert_eq!(finished, 1, "the fixture's service landed it");
    drop(service);
    assert!(!runtime.exists(), "the runtime directory went with the service");
    remove_settled(&fixture);
}

/// The directory holding a program on the search path.
fn directory_of(program: &str) -> Option<PathBuf> {
    return std::env::var_os("PATH").and_then(|paths| { return std::env::split_paths(&paths).find(|directory| { return directory.join(program).is_file(); }); });
}

/// Real Git with `git-lfs` reachable,
/// in the fixture's hermetic environment.
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

/// Whether a program is on the search path,
/// so a control that needs it can say plainly why it did not run.
fn on_path(program: &str) -> bool {
    return std::env::var_os("PATH").is_some_and(|paths| { return std::env::split_paths(&paths).any(|directory| { return directory.join(program).is_file(); }); });
}

/// A commit from inside a PID namespace reaches the service and lands:
/// nothing about the commit depends on process ids.
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
    let mut service: Running = start(&fixture, name.as_str());
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
    command.env("XDG_RUNTIME_DIR", runtime_for(name.as_str())).current_dir(&repo);
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

/// Issue 1770:
/// a scoped commit with an unrelated tracked LFS asset that is stat-dirty lands,
/// and no temporary path
///  is printed.
/// Needs the real `git-lfs`;
/// without it the control says so and stops.
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
    let mut service: Running = start(&fixture, name.as_str());
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "--quiet", "-m", "scoped", "--", "a.txt"]);
    let path: String = command.get_envs().find(|(key, _)| { return *key == "PATH"; }).and_then(|(_, value)| { return value.map(|set| { return set.to_string_lossy().into_owned(); }); }).unwrap_or_default();
    command.env("PATH", format!("{path}:{}", lfs.display())).env("XDG_RUNTIME_DIR", runtime_for(name.as_str()));
    let ran: Output = command.output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&ran.stderr).into_owned();
    assert!(ran.status.success(), "{stderr}");
    assert!(!stderr.contains("lfs/tmp") && !stderr.contains("Error cleaning"), "{stderr}");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "scoped");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
