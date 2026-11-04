//! What:
//!  Controls of the detached helper launch on a real fork: the helper inherits no descriptor of its launcher (a lock, a pipe, a socket), a
//!  lock the launcher releases is free while the helper lives, and the helper is a function run in a new session of the same process
//!  image, never a second start of the executable or a parse of its command line.
//! Why:
//!  The acceptance target lists "helpers retain no lock, gate, client-lock or pipe descriptor" and "launch is non-recursive and
//!  resource-bounded" as obligations that need a test of the launched process, not of the model.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the helper lists no descriptor beyond 0, 1 and 2, all on /dev/null', ...);
//! ```

use crate::model_fixture::scratch;
use git_policy_cli::commit_flusher::FlusherLock;
use git_policy_cli::commit_helper_launch::launch_detached;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The longest a test waits for its helper to report; the helper is a local process doing a few file operations.
const REPORT_WAIT: Duration = Duration::from_secs(10);

/// A value the launcher stores in this process's memory before the launch; only a forked copy of the process still holds it.
static LAUNCH_MARK: AtomicU64 = AtomicU64::new(0);

/// The value stored in `LAUNCH_MARK`.
const MARK_VALUE: u64 = 0x5eed_cafe_f00d;

/// Waits until `path` exists, up to `REPORT_WAIT`; its text.
fn wait_for(path: &Path) -> String {
    let deadline: Instant = Instant::now() + REPORT_WAIT;
    while Instant::now() < deadline {
        if let Ok(text) = fs::read_to_string(path) {
            return text;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("{} did not appear within {:?}", path.display(), REPORT_WAIT);
}

/// Writes `text` to `path` by rename, so a reader never sees half of it.
fn publish(path: &Path, text: &str) {
    let temporary: PathBuf = path.with_extension("part");
    fs::write(&temporary, text).expect("write the report");
    fs::rename(&temporary, path).expect("publish the report");
}

/// The open descriptors of this process as `number=target` lines, without the one the listing itself uses.
fn open_descriptors() -> Vec<(i32, String)> {
    let own_directory: String = format!("/proc/{}/fd", std::process::id());
    let mut found: Vec<(i32, String)> = Vec::new();
    for entry in fs::read_dir("/proc/self/fd").expect("list descriptors").flatten() {
        let number: i32 = entry.file_name().to_string_lossy().parse().expect("a descriptor number");
        let target: String = fs::read_link(entry.path()).map_or_else(|error| { return format!("unreadable: {}", error); }, |path| { return path.display().to_string(); });
        if target != own_directory {
            found.push((number, target));
        }
    }
    found.sort();
    return found;
}

/// Whether process `pid` exists.
fn alive(pid: i32) -> bool {
    // SAFETY: signal 0 only checks that the process exists.
    return unsafe { libc::kill(pid, 0) } == 0;
}

/// The helper starts with a lane lock, a pipe and a socket pair open in its launcher, all without close-on-exec protection mattering
/// (a fork shares every description): it lists descriptors 0, 1 and 2 on `/dev/null` and nothing else.
#[test]
fn a_launched_helper_inherits_no_descriptor_of_its_launcher() {
    let lane: PathBuf = scratch("helper-descriptors");
    let report: PathBuf = lane.join("report");
    let _lock = FlusherLock::try_acquire(&lane).expect("lock").expect("the lock is free");
    let mut pipe: [libc::c_int; 2] = [0; 2];
    // SAFETY: `pipe` is a live array of two integers the call fills.
    assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
    let (_left, _right) = std::os::unix::net::UnixStream::pair().expect("socket pair");
    let target: PathBuf = report.clone();
    launch_detached(move || {
        let lines: Vec<String> = open_descriptors().into_iter().map(|(number, target)| { return format!("{}={}", number, target); }).collect();
        publish(&target, &lines.join("\n"));
        return 0;
    })
    .expect("launch");
    let text: String = wait_for(&report);
    let listed: Vec<&str> = text.lines().collect();
    assert_eq!(listed, ["0=/dev/null", "1=/dev/null", "2=/dev/null"], "the helper holds only its own streams");
}

/// A lock the launcher releases is free while the helper is alive: the helper did not carry the open file description along.
#[test]
fn a_lane_lock_the_launcher_releases_is_free_while_the_helper_lives() {
    let lane: PathBuf = scratch("helper-lock");
    let started: PathBuf = lane.join("started");
    let release: PathBuf = lane.join("release");
    let lock = FlusherLock::try_acquire(&lane).expect("lock").expect("the lock is free");
    let (started_path, release_path) = (started.clone(), release.clone());
    launch_detached(move || {
        publish(&started_path, &std::process::id().to_string());
        let deadline: Instant = Instant::now() + REPORT_WAIT;
        while !release_path.exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        return 0;
    })
    .expect("launch");
    let helper: i32 = wait_for(&started).trim().parse().expect("the helper's process number");
    assert_ne!(helper as u32, std::process::id(), "the helper is another process");
    drop(lock);
    assert!(alive(helper), "the helper is still running");
    assert!(FlusherLock::try_acquire(&lane).expect("lock").is_some(), "the released lock is free while the helper lives");
    fs::write(&release, b"go").expect("release the helper");
}

/// The helper is the same program image forked (the mark stored before the launch is still there; an `exec` of the executable would have
/// reset it), in a session of its own, not a child of the launcher, and with the launcher's command line untouched.
#[test]
fn a_launched_entry_is_a_function_in_a_new_session_and_not_a_second_start_of_the_program() {
    let lane: PathBuf = scratch("helper-image");
    let report: PathBuf = lane.join("report");
    LAUNCH_MARK.store(MARK_VALUE, Ordering::SeqCst);
    let target: PathBuf = report.clone();
    launch_detached(move || {
        // SAFETY: `getsid` and `getppid` take plain integers.
        let (session, parent) = unsafe { (libc::getsid(0), libc::getppid()) };
        let image: String = fs::read_link("/proc/self/exe").map_or_else(|error| { return error.to_string(); }, |path| { return path.display().to_string(); });
        let command: Vec<u8> = fs::read("/proc/self/cmdline").unwrap_or_default();
        let text: String = format!(
            "pid={}\nparent={}\nsession={}\nmark={}\nimage={}\ncommand={}\n",
            std::process::id(),
            parent,
            session,
            LAUNCH_MARK.load(Ordering::SeqCst),
            image,
            command.iter().map(|byte| { return format!("{:02x}", byte); }).collect::<String>()
        );
        publish(&target, &text);
        return 0;
    })
    .expect("launch");
    let text: String = wait_for(&report);
    let field = |name: &str| -> String {
        return text.lines().find_map(|line| { return line.strip_prefix(&format!("{}=", name)).map(str::to_string); }).unwrap_or_else(|| { panic!("no {} in {}", name, text) });
    };
    let own_command: String = fs::read("/proc/self/cmdline").expect("cmdline").iter().map(|byte| { return format!("{:02x}", byte); }).collect();
    // SAFETY: `getsid` takes a plain integer.
    let own_session: i32 = unsafe { libc::getsid(0) };
    assert_ne!(field("pid"), std::process::id().to_string(), "another process ran the entry");
    assert_ne!(field("parent"), std::process::id().to_string(), "the helper is not the launcher's child");
    assert_ne!(field("session").parse::<i32>().expect("session"), own_session, "the helper leads a session of its own");
    assert_eq!(field("mark"), MARK_VALUE.to_string(), "no program start happened: the launcher's memory is still there");
    assert_eq!(field("image"), fs::read_link("/proc/self/exe").expect("exe").display().to_string());
    assert_eq!(field("command"), own_command, "no command line was built for the helper");
}
