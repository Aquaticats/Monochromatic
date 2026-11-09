//! What:
//!  The scratch directory of a commit in preparation,
//! whose life the service owns while the client's connection is open.
//! Why:
//!  A client that is killed without warning cannot remove its scratch directory,
//! and nothing afterwards can tell a dead client
//!  from a slow one without a process id or a lock.
//! The open connection is the signal:
//! the service removes the directory when
//!  the connection closes,
//! by whatever means,
//! and removes nothing a client merely names.

use super::support::{Fixture, fixture};
use git_policy_cli::service_client::{Activation, ClientConfig, ScratchWatch, watch_scratch};
use git_policy_cli::service_protocol::{ClientMessage, SCRATCH_PREFIX, encode_client};
use git_policy_cli::service_server::Bind;
use std::io::Write;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};


/// The longest a control waits for anything:
/// far beyond what a healthy run needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long a control waits before it takes a directory that stays to be one the service left alone.
const SETTLE: Duration = Duration::from_millis(1500);

/// A service process that is stopped when the control ends,
/// whether it passed or panicked.
struct Held(Child);

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

fn start(fixture: &Fixture, name: &str) -> Held {
    let child: Child = super::support::service_command()
        .args(["--journal"])
        .arg(fixture.root.join("journal"))
        .args(["--abstract", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the service starts");
    let began: Instant = Instant::now();
    while !SocketAddr::from_abstract_name(name.as_bytes()).and_then(|socket| { return UnixStream::connect_addr(&socket); }).is_ok() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return Held(child);
}

fn config(name: &str) -> ClientConfig {
    return ClientConfig { address: Bind::Abstract(name.to_string()), activation: Activation::Managed, start_wait: PATIENCE, heartbeat_silence: PATIENCE, start_watch: None, keep_fresh: None, answer_ceiling: git_policy_cli::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: git_policy_cli::wait_ceilings::WITHDRAW_CEILING, extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: git_policy_cli::wait_ceilings::START_CEILING, units_record_dir: None };
}

/// A repository's Git directory with a scratch directory in it,
/// holding a file,
/// made where a release before this one made it.
fn scratch_in(root: &Path, identity: &str) -> (PathBuf, PathBuf) {
    let git_dir: PathBuf = root.join("repo/.git");
    let scratch: PathBuf = git_dir.join(format!("{SCRATCH_PREFIX}{identity}"));
    std::fs::create_dir_all(&scratch).expect("scratch directory");
    std::fs::write(scratch.join("commit.index"), b"index").expect("file in the scratch directory");
    return (git_dir, scratch);
}

/// A repository's Git directory with a scratch directory in it,
/// holding a file,
/// made where this release makes it:
/// inside the repository scratch directory the sweep covers.
fn scratch_where_this_release_makes_it(root: &Path, identity: &str) -> (PathBuf, PathBuf) {
    let git_dir: PathBuf = root.join("repo/.git");
    let scratch: PathBuf = git_dir
        .join("monochromatic-git-policy-cli-scratch")
        .join(format!("{SCRATCH_PREFIX}{identity}"));
    std::fs::create_dir_all(&scratch).expect("scratch directory");
    std::fs::write(scratch.join("commit.index"), b"index").expect("file in the scratch directory");
    return (git_dir, scratch);
}

fn wait_until_gone(path: &Path) -> bool {
    let began: Instant = Instant::now();
    while path.exists() {
        if began.elapsed() >= PATIENCE {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    return true;
}

/// The directory stays while its holder is connected and goes when the holder lets go.
#[test]
fn a_scratch_directory_goes_with_the_connection_that_holds_it() {
    let fixture: Fixture = fixture("service-scratch");
    let name: String = format!("service-scratch-{}", std::process::id());
    let _service: Held = start(&fixture, name.as_str());
    let (git_dir, scratch) = scratch_in(fixture.root.as_path(), "0a1b2c3d-1");
    let watch: ScratchWatch = watch_scratch(&config(name.as_str()), git_dir.as_path(), scratch.as_path()).expect("the service takes the directory");
    std::thread::sleep(SETTLE);
    assert!(scratch.exists(), "the directory stays while its holder is connected");
    drop(watch);
    assert!(wait_until_gone(scratch.as_path()), "the directory is removed once the holder is gone");
    assert!(git_dir.exists(), "only the scratch directory goes");
    super::commit::remove_settled(&fixture);
}

/// A scratch directory in the repository scratch directory goes with the connection that holds it,
/// which is where this release makes it.
#[test]
fn a_scratch_directory_in_the_repository_scratch_directory_goes_with_its_connection() {
    let fixture: Fixture = fixture("service-scratch-covered");
    let name: String = format!("service-scratch-covered-{}", std::process::id());
    let _service: Held = start(&fixture, name.as_str());
    let (git_dir, scratch) = scratch_where_this_release_makes_it(fixture.root.as_path(), "0a1b2c3d-2");
    let watch: ScratchWatch = watch_scratch(&config(name.as_str()), git_dir.as_path(), scratch.as_path()).expect("the service takes the directory");
    std::thread::sleep(SETTLE);
    assert!(scratch.exists(), "the directory stays while its holder is connected");
    drop(watch);
    assert!(wait_until_gone(scratch.as_path()), "the directory is removed once the holder is gone");
    assert!(git_dir.join("monochromatic-git-policy-cli-scratch").exists(), "only the scratch directory goes");
    super::commit::remove_settled(&fixture);
}

/// A client can only hand over a scratch directory of the wrapper's own name inside the repository:
/// a path anywhere else,
/// or
///  of any other name,
/// is left alone when the connection closes.
#[test]
fn nothing_but_a_scratch_directory_of_the_repository_is_ever_removed() {
    let fixture: Fixture = fixture("service-scratch-hostile");
    let name: String = format!("service-scratch-hostile-{}", std::process::id());
    let _service: Held = start(&fixture, name.as_str());
    let (git_dir, _scratch) = scratch_in(fixture.root.as_path(), "0a1b2c3d-3");
    let precious: PathBuf = fixture.root.join("precious");
    std::fs::create_dir_all(&precious).expect("precious");
    let elsewhere: PathBuf = fixture.root.join(format!("{SCRATCH_PREFIX}elsewhere"));
    std::fs::create_dir_all(&elsewhere).expect("outside the repository");
    let notes: PathBuf = git_dir.join("notes");
    std::fs::create_dir_all(&notes).expect("notes");
    for path in [&precious, &elsewhere, &notes] {
        let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("address");
        let mut stream: UnixStream = UnixStream::connect_addr(&socket).expect("connect");
        let line: String = encode_client(&ClientMessage::Scratch { git_dir: git_dir.clone(), path: path.clone() });
        stream.write_all(format!("{line}\n").as_bytes()).expect("send");
        drop(stream);
    }
    std::thread::sleep(SETTLE);
    for path in [&precious, &elsewhere, &notes] {
        assert!(path.exists(), "{} was removed", path.display());
    }
    super::commit::remove_settled(&fixture);
}

/// A service of another release takes nothing:
/// the client learns that and keeps the directory's removal to itself.
#[test]
fn no_service_means_no_watch() {
    let fixture: Fixture = fixture("service-scratch-none");
    let (git_dir, scratch) = scratch_in(fixture.root.as_path(), "0a1b2c3d-4");
    let nobody: String = format!("service-scratch-nobody-{}", std::process::id());
    assert!(watch_scratch(&config(nobody.as_str()), git_dir.as_path(), scratch.as_path()).is_none());
    assert!(scratch.exists());
    super::commit::remove_settled(&fixture);
}

/// A service that is frozen (a busy host,
/// a stopped process) still takes the directory:
/// the announcement is written at once
/// and read when the service runs again,
/// so a client killed while the service is frozen loses its directory too.
#[test]
fn a_frozen_service_takes_the_directory_when_it_runs_again() {
    let fixture: Fixture = fixture("service-scratch-frozen");
    let name: String = format!("service-scratch-frozen-{}", std::process::id());
    let service: Held = start(&fixture, name.as_str());
    let (git_dir, scratch) = scratch_in(fixture.root.as_path(), "0a1b2c3d-5");
    let pid: i32 = i32::try_from(service.0.id()).expect("a process number");
    // SAFETY: `kill` takes a process number and a signal number and touches no memory.
    assert_eq!(unsafe { libc::kill(pid, libc::SIGSTOP) }, 0, "the service freezes");
    // A client that waited for the frozen service to greet it would sit out this wait, so the announcement is the proof that it did not.
    let quick = ClientConfig { start_wait: Duration::from_secs(3600), ..config(name.as_str()) };
    let offered: Option<ScratchWatch> = watch_scratch(&quick, git_dir.as_path(), scratch.as_path());
    let watch: ScratchWatch = offered.expect("the announcement is written without waiting for a greeting");
    // The client dies while the service is frozen.
    drop(watch);
    // SAFETY: as above.
    assert_eq!(unsafe { libc::kill(pid, libc::SIGCONT) }, 0, "the service runs again");
    assert!(wait_until_gone(scratch.as_path()), "the directory is removed once the service reads the announcement");
    super::commit::remove_settled(&fixture);
}
