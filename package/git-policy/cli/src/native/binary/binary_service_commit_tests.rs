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
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use git_policy_cli::service_instance::{instance_of, socket_for};
use std::ffi::{OsStr, OsString};
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

    /// What:
    ///  Ends the service's processes and waits for them,
    /// leaving the runtime directory to its owner.
    /// Why:
    ///   The stand-in manager's stop ends the unit's processes before the next start of the same instance,
    /// and a start must not
    ///  race a process that still holds the socket.
    pub(super) fn stop(&mut self) {
        for child in &mut self.children {
            let _killed = child.kill();
            let _waited = child.wait();
        }
        self.children.clear();
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

/// What:
///  A replace ref added to a commit between two landings of one service reaches the second landing.
/// Why:
///  The service's landing state is entered once around its whole loop,
///  so a memory of commits read that no landing empties lives as long as the service:
///  the amend then takes the commit's own parents where Git reads the replacement's,
///  and lands on a parent Git gives it none for.
///  A service of this test lands a commit,
///  the replacement is added,
///  and the amend is the second landing of the same service,
///  in a reftable repository and in a files repository holding an unrelated replace ref.
#[test]
fn a_replace_ref_added_between_two_landings_reaches_the_second_landing() {
    let fixture: Fixture = fixture("service-two-landings-replaced");
    let arms: [(&str, &str, &[&str], bool); 2] = [
        ("reftable", "reftable", &["--ref-format=reftable"], false),
        ("files-replace", "files beside a replace ref", &[], true),
    ];
    let mut repos: Vec<PathBuf> = Vec::new();
    for (name, arm, format, unrelated) in arms {
        let repo: PathBuf = fixture.root.join(name);
        std::fs::create_dir(repo.as_path()).expect("repository directory");
        let mut init: Vec<&str> = vec!["init", "--quiet", "--initial-branch=main"];
        init.extend_from_slice(format);
        git(&fixture, repo.as_path(), init.as_slice());
        std::fs::write(repo.join("a.txt"), "base\n").expect("a");
        git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
        if unrelated {
            // A replace ref that stands on an object nothing else names: it makes the reader decline the store
            //  without changing what Git answers for this history.
            let root: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
            let tree: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
            let dangling: String = query(&fixture, repo.as_path(), &["commit-tree", tree.as_str(), "-m", "unrelated"]);
            git(&fixture, repo.as_path(), &["replace", dangling.as_str(), root.as_str()]);
            assert_eq!(query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), root, "{arm}: the unrelated ref leaves the branch where it was");
        }
        git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
        repos.push(repo);
    }
    let service_name: String = format!("two-landings-replaced-{}", std::process::id());
    let mut service: Running = start(&fixture, service_name.as_str());
    for (index, (_name, arm, _format, _unrelated)) in arms.iter().enumerate() {
        let repo: &PathBuf = &repos[index];
        // The first landing of the service reads the commit it builds for its objects and for its reflog row.
        std::fs::write(repo.join("a.txt"), "one\n").expect("a");
        let served: Output = through_service(&fixture, repo.as_path(), service_name.as_str(), &["commit", "--quiet", "--message=first", "--", "a.txt"]);
        assert!(served.status.success(), "{arm}: {}", String::from_utf8_lossy(served.stderr.as_slice()));
        let first: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let parent: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD~1"]);
        assert_eq!(query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", first.as_str()]), format!("{first} {parent}"), "{arm}: the landed commit stands on the base");
        // The replacement stands on no parent, so Git gives the amended commit none either.
        let tree: String = query(&fixture, repo.as_path(), &["rev-parse", format!("{first}^{{tree}}").as_str()]);
        let replacement: String = query(&fixture, repo.as_path(), &["commit-tree", tree.as_str(), "-m", "replacement"]);
        git(&fixture, repo.as_path(), &["replace", first.as_str(), replacement.as_str()]);
        // Positive control: Git now reads no parent for the commit the second landing amends.
        assert_eq!(query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", first.as_str()]), first, "{arm}: Git reads the replacement's parents");
        std::fs::write(repo.join("a.txt"), "two\n").expect("a");
        let amended: Output = through_service(&fixture, repo.as_path(), service_name.as_str(), &["commit", "--amend", "--message=amended", "--", "a.txt"]);
        assert!(amended.status.success(), "{arm}: {}", String::from_utf8_lossy(amended.stderr.as_slice()));
        let landed: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        assert_ne!(landed, first, "{arm}: the branch moved to the amended commit");
        assert_eq!(
            query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", landed.as_str()]),
            landed,
            "{arm}: the amended commit stands on the parents Git reads, which are none"
        );
    }
    let _killed = service.kill();
    let _waited = service.wait();
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

/// What:
///  An amend of a commit a replace ref stands for lands with the parents Git reads,
///  in a files-backend repository and in a reftable one.
/// Why:
///   A reftable repository keeps `refs/replace/<name>` in its tables,
///  and the object reader the landing asks for the amended commit's parents scans
///  `refs/replace` and `packed-refs` only:
///  it answered the replaced commit's own parents while Git answers the replacement's,
///  so the amend built and landed a commit on a parent Git does not name.
///  The files-backend arm is the control that the difference is the ref store:
///  there the reader declines the database and Git's answer is used.
///  A linked worktree of the reftable repository is the third arm:
///  its replace ref reaches it through the common directory the reader declines.
#[test]
fn an_amend_of_a_replaced_commit_lands_with_gits_parents() {
    let fixture: Fixture = fixture("service-amend-replaced");
    let arms: [(&str, &[&str]); 2] = [("files", &[]), ("reftable", &["--ref-format=reftable"])];
    let mut repos: Vec<PathBuf> = Vec::new();
    for (arm, format) in arms {
        let repo: PathBuf = fixture.root.join(arm);
        std::fs::create_dir(repo.as_path()).expect("repository directory");
        let mut init: Vec<&str> = vec!["init", "--quiet", "--initial-branch=main"];
        init.extend_from_slice(format);
        git(&fixture, repo.as_path(), init.as_slice());
        std::fs::write(repo.join("a.txt"), "one\n").expect("a");
        git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=first"]);
        std::fs::write(repo.join("a.txt"), "two\n").expect("a");
        git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second"]);
        git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
        let head: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        // A replacement commit with the head's tree and no parent, so Git gives the amended commit no parent either.
        let tree: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        let replacement: String = query(&fixture, repo.as_path(), &["commit-tree", tree.as_str(), "-m", "replacement"]);
        git(&fixture, repo.as_path(), &["replace", head.as_str(), replacement.as_str()]);
        // Positive control: Git answers the replacement, which stands on no parent.
        assert_eq!(query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", head.as_str()]), head, "{arm}: Git reads the replacement's parents");
        repos.push(repo);
    }
    // A linked worktree of the reftable repository reads the same replace ref through its common directory,
    // and its own Git directory is served before the service scans the fixture for the repositories it serves.
    let linked: PathBuf = fixture.root.join("reftable-linked");
    git(&fixture, repos[1].as_path(), &["worktree", "add", "--quiet", linked.to_str().expect("UTF-8"), "-b", "topic"]);
    let name: String = format!("amend-replaced-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for (index, (arm, _format)) in arms.iter().enumerate() {
        let repo: &PathBuf = &repos[index];
        let before: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        std::fs::write(repo.join("a.txt"), "three\n").expect("a");
        let served: Output = through_service(&fixture, repo.as_path(), name.as_str(), &["commit", "--amend", "--message=amended", "--", "a.txt"]);
        assert!(served.status.success(), "{arm}: {}", String::from_utf8_lossy(&served.stderr));
        let landed: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        assert_ne!(landed, before, "{arm}: the branch moved to the amended commit");
        assert_eq!(
            query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", landed.as_str()]),
            landed,
            "{arm}: the amended commit stands on the parents Git reads, which are none"
        );
    }
    // The control for the linked arm: Git reads the replacement's parents there as well.
    let linked_before: String = query(&fixture, linked.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(
        query(&fixture, linked.as_path(), &["rev-list", "--parents", "--max-count=1", linked_before.as_str()]),
        linked_before,
        "linked: Git reads the replacement's parents in a linked worktree too"
    );
    std::fs::write(linked.join("a.txt"), "four\n").expect("a");
    let served: Output = through_service(&fixture, linked.as_path(), name.as_str(), &["commit", "--amend", "--message=amended", "--", "a.txt"]);
    assert!(served.status.success(), "linked: {}", String::from_utf8_lossy(&served.stderr));
    let landed: String = query(&fixture, linked.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(
        query(&fixture, linked.as_path(), &["rev-list", "--parents", "--max-count=1", landed.as_str()]),
        landed,
        "linked: the amended commit stands on the parents Git reads, which are none"
    );
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The same settings as a *service* is given them,
/// each with a value that moves what the service's Git reads.
/// Why:
///  A `files://` payload store names the whole ref store of the command,
/// and a store that holds only a branch and a
///  replacement is read by Git for `cat-file` and not for the branch;
///  a reftable store that holds no reference at all is the shape the service fails on,
/// which is what Git does under it.
const SETTINGS_ONLY_A_SERVICE_SEES: [(&str, &str); 4] = [
    ("GIT_REFERENCE_BACKEND", "reftable"),
    ("GIT_REPLACE_REF_BASE", "refs/probe/"),
    ("GIT_OBJECT_DIRECTORY", "emptystore"),
    ("GIT_ALTERNATE_OBJECT_DIRECTORIES", "emptystore"),
];

/// The value Git is given when a control needs a variable that moves nothing:
/// a name Git reads and the fixture's empty environment already honours.
const A_SETTING_THAT_MOVES_NOTHING: (&str, &str) = ("GIT_ATTR_NOSYSTEM", "1");

/// A repository of two commits with an edit staged for an amend,
/// and the store or the replacement base `variable` names written under `value`.
/// Why:
///  The setting has to move what Git answers for the base,
/// or the control proves nothing:
///  a replacement that stands on no parent makes an amended commit stand on none,
///  an empty object store makes Git find no base at all,
///  and a base moved into the alternate store is found only through it.
fn a_repository_a_setting_moves(fixture: &Fixture, repo: &Path, variable: &str, value: &str) {
    std::fs::create_dir(repo).expect("repository directory");
    git(fixture, repo, &["init", "--quiet", "--initial-branch=main"]);
    git(fixture, repo, &["commit", "--quiet", "--allow-empty", "--message=root"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(fixture, repo, &["add", "--", "a.txt"]);
    git(fixture, repo, &["commit", "--quiet", "--message=base"]);
    git(fixture, repo, &["config", "maintenance.autoDetach", "false"]);
    let base: String = query(fixture, repo, &["rev-parse", "HEAD"]);
    let tree: String = query(fixture, repo, &["rev-parse", "HEAD^{tree}"]);
    let replacement: String = query(fixture, repo, &["commit-tree", tree.as_str(), "-m", "replacement"]);
    match variable {
        // Git resolves a relative payload against the common directory, so the store stands beside `config`.
        "GIT_REFERENCE_BACKEND" if value.starts_with("files://") => {
            let store: PathBuf = repo.join(".git").join(value.trim_start_matches("files://"));
            std::fs::create_dir_all(store.join("refs/replace")).expect("store");
            std::fs::create_dir_all(store.join("refs/heads")).expect("the store's branches");
            // The branch the amend moves, so the store can answer for `HEAD` as a whole ref store does.
            std::fs::write(store.join("refs/heads/main"), format!("{base}\n")).expect("the branch in the store");
            std::fs::write(store.join("refs/replace").join(base.as_str()), format!("{replacement}\n")).expect("the replacement ref");
        }
        "GIT_REPLACE_REF_BASE" => {
            std::fs::create_dir_all(repo.join(".git").join(value)).expect("the named base");
            std::fs::write(repo.join(".git").join(value).join(base.as_str()), format!("{replacement}\n")).expect("the replacement ref");
        }
        // The alternate store holds the base commit alone, so only a read through it finds the base.
        "GIT_ALTERNATE_OBJECT_DIRECTORIES" => {
            let store: PathBuf = repo.join(value);
            std::fs::create_dir_all(store.join(&base[..2])).expect("the named store");
            std::fs::rename(repo.join(".git/objects").join(&base[..2]).join(&base[2..]), store.join(&base[..2]).join(&base[2..])).expect("the move");
        }
        _ => {
            std::fs::create_dir_all(repo.join(value)).expect("the named store");
        }
    }
    std::fs::write(repo.join("a.txt"), "two\n").expect("edit");
    git(fixture, repo, &["add", "--", "a.txt"]);
}

/// One real Git run in a fixture with `held` given to it,
/// bounded as every fixture Git run is.
fn git_holding(fixture: &Fixture, repo: &Path, held: &[(&str, &str)], arguments: &[&str]) -> Output {
    let bound: &str = super::support::NATIVE_GIT_BOUND_SECONDS;
    let mut command: Command = super::support::native_git_bound(fixture, bound);
    for (name, value) in held {
        command.env(name, value);
    }
    command.args(arguments);
    return super::support::native_git_output(fixture, &mut command, repo, bound);
}

/// What:
///  A setting given to the service and not to the client lands as Git reads it,
///  or leaves the branch alone where Git cannot read the base either.
/// Why:
///  The service's own Git children inherit its environment,
///  so a replacement base it holds makes the amend stand on the parents Git reads,
///  and a store it names is what the landing asks;
///  a service that answered from its own reading while Git read elsewhere would be the wrong answer.
///  The answer is kept right by two doors together:
/// `history_is_plain` lists the variables,
/// so the landing leaves the
///  parents to Git,
/// and the reader declines them,
/// so a landing that read in process asks Git too.
///  This control is therefore red only with both doors closed;
///  `a_graft_file_the_service_names_lands_as_git_reads_it` and
///  `a_ref_store_the_service_names_lands_as_git_reads_it` hold one door each.
#[test]
fn a_setting_only_the_service_sees_lands_as_git_reads_it() {
    for (variable, value) in SETTINGS_ONLY_A_SERVICE_SEES {
        let fixture: Fixture = fixture(format!("service-setting-{variable}").as_str());
        let repo: PathBuf = fixture.root.join("repo");
        a_repository_a_setting_moves(&fixture, repo.as_path(), variable, value);
        let base: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        // Git's own answers taken before the commit runs: whether the command could prepare this base at all, and what the
        // service's Git reads for it.
        let plain_read: Output = git_holding(&fixture, repo.as_path(), &[A_SETTING_THAT_MOVES_NOTHING], &["cat-file", "commit", base.as_str()]);
        let under_read: Output = git_holding(&fixture, repo.as_path(), &[(variable, value)], &["cat-file", "commit", base.as_str()]);
        {
            let mut environment = fixture.service_environment.lock().expect("the service environment");
            environment.push((OsString::from(variable), OsString::from(value)));
        }
        let service: FixtureService = start_fixture_service(&fixture);
        service.serve(&fixture, repo.join(".git").as_path());
        let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--amend", "--message=amended", "--", "a.txt"]);
        command.env("XDG_RUNTIME_DIR", &service.runtime);
        let served: Output = command.output().expect("the wrapped commit runs");
        let said: String = String::from_utf8_lossy(served.stderr.as_slice()).into_owned();
        let under_head: Output = git_holding(&fixture, repo.as_path(), &[(variable, value)], &["rev-parse", "HEAD"]);
        if plain_read.status.success() && under_read.status.success() && under_head.status.success() {
            assert!(served.status.success(), "{variable}: Git reads the base under the setting: {said}");
            let landed: String = String::from_utf8_lossy(under_head.stdout.as_slice()).trim().to_string();
            assert_ne!(landed, base, "{variable}: the branch moved to the amended commit");
            let parents: Vec<String> = String::from_utf8_lossy(under_read.stdout.as_slice())
                .lines()
                .filter_map(|line| { return line.strip_prefix("parent ").map(str::to_string); })
                .collect();
            let expected: String = std::iter::once(landed.clone()).chain(parents).collect::<Vec<String>>().join(" ");
            assert_eq!(
                query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", landed.as_str()]),
                expected,
                "{variable}: the amended commit stands on the parents Git reads under the setting"
            );
        } else {
            assert!(
                !served.status.success(),
                "{variable}: Git cannot read the base under the setting either (plain {:?}, under {:?}, head {:?}): {said}",
                plain_read.status.code(),
                under_read.status.code(),
                under_head.status.code()
            );
            assert_eq!(query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), base, "{variable}: nothing landed");
        }
        remove_settled(&fixture);
    }
}

/// The paths one environment row works in.
struct RowPaths {
    /// The repository the row commits in.
    repo: PathBuf,
    /// Its object store.
    objects: PathBuf,
    /// The directory the row's alternate reads are given.
    cache: PathBuf,
}

/// The paths of the row `fixture` and `repo` belong to.
fn row_paths(fixture: &Fixture, repo: &Path) -> RowPaths {
    return RowPaths { repo: repo.to_path_buf(), objects: repo.join(".git/objects"), cache: fixture.root.join("cache") };
}

/// Nothing held by the process this is asked of.
fn nothing_held(_paths: &RowPaths) -> Vec<(&'static str, String)> {
    return Vec::new();
}

/// The repository's own object store,
/// as Git reads it when told nothing.
fn the_object_store_in_use(paths: &RowPaths) -> Vec<(&'static str, String)> {
    return vec![("GIT_OBJECT_DIRECTORY", paths.objects.display().to_string())];
}

/// The same store,
/// spelled the way the worktree's own pathspecs would spell it.
fn the_object_store_in_use_spelled_relatively(_paths: &RowPaths) -> Vec<(&'static str, String)> {
    return vec![("GIT_OBJECT_DIRECTORY", String::from(".git/objects"))];
}

/// An empty directory beside the repository,
/// as a cache a CI job would name.
fn an_empty_cache_directory(paths: &RowPaths) -> Vec<(&'static str, String)> {
    return vec![("GIT_ALTERNATE_OBJECT_DIRECTORIES", paths.cache.display().to_string())];
}

/// The repository's own store as an alternate,
/// which adds no object at all.
fn the_object_store_as_an_alternate(paths: &RowPaths) -> Vec<(&'static str, String)> {
    return vec![("GIT_ALTERNATE_OBJECT_DIRECTORIES", paths.objects.display().to_string())];
}

/// A base of replacement refs with nothing under it.
fn a_replacement_base_with_nothing_under_it(_paths: &RowPaths) -> Vec<(&'static str, String)> {
    return vec![("GIT_REPLACE_REF_BASE", String::from("refs/probe/"))];
}

/// The reference store the repository already keeps,
/// named for the command.
fn the_ref_store_the_repository_already_keeps(_paths: &RowPaths) -> Vec<(&'static str, String)> {
    return vec![("GIT_REFERENCE_BACKEND", String::from("reftable"))];
}

/// A row that changes nothing before the commit.
fn nothing_to_prepare(_fixture: &Fixture, _paths: &RowPaths) {}

/// Move every loose object of the base history into the row's cache directory,
/// as a repository whose history only the alternate lets Git read.
fn move_the_history_into_the_cache(_fixture: &Fixture, paths: &RowPaths) {
    std::fs::create_dir_all(paths.cache.as_path()).expect("the cache directory");
    for entry in std::fs::read_dir(paths.objects.as_path()).expect("the object store").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let a_fan_out: bool = name.len() == 2 && name.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
        if a_fan_out {
            std::fs::rename(entry.path(), paths.cache.join(name.as_str())).expect("the move");
        }
    }
}

/// Put a replacement that stands on no parent under the base's name in the row's replacement base.
fn put_a_parentless_replacement_under_the_base(fixture: &Fixture, paths: &RowPaths) {
    let base: String = query(fixture, paths.repo.as_path(), &["rev-parse", "HEAD"]);
    let tree: String = query(fixture, paths.repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let replacement: String = query(fixture, paths.repo.as_path(), &["commit-tree", tree.as_str(), "-m", "replacement"]);
    std::fs::create_dir_all(paths.repo.join(".git/refs/probe")).expect("the named base");
    std::fs::write(paths.repo.join(".git/refs/probe").join(base.as_str()), format!("{replacement}\n")).expect("the replacement ref");
}

/// One row of the integration recheck's environment situations:
/// what the command's own environment holds,
/// what the service's holds,
/// what the row does first,
/// and whether its commit is an amend.
struct AnEnvironmentRow {
    /// What the row is called here.
    name: &'static str,
    /// Whether the repository keeps its references in reftable tables.
    reftable: bool,
    /// The variables the command's own environment is given.
    command: fn(&RowPaths) -> Vec<(&'static str, String)>,
    /// The variables the service's environment is given.
    service: fn(&RowPaths) -> Vec<(&'static str, String)>,
    /// What the row does to the repository before the commit.
    prepare: fn(&Fixture, &RowPaths),
    /// Whether the commit is an amend.
    amend: bool,
}

/// The rows of the integration recheck's environment situations in which `main` lands the right commit,
/// under the names it gives them (`r01` to `r32` of its `env-situations.ts`).
/// Why:
///  These are the rows the commit refusal this round added refused:
/// a value that names what the repository already
///  uses,
/// an empty cache directory,
/// a replacement base with nothing under it,
/// a reftable store naming what it is,
///  and an effective setting the service holds too.
fn rows_of_the_recheck() -> Vec<AnEnvironmentRow> {
    return vec![
        AnEnvironmentRow { name: "r01 the object store in use", reftable: false, command: the_object_store_in_use, service: nothing_held, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r02 the object store in use, held by the service too", reftable: false, command: the_object_store_in_use, service: the_object_store_in_use, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r03 the object store in use, spelled relatively", reftable: false, command: the_object_store_in_use_spelled_relatively, service: nothing_held, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r04 the object store in use, a reftable repository", reftable: true, command: the_object_store_in_use, service: nothing_held, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r10 an empty cache directory", reftable: false, command: an_empty_cache_directory, service: nothing_held, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r11 an empty cache directory, held by the service too", reftable: false, command: an_empty_cache_directory, service: an_empty_cache_directory, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r12 the object store as an alternate", reftable: false, command: the_object_store_as_an_alternate, service: nothing_held, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r13 the history in the alternate, held by the service too", reftable: false, command: an_empty_cache_directory, service: an_empty_cache_directory, prepare: move_the_history_into_the_cache, amend: false },
        AnEnvironmentRow { name: "r14 a parentless replacement under the base, held by the service too", reftable: false, command: a_replacement_base_with_nothing_under_it, service: a_replacement_base_with_nothing_under_it, prepare: put_a_parentless_replacement_under_the_base, amend: true },
        AnEnvironmentRow { name: "r21 a replacement base with nothing under it", reftable: false, command: a_replacement_base_with_nothing_under_it, service: nothing_held, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r22 a replacement base with nothing under it, held by the service too", reftable: false, command: a_replacement_base_with_nothing_under_it, service: a_replacement_base_with_nothing_under_it, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r31 a reftable store naming what it is", reftable: true, command: the_ref_store_the_repository_already_keeps, service: nothing_held, prepare: nothing_to_prepare, amend: false },
        AnEnvironmentRow { name: "r32 a reftable store naming what it is, held by the service too", reftable: true, command: the_ref_store_the_repository_already_keeps, service: the_ref_store_the_repository_already_keeps, prepare: nothing_to_prepare, amend: false },
    ];
}

/// What:
///  Every row of the recheck's environment situations in which `main` lands the right commit still lands it,
///  with the parents Git reads and a whole commit.
/// Why:
///  The commit refusal this round added refused these rows,
/// and the merge bar is that a branch may not make anything
///  `main` does well worse:
///  a value that names what the repository already uses,
/// an empty cache directory,
/// a replacement base with nothing under
///  it,
/// a reftable store naming what it is,
/// and an effective setting the service holds too are all cases `main` lands.
///  Each row has a fixture and a commit service of its own.
#[test]
fn a_setting_the_command_or_the_service_holds_still_lands() {
    for (index, row) in rows_of_the_recheck().into_iter().enumerate() {
        let fixture: Fixture = fixture(format!("environment-row-{index}").as_str());
        let repo: PathBuf = fixture.root.join("repo");
        std::fs::create_dir(repo.as_path()).expect("repository directory");
        let mut init: Vec<&str> = vec!["init", "--quiet", "--initial-branch=main"];
        if row.reftable {
            init.push("--ref-format=reftable");
        }
        git(&fixture, repo.as_path(), init.as_slice());
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=root"]);
        std::fs::write(repo.join("a.txt"), "one\n").expect("a");
        git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
        git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
        let paths: RowPaths = row_paths(&fixture, repo.as_path());
        std::fs::create_dir_all(paths.cache.as_path()).expect("the directory the row names");
        (row.prepare)(&fixture, &paths);
        let base: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let held_by_the_command: Vec<(&'static str, String)> = (row.command)(&paths);
        {
            let mut environment = fixture.service_environment.lock().expect("the service environment");
            for (name, value) in (row.service)(&paths) {
                environment.push((OsString::from(name), OsString::from(value)));
            }
        }
        let service: FixtureService = start_fixture_service(&fixture);
        service.serve(&fixture, repo.join(".git").as_path());
        std::fs::write(repo.join("a.txt"), "two\n").expect("edit");
        git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
        let arguments: &[&str] = match row.amend {
            true => &["commit", "--quiet", "--amend", "--message=amended", "--", "a.txt"],
            false => &["commit", "--quiet", "--message=under test", "--", "a.txt"],
        };
        let mut command: Command = wrapped_commit(&fixture, repo.as_path(), arguments);
        command.env("XDG_RUNTIME_DIR", &service.runtime);
        for (name, value) in &held_by_the_command {
            command.env(name, value);
        }
        let served: Output = command.output().expect("the wrapped commit runs");
        let said: String = String::from_utf8_lossy(served.stderr.as_slice()).into_owned();
        assert_eq!(served.status.code(), Some(0), "{}: this row lands on main: {said}", row.name);
        // What Git reads for the commit under the same environment, and what the landing made of it.
        let judging: Vec<(&str, &str)> = held_by_the_command.iter().map(|(name, value)| { return (*name, value.as_str()); }).collect();
        let landed: String = text_of(git_holding(&fixture, repo.as_path(), judging.as_slice(), &["rev-parse", "HEAD"]));
        assert_ne!(landed, base, "{}: the branch moved", row.name);
        let expected: String = match row.amend {
            true => landed.clone(),
            false => format!("{landed} {base}"),
        };
        assert_eq!(
            text_of(git_holding(&fixture, repo.as_path(), judging.as_slice(), &["rev-list", "--parents", "--max-count=1", landed.as_str()])),
            expected,
            "{}: the commit stands on the parents Git reads",
            row.name
        );
        let whole: Output = git_holding(&fixture, repo.as_path(), judging.as_slice(), &["fsck", "--no-progress"]);
        assert!(whole.status.success(), "{}: Git finds every object the commit names: {}", row.name, String::from_utf8_lossy(whole.stderr.as_slice()));
        assert_eq!(
            text_of(git_holding(&fixture, repo.as_path(), judging.as_slice(), &["status", "--porcelain"])),
            "",
            "{}: the worktree is clean",
            row.name
        );
        remove_settled(&fixture);
    }
}

/// The text of one Git answer,
/// without its trailing line feed.
fn text_of(output: Output) -> String {
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
}

/// What:
///  A graft file the service names is what Git reads,
/// and the amendment stands where Git reads it.
/// Why:
///  This is the landing's first door for a history Git may show differently:
/// `history_is_plain` lists
///  `GIT_GRAFT_FILE`,
/// so the parents are left to Git.
///  The repository's own `info/grafts` is empty here,
/// so the reader opens and the second door does not apply:
///  this control is red with `GIT_GRAFT_FILE` taken out of that list and nothing else.
#[test]
fn a_graft_file_the_service_names_lands_as_git_reads_it() {
    let fixture: Fixture = fixture("service-graft-file");
    let repo: PathBuf = fixture.root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(&fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=root"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    let base: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    // A graft file beside the repository, not the repository's own `info/grafts`: Git reads the base as standing on no parent.
    let graft: PathBuf = fixture.root.join("grafts");
    std::fs::write(graft.as_path(), format!("{base}\n")).expect("the graft file");
    let named: &str = graft.to_str().expect("a text path");
    let held: &[(&str, &str)] = &[("GIT_GRAFT_FILE", named)];
    assert_eq!(
        text_of(git_holding(&fixture, repo.as_path(), held, &["rev-list", "--parents", "--max-count=1", base.as_str()])),
        base,
        "the positive control: Git reads the graft file, and the base stands on no parent"
    );
    {
        let mut environment = fixture.service_environment.lock().expect("the service environment");
        environment.push((OsString::from("GIT_GRAFT_FILE"), OsString::from(named)));
    }
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    std::fs::write(repo.join("a.txt"), "two\n").expect("edit");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--amend", "--message=amended", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &service.runtime);
    let served: Output = command.output().expect("the wrapped commit runs");
    let said: String = String::from_utf8_lossy(served.stderr.as_slice()).into_owned();
    assert!(served.status.success(), "the amendment lands: {said}");
    let landed: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    assert_ne!(landed, base, "the branch moved to the amended commit");
    assert_eq!(
        query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", landed.as_str()]),
        landed,
        "the amended commit stands on the parents Git reads under the graft file, which are none"
    );
    remove_settled(&fixture);
}

/// What:
///  A replace ref keeps the amendment on the parents Git reads,
///  with no variable of the landing's history list set anywhere.
/// Why:
///  This is the landing's second door alone:
/// the reader declines a repository holding a replace ref,
///  so the base's parents are asked of Git,
/// and the fixture closes the first door by setting nothing that
///  `history_is_plain` lists (a `refs/replace` entry is not one of its variables).
///  This control is red with the replace refs taken out of the reader's decline and nothing else.
#[test]
fn a_replace_ref_keeps_the_amendment_on_the_parents_git_reads() {
    let fixture: Fixture = fixture("service-replace-ref-door");
    let repo: PathBuf = fixture.root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(&fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=root"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    let base: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let tree: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let replacement: String = query(&fixture, repo.as_path(), &["commit-tree", tree.as_str(), "-m", "replacement"]);
    // A replacement that stands on no parent, under `refs/replace`, where Git reads it and the reader declines the store.
    git(&fixture, repo.as_path(), &["replace", base.as_str(), replacement.as_str()]);
    assert_eq!(
        query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", base.as_str()]),
        base,
        "the positive control: Git reads the replacement, which stands on no parent"
    );
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    std::fs::write(repo.join("a.txt"), "two\n").expect("edit");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--amend", "--message=amended", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &service.runtime);
    let served: Output = command.output().expect("the wrapped commit runs");
    let said: String = String::from_utf8_lossy(served.stderr.as_slice()).into_owned();
    assert!(served.status.success(), "the amendment lands: {said}");
    let landed: String = query(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    assert_ne!(landed, base, "the branch moved to the amended commit");
    assert_eq!(
        query(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", landed.as_str()]),
        landed,
        "the amended commit stands on the parents Git reads under the replace ref, which are none"
    );
    remove_settled(&fixture);
}
