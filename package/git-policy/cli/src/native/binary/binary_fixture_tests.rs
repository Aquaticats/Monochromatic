//! Binary-level controls for the fixtures themselves:
//! where they live and that they go away however their test ends
//! (issue 1809).

use super::support::{
    FIXTURE_ROOT_VARIABLE, Fixture, FixtureService, ServiceLaunch, fixture, missing_fixture_root_report, give_service_environment, remove_patiently, repository, service_runtime,
    service_is_serving, settle_fixture, spawn_owned, start_fixture_service,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What the report of a vanished fixture root says:
/// that the root does not exist,
/// and what its parent and its grandparent hold (or that they are gone too),
/// so the next failure of the kind names the folder that went.
#[test]
fn the_report_of_a_vanished_fixture_root_names_it_and_lists_its_parent_and_grandparent() {
    let present: Fixture = fixture("missing-root-report");
    let grandparent: PathBuf = present.root.join("grand");
    let parent: PathBuf = grandparent.join("parent");
    std::fs::create_dir_all(&grandparent).expect("grandparent");
    std::fs::write(grandparent.join("marker-file"), "x").expect("marker");
    let root: PathBuf = parent.join("root");
    let report: String = missing_fixture_root_report(root.as_path());
    assert!(report.contains(format!("the fixture root {} does not exist", root.display()).as_str()), "{report}");
    assert!(report.contains(format!("its parent {} does not exist", parent.display()).as_str()), "{report}");
    assert!(report.contains(format!("its grandparent {} holds: marker-file", grandparent.display()).as_str()), "{report}");
    std::fs::create_dir_all(&root).expect("root");
    let existing: String = missing_fixture_root_report(root.as_path());
    assert!(existing.contains(format!("the fixture root {} exists", root.display()).as_str()), "{existing}");
}

/// A fixture whose test panics after creating it leaves nothing behind.
#[test]
fn a_fixture_is_removed_when_its_test_panics() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    let handle = std::thread::spawn(move || {
        let created: Fixture = fixture("drop-on-panic");
        std::fs::write(created.root.join("file"), "content").expect("file");
        sender.send(created.root.clone()).expect("send the root");
        panic!("the test fails before it removes its fixture");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let root: PathBuf = receiver.recv().expect("the root");
    assert!(!root.exists(), "{} was left behind", root.display());
}

/// How many seconds the helper of the control below would sleep if nothing ended it:
/// far longer than the control waits.
const HELPER_SECONDS: &str = "600";

/// How long the control below waits for the helper to be gone,
/// so a missing guard fails it and never holds it.
const HELPER_GONE_PATIENCE: Duration = Duration::from_secs(20);

/// How often the control below looks at the helper.
const HELPER_LOOK: Duration = Duration::from_millis(50);

/// What:
///  A helper started with `spawn_owned` by a test that then panics is gone when the test thread ends.
/// Why:
///   `Child` does not kill on drop,
/// so a service or a shell loop that a failing control never reached the end of ran on after it
///  (87 minutes,
/// for the loop of a unit control).
/// The thread's list of helper groups is killed when the thread ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pid = await startThatThrows(() => { const child = spawnOwned('sleep', ['600']); throw new Error('fails'); });
/// await waitUntil(() => !isRunning(pid));
/// ```
#[test]
fn a_helper_started_by_a_panicking_test_is_gone_when_the_test_ends() {
    let (sender, receiver) = std::sync::mpsc::channel::<u32>();
    let handle = std::thread::spawn(move || {
        let mut child = spawn_owned(Command::new("sleep").arg(HELPER_SECONDS).stdin(Stdio::null())).expect("the helper starts");
        sender.send(child.id()).expect("send the pid");
        // A look that does not wait: the helper is still running when the test fails, as in the leak this control is for.
        let _still_running = child.try_wait();
        panic!("the test fails before it ends the helper");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let pid: u32 = receiver.recv().expect("the pid");
    let number: i32 = i32::try_from(pid).expect("a process ID fits");
    let began: Instant = Instant::now();
    while super::fault_run::is_running(number) && began.elapsed() < HELPER_GONE_PATIENCE {
        std::thread::sleep(HELPER_LOOK);
    }
    let left: bool = super::fault_run::is_running(number);
    if left {
        // Tidy up the leak this control exists to catch, so a red run leaves nothing behind either.
        // SAFETY: `kill` takes two integers and touches no memory of this process.
        unsafe { libc::kill(number, libc::SIGKILL) };
    }
    assert!(!left, "the helper {pid} outlived the test that started it");
}

/// A fixture removed explicitly and then dropped does not fail the drop.
#[test]
fn a_fixture_removed_explicitly_is_dropped_quietly() {
    let created: Fixture = fixture("drop-after-remove");
    let root: PathBuf = created.root.clone();
    super::support::remove(&created);
    assert!(!root.exists());
    drop(created);
}

/// Settling keeps a tree on request,
/// removes it otherwise,
/// and accepts a tree that is already gone.
#[test]
fn settling_keeps_removes_and_accepts_a_tree_that_is_gone() {
    let created: Fixture = fixture("settle");
    let root: &Path = created.root.as_path();
    std::fs::write(root.join("file"), "content").expect("file");
    settle_fixture(root, true);
    assert!(root.join("file").exists(), "a kept fixture stays");
    settle_fixture(root, false);
    assert!(!root.exists(), "a settled fixture goes");
    settle_fixture(root, false);
}

/// Fixtures are created in Cargo's per-target scratch directory,
/// not in the system temporary directory,
/// unless the
/// person moved the root on purpose.
#[test]
fn fixtures_are_created_outside_the_system_temporary_directory() {
    if std::env::var_os(FIXTURE_ROOT_VARIABLE).is_some() {
        return;
    }
    let created: Fixture = fixture("location");
    assert!(
        created.root.starts_with(std::fs::canonicalize(env!("CARGO_TARGET_TMPDIR")).expect("target scratch")),
        "{} is not under the target scratch directory",
        created.root.display()
    );
}

/// What:
///  How many of the first extents of a file the filesystem reports as shared with another file,
///  absent when the filesystem cannot report extents.
/// Why:
///  A reflink clone shares every extent with its source (`FIEMAP_EXTENT_SHARED`),
/// and a clone has no pages in the
///  page cache,
/// so starting it reads the executable from disk.
fn shared_extent_count(path: &Path) -> Option<usize> {
    use std::os::fd::AsRawFd;
    /// `_IOWR('f', 11, struct fiemap)`.
    const FS_IOC_FIEMAP: u64 = 0xC020_660B;
    /// Extent flag:
    /// the extent is shared with another file.
    const FIEMAP_EXTENT_SHARED: u32 = 0x2000;
    /// Size of `struct fiemap` before its extents.
    const HEADER_BYTES: usize = 32;
    /// Size of one `struct fiemap_extent`.
    const EXTENT_BYTES: usize = 56;
    /// Extents asked for;
    /// a clone shares all of them,
    /// so the first ones are enough.
    const EXTENT_SLOTS: usize = 256;
    let file: std::fs::File = std::fs::File::open(path).expect("open the copy");
    let mut buffer: Vec<u8> = vec![0; HEADER_BYTES + EXTENT_BYTES * EXTENT_SLOTS];
    // fm_start stays 0; fm_length covers the whole file; fm_flags stays 0 so no write-back is forced.
    buffer[8..16].copy_from_slice(&u64::MAX.to_ne_bytes());
    buffer[24..28].copy_from_slice(&u32::try_from(EXTENT_SLOTS).expect("slot count").to_ne_bytes());
    // SAFETY: `buffer` is a live allocation of the size the request describes and the kernel writes only inside it.
    let result: i32 = unsafe { libc::ioctl(file.as_raw_fd(), FS_IOC_FIEMAP as _, buffer.as_mut_ptr()) };
    if result != 0 {
        return None;
    }
    let mapped: usize = usize::try_from(u32::from_ne_bytes(buffer[20..24].try_into().expect("count bytes")))
        .expect("mapped count");
    let mut shared: usize = 0;
    for index in 0..mapped.min(EXTENT_SLOTS) {
        let flags_at: usize = HEADER_BYTES + EXTENT_BYTES * index + 40;
        let flags: u32 = u32::from_ne_bytes(buffer[flags_at..flags_at + 4].try_into().expect("flag bytes"));
        if flags & FIEMAP_EXTENT_SHARED != 0 {
            shared += 1;
        }
    }
    return Some(shared);
}

/// A byte copy a control makes is a data copy:
/// a clone starts by reading most of the executable from disk,
/// which stalled the first wrapped command of a test for seconds on a busy disk and hit the 5-second bound.
#[test]
fn a_byte_copy_of_the_wrapper_shares_no_storage_with_the_built_executable() {
    let created: Fixture = fixture("copy-storage");
    let copy: PathBuf = created.root.join("byte-copy");
    super::support::copy_executable(Path::new(super::support::WRAPPER), copy.as_path());
    match shared_extent_count(copy.as_path()) {
        Some(shared) => assert_eq!(
            shared,
            0,
            "{} shares {shared} extents with {}",
            copy.display(),
            super::support::WRAPPER
        ),
        None => eprintln!("the filesystem under {} reports no extents; nothing to check", created.root.display()),
    }
}

/// A fixture's service keeps running after the thread that started it has ended:
/// a control that commits in two worktrees from
///  two threads starts the services from whichever thread gets there first,
/// and the other thread's commit must still find its
///  service.
/// The kernel sends the parent-death signal when the parent thread ends,
/// not the parent process,
/// so a service
///  spawned from the starting thread itself died with that thread and left the other commit's work claimed for good.
#[test]
fn a_fixture_service_outlives_the_thread_that_started_it() {
    let created: Fixture = fixture("service-outlives-thread");
    let git_dir: PathBuf = created.root.join("repo/.git");
    std::fs::create_dir_all(&git_dir).expect("a Git directory to serve");
    let service = super::support::start_fixture_service(&created);
    std::thread::scope(|scope| {
        return scope.spawn(|| { service.serve(&created, git_dir.as_path()); }).join().expect("the thread that starts the service ends");
    });
    let socket: PathBuf = git_policy_cli::service_instance::socket_for(service.runtime.as_path(), std::fs::canonicalize(&git_dir).expect("canonical").as_path());
    // The signal arrives as the thread ends, but the kernel delivers it asynchronously: give a killed service time to be gone.
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(std::os::unix::net::UnixStream::connect(&socket).is_ok(), "the service stopped with the thread that started it");
}

/// A directory that is still gaining entries when the removal looks (a child of a command finishing) is removed on a later try;
/// a tree that is gone is not a failure;
/// another error is returned at once;
/// and a directory that never empties is an error.
#[test]
fn a_removal_waits_for_a_directory_that_is_still_gaining_entries() {
    let pause: std::time::Duration = std::time::Duration::from_millis(1);
    let busy = || { return std::io::Error::from(std::io::ErrorKind::DirectoryNotEmpty); };
    let calls: std::cell::Cell<usize> = std::cell::Cell::new(0);
    let twice_busy = |_: &Path| -> std::io::Result<()> {
        calls.set(calls.get() + 1);
        return if calls.get() <= 2 { Err(busy()) } else { Ok(()) };
    };
    assert!(remove_patiently(Path::new("/gone"), &twice_busy, 5, pause).is_ok());
    assert_eq!(calls.get(), 3, "removed on the third try");
    calls.set(0);
    let missing = |_: &Path| -> std::io::Result<()> { calls.set(calls.get() + 1); return Err(std::io::Error::from(std::io::ErrorKind::NotFound)); };
    assert!(remove_patiently(Path::new("/gone"), &missing, 5, pause).is_ok(), "a tree that is gone is removed");
    assert_eq!(calls.get(), 1);
    calls.set(0);
    let refused = |_: &Path| -> std::io::Result<()> { calls.set(calls.get() + 1); return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)); };
    assert_eq!(remove_patiently(Path::new("/gone"), &refused, 5, pause).map_err(|error| { return error.kind(); }), Err(std::io::ErrorKind::PermissionDenied));
    assert_eq!(calls.get(), 1, "another error is not retried");
    calls.set(0);
    let never = |_: &Path| -> std::io::Result<()> { calls.set(calls.get() + 1); return Err(busy()); };
    assert_eq!(remove_patiently(Path::new("/gone"), &never, 3, pause).map_err(|error| { return error.kind(); }), Err(std::io::ErrorKind::DirectoryNotEmpty));
    assert_eq!(calls.get(), 3, "and gives up after its tries");
}

/// How long a fixture may take to say that its commit service ended before it listened.
/// The patience for a service that is
///  merely slow is a minute,
/// so anything near it means the fixture waited for a service that was already gone.
const ENDED_SERVICE_VERDICT: std::time::Duration = std::time::Duration::from_secs(10);

/// What:
///  A commit service that ends before it listens fails the fixture's start at once and says how it ended.
/// Why:
///   Under a memory limit close to what the service needs,
/// it binds its socket,
/// cannot start its acceptor thread and exits
///  within a millisecond.
/// The fixture used to poll for a listener until its minute of patience ran out ("never listened"),
///  although the service had ended at the start:
/// a poll that missed that millisecond waited for nothing.
/// A program placed
///  in front of the service that ends at once is the same event without the timing.
#[test]
fn a_fixture_service_that_ends_before_it_listens_fails_the_start_at_once() {
    let created: Fixture = fixture("service-ends-first");
    *created.service_launch.lock().expect("the service launch") =
        ServiceLaunch { prefix: vec![std::ffi::OsString::from("/usr/bin/false")], ..ServiceLaunch::default() };
    let services: FixtureService = start_fixture_service(&created);
    let git_dir: PathBuf = created.root.join("repo").join(".git");
    let began: std::time::Instant = std::time::Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { services.serve(&created, git_dir.as_path()); }));
    let said: String = match outcome {
        Ok(()) => panic!("the fixture reported a listening service although the service ended"),
        Err(payload) => payload.downcast_ref::<String>().cloned().or_else(|| { return payload.downcast_ref::<&str>().map(ToString::to_string); }).unwrap_or_default(),
    };
    assert!(began.elapsed() < ENDED_SERVICE_VERDICT, "the fixture waited {:?} for a service that had ended: {said}", began.elapsed());
    assert!(said.contains("ended"), "the failure must say that the service ended: {said}");
    let status: String = std::process::Command::new("/usr/bin/false").status().expect("the stand-in program runs").to_string();
    assert!(said.contains(status.as_str()), "the failure must give the exit status ({status}): {said}");
}

/// How long the stand-in for an earlier service that is still dying keeps its listener.
const DYING_SERVICE_LINGER: std::time::Duration = std::time::Duration::from_millis(500);

/// How long a connection to the fixture's new service may take to be greeted once the fixture said the service listens.
const GREETING_PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);

/// What:
///  A fixture does not report its new commit service as listening while an earlier service of the same worktree still holds the
///  socket.
/// Why:
///   Ending a service that runs under a tracer reaps the tracer,
/// which leaves the traced service to die a moment later.
/// The
///  fixture's readiness poll connected to that dying listener,
/// reported the new service as listening,
/// and the new service,
/// which
///  cannot take a socket that still answers,
/// ended with its address in use.
/// Nothing restarts a service in a fixture,
/// so the
///  next commit waited out the client's whole start wait and failed with "did not start".
/// The stand-in here is a listener that
///  answers nothing and goes away after a moment,
/// as the dying service's does.
#[test]
fn a_fixture_service_starts_after_an_earlier_service_that_is_still_dying() {
    let created: Fixture = fixture("service-after-dying");
    let services: FixtureService = start_fixture_service(&created);
    let git_dir: PathBuf = created.root.join("repo").join(".git");
    let socket: PathBuf = git_policy_cli::service_instance::socket_for(services.runtime.as_path(), git_dir.as_path());
    let dying: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(&socket).expect("the stand-in listener");
    let lingering = std::thread::spawn(move || {
        std::thread::sleep(DYING_SERVICE_LINGER);
        drop(dying);
    });
    services.serve(&created, git_dir.as_path());
    lingering.join().expect("the stand-in ends");
    let stream = std::os::unix::net::UnixStream::connect(&socket).expect("the fixture's service listens once the earlier one is gone");
    stream.set_read_timeout(Some(GREETING_PATIENCE)).expect("a read timeout");
    let mut greeting: String = String::new();
    std::io::BufRead::read_line(&mut std::io::BufReader::new(stream), &mut greeting).expect("the service greets");
    assert!(
        matches!(git_policy_cli::service_protocol::decode_service(greeting.trim_end()), Ok(git_policy_cli::service_protocol::ServiceMessage::Welcome { .. })),
        "the new service must greet, said {greeting:?}"
    );
}

/// A fixture root that cannot be created says what became of it,
/// and of its parent,
/// not just that the call failed.
/// Why:
///  A container gate lost its fixture root after test 5,018 of 5,265,
///  and 217 later tests failed with a bare `fresh fixture: NotFound`,
///  so nothing said what removed the folder or how far up the loss went
///  (item 9 of `~/temp/agent/pi-runs/queue-audit-report.md`).
#[test]
fn a_fixture_root_that_cannot_be_created_says_what_became_of_it() {
    let held: Fixture = fixture("vanished-root");
    let root: PathBuf = held.root.clone();
    let wanted: PathBuf = root.join("gone/parent");
    // A plain file where the root must be a folder, so creating the folder fails at once.
    std::fs::write(root.join("gone"), b"a file where a folder is needed").expect("a file in place of the folder");
    let probe: PathBuf = wanted.clone();
    let caught = std::panic::catch_unwind(move || {
        let _made = super::support::fixture_in(probe.as_path(), "vanished");
    });
    let message: String = match caught {
        Err(payload) => payload.downcast_ref::<String>().cloned().unwrap_or_else(|| { return String::from("<not text>"); }),
        Ok(()) => panic!("a fixture root that cannot be created must be refused"),
    };
    assert!(message.contains("fixture directory root"), "the failure names the step: {message}");
    assert!(message.contains(wanted.display().to_string().as_str()), "the failure names the root: {message}");
    assert!(message.contains("its parent"), "the failure names what became of the parent: {message}");
    std::fs::remove_dir_all(root.as_path()).expect("the fixture goes");
}

/// A process the binary tests start (a service that is killed,
/// a wrapper that aborts) may not write a core dump:
/// each dump
///  costs the system's core-dump handler processor time and disk writes on a shared disk.
/// The harness sets the limit;
/// the
///  product sets none.
#[test]
fn a_process_a_binary_test_starts_may_not_write_a_core_dump() {
    let fixture: Fixture = fixture("core-limit");
    let output = std::process::Command::new("sh").args(["-c", "ulimit -c; ulimit -H -c"]).output().expect("sh runs");
    assert_eq!(String::from_utf8_lossy(&output.stdout).split_whitespace().collect::<Vec<&str>>(), vec!["0", "0"]);
    drop(fixture);
}

/// How long the control below waits for two threads that give the service a variable and then start it,
/// so a hang fails it and
///  never holds it.
/// The two calls take well under a second when nothing blocks (a service start and a variable).
const SERVICE_ENVIRONMENT_PATIENCE: Duration = Duration::from_secs(60);

/// What:
///  Two threads of one control that each give the fixture's service a variable and then start it do not wait for each other for ever.
/// Why:
///   `service_runtime` holds the service's lock and reads the environment while it starts a service,
/// and
///  `give_service_environment` held the environment's lock and then took the service's:
/// each waited for the lock the other held
///  (measured with `gdb` on the hung test process:
/// one thread in `service_runtime` at the environment lock,
/// the other in
///  `give_service_environment` at the service lock,
/// 3 of 3 runs of the two-worktree LFS control when it passed an environment).
///  The threads are the two worktrees' commits of that control.
#[test]
fn two_threads_that_give_the_service_a_variable_and_start_it_do_not_wait_for_each_other_for_ever() {
    let fixture: Fixture = fixture("service-environment-order");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let fixture_ref: &'static Fixture = Box::leak(Box::new(fixture));
    for _ in 0..2 {
        let done = sender.clone();
        std::thread::spawn(move || {
            give_service_environment(fixture_ref, "CLI_GIT_WAIT_NOTICE_SECONDS", "1");
            let _runtime = service_runtime(fixture_ref);
            let _sent = done.send(());
        });
    }
    for _ in 0..2 {
        receiver.recv_timeout(SERVICE_ENVIRONMENT_PATIENCE).expect("the thread ended: the service's two locks are taken in one order");
    }
    assert!(repo.exists());
}

/// How long the control below lets a listener sit without greeting before it says the service is not serving.
const SILENT_LISTENER_WAIT: Duration = Duration::from_millis(300);

/// A service is serving when it greets a connection,
/// not when the kernel accepts one:
/// a listener that has not started its
/// threads accepts a connection and says nothing,
/// and the fixture must not count it as started (a control that lowered the
/// service's process limit at that moment ended the service with status 4,
/// `cannot start the thread for the after-commit work`).
#[test]
fn a_listener_that_accepts_a_connection_and_says_nothing_is_not_a_service_that_is_serving() {
    use std::io::Write;
    let held: Fixture = fixture("silent-listener");
    // The socket's name must fit `SUN_LEN`, which a fixture root under a deep checkout does not; the short directory is the one
    //  the fixture's own services put their sockets in.
    let runtime: PathBuf = super::support::short_runtime_directory(&held);
    let socket: PathBuf = runtime.join("socket");
    let listener = std::os::unix::net::UnixListener::bind(&socket).expect("a listening socket");
    assert!(std::os::unix::net::UnixStream::connect(&socket).is_ok(), "the kernel accepts a connection for a listener that has not answered");
    let began: Instant = Instant::now();
    let silent: bool = !service_is_serving(&socket);
    assert!(silent, "no greeting came, so the service is not serving");
    assert!(began.elapsed() < SILENT_LISTENER_WAIT * 20, "the check ended within its own bound");
    let greeter = std::thread::spawn(move || {
        for connection in listener.incoming().take(3).flatten() {
            let mut stream = connection;
            let _greeted = stream.write_all(b"{\"greeting\":true}\n");
        }
    });
    assert!(service_is_serving(&socket), "a listener that greets is serving");
    greeter.join().expect("the greeter ends");
    let _removed = std::fs::remove_dir_all(runtime.as_path());
}
