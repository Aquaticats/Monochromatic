//! What:
//!  The built executable's service mode (`cli-git-native cli-git service run`): the invariants that the service is a mode of
//!  the wrapper and not a second program.
//! Why:
//!  The release ships one executable, so a person's `git cli-git service run` reaches the same code the service manager starts.
//!  A start with no socket from the manager and no named address must be refused and leave no server and no directory; a
//!  `--git` that is the wrapper itself must be refused (owner: "We should never invoke ourselves recursively."); the words
//!  count only first, so a global option of Git in front never selects the mode.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(run(wrapper, ['cli-git', 'service', 'run', '--journal', dir]).code).toBe(3); expect(exists(dir)).toBe(false);
//! ```

use super::support::{Fixture, fixture, hermetic, remove, repository, service_command, service_executable, spawn_owned, wrapped};
use std::path::{Path, PathBuf};
use std::process::{Child, Output, Stdio};
use std::time::{Duration, Instant};

/// The longest a control waits for a start to end: far beyond what a refusal needs, so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long a control waits before it takes an absent file to stay absent.
const SETTLE: Duration = Duration::from_millis(500);

/// Waits for `child` to end within `PATIENCE`, and gives its output; a child still running then is killed and the control fails.
fn finished(mut child: Child) -> Output {
    let began: Instant = Instant::now();
    loop {
        if child.try_wait().expect("the child's state").is_some() {
            return child.wait_with_output().expect("the child's output");
        }
        if began.elapsed() >= PATIENCE {
            let _killed = child.kill();
            let _waited = child.wait();
            panic!("the start was still running after {PATIENCE:?}: a refusal ends at once");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A person types `git cli-git service run`: the manager passed no socket and no address is named, so the start is refused with
///  the reason and the way forward, exits without listening, and creates nothing, not even the journal directory.
#[test]
fn a_start_with_no_socket_and_no_address_is_refused_and_creates_nothing() {
    let fixture: Fixture = fixture("service-mode-no-socket");
    let journal: PathBuf = fixture.root.join("journal-not-made");
    for (label, listen_pid, listen_fds) in [("no variables", None, None), ("a socket for another process", Some("1"), Some("1")), ("no descriptor", Some(std::process::id().to_string()), Some(String::from("0")))] {
        let mut command = wrapped(&fixture);
        command.args(["cli-git", "service", "run", "--journal"]).arg(&journal);
        if let (Some(pid), Some(fds)) = (listen_pid.as_ref(), listen_fds.as_ref()) {
            command.env("LISTEN_PID", pid).env("LISTEN_FDS", fds);
        }
        let output: Output = finished(command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the wrapper starts"));
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(output.status.code(), Some(git_policy_cli::service_main::EXIT_NOT_LISTENING), "{label}: {stderr}");
        assert!(stderr.contains("started by the service manager") && stderr.contains("--path <socket>"), "{label}: {stderr}");
        assert!(!journal.exists(), "{label}: a refused start left a directory");
    }
    // Also without any option: the refusal names the manager, not a missing option.
    let bare: Output = finished(wrapped(&fixture).args(["cli-git", "service", "run"]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the wrapper starts"));
    assert!(String::from_utf8_lossy(&bare.stderr).contains("started by the service manager"), "{}", String::from_utf8_lossy(&bare.stderr));
    remove(&fixture);
}

/// The words select the mode only first: behind a global option of Git they are an unknown management command, which the
///  wrapper refuses with its usage, and no server starts and no socket appears.
#[test]
fn the_words_behind_a_global_option_start_no_server() {
    let fixture: Fixture = fixture("service-mode-behind-option");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    let socket: PathBuf = fixture.root.join("stray.sock");
    let output: Output = finished(
        wrapped(&fixture)
            .arg("-C")
            .arg(&repo)
            .args(["cli-git", "service", "run", "--journal"])
            .arg(fixture.root.join("journal-not-made"))
            .arg("--path")
            .arg(&socket)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the wrapper starts"),
    );
    assert!(!output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage: git cli-git"), "{}", String::from_utf8_lossy(&output.stderr));
    std::thread::sleep(SETTLE);
    assert!(!socket.exists(), "a socket appeared");
    assert!(!fixture.root.join("journal-not-made").exists());
    remove(&fixture);
}

/// Starts the service mode with `--git` set to `git`, on a path socket of the fixture, and gives its end.
fn run_with_git(fixture: &Fixture, git: &Path, label: &str) -> (Output, PathBuf) {
    let socket: PathBuf = fixture.root.join(format!("{label}.sock"));
    let output: Output = finished(
        service_command()
            .arg("--journal")
            .arg(fixture.root.join(format!("journal-{label}")))
            .arg("--git")
            .arg(git)
            .arg("--path")
            .arg(&socket)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the service mode starts"),
    );
    return (output, socket);
}

/// A `--git` that is the wrapper (the built file, the fixture's hard link of it, the `git` link to it, or a byte copy) is
///  refused at once with the way out, and nothing listens.
#[test]
fn a_git_that_is_the_wrapper_is_refused() {
    let fixture: Fixture = fixture("service-mode-git-is-wrapper");
    let copy: PathBuf = fixture.root.join("copy-of-the-wrapper");
    std::fs::copy(service_executable(), &copy).expect("a byte copy of the executable");
    for (label, git) in [
        ("built", PathBuf::from(service_executable())),
        ("linked", fixture.wrapper.clone()),
        ("git-link", fixture.root.join("bin/git")),
        ("copy", copy),
    ] {
        let (output, socket) = run_with_git(&fixture, git.as_path(), label);
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(output.status.code(), Some(git_policy_cli::service_main::EXIT_REFUSED), "{label}: {stderr}");
        assert!(stderr.contains("would start cli-git again") && stderr.contains("/usr/bin/git"), "{label}: {stderr}");
        assert!(!socket.exists(), "{label}: a refused start listened");
    }
    remove(&fixture);
}

/// The control that the refusal can fail: the same start with the real Git listens, answers a connection and ends when told.
#[test]
fn the_same_start_with_the_real_git_listens() {
    let fixture: Fixture = fixture("service-mode-real-git");
    let socket: PathBuf = fixture.root.join("real.sock");
    let mut command = service_command();
    hermetic(&mut command, &fixture, std::ffi::OsStr::new("/usr/bin:/bin"));
    let mut child: Child = spawn_owned(
        command
            .arg("--journal")
            .arg(fixture.root.join("journal-real"))
            .args(["--git", "/usr/bin/git", "--path"])
            .arg(&socket)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null()),
    )
    .expect("the service mode starts");
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&socket).is_err() {
        assert!(child.try_wait().expect("state").is_none(), "the service ended before it listened");
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    let _killed = child.kill();
    let _waited = child.wait();
    remove(&fixture);
}
