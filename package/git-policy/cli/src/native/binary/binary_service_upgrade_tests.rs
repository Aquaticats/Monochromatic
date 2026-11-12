//! What:
//!  The upgrade from the 0.5.4 release,
//! run with that release's own service program:
//! the single per-user service of protocol
//!  4 that sends no release in its greeting,
//! still running from a program path the upgrade deleted,
//! meets the wrapper of this
//!  build,
//! which gives each worktree its own service.
//! Why:
//!  Nothing but the real 0.5.4 program shows what its reader accepts.
//! This build speaks a newer protocol,
//! and a request to
//!  retire that carries only this build's version is refused by a reader that wants its own,
//! which would leave the old
//!  service in place for good.
//! The service manager is the stand-in bus of
//!  `binary_service_provision_tests.rs`,
//! so no real unit and no real runtime directory is touched.

use super::commit::git_text;
use super::service_commit::Running;
use super::service_provision::{PATIENCE, activation, commit, fake_bus_many, finish, finished_batches, prepared, subject};
use super::support::git;
use git_policy_cli::service_instance::LEGACY_SOCKET_RELATIVE;
use git_policy_cli::service_provision::{LEGACY_SERVICE_UNIT_NAME, LEGACY_SOCKET_UNIT_NAME};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The variable naming the 0.5.4 `cli-git-service` program,
/// copied out of the toolchain directory before the install that
///  deletes it.
const OLD_SERVICE_VARIABLE: &str = "CLI_GIT_OLD_SERVICE";

/// Starts the 0.5.4 service program on the socket the wrapper looks for,
/// as the manager's socket activation left it.
fn start_old_service(program: &Path, root: &Path, runtime: &Path) -> Child {
    let socket: PathBuf = runtime.join(LEGACY_SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    // Owned: its idle exit is hours away, so a control that fails before the wrapper retires it would leave it running.
    let child: Child = super::support::spawn_owned(
        Command::new(program)
            .args(["--journal", root.join("journal").to_str().expect("UTF-8"), "--path", socket.to_str().expect("UTF-8")])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", root.join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit()),
    )
    .expect("the 0.5.4 service starts");
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&socket).is_err() {
        assert!(began.elapsed() < PATIENCE, "the 0.5.4 service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return child;
}

/// Waits for the process to exit by itself for at most `limit`.
fn exited_within(child: &mut Child, limit: Duration) -> Option<std::process::ExitStatus> {
    let began: Instant = Instant::now();
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

/// A 0.5.4 service is running from a program the upgrade deleted,
/// on the one socket every worktree shared,
/// and its two units are
///  installed.
/// The next signed commit asks the old service to retire (which that service accepts although this build speaks a
///  newer protocol),
/// has the manager stop its two units and reload,
/// removes them,
/// and lands on the worktree's own service,
///  signed.
/// The old service exits by itself with success,
/// and the worktree's service is still serving at the end.
#[test]
#[ignore = "needs CLI_GIT_OLD_SERVICE, the 0.5.4 cli-git-service program"]
fn a_signed_commit_replaces_a_running_0_5_4_service_whose_program_was_deleted() {
    let program: PathBuf = PathBuf::from(std::env::var_os(OLD_SERVICE_VARIABLE).expect("CLI_GIT_OLD_SERVICE names the 0.5.4 service program"));
    assert!(program.is_file(), "{} is not a file", program.display());
    let (fixture, repo, runtime) = prepared("upgrade-054");
    let key: PathBuf = fixture.root.join("signing-key");
    let made = Command::new("ssh-keygen").args(["-q", "-t", "ed25519", "-N", "", "-C", "fixture", "-f"]).arg(&key).output().expect("ssh-keygen starts");
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let public: String = std::fs::read_to_string(key.with_extension("pub")).expect("public key");
    let allowed: PathBuf = fixture.root.join("allowed-signers");
    std::fs::write(&allowed, format!("fixture@example.invalid {}", public.trim())).expect("allowed signers");
    for (setting, value) in [
        ("gpg.format", "ssh"),
        ("user.signingkey", key.to_str().expect("UTF-8")),
        ("gpg.ssh.allowedsignersfile", allowed.to_str().expect("UTF-8")),
        ("commit.gpgsign", "true"),
    ] {
        git(&fixture, &repo, &["config", setting, value]);
    }
    // The two units the 0.5.4 release installed.
    let units: PathBuf = runtime.join("systemd/user");
    std::fs::create_dir_all(&units).expect("unit directory");
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        std::fs::write(units.join(name), "# Written by the git wrapper for its commit service (cli-git-unit)\n[Unit]\n").expect("old unit");
    }
    let mut old: Child = start_old_service(program.as_path(), fixture.root.as_path(), runtime.as_path());
    let keep: Arc<Mutex<Vec<Running>>> = Arc::new(Mutex::new(Vec::new()));
    let bus = fake_bus_many(runtime.as_path(), false, 1, activation(fixture.root.clone(), runtime.clone(), Arc::clone(&keep)));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    let status = exited_within(&mut old, PATIENCE / 6).expect("the 0.5.4 service retired when the wrapper asked it to");
    assert!(status.success(), "the old service retired by itself: {status:?}");
    assert_eq!(subject(&fixture, &repo), "subject");
    assert_eq!(finished_batches(&fixture), 1, "the worktree's own service landed it");
    assert!(git_text(&fixture, &repo, &["cat-file", "commit", "HEAD"]).contains("\ngpgsig -----BEGIN SSH SIGNATURE-----"), "the commit is signed");
    git(&fixture, &repo, &["verify-commit", "HEAD"]);
    // The socket is stopped, the wrapper asks for the service's state until the stand-in reports it stopped (it left when it was
    //  asked to retire; the stop of the service goes after it left), then it asks where a held socket unit listens before it starts
    //  anything, and then for each unit's state, which the stand-in reports stopped on the first ask.
    assert_eq!(
        bus.join().expect("bus"),
        vec![vec![
            "Hello",
            "StopUnit",
            "GetUnit",
            "Get",
            "StopUnit",
            "GetUnit",
            "Get",
            "GetUnit",
            "Get",
            "GetUnit",
            "Get",
            "Reload",
            "StartUnit",
        ]]
    );
    assert!(!units.join(LEGACY_SOCKET_UNIT_NAME).exists() && !units.join(LEGACY_SERVICE_UNIT_NAME).exists(), "the old units are gone");
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}
