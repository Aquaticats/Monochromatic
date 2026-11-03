//! Binary-level controls for the fixtures themselves: where they live and that they go away however their test ends
//! (issue 1809).

use super::support::{FIXTURE_ROOT_VARIABLE, Fixture, FixtureService, ServiceLaunch, fixture, settle_fixture, start_fixture_service};
use std::path::{Path, PathBuf};

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

/// A fixture removed explicitly and then dropped does not fail the drop.
#[test]
fn a_fixture_removed_explicitly_is_dropped_quietly() {
    let created: Fixture = fixture("drop-after-remove");
    let root: PathBuf = created.root.clone();
    super::support::remove(&created);
    assert!(!root.exists());
    drop(created);
}

/// Settling keeps a tree on request, removes it otherwise, and accepts a tree that is already gone.
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

/// Fixtures are created in Cargo's per-target scratch directory, not in the system temporary directory, unless the
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
///  A reflink clone shares every extent with its source (`FIEMAP_EXTENT_SHARED`), and a clone has no pages in the
///  page cache, so starting it reads the executable from disk.
fn shared_extent_count(path: &Path) -> Option<usize> {
    use std::os::fd::AsRawFd;
    /// `_IOWR('f', 11, struct fiemap)`.
    const FS_IOC_FIEMAP: u64 = 0xC020_660B;
    /// Extent flag: the extent is shared with another file.
    const FIEMAP_EXTENT_SHARED: u32 = 0x2000;
    /// Size of `struct fiemap` before its extents.
    const HEADER_BYTES: usize = 32;
    /// Size of one `struct fiemap_extent`.
    const EXTENT_BYTES: usize = 56;
    /// Extents asked for; a clone shares all of them, so the first ones are enough.
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

/// A byte copy a control makes is a data copy: a clone starts by reading most of the executable from disk,
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

/// A fixture's service keeps running after the thread that started it has ended: a control that commits in two worktrees from
///  two threads starts the services from whichever thread gets there first, and the other thread's commit must still find its
///  service. The kernel sends the parent-death signal when the parent thread ends, not the parent process, so a service
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

/// How long a fixture may take to say that its commit service ended before it listened. The patience for a service that is
///  merely slow is a minute, so anything near it means the fixture waited for a service that was already gone.
const ENDED_SERVICE_VERDICT: std::time::Duration = std::time::Duration::from_secs(10);

/// What:
///  A commit service that ends before it listens fails the fixture's start at once and says how it ended.
/// Why:
///   Under a memory limit close to what the service needs, it binds its socket, cannot start its acceptor thread and exits
///  within a millisecond. The fixture used to poll for a listener until its minute of patience ran out ("never listened"),
///  although the service had ended at the start: a poll that missed that millisecond waited for nothing. A program placed
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
