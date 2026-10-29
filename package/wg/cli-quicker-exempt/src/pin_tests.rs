//! Exercises persisted cgroup links and all supported socket-address hooks as root.

/// Direct marker attach used to force a partial pin transaction failure.
use crate::bpf::{
    attach_marker_unpinned,
    attach_marker_unpinned_failing_after,
    HOOK_NAMES,
};
/// Production user app-slice path used by watcher service fixture.
use crate::application_targets::user_app_slice;
/// Process start-time reader for wrong-owner state fixture.
use crate::keeper_process::process_start_time;
/// Runtime keeper state used to verify fail-closed identity cleanup.
use crate::keeper_state::{
    active_state,
    read_state,
    replacing_state,
    write_state,
    KeeperProcess,
};
/// Production lifecycle and pin paths under test.
use crate::pin::{
    commit_staging_for_test,
    detach_cgroup,
    pin_dir_for,
    remove_exact_pin_dir_for_test,
    BPF_FS,
    PIN_ROOT,
};
/// Standard I/O failures used by syscall helpers.
use std::io::{self, BufRead, BufReader, Write};
/// Raw descriptor access and ownership for sockets.
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
/// Child setup hook that moves itself before executing socket assertions.
use std::os::unix::process::CommandExt;
/// Paths for disposable cgroups and pin directories.
use std::path::{Path, PathBuf};
/// Child process launcher for cross-process persistence checks.
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Environment flag selecting socket-check child behavior.
const CHILD_FLAG: &str = "WG_QUICKER_EXEMPT_SOCKET_CHILD";
/// Environment value carrying expected mark into socket-check child.
const EXPECTED_MARK: &str = "WG_QUICKER_EXEMPT_EXPECTED_MARK";
/// Debug environment flag making public CLI exercise descriptor-keeper fallback on every kernel.
const FORCE_PIN_INVALID_ENV: &str = "WG_QUICKER_EXEMPT_TEST_FORCE_PIN_INVALID";
/// First mark used to verify persisted initial attachment.
const FIRST_MARK: u32 = 8_888;
/// Second mark used to verify atomic replacement.
const SECOND_MARK: u32 = 9_999;
/// Third mark distinguishes recovery replacement from both recorded holders.
const RECOVERY_MARK: u32 = 7_777;

/// Locates normal debug CLI from any Cargo test-artifact subdirectory.
fn cli_binary() -> io::Result<PathBuf> {
    let test_binary = std::env::current_exe()?;
    // Cargo toolchains can place test artifacts under `debug/deps` or deeper `debug/build` directories.
    for directory in test_binary.ancestors() {
        if !directory.ends_with("debug") {
            continue;
        }
        let binary = directory.join("wg-quicker-exempt");
        if binary.exists() {
            return Ok(binary);
        }
    }
    return Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "build debug CLI beside debug artifact tree containing {}",
            test_binary.display()
        ),
    ));
}

/// Runs real CLI and includes stderr when command fails.
fn run_cli(arguments: &[&str]) -> io::Result<()> {
    let output = Command::new(cli_binary()?).args(arguments).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "CLI {:?} failed with {}: {}",
            arguments,
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    return Ok(());
}

/// Runs real debug CLI through deterministic descriptor-keeper fallback.
fn run_cli_with_forced_keeper(arguments: &[&str]) -> io::Result<()> {
    let output = Command::new(cli_binary()?)
        .args(arguments)
        .env(FORCE_PIN_INVALID_ENV, "1")
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "forced-keeper CLI {:?} failed with {}: {}",
            arguments,
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    return Ok(());
}

/// Direct holder child and handshake pipes used to model parent crash windows.
struct HolderCandidate {
    /// Child is reaped after recovery terminates it.
    child: Child,
    /// Commit channel whose EOF rolls back uncommitted links.
    input: ChildStdin,
    /// Readiness and commit acknowledgement channel.
    output: BufReader<ChildStdout>,
    /// Exact identity persisted in synthetic transition state.
    process: KeeperProcess,
}

/// Starts holder directly and waits until its four uncommitted links exist.
fn spawn_holder_candidate(
    cgroup: &Path,
    mark: u32,
) -> io::Result<HolderCandidate> {
    let mark_text = mark.to_string();
    let mut child = Command::new(cli_binary()?)
        .args(["__hold", &mark_text])
        .arg(cgroup)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let input = child.stdin.take().ok_or_else(|| return io::Error::other("holder stdin absent"))?;
    let stdout = child.stdout.take().ok_or_else(|| return io::Error::other("holder stdout absent"))?;
    let mut output = BufReader::new(stdout);
    let mut readiness = String::new();
    output.read_line(&mut readiness)?;
    if readiness.trim_end() != "READY" {
        return Err(io::Error::other("holder did not report readiness"));
    }
    let pid = child.id() as i32;
    return Ok(HolderCandidate {
        child,
        input,
        output,
        process: KeeperProcess {
            pid,
            start_time: process_start_time(pid)?,
            mark,
        },
    });
}

/// Sends commit to direct holder and waits for durable marker acknowledgement.
fn commit_holder(candidate: &mut HolderCandidate) -> io::Result<()> {
    writeln!(candidate.input, "COMMIT")?;
    candidate.input.flush()?;
    let mut acknowledgement = String::new();
    candidate.output.read_line(&mut acknowledgement)?;
    if acknowledgement.trim_end() != "COMMITTED" {
        return Err(io::Error::other("holder did not acknowledge commit"));
    }
    return Ok(());
}

/// Removes links and cgroup even when assertion unwinds.
struct Fixture {
    /// Unique disposable cgroup path.
    cgroup: PathBuf,
}

impl Drop for Fixture {
    /// Best-effort cleanup leaves unrelated cgroups and pins untouched.
    fn drop(&mut self) {
        if let Some(cgroup) = self.cgroup.to_str() {
            let _detach_result = run_cli(&["detach", cgroup]);
        }
        let _detach_result = detach_cgroup(&self.cgroup);
        let _remove_result = std::fs::remove_dir(&self.cgroup);
    }
}

/// Creates unique leaf cgroup under host's unified cgroup v2 mount.
fn create_fixture(label: &str) -> io::Result<Fixture> {
    let cgroup = Path::new("/sys/fs/cgroup").join(format!(
        "wg-quicker-exempt-{label}-{}",
        std::process::id()
    ));
    if cgroup.exists() {
        std::fs::remove_dir(&cgroup)?;
    }
    std::fs::create_dir(&cgroup)?;
    return Ok(Fixture { cgroup });
}

/// Creates owned socket descriptor for one address family and transport type.
fn socket(domain: libc::c_int, socket_type: libc::c_int) -> io::Result<OwnedFd> {
    // SAFETY: domain and type are Linux socket constants and successful result is newly owned.
    let raw_fd = unsafe { libc::socket(domain, socket_type | libc::SOCK_CLOEXEC, 0) };
    if raw_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful `socket` returns a descriptor this scope owns.
    return Ok(unsafe { OwnedFd::from_raw_fd(raw_fd) });
}

/// Reads socket mark after one cgroup hook operation.
fn read_mark(fd: &OwnedFd) -> io::Result<u32> {
    let mut mark: libc::c_int = 0;
    let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    // SAFETY: output pointer and length describe writable `mark` storage.
    let result = unsafe {
        libc::getsockopt(
            fd.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_MARK,
            std::ptr::addr_of_mut!(mark).cast(),
            &mut length,
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(mark as u32);
}

/// Triggers IPv4 connect or sendmsg hook and returns resulting socket mark.
fn ipv4_mark(socket_type: libc::c_int) -> io::Result<u32> {
    let fd = socket(libc::AF_INET, socket_type)?;
    let address = libc::sockaddr_in {
        sin_family: libc::AF_INET as libc::sa_family_t,
        sin_port: 9_u16.to_be(),
        sin_addr: libc::in_addr {
            s_addr: u32::from_ne_bytes([127, 0, 0, 1]),
        },
        sin_zero: [0; 8],
    };
    let address_pointer = std::ptr::addr_of!(address).cast::<libc::sockaddr>();
    if socket_type == libc::SOCK_DGRAM {
        let byte = [1_u8];
        // SAFETY: data and destination pointers remain valid for synchronous syscall.
        let _send_result = unsafe {
            libc::sendto(
                fd.as_raw_fd(),
                byte.as_ptr().cast(),
                byte.len(),
                0,
                address_pointer,
                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
            )
        };
    } else {
        // SAFETY: destination pointer and length describe initialized IPv4 address.
        let _connect_result = unsafe {
            libc::connect(
                fd.as_raw_fd(),
                address_pointer,
                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
            )
        };
    }
    return read_mark(&fd);
}

/// Triggers IPv6 connect or sendmsg hook and returns resulting socket mark.
fn ipv6_mark(socket_type: libc::c_int) -> io::Result<u32> {
    let fd = socket(libc::AF_INET6, socket_type)?;
    let address = libc::sockaddr_in6 {
        sin6_family: libc::AF_INET6 as libc::sa_family_t,
        sin6_port: 9_u16.to_be(),
        sin6_flowinfo: 0,
        sin6_addr: libc::in6_addr {
            s6_addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        },
        sin6_scope_id: 0,
    };
    let address_pointer = std::ptr::addr_of!(address).cast::<libc::sockaddr>();
    if socket_type == libc::SOCK_DGRAM {
        let byte = [1_u8];
        // SAFETY: data and destination pointers remain valid for synchronous syscall.
        let _send_result = unsafe {
            libc::sendto(
                fd.as_raw_fd(),
                byte.as_ptr().cast(),
                byte.len(),
                0,
                address_pointer,
                std::mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t,
            )
        };
    } else {
        // SAFETY: destination pointer and length describe initialized IPv6 address.
        let _connect_result = unsafe {
            libc::connect(
                fd.as_raw_fd(),
                address_pointer,
                std::mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t,
            )
        };
    }
    return read_mark(&fd);
}

/// Runs all four hook operations in subprocess moved into fixture cgroup before exec.
fn assert_marks_from_child(cgroup: &Path, expected_mark: u32) -> io::Result<()> {
    let cgroup_procs = std::ffi::CString::new(cgroup.join("cgroup.procs").as_os_str().as_encoded_bytes()).map_err(|_| {
        return io::Error::new(io::ErrorKind::InvalidInput, "cgroup.procs path contains NUL");
    })?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg("--exact")
        .arg("pin_tests::socket_child")
        .arg("--nocapture")
        .env(CHILD_FLAG, "1")
        .env(EXPECTED_MARK, expected_mark.to_string());
    // SAFETY: callback uses only async-signal-safe libc calls before immediate exec.
    unsafe {
        command.pre_exec(move || {
            let fd = libc::open(cgroup_procs.as_ptr(), libc::O_WRONLY | libc::O_CLOEXEC);
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            let moved = libc::write(fd, b"0".as_ptr().cast(), 1);
            let close_result = libc::close(fd);
            if moved != 1 || close_result < 0 {
                return Err(io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    let status = command.status()?;
    if !status.success() {
        return Err(io::Error::other(format!("socket child failed with {status}")));
    }
    return Ok(());
}

/// Subprocess assertion target skipped during ordinary parent test execution.
#[test]
fn socket_child() -> io::Result<()> {
    if std::env::var_os(CHILD_FLAG).is_none() {
        return Ok(());
    }
    let expected: u32 = std::env::var(EXPECTED_MARK)
        .map_err(io::Error::other)?
        .parse()
        .map_err(io::Error::other)?;
    assert_eq!(ipv4_mark(libc::SOCK_STREAM)?, expected, "TCP4 mark");
    assert_eq!(ipv6_mark(libc::SOCK_STREAM)?, expected, "TCP6 mark");
    assert_eq!(ipv4_mark(libc::SOCK_DGRAM)?, expected, "UDP4 mark");
    assert_eq!(ipv6_mark(libc::SOCK_DGRAM)?, expected, "UDP6 mark");
    return Ok(());
}

/// Foreground watcher covers named targets and future Ghostty scopes,
///  then drops all links on signal.
#[test]
#[ignore = "requires root and writable cgroup v2 mount"]
fn application_watcher_covers_named_targets_and_future_ghostty() -> io::Result<()> {
    let app_slice = Path::new("/sys/fs/cgroup").join(format!(
        "wg-quicker-exempt-watch-{}",
        std::process::id()
    ));
    let service = app_slice.join("app-com.mitchellh.ghostty@fixture.service");
    let surface = app_slice.join("app-ghostty-surface-transient-123.scope");
    let firefox_nightly = app_slice.join("app-firefox\\x2dnightly@fixture.service");
    let firefox_esr = app_slice.join("app-firefox\\x2desr@fixture.service");
    let unrelated = app_slice.join("app-org.example.Other.scope");
    std::fs::create_dir(&app_slice)?;
    std::fs::create_dir(&service)?;
    std::fs::create_dir(&firefox_nightly)?;
    std::fs::create_dir(&firefox_esr)?;
    std::fs::create_dir(&unrelated)?;
    let mut watcher = Command::new(cli_binary()?)
        .args(["__watch-test", "8888"])
        .arg(&app_slice)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = watcher.stdin.take().ok_or_else(|| return io::Error::other("watcher stdin absent"))?;
    let stdout = watcher.stdout.take().ok_or_else(|| return io::Error::other("watcher stdout absent"))?;
    let mut output = BufReader::new(stdout);
    let mut readiness = String::new();
    output.read_line(&mut readiness)?;
    assert_eq!(readiness.trim_end(), "READY");
    writeln!(input, "COMMIT")?;
    input.flush()?;
    let mut committed = String::new();
    output.read_line(&mut committed)?;
    assert_eq!(committed.trim_end(), "COMMITTED");
    assert_marks_from_child(&service, FIRST_MARK)?;
    assert_marks_from_child(&firefox_nightly, FIRST_MARK)?;
    assert_marks_from_child(&firefox_esr, 0)?;
    assert_marks_from_child(&unrelated, 0)?;
    std::fs::create_dir(&surface)?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert_marks_from_child(&surface, FIRST_MARK)?;
    // SAFETY: watcher child PID is owned by this test and handles SIGTERM through signalfd.
    let signal_result = unsafe { libc::kill(watcher.id() as i32, libc::SIGTERM) };
    if signal_result < 0 {
        return Err(io::Error::last_os_error());
    }
    let status = watcher.wait()?;
    assert!(status.success());
    assert_marks_from_child(&service, 0)?;
    assert_marks_from_child(&firefox_nightly, 0)?;
    std::fs::remove_dir(&surface)?;
    std::fs::remove_dir(&unrelated)?;
    std::fs::remove_dir(&firefox_esr)?;
    std::fs::remove_dir(&firefox_nightly)?;
    std::fs::remove_dir(&service)?;
    std::fs::remove_dir(&app_slice)?;
    return Ok(());
}

/// Public watcher service persists after start CLI exits and stops by validated key.
#[test]
#[ignore = "requires root and writable cgroup v2 mount"]
fn application_watcher_service_starts_and_stops() -> io::Result<()> {
    let uid = 3_000_000_000_u32 + (std::process::id() % 100_000_000);
    let app_slice = user_app_slice(uid);
    let service = app_slice.join("app-com.mitchellh.ghostty@fixture.service");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&service)?;
    let key = format!("wgq-watch-{}", std::process::id());
    let uid_text = uid.to_string();
    run_cli(&["watch-start", &key, "8888", &uid_text])?;
    assert_marks_from_child(&service, FIRST_MARK)?;
    run_cli(&["watch-stop", &key])?;
    assert_marks_from_child(&service, 0)?;
    std::fs::remove_dir(&service)?;
    std::fs::remove_dir(&app_slice)?;
    let user_service = app_slice.parent().ok_or_else(|| return io::Error::other("app slice lacks service parent"))?;
    let user_slice = user_service.parent().ok_or_else(|| return io::Error::other("user service lacks slice parent"))?;
    std::fs::remove_dir(user_service)?;
    std::fs::remove_dir(user_slice)?;
    return Ok(());
}

/// Parent death before transition state closes stdin and leaves no untracked links.
#[test]
#[ignore = "requires root and writable cgroup v2 plus bpffs mounts"]
fn pretransition_parent_death_rolls_back_candidate() -> io::Result<()> {
    let fixture = create_fixture("pretransition")?;
    let mut candidate = spawn_holder_candidate(&fixture.cgroup, FIRST_MARK)?;
    drop(candidate.input);
    let status = candidate.child.wait()?;
    assert!(status.success());
    assert_marks_from_child(&fixture.cgroup, 0)?;
    assert!(read_state(&fixture.cgroup)?.is_none());
    return Ok(());
}

/// Recovery stops live uncommitted candidate and preserves prior holder before replacement.
#[test]
#[ignore = "requires root and writable cgroup v2 plus bpffs mounts"]
fn recovery_rejects_uncommitted_candidate() -> io::Result<()> {
    let fixture = create_fixture("recover-uncommitted")?;
    let cgroup = fixture
        .cgroup
        .to_str()
        .ok_or_else(|| return io::Error::other("non-UTF-8 cgroup fixture"))?;
    run_cli_with_forced_keeper(&["attach", "8888", cgroup])?;
    let active = read_state(&fixture.cgroup)?
        .and_then(|state| return state.active)
        .ok_or_else(|| return io::Error::other("active keeper absent"))?;
    let candidate = spawn_holder_candidate(&fixture.cgroup, SECOND_MARK)?;
    write_state(
        &fixture.cgroup,
        &replacing_state(Some(active), candidate.process),
    )?;
    let mut candidate_child = candidate.child;
    let reaper = std::thread::spawn(move || return candidate_child.wait());
    run_cli_with_forced_keeper(&["attach", "7777", cgroup])?;
    let _candidate_status = reaper
        .join()
        .map_err(|_| return io::Error::other("candidate reaper panicked"))??;
    assert_marks_from_child(&fixture.cgroup, RECOVERY_MARK)?;
    run_cli(&["detach", cgroup])?;
    return Ok(());
}

/// Recovery adopts committed candidate,
///  removes marker,
///  then performs next replacement.
#[test]
#[ignore = "requires root and writable cgroup v2 plus bpffs mounts"]
fn recovery_adopts_committed_candidate() -> io::Result<()> {
    let fixture = create_fixture("recover-committed")?;
    let cgroup = fixture
        .cgroup
        .to_str()
        .ok_or_else(|| return io::Error::other("non-UTF-8 cgroup fixture"))?;
    let mut candidate = spawn_holder_candidate(&fixture.cgroup, FIRST_MARK)?;
    write_state(
        &fixture.cgroup,
        &replacing_state(None, candidate.process),
    )?;
    commit_holder(&mut candidate)?;
    let mut candidate_child = candidate.child;
    let reaper = std::thread::spawn(move || return candidate_child.wait());
    run_cli_with_forced_keeper(&["attach", "9999", cgroup])?;
    let _candidate_status = reaper
        .join()
        .map_err(|_| return io::Error::other("candidate reaper panicked"))??;
    assert_marks_from_child(&fixture.cgroup, SECOND_MARK)?;
    run_cli(&["detach", cgroup])?;
    return Ok(());
}

/// Removed cgroup still maps to prior state key so holder can be stopped and forgotten.
#[test]
#[ignore = "requires root and writable cgroup v2 plus bpffs mounts"]
fn removed_cgroup_detaches_by_lexical_identity() -> io::Result<()> {
    let fixture = create_fixture("removed")?;
    let cgroup = fixture
        .cgroup
        .to_str()
        .ok_or_else(|| return io::Error::other("non-UTF-8 cgroup fixture"))?;
    run_cli_with_forced_keeper(&["attach", "8888", cgroup])?;
    std::fs::remove_dir(&fixture.cgroup)?;
    run_cli(&["detach", cgroup])?;
    assert!(read_state(&fixture.cgroup)?.is_none());
    return Ok(());
}

/// Wrong-owner state blocks detach and preserves live holder until identity is restored.
#[test]
#[ignore = "requires root and writable cgroup v2 plus bpffs mounts"]
fn wrong_owner_state_fails_closed() -> io::Result<()> {
    let fixture = create_fixture("wrong-owner")?;
    let cgroup = fixture
        .cgroup
        .to_str()
        .ok_or_else(|| return io::Error::other("non-UTF-8 cgroup fixture"))?;
    run_cli_with_forced_keeper(&["attach", "8888", cgroup])?;
    let original_state = read_state(&fixture.cgroup)?
        .ok_or_else(|| return io::Error::other("keeper state absent"))?;
    let original = original_state
        .active
        .ok_or_else(|| return io::Error::other("active keeper absent"))?;
    let current_pid = std::process::id() as i32;
    let wrong = KeeperProcess {
        pid: current_pid,
        start_time: process_start_time(current_pid)?,
        mark: original.mark,
    };
    write_state(&fixture.cgroup, &active_state(wrong))?;
    let output = Command::new(cli_binary()?)
        .args(["detach", cgroup])
        .output()?;
    write_state(&fixture.cgroup, &active_state(original))?;
    assert!(!output.status.success());
    assert_marks_from_child(&fixture.cgroup, FIRST_MARK)?;
    run_cli(&["detach", cgroup])?;
    return Ok(());
}

/// Partial-link failure drops earlier descriptors and leaves cgroup unmarked.
#[test]
#[ignore = "requires root and writable cgroup v2 mount"]
fn partial_link_attach_rolls_back() -> io::Result<()> {
    let fixture = create_fixture("partial-links")?;
    let cgroup = std::fs::File::open(&fixture.cgroup)?;
    let result = attach_marker_unpinned_failing_after(cgroup.as_raw_fd(), FIRST_MARK, 2);
    assert!(result.is_err());
    assert_marks_from_child(&fixture.cgroup, 0)?;
    return Ok(());
}

/// Verifies eBPF behavior on every hook while link descriptors remain owned.
#[test]
#[ignore = "requires root and writable cgroup v2 mount"]
fn all_protocol_hooks_mark_and_drop_cleanly() -> io::Result<()> {
    let fixture = create_fixture("protocols")?;
    let cgroup = std::fs::File::open(&fixture.cgroup)?;
    let links = attach_marker_unpinned(cgroup.as_raw_fd(), FIRST_MARK)?;
    assert_marks_from_child(&fixture.cgroup, FIRST_MARK)?;
    drop(links);
    assert_marks_from_child(&fixture.cgroup, 0)?;
    return Ok(());
}

/// Verifies process-exit persistence,
///  replacement,
///  and idempotent exact detach through real CLI.
#[test]
#[ignore = "requires root and writable cgroup v2 plus bpffs mounts"]
fn persisted_protocol_lifecycle() -> io::Result<()> {
    let fixture = create_fixture("lifecycle")?;
    let cgroup = fixture
        .cgroup
        .to_str()
        .ok_or_else(|| return io::Error::other("non-UTF-8 cgroup fixture"))?;
    run_cli(&["attach", "8888", cgroup])?;
    assert_marks_from_child(&fixture.cgroup, FIRST_MARK)?;
    run_cli(&["attach", "9999", cgroup])?;
    assert_marks_from_child(&fixture.cgroup, SECOND_MARK)?;
    run_cli(&["detach", cgroup])?;
    assert_marks_from_child(&fixture.cgroup, 0)?;
    run_cli(&["detach", cgroup])?;
    return Ok(());
}

/// Candidate failure after attaching links leaves prior persisted attachment unchanged.
#[test]
#[ignore = "requires root and writable cgroup v2 plus bpffs mounts"]
fn failed_attach_preserves_prior_attachment() -> io::Result<()> {
    let fixture = create_fixture("rollback")?;
    let cgroup = fixture
        .cgroup
        .to_str()
        .ok_or_else(|| return io::Error::other("non-UTF-8 cgroup fixture"))?;
    run_cli_with_forced_keeper(&["attach", "8888", cgroup])?;
    let output = Command::new(cli_binary()?)
        .args(["attach", "9999", cgroup])
        .env(FORCE_PIN_INVALID_ENV, "1")
        .env("WG_QUICKER_EXEMPT_TEST_FAIL_AFTER_ATTACH", "1")
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("injected holder failure"));
    assert_marks_from_child(&fixture.cgroup, FIRST_MARK)?;
    return Ok(());
}

/// Creates ordinary files shaped like four link pins for transaction unit tests.
fn write_pin_fixture(directory: &Path, marker: &[u8]) -> io::Result<()> {
    std::fs::create_dir(directory)?;
    for hook_name in HOOK_NAMES {
        std::fs::write(directory.join(hook_name), marker)?;
    }
    return Ok(());
}

/// Atomic exchange makes all new files visible and removes all old files.
#[test]
fn replacement_exchange_commits_complete_pin_set() -> io::Result<()> {
    let scratch = std::env::temp_dir().join(format!("wg-quicker-pin-exchange-{}", std::process::id()));
    std::fs::create_dir(&scratch)?;
    let final_dir = scratch.join("final");
    let staging_dir = scratch.join("staging");
    write_pin_fixture(&final_dir, b"old")?;
    write_pin_fixture(&staging_dir, b"new")?;
    commit_staging_for_test(&staging_dir, &final_dir)?;
    assert!(!staging_dir.exists());
    for hook_name in HOOK_NAMES {
        assert_eq!(std::fs::read(final_dir.join(hook_name))?, b"new");
    }
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Failed exchange leaves prior complete pin set untouched.
#[test]
fn failed_exchange_preserves_prior_pin_set() -> io::Result<()> {
    let scratch = std::env::temp_dir().join(format!("wg-quicker-pin-failure-{}", std::process::id()));
    std::fs::create_dir(&scratch)?;
    let final_dir = scratch.join("final");
    let missing_staging = scratch.join("missing");
    write_pin_fixture(&final_dir, b"old")?;
    assert!(commit_staging_for_test(&missing_staging, &final_dir).is_err());
    for hook_name in HOOK_NAMES {
        assert_eq!(std::fs::read(final_dir.join(hook_name))?, b"old");
    }
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Exact cleanup removes owned names but refuses an unrelated directory entry.
#[test]
fn exact_cleanup_preserves_unrelated_entry() -> io::Result<()> {
    let scratch = std::env::temp_dir().join(format!("wg-quicker-pin-cleanup-{}", std::process::id()));
    write_pin_fixture(&scratch, b"owned")?;
    let unrelated = scratch.join("unrelated");
    std::fs::write(&unrelated, b"keep")?;
    assert!(remove_exact_pin_dir_for_test(&scratch).is_err());
    assert!(unrelated.exists());
    for hook_name in HOOK_NAMES {
        assert!(!scratch.join(hook_name).exists());
    }
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Hex path encoding distinguishes slash boundaries while avoiding reserved dots.
#[test]
fn canonical_pin_path_is_injective_for_slash_boundaries() -> io::Result<()> {
    let scratch = std::env::temp_dir().join(format!("wg-quicker-pin-path-{}", std::process::id()));
    let first = scratch.join("a").join("b__c.scope");
    let second = scratch.join("a__b").join("c.scope");
    std::fs::create_dir_all(&first)?;
    std::fs::create_dir_all(&second)?;
    let first_mapped = pin_dir_for(&first)?;
    let second_mapped = pin_dir_for(&second)?;
    assert_ne!(first_mapped, second_mapped);
    for component in first_mapped.strip_prefix(PIN_ROOT).map_err(io::Error::other)?.components() {
        let text = component.as_os_str().to_str().ok_or_else(|| return io::Error::other("non-UTF-8 key"))?;
        assert!(!text.contains('.'));
        assert!(text.len() <= 200);
    }
    std::fs::remove_dir_all(&scratch)?;
    assert!(Path::new(BPF_FS).is_absolute());
    return Ok(());
}
