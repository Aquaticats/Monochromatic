//! What:
//!  The fixture the forced act's guest arm runs in:
//!  one file-backed pool,
//!  a checkout dataset mounted inside a private namespace of the arm's own,
//!  a clone mounted at its slot,
//!  holders in sessions and mount namespaces of their own,
//!  and the reads the phases judge by.
//! Why:
//!  Every call the act makes is a mount change,
//!  so no control on a person's machine may run it,
//!  and the arm runs only inside the throwaway `mvm` guest the package's e2e
//!  recipe drives
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Measurements owed",
//!  and this round's brief).
//!  Each phase runs in a forked child that unshares a private mount namespace
//!  first,
//!  so the phase's mounts die with it and no phase can see another's.
//!  A run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = Fixture.fromEnvironment('the forced act'); fixture.buildThePool();
//! const phase = inAPrivateNamespace(() => { fixture.mountTheClone(); … });
//! ```

use crate::worktree_clone_forced_removal::receiver_closure::{MountRow, NamespaceTable};
use crate::worktree_clone_forced_removal::the_copy_taken_out_of_a_namespace::{row_of, rows_of, topmost_at};
use std::io::Read;
use std::os::fd::{FromRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The variable naming the guest this arm may run in.
pub(crate) const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_FORCED_ACT_GUEST";
/// The variable naming the file-backed pool the arm builds.
pub(crate) const POOL_VARIABLE: &str = "CLI_GIT_ZFS_FORCED_ACT_POOL";
/// The variable naming the OpenZFS prefix the guest carries.
pub(crate) const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_FORCED_ACT_PREFIX";
/// The variable naming the folder the pool's image lives under.
pub(crate) const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_FORCED_ACT_ROOT";

/// The guest directory the fixture lives in when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-forced-act-fixture";
/// The pool the arm builds when the driver names none.
const DEFAULT_POOL: &str = "forcedact";
/// The image the pool stands on,
/// in gibibytes.
const POOL_IMAGE_SIZE: &str = "2G";
/// The slot the removal is about.
pub(crate) const SLOT: &str = "one";
/// The most bytes one phase's line may take.
const MAXIMUM_PHASE_LINE_BYTES: usize = 8192;
/// What:
///  The guest's name, its pool and the paths its commands come from.
/// Why:
///  The arm refuses to run anywhere but the guest its own variable names,
///  and every command it makes needs the guest's OpenZFS `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fixture = { guest; pool; prefix; root };
/// ```
pub(crate) struct Fixture {
    /// The guest's host name.
    pub(crate) guest: String,
    /// The pool the arm builds and destroys.
    pub(crate) pool: String,
    /// The OpenZFS prefix whose `sbin` and `bin` the commands come from.
    pub(crate) prefix: PathBuf,
    /// The folder the pool's image lives under.
    pub(crate) root: PathBuf,
}

impl Fixture {
    /// The fixture the environment names,
    /// or nothing on a host run,
    /// where no pool command may be made.
    pub(crate) fn from_environment(test: &str) -> Option<Fixture> {
        let named: std::ffi::OsString = std::env::var_os(GUEST_VARIABLE)?;
        let guest: String = named.to_string_lossy().into_owned();
        let host: Output = Command::new("hostname").output().expect("the guest has `hostname`");
        let here: String = String::from_utf8_lossy(host.stdout.as_slice()).trim().to_string();
        if here != guest {
            panic!("{test}: {GUEST_VARIABLE} names the guest `{guest}` and this machine calls itself `{here}`, so the arm would reach the wrong pool");
        }
        return Some(Fixture {
            guest,
            pool: std::env::var(POOL_VARIABLE).unwrap_or_else(|_error| return String::from(DEFAULT_POOL)),
            prefix: PathBuf::from(std::env::var(PREFIX_VARIABLE).unwrap_or_else(|_error| return String::from("/opt/openzfs-2.4.4"))),
            root: PathBuf::from(std::env::var(ROOT_VARIABLE).unwrap_or_else(|_error| return String::from(DEFAULT_ROOT))),
        });
    }

    /// The `PATH` the guest's OpenZFS userspace needs.
    pub(crate) fn search_path(&self) -> String {
        return format!("{}:{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", self.prefix.join("sbin").display(), self.prefix.join("bin").display());
    }

    /// One command,
    /// its answer kept whichever way it went.
    pub(crate) fn ask(&self, program: &str, arguments: &[&str]) -> (bool, String) {
        let output: Result<Output, std::io::Error> = Command::new(program).args(arguments).env("PATH", self.search_path()).output();
        return match output {
            Ok(answer) => (
                answer.status.success(),
                format!("{}{}", String::from_utf8_lossy(answer.stdout.as_slice()).trim(), String::from_utf8_lossy(answer.stderr.as_slice()).trim()),
            ),
            Err(error) => (false, format!("`{program}` could not start: {error}")),
        };
    }

    /// One pool command that must work,
    /// whose failure names the command and what it wrote.
    pub(crate) fn run(&self, program: &str, arguments: &[&str]) -> String {
        let (worked, said) = self.ask(program, arguments);
        assert!(worked, "`{program} {}` failed: {said}", arguments.join(" "));
        return said;
    }

    /// The full name of one dataset in this pool.
    pub(crate) fn named(&self, below: &str) -> String {
        return format!("{}/{below}", self.pool);
    }

    /// The checkout's dataset name,
    /// which is the parent mount of the slot and of way 7's directory.
    pub(crate) fn checkout(&self) -> String {
        return self.named("checkout");
    }

    /// The clone's dataset name.
    pub(crate) fn clone(&self) -> String {
        return self.named("clones/one");
    }

    /// Where the checkout is mounted inside a phase's namespace.
    pub(crate) fn checkout_path(&self) -> PathBuf {
        return self.root.join("checkout");
    }

    /// What:
    ///  Build the pool,
    ///  the checkout's own dataset and its snapshot,
    ///  which every phase clones from.
    /// Why:
    ///  Each phase destroys its clone to show the head destroy answers `gone`,
    ///  so the snapshot is what a later phase stands its clone on.
    pub(crate) fn build_the_pool(&self) {
        self.teardown();
        std::fs::create_dir_all(self.root.as_path()).expect("the fixture root");
        let image: PathBuf = self.root.join("pool.img");
        let image_text: &str = image.to_str().expect("the image path is text");
        self.run("truncate", &["--size", POOL_IMAGE_SIZE, image_text]);
        self.run("zpool", &["create", "-f", self.pool.as_str(), image_text]);
        self.run("zfs", &["create", "-o", "mountpoint=legacy", self.checkout().as_str()]);
        // The container every phase's clone is made under, which no clone may be created without.
        self.run("zfs", &["create", "-o", "mountpoint=none", "-o", "canmount=off", self.named("clones").as_str()]);
        self.run("zfs", &["snapshot", format!("{}@origin", self.checkout()).as_str()]);
        self.run("mkdir", &["--parents", self.checkout_path().to_str().expect("the checkout path is text")]);
    }

    /// Destroy everything this arm built,
    /// whatever a phase left.
    pub(crate) fn teardown(&self) {
        let _answer: (bool, String) = self.ask("zpool", &["destroy", "-f", self.pool.as_str()]);
        let image: PathBuf = self.root.join("pool.img");
        let _removed: std::io::Result<()> = std::fs::remove_file(image.as_path());
    }

    /// What:
    ///  Destroy a clone a finished phase left and make this phase's own,
    ///  with no mount point of its own,
    ///  so the one mount the phase makes stands for the clone.
    /// Why:
    ///  A phase's last act is the head destroy,
    ///  and a later phase must stand on the same snapshot with a fresh clone.
    pub(crate) fn a_fresh_clone(&self) {
        let _gone: (bool, String) = self.ask("zfs", &["destroy", "-r", self.clone().as_str()]);
        self.run("zfs", &["clone", "-o", "mountpoint=none", format!("{}@origin", self.checkout()).as_str(), self.clone().as_str()]);
    }
}

/// What:
///  One phase's own line,
///  which the recipe greps for.
/// Why:
///
/// In TS you'd write (pseudocode):
/// ```ts
/// phaseLine('path-holder-asked', { control: 'EBUSY', end: 'exited-after-the-ask' });
/// ```
pub(crate) fn phase_line(phase: &str, fields: &[(&str, &str)]) -> String {
    let mut line: String = format!("FORCEDACT phase={phase}");
    for (key, value) in fields {
        line.push(' ');
        line.push_str(key);
        line.push('=');
        line.push_str(value);
    }
    return line;
}

/// What:
///  Run one phase in a forked child that unshares a private mount namespace
///  first,
///  and answer the line it wrote.
/// Why:
///  A phase's mounts must die with it,
///  and the child is single-threaded,
///  so a mount change there can never reach the arm's own or the guest's
///  namespaces.
///  The child ends at once and never returns through the arm's frames,
///  and the parent waits for it on every path.
pub(crate) fn in_a_private_namespace(phase: &str, body: &dyn Fn() -> String) -> String {
    let started: std::time::Instant = std::time::Instant::now();
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: `ends` is a live two-element array the call fills with the two descriptors it makes.
    let piped: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(piped, 0, "the phase's report pipe: {}", std::io::Error::last_os_error());
    let read_end: RawFd = ends[0];
    let write_end: RawFd = ends[1];
    // SAFETY: `fork` takes no argument and the child below returns through `_exit` only.
    let child: libc::pid_t = unsafe { libc::fork() };
    assert!(child >= 0, "the phase's child: {}", std::io::Error::last_os_error());
    if child == 0 {
        // SAFETY: the read end is the parent's and this child closes its own copy exactly once.
        unsafe { libc::close(read_end) };
        match unshare_a_private_mount_namespace() {
            Ok(()) => {}
            Err(error) => {
                write_the_line(write_end, phase_line(phase, &[("refused", "unshare"), ("errno", error.to_string().as_str())]).as_str());
                end_now(1);
            }
        }
        // A panic in the child would unwind through the arm's own frames and run a second copy of the arm, so the
        // child catches it, writes the failure and ends at once.
        let outcome: std::thread::Result<String> = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        match outcome {
            Ok(line) => {
                write_the_line(write_end, line.as_str());
                end_now(0);
            }
            Err(_payload) => {
                write_the_line(write_end, phase_line(phase, &[("failed", "panic")]).as_str());
                end_now(1);
            }
        }
    }
    // SAFETY: the write end is the child's and this parent closes its own copy exactly once.
    unsafe { libc::close(write_end) };
    // SAFETY: the read end is this call's own and nothing else reads it.
    let mut pipe: std::fs::File = unsafe { std::fs::File::from_raw_fd(read_end) };
    let mut bytes: Vec<u8> = Vec::new();
    let _read: std::io::Result<usize> = Read::by_ref(&mut pipe).take(MAXIMUM_PHASE_LINE_BYTES as u64).read_to_end(&mut bytes);
    drop(pipe);
    let mut status: libc::c_int = 0;
    let waited: libc::pid_t = loop {
        // SAFETY: the status word is this call's own and `waitpid` writes it.
        let answered: libc::pid_t = unsafe { libc::waitpid(child, &mut status, 0) };
        if answered >= 0 {
            break answered;
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        assert_eq!(error.raw_os_error(), Some(libc::EINTR), "the phase's child is waited for: {error}");
    };
    assert_eq!(waited, child);
    let line: String = String::from_utf8_lossy(bytes.as_slice()).trim().to_string();
    assert!(libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0, "the phase `{phase}` ended with status {status} and wrote: {line}");
    assert!(!line.is_empty(), "the phase `{phase}` wrote no line");
    // What the phase took, for the round's record: a measurement of this run and never a value the release ships.
    let took: u128 = started.elapsed().as_millis();
    return format!("{line} ms={took}");
}

/// What:
///  Make this child's mount namespace its own,
///  with every mount private.
/// Why:
///  A mount that propagated would reach the guest's own table,
///  which is the one thing this arm must never do.
fn unshare_a_private_mount_namespace() -> Result<(), std::io::Error> {
    // SAFETY: `unshare` reads one flag word and touches no memory of ours.
    let made: libc::c_int = unsafe { libc::unshare(libc::CLONE_NEWNS) };
    if made != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let root: std::ffi::CString = std::ffi::CString::new("/").expect("no NUL in the root path");
    // SAFETY: the root path outlives the call and the flag word asks for one private tree.
    let changed: libc::c_int = unsafe { libc::mount(std::ptr::null(), root.as_ptr(), std::ptr::null(), libc::MS_REC | libc::MS_PRIVATE, std::ptr::null()) };
    if changed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// Write one phase's line to the pipe,
/// and say nothing when the parent has gone.
fn write_the_line(descriptor: RawFd, line: &str) {
    let mut bytes: &[u8] = line.as_bytes();
    while !bytes.is_empty() {
        // SAFETY: the bytes are this call's own and the descriptor is the child's write end.
        let written: libc::ssize_t = unsafe { libc::write(descriptor, bytes.as_ptr().cast::<libc::c_void>(), bytes.len()) };
        if written <= 0 {
            return;
        }
        bytes = &bytes[usize::try_from(written).unwrap_or(0)..];
    }
}

/// End this child at once,
/// never through the arm's frames.
pub(crate) fn end_now(status: libc::c_int) -> ! {
    // SAFETY: `_exit` ends this process at once and takes no argument but its status.
    unsafe { libc::_exit(status) }
}

/// What:
///  The mount table of the namespace this phase is in.
/// Why:
///  Every phase judges the kernel's own rows,
///  not what the act said it did.
pub(crate) fn the_table() -> Vec<MountRow> {
    let text: String = std::fs::read_to_string("/proc/self/mountinfo").expect("this namespace's own table");
    return rows_of(text.as_str());
}

/// What:
///  One namespace's table,
///  read through that namespace's own file.
/// Why:
///  A holder's copy is a fact about the holder's namespace,
///  and reading it from outside the namespace is what shows the copy is there
///  or gone without entering it.
pub(crate) fn the_table_of_the_process(pid: i32) -> NamespaceTable {
    let path: String = format!("/proc/{pid}/mountinfo");
    let namespace: String = format!("/proc/{pid}/ns/mnt");
    let text: String = std::fs::read_to_string(path.as_str()).expect("a holder's own table is readable to the guest's root");
    let facts: std::fs::Metadata = std::fs::metadata(namespace.as_str()).expect("a holder's namespace link");
    use std::os::unix::fs::MetadataExt;
    return NamespaceTable { namespace: format!("mnt:[{}]", facts.ino()), rows: rows_of(text.as_str()), at_the_slot: None };
}

/// What:
///  The row one mount id names in one table,
///  or a panic naming what the table held instead.
pub(crate) fn a_row(rows: &[MountRow], mount_id: u64) -> Option<MountRow> {
    return row_of(rows, mount_id);
}

/// What:
///  Whether one table holds a mount of the dataset at the slot.
/// Why:
///  The two facts every phase judges are whether the copy is there and whether
///  it is gone,
///  and both are asked of the kernel's rows.
pub(crate) fn holds_the_copy_at(rows: &[MountRow], at: &Path, dataset: &str) -> bool {
    let Some(row) = topmost_at(rows, at) else {
        return false;
    };
    return row.filesystem == "zfs" && row.source == dataset;
}

/// What:
///  Make the checkout a peer group of its own inside this namespace,
///  which is the shape systemd leaves a unit's parent.
/// Why:
///  Way 8 is taken only where the parent shares with a group of its own,
///  so a phase that measures it must make that shape.
pub(crate) fn make_the_checkout_shared(checkout: &Path) -> Result<(), std::io::Error> {
    let path: std::ffi::CString = std::ffi::CString::new(checkout.as_os_str().as_encoded_bytes()).expect("no NUL in the checkout path");
    // SAFETY: the path outlives the call and the flag word asks for sharing on one mount.
    let changed: libc::c_int = unsafe { libc::mount(std::ptr::null(), path.as_ptr(), std::ptr::null(), libc::MS_SHARED, std::ptr::null()) };
    if changed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Die if one pool command may not run,
///  naming the guest and the release that answered.
/// Why:
///  Every sentence of the arm is about a kernel and an OpenZFS release,
///  and a record that did not name them could not be compared with another
///  run.
pub(crate) fn the_versions(fixture: &Fixture) -> String {
    let (_, kernel) = fixture.ask("uname", &["-r"]);
    let (_, zfs) = fixture.ask("zfs", &["version"]);
    return format!("guest={} kernel={kernel} zfs={zfs}", fixture.guest);
}
