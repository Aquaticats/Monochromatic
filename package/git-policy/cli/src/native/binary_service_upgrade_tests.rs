//! What:
//!  The upgrade from the 0.5.4 release, run with that release's own service program: a service of protocol 4 that sends no
//!  release in its greeting, still running from a program path the upgrade deleted, meets the wrapper of this build.
//! Why:
//!  Nothing but the real 0.5.4 program shows what its reader accepts. This build speaks a newer protocol, and a request to
//!  retire that carries only this build's version is refused by a reader that wants its own, which would leave the old
//!  service in place for good and fail every commit after the upgrade. The service manager is the stand-in bus of
//!  `binary_service_provision_tests.rs`, so no real unit and no real runtime directory is touched.

use super::commit::git_text;
use super::service_commit::start_in;
use super::service_provision::{PATIENCE, commit, fake_bus_many, finish, finished_batches, prepared, subject};
use super::support::git;
use git_policy_cli::service_messages::UPDATING_NOTICE;
use git_policy_cli::service_provision::{ProvisionHost, SERVICE_UNIT_NAME, runtime_unit_dir, unit_texts};
use git_policy_cli::service_route::SOCKET_RELATIVE;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The variable naming the 0.5.4 `cli-git-service` program, copied out of the toolchain directory before the install that
///  deletes it.
const OLD_SERVICE_VARIABLE: &str = "CLI_GIT_OLD_SERVICE";

/// Where the 0.5.4 program was installed, and so what the unit it left behind names once the next release removes it.
const REMOVED_PROGRAM: &str = "/home/a/.local/share/monochromatic-dev/toolchain/cargo/monochromatic-git-policy-cli/0.5.4/bin/cli-git-service";

/// Starts the 0.5.4 service program on the socket the wrapper looks for, as the manager's socket activation left it.
fn start_old_service(program: &Path, root: &Path, runtime: &Path) -> Child {
    let socket: PathBuf = runtime.join(SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let child: Child = Command::new(program)
        .args(["--journal", root.join("journal").to_str().expect("UTF-8"), "--path", socket.to_str().expect("UTF-8")])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
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

/// A 0.5.4 service is running from a program the upgrade deleted, and its unit names that program. The next signed commit
///  rewrites the unit, has the manager clear the failed state, asks the old service to retire (which that service accepts
///  although this build speaks a newer protocol), and lands on the current release's service, signed. The old service
///  exits by itself with success, the current one is still serving at the end, and the person is told only of the update.
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
    // The units the 0.5.4 release installed, naming the program that the upgrade removed.
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: runtime.clone(),
        user_unit_dir: None,
        bus: runtime.join("bus"),
        service_binary: PathBuf::from(REMOVED_PROGRAM),
        git: PathBuf::from("/usr/bin/git"),
    };
    let units: PathBuf = runtime_unit_dir(runtime.as_path());
    std::fs::create_dir_all(&units).expect("unit directory");
    for (name, text) in unit_texts(&host) {
        std::fs::write(units.join(name), text).expect("old unit");
    }
    let mut old: Child = start_old_service(program.as_path(), fixture.root.as_path(), runtime.as_path());
    // The manager starts the next service on the socket once the old one has gone; the test plays that part.
    let replacement = std::thread::spawn({
        let root: PathBuf = fixture.root.clone();
        let socket_directory: PathBuf = runtime.clone();
        move || {
            let Some(status) = exited_within(&mut old, PATIENCE / 6) else {
                let _killed = old.kill();
                let _waited = old.wait();
                return (None, None);
            };
            std::fs::remove_file(socket_directory.join(SOCKET_RELATIVE)).expect("the old socket goes");
            return (Some(status), Some(start_in(root.as_path(), socket_directory.as_path())));
        }
    });
    let bus = fake_bus_many(runtime.as_path(), false, 1, Box::new(|| {}));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    let (exit, started) = replacement.join().expect("replacement");
    let status = exit.expect("the 0.5.4 service retired when the wrapper asked it to");
    assert!(output.status.success(), "{stderr}");
    assert!(status.success(), "the old service retired by itself: {status:?}");
    let mut current = started.expect("the current service started");
    assert!(current.exited_within(Duration::from_millis(100)).is_none(), "the current service is serving at the end");
    assert!(stderr.contains(UPDATING_NOTICE), "{stderr}");
    assert_eq!(subject(&fixture, &repo), "subject");
    assert_eq!(finished_batches(&fixture), 1, "the current service landed it");
    assert!(git_text(&fixture, &repo, &["cat-file", "commit", "HEAD"]).contains("\ngpgsig -----BEGIN SSH SIGNATURE-----"), "the commit is signed");
    git(&fixture, &repo, &["verify-commit", "HEAD"]);
    // The manager was told to reload the rewritten unit and to clear the failed state of both units before it started the socket.
    assert_eq!(bus.join().expect("bus"), vec![vec!["Hello", "Reload", "ResetFailedUnit", "ResetFailedUnit", "StartUnit"]]);
    let unit: String = std::fs::read_to_string(units.join(SERVICE_UNIT_NAME)).expect("service unit");
    assert!(!unit.contains(REMOVED_PROGRAM), "the unit names the program that exists: {unit}");
    drop(current);
    finish(&fixture, runtime.as_path());
}
