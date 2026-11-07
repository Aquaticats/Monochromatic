//! What:
//! A delegated control group (cgroup v2) as the boundary of everything a bounded run starts:
//! a transient user scope that the manager makes with `Delegate=yes`.
//! Why:
//! Round 7 (sixth review 6a R1 to R3, 6b2 2, 6c R2):
//! a process group, and a child subreaper with a `/proc` scan, failed the one proof that matters,
//! that every writer a command started has stopped.
//! A control group is inherited by every child, also by those that call `setsid` or fork twice,
//! it keeps its members when this program dies,
//! `cgroup.events` says `populated 0` only when no thread is left,
//! and a recovery can ask the same question about a scope the journal recorded.
//! Round 8 (contract `doc/planning/cargo-shadow-guarantees.md`, "The spawn route"):
//! no thread of this program writes a `cgroup.procs` file.
//! The program is started as `systemd-run --user --scope --unit=<name> --property=Delegate=yes --collect -- <program>`,
//! and the manager moves it into the scope before it runs.
//! Round 7's self-move (the wrapper entering the group around a `spawn`) captured other threads' children and could kill the wrapper
//! itself (7a F1, F2, F3) and is gone.
//! The limits:
//! `cgroup.kill` alone does not end a thread group whose leader already exited (measured),
//! so the stop also signals each listed process through a pidfd;
//! a writer that leaves the scope is outside the contract (assumption a).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group.ts (Linux only)
//! export function detectRoute(settings: RouteSettings): Route  // throws GroupUnavailable
//! export class ControlGroup {
//!   static plan(route: Route): ControlGroup
//!   launcher(route: Route, command: Command): Command
//!   stop(boundMs: number): boolean
//! }
//! export function query(record: ScopeRecord): 'gone' | 'alive' | { unknown: string }
//! ```

/// Imports `std::fmt` for this file.
use std::fmt;

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::io::{Read, Write}` for this file.
use std::io::{Read, Write};

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::os::unix::fs::{FileExt, MetadataExt}` for this file.
use std::os::unix::fs::{FileExt, MetadataExt};

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports `std::sync::Arc` for this file.
use std::sync::Arc;

/// Imports `std::sync::atomic::{AtomicBool, Ordering}` for this file.
use std::sync::atomic::{AtomicBool, Ordering};

/// Imports `std::process::{Command, Stdio}` for this file.
use std::process::{Command, Stdio};

/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};

/// Imports the limits the scope of a run carries from its caller.
use crate::inherited_limits::{InheritedLimits, caller_limits};

/// Imports `rustix::event::{PollFd, PollFlags, Timespec, poll}` for this file.
use rustix::event::{PollFd, PollFlags, Timespec, poll};

/// Imports `rustix::fs::{AtFlags, CWD, StatxFlags, statx}` for this file.
use rustix::fs::{AtFlags, CWD, StatxFlags, statx};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// Imports `rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal}` for this file.
use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

/// Where the unified control group hierarchy is mounted.
pub(crate) const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// The file that names the control group of this process.
const OWN_CGROUP_FILE: &str = "/proc/self/cgroup";

/// The file system type number of a cgroup v2 mount (`CGROUP2_SUPER_MAGIC`, ASCII "cgrp"), from `statfs(2)`.
const CGROUP2_MAGIC: u64 = 0x6367_7270;

/// `STATX_MNT_ID_UNIQUE` of `linux/stat.h` (Linux 6.8): the mount id that is never reused while the system runs.
/// `rustix` 1.1.6 has the field and not the flag, so the bit is spelled here.
const STATX_MNT_ID_UNIQUE: u32 = 0x4000;

/// How long the stop waits for a change of `cgroup.events` before it looks for members to signal again.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open;
/// measurement beside it:
/// on 2026-11-04 `populated 0` followed `cgroup.kill` within 5 ms for ordinary members and 1 ms after a signal to a zombie leader,
/// so a wake-up every 50 ms only matters for a member that appears or survives the first signals.
const STRAGGLER_INTERVAL: Duration = Duration::from_millis(50);

/// How long the discovery of the manager's slice may take.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open;
/// measurement beside it:
/// `systemctl --user show app.slice` answered in a few milliseconds on 2026-11-04, so ten seconds only ends a manager that does
/// not answer.
const DISCOVERY_BOUND: Duration = Duration::from_secs(10);

/// How often the discovery asks whether `systemctl` has ended.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open; not a measurement.
const DISCOVERY_POLL: Duration = Duration::from_millis(5);

/// How many random bytes make a scope name unique.
/// 16 bytes are 128 bits, so a name repeated by chance is not a case to plan for.
const TOKEN_BYTES: usize = 16;

/// Why no control group could be had,
/// in words that name the missing capability.
#[derive(Debug)]
pub(crate) struct GroupUnavailable {
    /// The missing capability and what was tried.
    pub(crate) reason: String,
}

/// How the reason reads to a person.
impl fmt::Display for GroupUnavailable {
    /// What:
    /// `fn fmt(&self, formatter)` writes the reason.
    /// Why:
    /// The one-line notice of the weaker path is this text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.reason);
    }
}

/// How a path is seen by this process: the mount it is on and its inode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct View {
    /// The mount id (`STATX_MNT_ID`), which differs in another mount namespace.
    pub(crate) mount: u64,
    /// The inode number.
    pub(crate) inode: u64,
}

/// How a view reads in a journal line and in a message.
impl fmt::Display for View {
    /// What:
    /// `fn fmt(&self, formatter)` writes `mount:inode`.
    /// Why:
    /// Both numbers sit in one field of a record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}:{}", self.mount, self.inode);
    }
}

/// What:
/// `pub(crate) fn view_of(path: &Path) -> io::Result<View>` asks the kernel for the mount id and the inode of a path.
/// Why:
/// A path means the same thing in a recovery as in the run only when both are equal (contract assumption c).
/// A kernel without `STATX_MNT_ID` (before Linux 5.8) is an error, never a view of zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function viewOf(path: string): View
/// ```
pub(crate) fn view_of(path: &Path) -> io::Result<View> {
    // The unique id is asked for alone first (a kernel before 6.8 ignores the bit); the reusable one is the fallback.
    let unique_bit: StatxFlags = StatxFlags::from_bits_retain(STATX_MNT_ID_UNIQUE);
    let unique = statx(CWD, path, AtFlags::empty(), unique_bit | StatxFlags::INO).map_err(io::Error::from)?;
    if StatxFlags::from_bits_retain(unique.stx_mask).contains(unique_bit) && StatxFlags::from_bits_retain(unique.stx_mask).contains(StatxFlags::INO) {
        return Ok(View { mount: unique.stx_mnt_id, inode: unique.stx_ino });
    }
    let found = statx(CWD, path, AtFlags::empty(), StatxFlags::MNT_ID | StatxFlags::INO).map_err(io::Error::from)?;
    let reported: StatxFlags = StatxFlags::from_bits_retain(found.stx_mask);
    if !reported.contains(StatxFlags::MNT_ID) || !reported.contains(StatxFlags::INO) {
        return Err(io::Error::other(format!("the kernel did not report the mount id and inode of {}", path.display())));
    }
    return Ok(View { mount: found.stx_mnt_id, inode: found.stx_ino });
}

/// What the discovery of the route uses,
/// so a test can make the host look like one that lacks a capability.
#[derive(Clone, Debug)]
pub(crate) struct RouteSettings {
    /// The program that makes the scope.
    pub(crate) systemd_run: String,
    /// The program that asks the manager where the slice is.
    pub(crate) systemctl: String,
    /// Where the cgroup v2 hierarchy is mounted.
    pub(crate) cgroup_root: PathBuf,
    /// The slice the scopes are made in.
    pub(crate) slice: String,
    /// Variables set (`Some`) or removed (`None`) for the program that asks the manager.
    pub(crate) environment: Vec<(String, Option<String>)>,
    /// The arguments of the question to the manager when a test replaces the ordinary ones (`None` is the ordinary question).
    pub(crate) question: Option<Vec<String>>,
    /// How long the question may take.
    pub(crate) discovery_bound: Duration,
    /// The file that names this process's own control group.
    pub(crate) own_cgroup_file: PathBuf,
}

/// The settings of an ordinary host.
impl Default for RouteSettings {
    /// What:
    /// `fn default()` is the ordinary settings:
    /// `systemd-run`, `systemctl`, `/sys/fs/cgroup` and `app.slice`.
    /// Why:
    /// `app.slice` is where the user manager puts application scopes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const defaults: RouteSettings = { ... }
    /// ```
    fn default() -> RouteSettings {
        return RouteSettings {
            systemd_run: "systemd-run".to_string(),
            systemctl: "systemctl".to_string(),
            cgroup_root: PathBuf::from(CGROUP_ROOT),
            slice: "app.slice".to_string(),
            environment: Vec::new(),
            question: None,
            discovery_bound: DISCOVERY_BOUND,
            own_cgroup_file: PathBuf::from(OWN_CGROUP_FILE),
        };
    }
}

/// What a host can give: the way to a delegated scope.
#[derive(Clone, Debug)]
pub(crate) struct Route {
    /// The program that makes the scope.
    pub(crate) systemd_run: String,
    /// The slice the scopes are made in.
    pub(crate) slice: String,
    /// The directory of the slice in the cgroup file system,
    /// as the manager reports it.
    pub(crate) slice_dir: PathBuf,
    /// The inode of this process's cgroup namespace.
    pub(crate) namespace: u64,
    /// How the slice directory is seen now.
    pub(crate) slice_view: View,
    /// The limits of the caller's control group that every scope of this route carries.
    pub(crate) limits: InheritedLimits,
}

/// Who ran: the machine and the boot of the kernel that held the processes of a run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Identity {
    /// `/proc/sys/kernel/random/boot_id`: a new value for every boot of a kernel.
    pub(crate) boot: String,
    /// `/etc/machine-id`: the same value on every boot of one installation.
    pub(crate) machine: String,
}

#[cfg(test)]
thread_local! {
    /// The identity this thread reads instead of the machine's, so a test can be a recovery after a reboot or on another machine.
    static IDENTITY_OVERRIDE: std::cell::RefCell<Option<Result<Identity, String>>> = const { std::cell::RefCell::new(None) };
}

/// What:
/// `pub(crate) fn use_identity_for_test(identity: Option<Result<Identity, String>>)` makes this thread read `identity` as its own.
/// Why:
/// A recovery after a reboot, on another machine, or with an unreadable identity is made without rebooting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function useIdentityForTest(identity?: Identity | Error): void
/// ```
#[cfg(test)]
pub(crate) fn use_identity_for_test(identity: Option<Result<Identity, String>>) {
    IDENTITY_OVERRIDE.with(|slot| { *slot.borrow_mut() = identity; });
}

/// What:
/// `fn is_hex_id(text: &str, groups: &[usize]) -> bool` says whether `text` is lowercase hexadecimal in groups of exactly the given
/// lengths, separated by single hyphens, and not all zero.
/// Why:
/// The kernel writes the boot id as a lowercase `8-4-4-4-12` UUID, and the machine id is 32 lowercase hexadecimal digits
/// (the same formats systemd reads: `ID128_FORMAT_UUID` and plain);
/// anything else is not an identity, and a text that is not one cannot show that a boot or a machine differs (round 10, 9a D2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isHexId(text: string, groups: number[]): boolean
/// ```
fn is_hex_id(text: &str, groups: &[usize]) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != groups.len() {
        return false;
    }
    let shaped: bool = parts.iter().zip(groups.iter()).all(|(part, length)| { return part.len() == *length && part.bytes().all(|byte| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte); }); });
    return shaped && text.bytes().any(|byte| { return byte != b'0' && byte != b'-'; });
}

/// The group lengths of the boot id (`8-4-4-4-12`).
const BOOT_ID_GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

/// The group length of the machine id (one run of 32 digits).
const MACHINE_ID_GROUPS: [usize; 1] = [32];

/// The kernel file that holds the boot id.
pub(crate) const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// The file system type number of procfs (`PROC_SUPER_MAGIC`), from `statfs(2)`.
const PROC_MAGIC: u64 = 0x9fa0;

/// The machine id file.
const MACHINE_ID_PATH: &str = "/etc/machine-id";

/// The methods of the type above.
impl Identity {
    /// What:
    /// `pub(crate) fn is_well_formed(&self) -> bool` says whether both ids have the format of a boot id and a machine id.
    /// Why:
    /// A malformed id differs from every real one and would read as a reboot or as another machine (9a D2);
    /// the check applies to the recorded and to the current identity alike.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isWellFormed(): boolean
    /// ```
    pub(crate) fn is_well_formed(&self) -> bool {
        return is_hex_id(&self.boot, &BOOT_ID_GROUPS) && is_hex_id(&self.machine, &MACHINE_ID_GROUPS);
    }
}

/// What:
/// `fn is_view_line(line: &str) -> bool` says whether a line is exactly `view <mount>:<inode>` with two decimal numbers.
/// Why:
/// The header of the record is part of the format the identity is read from (round 12, 11a); a leading sign or space is not a number
/// of the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isViewLine(line: string): boolean
/// ```
fn is_view_line(line: &str) -> bool {
    let Some((mount_text, inode_text)) = line.strip_prefix("view ").and_then(|rest| { return rest.split_once(':'); }) else {
        return false;
    };
    return [mount_text, inode_text].iter().all(|number| { return !number.is_empty() && number.bytes().all(|byte| { return byte.is_ascii_digit(); }) && number.parse::<u64>().is_ok(); });
}

/// What:
/// `pub(crate) fn read_identity_text(text: &str) -> Option<Identity>` reads the `boot` and `machine` lines of a journal's view record.
/// Why:
/// A missing, repeated, unknown or malformed identity is uncertainty, never a match and never a different boot (round 10, 9a D2):
/// the record is the first line `view <mount>:<inode>`, then exactly one `boot <id>` and one `machine <id>`, each line exactly in
/// that form with a well-formed id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readIdentityText(text: string): Identity | undefined
/// ```
pub(crate) fn read_identity_text(text: &str) -> Option<Identity> {
    let mut boot_text: Option<String> = None;
    let mut machine_text: Option<String> = None;
    let mut lines = text.lines();
    if !is_view_line(lines.next()?) {
        return None;
    }
    for line in lines {
        if let Some(found) = line.strip_prefix("boot ") {
            if boot_text.replace(found.to_string()).is_some() {
                return None;
            }
        } else {
            // Any line that is neither a boot nor a machine line makes the record unusable.
            let found: &str = line.strip_prefix("machine ")?;
            if machine_text.replace(found.to_string()).is_some() {
                return None;
            }
        }
    }
    let identity: Identity = Identity { boot: boot_text?, machine: machine_text? };
    return identity.is_well_formed().then_some(identity);
}

/// What:
/// `pub(crate) fn read_authoritative_boot_id(path: &Path) -> Result<String, String>` opens the boot id file once and returns its text
/// only if the opened object is the kernel's own file and not one put over it.
/// Why:
/// A bind mount can replace `/proc/sys/kernel/random/boot_id` without a reboot (measured in a disposable user and mount namespace on
/// 2026-11-04T08:36:57-05:00), and `systemd-nspawn` does it on purpose to give each container its own id
/// (`src/nspawn/nspawn.c`, `setup_boot_id`).
/// The kernel serves the file from procfs; a file of another file system, or a file that is itself the root of a mount
/// (`STATX_ATTR_MOUNT_ROOT`), is a replacement.
/// A kernel that does not say whether the file is a mount root is an error, not a pass (round 10, 9a section 4).
/// Round 12 (11a): the file is opened without following a final symbolic link (a link to a genuine procfs file is a replacement that
/// the checks of the target would pass), and the checks and the read are made on the one descriptor, so a mount that appears after
/// the checks cannot change what is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readAuthoritativeBootId(path: string): string  // throws the reason
/// ```
pub(crate) fn read_authoritative_boot_id(path: &Path) -> Result<String, String> {
    let opened: OwnedFd = rustix::fs::open(path, rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
        .map_err(|error| { return format!("{} cannot be opened without following a link ({error})", path.display()); })?;
    let found = rustix::fs::fstatfs(&opened).map_err(|error| { return format!("{} cannot be examined ({error})", path.display()); })?;
    if u64::try_from(found.f_type).unwrap_or(0) != PROC_MAGIC {
        return Err(format!("{} is not served by procfs, so it may be a file put over the kernel's", path.display()));
    }
    let attributes = statx(&opened, "", AtFlags::EMPTY_PATH, StatxFlags::TYPE).map_err(|error| { return format!("{} cannot be examined ({error})", path.display()); })?;
    if !attributes.stx_attributes_mask.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) {
        return Err("the kernel does not say whether the boot id file is the root of a mount".to_string());
    }
    if attributes.stx_attributes.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) {
        return Err(format!("{} is the root of a mount of its own, so it was put over the kernel's file", path.display()));
    }
    let mut text: String = String::new();
    fs::File::from(opened).read_to_string(&mut text).map_err(|error| { return format!("the boot id cannot be read ({error})"); })?;
    return Ok(text);
}

/// What:
/// `pub(crate) fn read_current_identity(boot_path: &Path, machine_path: &Path) -> Result<Identity, String>` reads and validates the
/// boot id and the machine id from the given files.
/// Why:
/// The boot file must pass the authority check (made on the descriptor that is read) and both ids must have their format; the paths are parameters so a test can show
/// each refusal on a file it controls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readCurrentIdentity(bootPath: string, machinePath: string): Identity  // throws the reason
/// ```
pub(crate) fn read_current_identity(boot_path: &Path, machine_path: &Path) -> Result<Identity, String> {
    let single_line = |text: String| -> String { return text.strip_suffix('\n').unwrap_or(&text).to_string(); };
    let boot: String = single_line(read_authoritative_boot_id(boot_path)?);
    let machine: String = single_line(fs::read_to_string(machine_path).map_err(|error| { return format!("the machine id cannot be read ({error})"); })?);
    let identity: Identity = Identity { boot, machine };
    if !identity.is_well_formed() {
        return Err("the boot id or the machine id is empty or not in the format of an id".to_string());
    }
    return Ok(identity);
}

/// What:
/// `pub(crate) fn current_identity() -> Result<Identity, String>` reads the boot id and the machine id of this system.
/// Why:
/// Both are recorded with a journal, so a recovery can tell a reboot of the same machine from another machine.
/// Either unreadable, malformed, or not the kernel's own (`read_authoritative_boot_id`) is an error, and the caller keeps the refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function currentIdentity(): Identity  // throws the reason
/// ```
pub(crate) fn current_identity() -> Result<Identity, String> {
    #[cfg(test)]
    if let Some(chosen) = IDENTITY_OVERRIDE.with(|slot| { return slot.borrow().clone(); }) {
        return chosen;
    }
    return read_current_identity(Path::new(BOOT_ID_PATH), Path::new(MACHINE_ID_PATH));
}

/// What `launcher_state` found about the process a launcher record names.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LauncherState {
    /// No process with that id and start time exists in the namespace of the record: it can never join the scope.
    Gone,
    /// The process exists (a zombie that is not reaped yet counts, because only its reaping shows it ended).
    Alive,
    /// The question cannot be answered, with the reason.
    Unknown(
        /// Why.
        String,
    ),
}

/// What:
/// `pub(crate) fn pid_namespace_id() -> io::Result<u64>` is the inode number of this process's process id namespace.
/// Why:
/// A recorded process id means the same process only inside the namespace it was recorded in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pidNamespaceId(): number
/// ```
pub(crate) fn pid_namespace_id() -> io::Result<u64> {
    return Ok(fs::metadata("/proc/self/ns/pid")?.ino());
}

/// What `start_of` found about a process id.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum StartOf {
    /// No process has this id in this process's namespace.
    Gone,
    /// The process, with its start time, read through a procfs that numbers it with this id.
    Process(
        /// The start time in clock ticks since boot.
        u64,
    ),
    /// The question cannot be answered, with the reason.
    Unknown(
        /// Why.
        String,
    ),
}

/// What:
/// `pub(crate) fn missing_entry_verdict(number: i64, pid: i32, reason: &str) -> StartOf` reads the number that the procfs gives a pidfd's process
/// after the process's directory could not be read.
/// Why:
/// Only the value the kernel prints for a process that has ended and been reaped (`-1`) shows that the process is gone;
/// the same number as before means the process exists and procfs hides it, and any other number (another positive id, or `0` for a
/// process that this procfs does not show) says nothing about whether it ended (round 13, 12a).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingEntryVerdict(number: number, pid: number, reason: string): StartOf
/// ```
pub(crate) fn missing_entry_verdict(number: i64, pid: i32, reason: &str) -> StartOf {
    return match number {
        n if n == i64::from(pid) => StartOf::Unknown(format!("process {pid} exists but /proc/{pid}/stat cannot be read ({reason})")),
        -1 => StartOf::Gone,
        other => StartOf::Unknown(format!("process {pid} is numbered {other} by the procfs after its directory could not be read ({reason}), which does not show that it ended")),
    };
}

/// The procfs, opened once: every numeric lookup of one question goes through this descriptor.
struct ProcRoot {
    /// The root directory of the procfs mount.
    directory: OwnedFd,
    /// This process's own number in that procfs.
    own: i32,
}

/// What:
/// `fn open_proc_root() -> Result<ProcRoot, String>` opens `/proc` once and shows that it is the root of a procfs mount that numbers
/// this process as the caller's own namespace does.
/// Why:
/// The observation that a pidfd's process has a given number and the read of that process's stat file must come from one procfs,
/// so both go through this one descriptor (round 13, 12a); a procfs that numbers the caller differently is another namespace's.
/// This is a guard that costs a few system calls; a program that deliberately changes mounts or `/proc` while a run or a recovery
/// is in progress is outside the contract (assumption a, amended in round 13).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openProcRoot(): ProcRoot  // throws the reason
/// ```
fn open_proc_root() -> Result<ProcRoot, String> {
    let directory: OwnedFd = rustix::fs::open("/proc", rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
        .map_err(|error| { return format!("/proc cannot be opened ({error})"); })?;
    let found = rustix::fs::fstatfs(&directory).map_err(|error| { return format!("/proc cannot be examined ({error})"); })?;
    if u64::try_from(found.f_type).unwrap_or(0) != PROC_MAGIC {
        return Err("/proc is not a procfs".to_string());
    }
    let attributes = statx(&directory, "", AtFlags::EMPTY_PATH, StatxFlags::TYPE).map_err(|error| { return format!("/proc cannot be examined ({error})"); })?;
    if !attributes.stx_attributes_mask.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) || !attributes.stx_attributes.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) {
        return Err("/proc is not the root of a procfs mount".to_string());
    }
    let own: i32 = rustix::process::getpid().as_raw_nonzero().get();
    let link = rustix::fs::readlinkat(&directory, "self", Vec::new()).map_err(|error| { return format!("the procfs does not name this process ({error})"); })?;
    let named: i32 = link.to_str().ok().and_then(|text| { return text.parse::<i32>().ok(); }).ok_or_else(|| { return "the procfs names this process in a form that is not a number".to_string(); })?;
    if named != own {
        return Err(format!("the mounted procfs numbers this process {named} and this process is {own}, so it serves another process id namespace than this process is in"));
    }
    let root: ProcRoot = ProcRoot { directory, own };
    // Equal numbers do not prove the same namespace: an ancestor's procfs can number this process, by coincidence, as the process
    // numbers itself. The `NSpid` line lists the process's id in every namespace from the one the procfs belongs to down to its own,
    // so a procfs of the process's own namespace lists exactly one (certification, third pass).
    match read_in_proc(&root, &format!("{own}/status")) {
        Ok(status) => nspid_shows_own_namespace(&status)?,
        Err(error) => return Err(format!("the status of this process cannot be read to check its namespace ({error})")),
    }
    return Ok(root);
}

/// What:
/// `fn nspid_shows_own_namespace(status: &str) -> Result<(), String>` accepts a status file whose `NSpid` line has one entry (or that
/// has no such line, a kernel before 4.1) and refuses one that lists the process in more than one namespace.
/// Why:
/// A procfs of an ancestor namespace lists the process in its namespace and in each below it, even when two of the numbers coincide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nspidShowsOwnNamespace(status: string): void  // throws the reason
/// ```
pub(crate) fn nspid_shows_own_namespace(status: &str) -> Result<(), String> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("NSpid:") {
            let entries: usize = rest.split_whitespace().count();
            if entries != 1 {
                return Err(format!("the mounted procfs lists this process in {entries} process id namespaces, so it belongs to an ancestor namespace"));
            }
            return Ok(());
        }
    }
    return Ok(());
}

/// What:
/// `fn read_in_proc(root: &ProcRoot, relative: &str) -> io::Result<String>` reads a file below the procfs root, refusing to leave it,
/// to cross a mount point or to follow a link on the way.
/// Why:
/// `openat2` with `RESOLVE_BENEATH | RESOLVE_NO_XDEV | RESOLVE_NO_SYMLINKS` (Linux 5.6; a kernel without it gives an error, which the
/// callers read as unknown, never as gone) keeps a numeric lookup inside the procfs that `open_proc_root` checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readInProc(root: ProcRoot, relative: string): string
/// ```
fn read_in_proc(root: &ProcRoot, relative: &str) -> io::Result<String> {
    let opened: OwnedFd = rustix::fs::openat2(
        &root.directory,
        relative,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
        rustix::fs::ResolveFlags::BENEATH | rustix::fs::ResolveFlags::NO_XDEV | rustix::fs::ResolveFlags::NO_SYMLINKS,
    )
    .map_err(io::Error::from)?;
    let mut text: String = String::new();
    fs::File::from(opened).read_to_string(&mut text)?;
    return Ok(text);
}

/// What:
/// `fn start_of(pid: i32) -> StartOf` reads the start time of a process id after showing that the mounted procfs numbers processes
/// as this process's own namespace does.
/// Why:
/// `kill`, `pidfd_open` and the other system calls read a process id in the caller's active namespace, but `/proc/<pid>` reads it in
/// the namespace of the procfs mount, and `/proc/self/ns/pid` names only the caller's (round 12, 11a: a child namespace that still
/// uses an ancestor's procfs would make the two name different processes).
/// A pidfd names the process of the first reading, and its `fdinfo` (read through the procfs) gives the number that this procfs
/// has for that same process: only when it is the id asked for does `/proc/<pid>` name that process.
/// Anything else is unknown, never gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startOf(pid: number): { gone } | { start: number } | { unknown: string }
/// ```
fn start_of(pid: i32) -> StartOf {
    use std::os::fd::AsRawFd;
    let Some(found) = Pid::from_raw(pid) else {
        return StartOf::Unknown(format!("{pid} is not a process id"));
    };
    let pidfd: OwnedFd = match pidfd_open(found, PidfdFlags::empty()) {
        Ok(opened) => opened,
        Err(Errno::SRCH) => return StartOf::Gone,
        Err(error) => return StartOf::Unknown(format!("a pidfd for process {pid} cannot be opened ({error})")),
    };
    let root: ProcRoot = match open_proc_root() {
        Ok(opened) => opened,
        Err(reason) => return StartOf::Unknown(reason),
    };
    let number_in_procfs = || -> Result<i32, String> {
        let info: String = read_in_proc(&root, &format!("{}/fdinfo/{}", root.own, pidfd.as_raw_fd())).map_err(|error| { return format!("the pidfd of process {pid} cannot be examined through procfs ({error})"); })?;
        let line: &str = info.lines().find_map(|candidate| { return candidate.strip_prefix("Pid:"); }).ok_or_else(|| { return format!("the pidfd of process {pid} has no Pid line in procfs"); })?;
        return line.trim().parse::<i32>().map_err(|error| { return format!("the Pid line of the pidfd of process {pid} is not a number ({error})"); });
    };
    match number_in_procfs() {
        Ok(number) if number == pid => {}
        Ok(number) => return StartOf::Unknown(format!("the mounted procfs numbers process {pid} as {number}, so it serves another process id namespace than this process is in")),
        Err(reason) => return StartOf::Unknown(reason),
    }
    return match read_in_proc(&root, &format!("{pid}/stat")).and_then(|text| { return parse_stat(pid, &text); }) {
        Ok((start, _)) => StartOf::Process(start),
        // The same process was numbered `pid` a moment ago: if the procfs now has no entry, it ended (its number is then -1), or hides it.
        Err(error) if error.kind() == io::ErrorKind::NotFound => match number_in_procfs() {
            Ok(number) => missing_entry_verdict(i64::from(number), pid, &error.to_string()),
            Err(reason) => StartOf::Unknown(reason),
        },
        Err(error) => StartOf::Unknown(format!("/proc/{pid}/stat cannot be read ({error})")),
    };
}

/// What:
/// `pub(crate) fn procfs_matches_namespace() -> Result<(), String>` says whether the mounted procfs numbers processes as this
/// process's own namespace does, and why not when it does not.
/// Why:
/// An oracle of the tests that reads a numeric entry of `/proc` (a thread, for one) must not run in an environment whose procfs
/// names other processes than the caller's namespace does: it would read "no such entry" for a live process (certification, second
/// pass).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function procfsMatchesNamespace(): void  // throws the reason
/// ```
#[cfg(test)]
pub(crate) fn procfs_matches_namespace() -> Result<(), String> {
    return open_proc_root().map(|_root| {});
}

/// What:
/// `fn names_an_absent_process(error: &io::Error) -> bool` says whether a failed read of a process's file shows that the process is
/// not there: `ENOENT` (the entry is gone) or `ESRCH` (the entry was there when it was opened and the process ended before the read).
/// Why:
/// A read of `/proc/<pid>/stat` of a process that ends under it fails with either; the map says both are read as gone, and the pidfd
/// then confirms it, so neither is taken as a failure of the oracle (fourth certification pass).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesAnAbsentProcess(error: NodeJS.ErrnoException): boolean { return error.code === 'ENOENT' || error.code === 'ESRCH'; }
/// ```
#[cfg(test)]
pub(crate) fn names_an_absent_process(error: &io::Error) -> bool {
    return error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(Errno::SRCH.raw_os_error());
}

/// What:
/// `pub(crate) fn process_is_running(pid: i32) -> Result<bool, String>` says whether a process with this id is running (not ended,
/// not a zombie that is waiting to be reaped), and fails when it cannot tell.
/// Why:
/// An oracle of the tests that decides "this process is stopped" must not fail open: an inspection that cannot tell (a procfs of
/// another namespace, an unreadable entry) is an error, never "gone" (certification group 1).
/// It reads through the pinned procfs descriptor and a pidfd, as the launcher question does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processIsRunning(pid: number): boolean  // throws when it cannot tell
/// ```
#[cfg(test)]
pub(crate) fn process_is_running(pid: i32) -> Result<bool, String> {
    use std::os::fd::AsRawFd;
    let found: Pid = Pid::from_raw(pid).ok_or_else(|| { return format!("{pid} is not a process id"); })?;
    let pidfd: OwnedFd = match pidfd_open(found, PidfdFlags::empty()) {
        Ok(opened) => opened,
        Err(Errno::SRCH) => return Ok(false),
        Err(error) => return Err(format!("a pidfd for process {pid} cannot be opened ({error})")),
    };
    let root: ProcRoot = open_proc_root()?;
    let number = || -> Result<i64, String> {
        let info: String = read_in_proc(&root, &format!("{}/fdinfo/{}", root.own, pidfd.as_raw_fd())).map_err(|error| { return format!("the pidfd of process {pid} cannot be examined ({error})"); })?;
        let line: &str = info.lines().find_map(|candidate| { return candidate.strip_prefix("Pid:"); }).ok_or_else(|| { return format!("the pidfd of process {pid} has no Pid line"); })?;
        return line.trim().parse::<i64>().map_err(|error| { return format!("the Pid line of process {pid} is not a number ({error})"); });
    };
    match number()? {
        -1 => return Ok(false),
        other if other == i64::from(pid) => {}
        other => return Err(format!("the mounted procfs numbers process {pid} as {other}, so it serves another process id namespace")),
    }
    return match read_in_proc(&root, &format!("{pid}/stat")).and_then(|text| { return parse_stat(pid, &text); }) {
        Ok((_, state)) => Ok(state != 'Z' && state != 'X'),
        Err(error) if names_an_absent_process(&error) => match number()? {
            -1 => Ok(false),
            _ => Err(format!("process {pid} exists but /proc/{pid}/stat cannot be read ({error})")),
        },
        Err(error) => Err(format!("/proc/{pid}/stat cannot be read ({error})")),
    };
}

/// What:
/// `pub(crate) fn process_start(pid: i32) -> io::Result<u64>` is the start time of a process, in clock ticks since boot
/// (field 22 of `/proc/<pid>/stat`).
/// Why:
/// A process id alone can be reused; the id with its start time cannot name two processes in one boot.
/// It is an error, never a guess, when the procfs does not number processes as this process's namespace does (`start_of`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processStart(pid: number): number
/// ```
pub(crate) fn process_start(pid: i32) -> io::Result<u64> {
    return match start_of(pid) {
        StartOf::Process(start) => Ok(start),
        StartOf::Gone => Err(io::Error::new(io::ErrorKind::NotFound, format!("process {pid} does not exist"))),
        StartOf::Unknown(reason) => Err(io::Error::other(reason)),
    };
}

/// What:
/// `fn parse_stat(pid: i32, text: &str) -> io::Result<(u64, char)>` reads the start time and the state letter from the text of a stat
/// file.
/// Why:
/// The name field of the stat file may hold spaces and brackets, so the fields are counted after its last closing parenthesis.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStat(pid: number, text: string): [number, string]
/// ```
fn parse_stat(pid: i32, text: &str) -> io::Result<(u64, char)> {
    let invalid = || -> io::Error { return io::Error::new(io::ErrorKind::InvalidData, format!("/proc/{pid}/stat is not understood: {text:?}")); };
    let after: &str = text.rsplit_once(')').ok_or_else(invalid)?.1;
    let fields: Vec<&str> = after.split_whitespace().collect();
    // After the name: state is field 3, start time is field 22, so index 0 and 19 of what follows.
    let state: char = fields.first().and_then(|field| { return field.chars().next(); }).ok_or_else(invalid)?;
    let start: u64 = fields.get(19).ok_or_else(invalid)?.parse::<u64>().map_err(|_| { return invalid(); })?;
    return Ok((start, state));
}

/// What:
/// `pub(crate) fn launcher_state(pid: i32, start: u64, pid_namespace: u64) -> LauncherState` says whether the recorded launcher can
/// still join its scope.
/// Why:
/// The launcher becomes the program, and a process that is gone cannot be attached to a scope later (round 11).
/// It is gone only when this process is in the process id namespace of the record and no process has that id (a different start
/// time is unknown, not gone),
/// read through a procfs that numbers processes as this namespace does (`start_of`, round 12);
/// a different namespace, another namespace's procfs, a hidden or unreadable process and a state that is not understood are
/// "unknown", never "gone".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function launcherState(pid: number, start: number, pidNamespace: number): 'gone' | 'alive' | { unknown: string }
/// ```
pub(crate) fn launcher_state(pid: i32, start: u64, pid_namespace: u64) -> LauncherState {

    match pid_namespace_id() {
        Ok(found) if found == pid_namespace => {}
        Ok(_) => return LauncherState::Unknown(format!("the launcher {pid} was recorded in another process id namespace than this process is in")),
        Err(error) => return LauncherState::Unknown(format!("the process id namespace of this process cannot be identified ({error})")),
    }
    return match start_of(pid) {
        StartOf::Gone => LauncherState::Gone,
        // The start time in procfs is relative to the boot-time offset of the reader's time namespace, and that the reader is in the
        // writer's time namespace cannot be proven (a namespace number is recycled once the namespace has no process), so a start time
        // that differs from the record does not show a reused id: it is unknown (round 15, 14 section 2).
        // An equal start time keeps the launcher alive, and an id that no process has is gone whatever the basis.
        StartOf::Process(seen) if seen != start => LauncherState::Unknown(format!(
            "process {pid} has start time {seen} and the launcher was recorded with start time {start}; the start time in procfs is relative to the reader's time namespace, which cannot be shown to be the writer's, so it cannot be told whether the launcher is gone or its id was reused"
        )),
        StartOf::Process(_) => LauncherState::Alive,
        StartOf::Unknown(reason) => LauncherState::Unknown(reason),
    };
}

/// What:
/// `fn is_cgroup2(path: &Path) -> io::Result<bool>` says whether the directory is on a cgroup v2 file system.
/// Why:
/// Only the file system type shows that a directory is part of the control group hierarchy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCgroup2(path: string): boolean
/// ```
fn is_cgroup2(path: &Path) -> io::Result<bool> {
    let found = rustix::fs::statfs(path).map_err(io::Error::from)?;
    return Ok(u64::try_from(found.f_type).unwrap_or(0) == CGROUP2_MAGIC);
}

/// What:
/// `fn namespace_id() -> io::Result<u64>` is the inode number of this process's cgroup namespace.
/// Why:
/// A recorded path means the same thing only inside the same namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namespaceId(): number
/// ```
fn namespace_id() -> io::Result<u64> {
    return Ok(fs::metadata("/proc/self/ns/cgroup")?.ino());
}

/// What:
/// `fn on_path(program: &str) -> bool` says whether an executable of that name is on `PATH` (or is itself a path to one).
/// Why:
/// A missing `systemd-run` is a missing capability to name, found without starting anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onPath(program: string): boolean
/// ```
fn on_path(program: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let executable = |candidate: &Path| -> bool { return fs::metadata(candidate).map(|found| { return found.is_file() && found.permissions().mode() & 0o111 != 0; }).unwrap_or(false); };
    if program.contains('/') {
        return executable(Path::new(program));
    }
    let search: String = std::env::var("PATH").unwrap_or_default();
    return search.split(':').any(|directory| { return !directory.is_empty() && executable(&Path::new(directory).join(program)); });
}

/// What:
/// `fn run_plain(program, arguments, environment) -> Result<String, String>` runs a short program under a deadline and returns its
/// standard output, or the reason it failed.
/// Why:
/// The question to the manager must not wait without end (contract V6);
/// the program is not a run of the user's, so it needs no containment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runPlain(program: string, args: string[], env: [string, string | undefined][]): string  // throws the reason
/// ```
fn run_plain(program: &str, arguments: &[&str], environment: &[(String, Option<String>)], bound: Duration) -> Result<String, String> {
    let mut command: Command = Command::new(program);
    command.args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    for (name, value) in environment {
        match value {
            Some(set) => command.env(name, set),
            None => command.env_remove(name),
        };
    }
    let mut child = command.spawn().map_err(|error| { return format!("{program} could not be started ({error})"); })?;
    // One deadline for the reading, the killing and the reaping (contract V6; round 10, 9a D3).
    let deadline: Instant = Instant::now() + bound;
    let cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let (Some(out_pipe), Some(err_pipe)) = (child.stdout.take(), child.stderr.take()) else {
        let _ = child.kill();
        let reaped: bool = reap_within(&mut child, deadline);
        return Err(format!("{program} has no output pipes{}", unreaped_note(reaped)));
    };
    let out_reader = crate::bounded_io::collect(OwnedFd::from(out_pipe), Arc::clone(&cancel));
    let err_reader = crate::bounded_io::collect(OwnedFd::from(err_pipe), Arc::clone(&cancel));
    // Every exit from here ends the readers, kills a child that is still running, and reaps it within what is left of the deadline.
    // It returns the text that says so when the child was not reaped in time.
    let ended = |running: &mut std::process::Child, readers: &Arc<AtomicBool>| -> &'static str {
        readers.store(true, Ordering::Relaxed);
        let _ = running.kill();
        return unreaped_note(reap_within(running, deadline));
    };
    let status = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => {
                let note: &str = ended(&mut child, &cancel);
                return Err(format!("{program} could not be waited for ({error}){note}"));
            }
        }
        if Instant::now() >= deadline {
            let note: &str = ended(&mut child, &cancel);
            return Err(format!("{program} did not answer within {bound:?}{note}"));
        }
        std::thread::sleep(DISCOVERY_POLL);
    };
    // The direct child ended; a descendant may still hold the pipes, so the draining has the same deadline.
    let out_wait: Duration = deadline.saturating_duration_since(Instant::now());
    let out: Vec<u8> = match out_reader.recv_timeout(out_wait) {
        Ok(Ok(bytes)) => bytes,
        _ => {
            let note: &str = ended(&mut child, &cancel);
            return Err(format!("{program} did not answer within {bound:?} (its output was not closed){note}"));
        }
    };
    let err_wait: Duration = deadline.saturating_duration_since(Instant::now());
    let err: Vec<u8> = match err_reader.recv_timeout(err_wait) {
        Ok(Ok(bytes)) => bytes,
        _ => {
            let note: &str = ended(&mut child, &cancel);
            return Err(format!("{program} did not answer within {bound:?} (its error output was not closed){note}"));
        }
    };
    cancel.store(true, Ordering::Relaxed);
    if !status.success() {
        let message: String = String::from_utf8_lossy(&err).to_string();
        return Err(format!("{program} failed ({})", message.lines().next().unwrap_or("no message").trim()));
    }
    return Ok(String::from_utf8_lossy(&out).to_string());
}

/// What:
/// `fn unreaped_note(reaped: bool) -> &'static str` is the text that a discovery adds to its failure when its killed program was
/// not reaped before the deadline.
/// Why:
/// The failure is conservative: the capability is missing, and the message says that a process may still exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreapedNote(reaped: boolean): string
/// ```
fn unreaped_note(reaped: bool) -> &'static str {
    return if reaped { "" } else { "; the killed program could not be reaped before the deadline" };
}

/// What:
/// `fn reap_within(child: &mut std::process::Child, deadline: Instant) -> bool` waits for a killed child until `deadline` and says
/// whether it was reaped.
/// Why:
/// `Child::wait` has no deadline (contract V6), and the discovery's one deadline covers its reading, its killing and its reaping:
/// a reaping with a budget of its own would end after the promised time (round 10, 9a D3).
/// One non-blocking attempt is made even when the deadline has passed, so a child that is already reapable is reaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reapWithin(child: ChildProcess, deadline: number): boolean
/// ```
fn reap_within(child: &mut std::process::Child, deadline: Instant) -> bool {
    // A test makes the killed child look unreapable for as long as the budget lasts.
    #[cfg(test)]
    let stuck: bool = crate::failpoints::fires(crate::failpoints::Point::ReapStuck);
    #[cfg(not(test))]
    let stuck: bool = false;
    #[cfg(test)]
    let injected_error: bool = crate::failpoints::fires(crate::failpoints::Point::ReapError);
    #[cfg(not(test))]
    let injected_error: bool = false;
    loop {
        let waited: io::Result<Option<std::process::ExitStatus>> = if injected_error { Err(io::Error::other("injected wait failure")) } else { child.try_wait() };
        if !stuck && matches!(waited, Ok(Some(_))) {
            return true;
        }
        let now: Instant = Instant::now();
        if now >= deadline {
            return false;
        }
        std::thread::sleep(DISCOVERY_POLL.min(deadline - now));
    }
}

/// What:
/// `pub(crate) fn kill_file_present(directory: &Path) -> bool` says whether a group directory has `cgroup.kill`.
/// Why:
/// The file exists since Linux 5.14; a kernel without it takes the weaker path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function killFilePresent(directory: string): boolean
/// ```
pub(crate) fn kill_file_present(directory: &Path) -> bool {
    return fs::metadata(directory.join("cgroup.kill")).is_ok();
}

/// What:
/// `pub(crate) fn detect_route(settings: &RouteSettings) -> Result<Route, GroupUnavailable>` finds out whether this host can give a
/// delegated scope, and where the manager keeps its scopes.
/// Why:
/// The manager's real hierarchy is asked for, never assumed from this process's own path (contract, "The spawn route").
/// Every failure names the missing capability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function detectRoute(settings: RouteSettings): Route  // throws GroupUnavailable
/// ```
pub(crate) fn detect_route(settings: &RouteSettings) -> Result<Route, GroupUnavailable> {
    let unavailable = |reason: String| -> GroupUnavailable { return GroupUnavailable { reason }; };
    match is_cgroup2(&settings.cgroup_root) {
        Ok(true) => {}
        Ok(false) => return Err(unavailable(format!("{} is not a cgroup v2 mount", settings.cgroup_root.display()))),
        Err(error) => return Err(unavailable(format!("{} cannot be examined ({error})", settings.cgroup_root.display()))),
    }
    if !on_path(&settings.systemd_run) {
        return Err(unavailable(format!("{} is not installed", settings.systemd_run)));
    }
    // An inactive slice has no control group yet (a fresh login, a runner whose manager has run no application): starting it is
    // idempotent, and it is still the discovery of a capability, so a manager that cannot start it is a missing capability.
    if settings.question.is_none() {
        run_plain(&settings.systemctl, &["--user", "start", &settings.slice], &settings.environment, settings.discovery_bound)
            .map_err(|reason| { return unavailable(format!("the user manager cannot start {} ({reason})", settings.slice)); })?;
    }
    let ordinary: Vec<String> = vec!["--user".to_string(), "show".to_string(), settings.slice.clone(), "--property=ControlGroup".to_string(), "--value".to_string()];
    let question: Vec<&str> = settings.question.as_ref().unwrap_or(&ordinary).iter().map(String::as_str).collect();
    let answer: String = run_plain(&settings.systemctl, &question, &settings.environment, settings.discovery_bound)
        .map_err(|reason| { return unavailable(format!("the user manager cannot be asked where {} is ({reason})", settings.slice)); })?;
    let reported: &str = answer.trim();
    if reported.is_empty() || !reported.starts_with('/') || reported.contains('\n') {
        return Err(unavailable(format!("the user manager reports no control group for {}", settings.slice)));
    }
    let slice_dir: PathBuf = settings.cgroup_root.join(reported.trim_start_matches('/'));
    if slice_dir.to_str().is_none() {
        return Err(unavailable(format!("{} is not UTF-8, which the journal record cannot hold", slice_dir.display())));
    }
    if !fs::metadata(&slice_dir).map(|found| { return found.is_dir(); }).unwrap_or(false) {
        return Err(unavailable(format!("{} does not exist in the cgroup file system", slice_dir.display())));
    }
    if !kill_file_present(&slice_dir) {
        return Err(unavailable(format!("{} has no cgroup.kill; Linux 5.14 or later is needed", slice_dir.display())));
    }
    let namespace: u64 = namespace_id().map_err(|error| { return unavailable(format!("the cgroup namespace cannot be identified ({error})")); })?;
    let slice_view: View = view_of(&slice_dir).map_err(|error| { return unavailable(format!("{} cannot be examined ({error})", slice_dir.display())); })?;
    let limits: InheritedLimits = caller_limits(&settings.own_cgroup_file, &settings.cgroup_root, &slice_dir)
        .map_err(|error| { return unavailable(format!("the resource limits of the caller's control group cannot be read ({error}), and a scope without them would loosen the run")); })?;
    return Ok(Route { systemd_run: settings.systemd_run.clone(), slice: settings.slice.clone(), slice_dir, namespace, slice_view, limits });
}

/// The scope of one run.
#[derive(Debug)]
pub(crate) struct ControlGroup {
    /// The unit name without the `.scope` suffix.
    name: String,
    /// The directory the scope has once it runs.
    directory: PathBuf,
    /// The directory that holds it.
    parent: PathBuf,
    /// How that directory was seen when the scope was planned.
    parent_view: View,
    /// The inode of this process's cgroup namespace.
    namespace: u64,
}

/// What a journal remembers of a run's containment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Record {
    /// A transient scope: the cgroup namespace and the view of the directory that holds it,
    /// recorded before the program starts,
    /// and the scope's path.
    Scope {
        /// The inode number of the cgroup namespace of the process that planned it.
        namespace: u64,
        /// How the directory that holds the scope was seen.
        parent_view: View,
        /// The directory of the scope.
        path: PathBuf,
    },
    /// The run that recorded this scope ended and every process of its launcher was reaped, so no launcher can still create the scope:
    /// from here an absent scope means a scope that ended, not one that was never made (round 9, 8a F1).
    Settled {
        /// The directory of the scope that is settled.
        path: PathBuf,
    },
    /// The launcher of the scope with this path, noted right after it was spawned (round 11): its process id, its start time (clock
    /// ticks since boot, field 22 of `/proc/<pid>/stat`) and the inode of its process id namespace.
    /// A launcher that is gone can never join the scope later, so a recovery that finds it gone may read the scope as final.
    Launcher {
        /// The process id of the launcher, which becomes the program.
        pid: i32,
        /// The start time of that process.
        start: u64,
        /// The inode number of the process id namespace of the wrapper, which is the one the pid belongs to.
        pid_namespace: u64,
        /// The directory of the scope the launcher was to enter.
        path: PathBuf,
    },
    /// The weaker path: only a process group (round 5), recorded by the process id of its leader.
    ProcessGroup(
        /// The leader's process id, which is the group's number.
        i32,
    ),
    /// The weaker path is about to start a child whose process group is not known yet;
    /// the line that follows it names the group.
    /// A killed wrapper that leaves this line alone may have a running child that no record names.
    ProcessGroupStarting,
}

/// What a recovery learns about a recorded scope.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Membership {
    /// No member can exist: the scope is verifiably absent, or present and empty.
    Gone,
    /// At least one thread of the scope lives.
    Alive,
    /// The question cannot be answered, with the reason.
    Unknown(
        /// Why.
        String,
    ),
}

/// What:
/// `fn random_token() -> io::Result<String>` is 128 random bits as hexadecimal text.
/// Why:
/// A scope name must never repeat while any journal can reference it,
/// so it carries no process id and no clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function randomToken(): string
/// ```
fn random_token() -> io::Result<String> {
    let mut bytes: [u8; TOKEN_BYTES] = [0; TOKEN_BYTES];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let mut text: String = String::with_capacity(TOKEN_BYTES * 2);
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    return Ok(text);
}

/// What:
/// `fn escaped(argument: &std::ffi::OsStr) -> std::ffi::OsString` doubles every dollar sign of an argument.
/// Why:
/// `systemd-run` reads the command it is given like the command line of a unit:
/// it turns two dollar signs into one and replaces a braced variable name by the value of the variable (probed on 2026-11-04),
/// so an argument is encoded for that grammar at the point where it is handed over (STB), and the program receives the bytes it was
/// given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function escaped(argument: string): string
/// ```
fn escaped(argument: &std::ffi::OsStr) -> std::ffi::OsString {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let mut bytes: Vec<u8> = Vec::with_capacity(argument.len());
    for byte in argument.as_bytes() {
        bytes.push(*byte);
        if *byte == b'$' {
            bytes.push(b'$');
        }
    }
    return std::ffi::OsString::from_vec(bytes);
}

/// The methods of the type above.
impl ControlGroup {
    /// What:
    /// `pub(crate) fn plan(route: &Route) -> io::Result<ControlGroup>` names the scope of one run before anything starts.
    /// Why:
    /// The path is known before the program exists, so the record can be forced to disk first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static plan(route: Route): ControlGroup
    /// ```
    pub(crate) fn plan(route: &Route) -> io::Result<ControlGroup> {
        let name: String = format!("cargo-shadow-{}", random_token()?);
        let directory: PathBuf = route.slice_dir.join(format!("{name}.scope"));
        return Ok(ControlGroup { name, directory, parent: route.slice_dir.clone(), parent_view: route.slice_view, namespace: route.namespace });
    }

    /// What:
    /// `pub(crate) fn directory(&self) -> &Path` is the scope's directory.
    /// Why:
    /// Tests and the record name it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// directory(): string
    /// ```
    pub(crate) fn directory(&self) -> &Path {
        return &self.directory;
    }

    /// What:
    /// `pub(crate) fn record(&self) -> Record` is what a journal keeps of this scope.
    /// Why:
    /// The journal line is written and forced to disk before the program starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record(): Record
    /// ```
    pub(crate) fn record(&self) -> Record {
        return Record::Scope { namespace: self.namespace, parent_view: self.parent_view, path: self.directory.clone() };
    }

    /// What:
    /// `pub(crate) fn launcher(&self, route: &Route, original: &Command, clean_environment: bool) -> Command` is the command that
    /// starts `original` in this scope.
    /// Why:
    /// `systemd-run --scope` asks the manager to move its own process into the scope and then becomes the program,
    /// so the program runs in the scope from its first instruction and no thread of this program touches a `cgroup.procs`.
    /// The working directory and the variables of the original are given to the launcher;
    /// a command that clears the environment runs through `env --ignore-environment`,
    /// so the manager is still reachable by the launcher and the program sees only the variables it was given.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// launcher(route: Route, original: Command, cleanEnvironment: boolean): Command
    /// ```
    pub(crate) fn launcher(&self, route: &Route, original: &Command, clean_environment: bool) -> Command {
        let mut launch: Command = Command::new(&route.systemd_run);
        launch.args(["--user", "--scope", "--quiet", "--collect", "--property=Delegate=yes"]);
        launch.arg(format!("--unit={}", self.name));
        launch.arg(format!("--slice={}", route.slice));
        launch.arg("--");
        if let Some(directory) = original.get_current_dir() {
            launch.current_dir(directory);
        }
        if clean_environment {
            launch.arg("env").arg("--ignore-environment");
            for (name, value) in original.get_envs() {
                if let Some(set) = value {
                    let mut assignment: std::ffi::OsString = name.to_os_string();
                    assignment.push("=");
                    assignment.push(set);
                    launch.arg(escaped(&assignment));
                }
            }
        } else {
            for (name, value) in original.get_envs() {
                match value {
                    Some(set) => launch.env(name, set),
                    None => launch.env_remove(name),
                };
            }
        }
        launch.arg(escaped(original.get_program()));
        for argument in original.get_args() {
            launch.arg(escaped(argument));
        }
        return launch;
    }

    /// What:
    /// `pub(crate) fn members(&self) -> io::Result<Vec<i32>>` lists the processes in the scope (the leader of each thread group).
    /// Why:
    /// A thread group whose leader exited is still listed while another of its threads lives.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// members(): number[]
    /// ```
    pub(crate) fn members(&self) -> io::Result<Vec<i32>> {
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::ContainmentInspection) {
            return Err(io::Error::other("injected failure to list the members"));
        }
        let text: String = match fs::read_to_string(self.directory.join("cgroup.procs")) {
            Ok(found) => found,
            // A scope that was removed has no members; `ENODEV` is what a read of a removed group's file says.
            Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut found: Vec<i32> = Vec::new();
        for line in text.lines() {
            found.push(line.trim().parse::<i32>().map_err(|error| { return io::Error::new(io::ErrorKind::InvalidData, format!("cgroup.procs holds {line:?} ({error})")); })?);
        }
        return Ok(found);
    }

    /// What:
    /// `pub(crate) fn populated(&self) -> io::Result<bool>` says whether any thread of the scope lives.
    /// Why:
    /// This is the kernel's own statement of quiescence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// populated(): boolean
    /// ```
    #[cfg(test)]
    pub(crate) fn populated(&self) -> io::Result<bool> {
        return populated_at(&self.directory);
    }

    /// What:
    /// `fn signal_members(&self) -> io::Result<()>` sends `SIGKILL` to every listed process through a pidfd that was checked
    /// against the listing a second time.
    /// Why:
    /// `cgroup.kill` leaves a thread group alive when its leader already exited (measured on Linux 7.2.15),
    /// and a signal to the process ends it.
    /// A process id read from a list can be reused,
    /// so the pidfd is opened first and the process must still be listed after that.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// signalMembers(): void
    /// ```
    fn signal_members(&self) -> io::Result<()> {
        let mut held: Vec<(i32, OwnedFd)> = Vec::new();
        for pid in self.members()? {
            let Some(found) = Pid::from_raw(pid) else {
                continue;
            };
            // A process that ended between the listing and now is simply not signalled.
            if let Ok(descriptor) = pidfd_open(found, PidfdFlags::empty()) {
                held.push((pid, descriptor));
            }
        }
        let still_listed: Vec<i32> = self.members()?;
        for (pid, descriptor) in &held {
            if still_listed.contains(pid) {
                let _ = pidfd_send_signal(descriptor, Signal::KILL);
            }
        }
        return Ok(());
    }

    /// What:
    /// `fn absence_is_verified(&self) -> io::Result<()>` checks that the absence of the scope can be read as emptiness.
    /// Why:
    /// The manager removes a scope only when it is empty or after stopping its members (probed),
    /// so a scope that is absent from the directory that held it had no member.
    /// That holds only when the directory is seen as it was when the scope was planned (contract assumption c).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// absenceIsVerified(): void  // throws when the view differs
    /// ```
    fn absence_is_verified(&self) -> io::Result<()> {
        let namespace: u64 = namespace_id()?;
        if namespace != self.namespace {
            return Err(io::Error::other("this process is in another cgroup namespace than the one the scope was made in"));
        }
        let seen: View = view_of(&self.parent)?;
        if seen != self.parent_view {
            return Err(io::Error::other(format!("{} is seen as {seen} now and was {} when the scope was planned", self.parent.display(), self.parent_view)));
        }
        return Ok(());
    }

    /// What:
    /// `pub(crate) fn stop(&self, bound: Duration) -> io::Result<bool>` kills every member and returns `true` once the kernel
    /// reports the scope unpopulated or its absence is verified,
    /// or `false` when `bound` passes first.
    /// Why:
    /// `cgroup.kill` is recursive and asynchronous,
    /// so the answer is the wait for `populated 0`, which can only be earned.
    /// A member in an uninterruptible wait cannot be killed and that is `false`.
    /// Every other failure is an error,
    /// and the caller reads both as "not quiescent".
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stop(boundMs: number): boolean
    /// ```
    pub(crate) fn stop(&self, bound: Duration) -> io::Result<bool> {
        #[cfg(test)]
        {
            if crate::failpoints::fires(crate::failpoints::Point::ContainmentStopError) {
                return Err(io::Error::other("injected failure to stop the scope"));
            }
            if crate::failpoints::fires(crate::failpoints::Point::ContainmentStopStuck) {
                return Ok(false);
            }
        }
        let deadline: Instant = Instant::now() + bound;
        // A test makes a member survive every signal, so the loop below runs to its deadline as it would for a member in an
        // uninterruptible wait (round 16, V6: an unsuccessful shutdown has a deadline).
        #[cfg(test)]
        let survivor: bool = crate::failpoints::fires(crate::failpoints::Point::ContainmentSurvivor);
        #[cfg(not(test))]
        let survivor: bool = false;
        match write_text(&self.directory.join("cgroup.kill"), "1") {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.absence_is_verified()?;
                #[cfg(test)]
                crate::bounded_run::trace(|| { return "scope proved absent".to_string(); });
                return Ok(true);
            }
            // The signal pass below does the stopping when the kill file cannot be written.
            Err(error) => eprintln!("cargo-shadow: warning: cgroup.kill of {} could not be written ({error}); signalling the members one by one", self.directory.display()),
        }
        let events = match fs::File::open(self.directory.join("cgroup.events")) {
            Ok(found) => found,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.absence_is_verified()?;
                #[cfg(test)]
                crate::bounded_run::trace(|| { return "scope proved absent".to_string(); });
                return Ok(true);
            }
            Err(error) => return Err(error),
        };
        loop {
            if !populated_through(&events)? && !survivor {
                // The authoritative proof: the kernel reports the scope unpopulated after the kill (a test build traces it, so a
                // test sees that a successful stop went through this check and not through a snapshot of the members).
                #[cfg(test)]
                crate::bounded_run::trace(|| { return "scope proved empty".to_string(); });
                return Ok(true);
            }
            self.signal_members()?;
            let now: Instant = Instant::now();
            if now >= deadline {
                return Ok(false);
            }
            let wait: Duration = STRAGGLER_INTERVAL.min(deadline - now);
            let timeout: Timespec = Timespec { tv_sec: i64::try_from(wait.as_secs()).unwrap_or(0), tv_nsec: i64::from(wait.subsec_nanos()) };
            let mut watched = [PollFd::new(&events, PollFlags::PRI)];
            match poll(&mut watched, Some(&timeout)) {
                Ok(_) | Err(Errno::INTR) => {}
                Err(error) => return Err(io::Error::from(error)),
            }
        }
    }
}

/// What:
/// `fn write_text(path: &Path, text: &str) -> io::Result<()>` writes `text` to a control file in one `write`.
/// Why:
/// The kernel reads a control file's command from a single write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeText(path: string, text: string): void
/// ```
fn write_text(path: &Path, text: &str) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    return file.write_all(text.as_bytes());
}

/// What:
/// `fn parse_populated(text: &str) -> io::Result<bool>` reads the `populated` line of `cgroup.events`.
/// Why:
/// Anything but `populated 0` or `populated 1` is an error, never "empty".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePopulated(text: string): boolean
/// ```
fn parse_populated(text: &str) -> io::Result<bool> {
    for line in text.lines() {
        match line.trim() {
            "populated 1" => return Ok(true),
            "populated 0" => return Ok(false),
            _ => {}
        }
    }
    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("cgroup.events has no populated line: {text:?}")));
}

/// What:
/// `fn populated_through(events: &fs::File) -> io::Result<bool>` re-reads an open `cgroup.events` from its start.
/// Why:
/// The read also arms the file so that a later `poll` wakes on the next change.
/// A removed group reads as unpopulated (`ENODEV`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function populatedThrough(events: FileHandle): boolean
/// ```
fn populated_through(events: &fs::File) -> io::Result<bool> {
    let mut buffer: [u8; 128] = [0; 128];
    let count: usize = match events.read_at(&mut buffer, 0) {
        Ok(found) => found,
        Err(error) if error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) => return Ok(false),
        Err(error) => return Err(error),
    };
    return parse_populated(&String::from_utf8_lossy(&buffer[..count]));
}

/// What:
/// `fn populated_at(directory: &Path) -> io::Result<bool>` reads `cgroup.events` of a group directory once.
/// Why:
/// A recovery has no open file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function populatedAt(directory: string): boolean
/// ```
pub(crate) fn populated_at(directory: &Path) -> io::Result<bool> {
    return parse_populated(&fs::read_to_string(directory.join("cgroup.events"))?);
}

/// How a record reads as a journal line.
impl fmt::Display for Record {
    /// What:
    /// `fn fmt(&self, formatter)` writes the one-line form of the record, without the line feed.
    /// Why:
    /// The journal's `groups` file holds one line per record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Record::Scope { namespace, parent_view, path } => write!(formatter, "scope {namespace} {parent_view} {}", path.display()),
            Record::Settled { path } => write!(formatter, "settled {}", path.display()),
            Record::Launcher { pid, start, pid_namespace, path } => write!(formatter, "launcher {pid} {start} {pid_namespace} {}", path.display()),
            Record::ProcessGroup(leader) => write!(formatter, "pgroup {leader}"),
            Record::ProcessGroupStarting => write!(formatter, "pgroup starting"),
        };
    }
}

/// What:
/// `fn parse_number(text: &str, what: &str, line: &str) -> Result<u64, String>` reads one number of a record.
/// Why:
/// The reason names the line and the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseNumber(text: string, what: string, line: string): number
/// ```
fn parse_number(text: &str, what: &str, line: &str) -> Result<u64, String> {
    return text.parse::<u64>().map_err(|error| { return format!("{line:?} has no {what} ({error})"); });
}

/// What:
/// `pub(crate) fn parse_record(line: &str) -> Result<Record, String>` reads one line of a `groups` file.
/// Why:
/// A line that is not understood is uncertainty about a possible writer, and the caller says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseRecord(line: string): Record  // throws the reason
/// ```
pub(crate) fn parse_record(line: &str) -> Result<Record, String> {
    if line == "pgroup starting" {
        return Ok(Record::ProcessGroupStarting);
    }
    if let Some(rest) = line.strip_prefix("pgroup ") {
        let leader: i32 = rest.trim().parse::<i32>().map_err(|error| { return format!("{line:?} is not a process group ({error})"); })?;
        if leader <= 0 {
            return Err(format!("{line:?} is not a process group"));
        }
        return Ok(Record::ProcessGroup(leader));
    }
    if let Some(rest) = line.strip_prefix("launcher ") {
        let mut parts = rest.splitn(4, ' ');
        let pid_text: &str = parts.next().unwrap_or("");
        let pid: i32 = pid_text.parse::<i32>().map_err(|error| { return format!("{line:?} has no launcher process id ({error})"); })?;
        if pid <= 0 {
            return Err(format!("{line:?} has no launcher process id"));
        }
        let start: u64 = parse_number(parts.next().unwrap_or(""), "start time", line)?;
        let pid_namespace: u64 = parse_number(parts.next().unwrap_or(""), "process id namespace", line)?;
        let path: &str = parts.next().unwrap_or("");
        // The path is absolute: a line with anything else there (a field of another version) is not understood.
        if !path.starts_with('/') {
            return Err(format!("{line:?} has no absolute path"));
        }
        return Ok(Record::Launcher { pid, start, pid_namespace, path: PathBuf::from(path) });
    }
    if let Some(rest) = line.strip_prefix("settled ") {
        if rest.is_empty() {
            return Err(format!("{line:?} has no path"));
        }
        return Ok(Record::Settled { path: PathBuf::from(rest) });
    }
    if let Some(rest) = line.strip_prefix("scope ") {
        let mut parts = rest.splitn(3, ' ');
        let namespace: u64 = parse_number(parts.next().unwrap_or(""), "namespace id", line)?;
        let view_text: &str = parts.next().unwrap_or("");
        let (mount_text, inode_text) = view_text.split_once(':').ok_or_else(|| { return format!("{line:?} has no mount view"); })?;
        let parent_view: View = View { mount: parse_number(mount_text, "mount id", line)?, inode: parse_number(inode_text, "inode", line)? };
        let path: &str = parts.next().unwrap_or("");
        if path.is_empty() {
            return Err(format!("{line:?} has no path"));
        }
        return Ok(Record::Scope { namespace, parent_view, path: PathBuf::from(path) });
    }
    return Err(format!("{line:?} is not a record of a scope or a process group"));
}

/// What:
/// `pub(crate) fn query(namespace: u64, parent_view: View, path: &Path) -> Membership` says whether the recorded scope has a member.
/// Why:
/// An absent scope is read as gone only when this process sees the cgroup file system as the run did:
/// the same cgroup namespace, a mounted cgroup v2, and the directory that held the scope with the recorded mount id and inode
/// (contract assumption c, the second check).
/// A scope that is present is asked for its `populated` flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function query(namespace: number, parentView: View, path: string): 'gone' | 'alive' | { unknown: string }
/// ```
pub(crate) fn query(namespace: u64, parent_view: View, path: &Path) -> Membership {
    match namespace_id() {
        Ok(found) if found == namespace => {}
        Ok(_) => return Membership::Unknown(format!("{} was made in another cgroup namespace than this process is in", path.display())),
        Err(error) => return Membership::Unknown(format!("the cgroup namespace of this process cannot be identified ({error})")),
    }
    match is_cgroup2(Path::new(CGROUP_ROOT)) {
        Ok(true) => {}
        Ok(false) => return Membership::Unknown(format!("{CGROUP_ROOT} is not a cgroup v2 mount, so {} cannot be examined", path.display())),
        Err(error) => return Membership::Unknown(format!("{CGROUP_ROOT} cannot be examined ({error})")),
    }
    let Some(parent) = path.parent() else {
        return Membership::Unknown(format!("{} has no parent directory", path.display()));
    };
    match view_of(parent) {
        Ok(seen) if seen == parent_view => {}
        Ok(seen) => {
            return Membership::Unknown(format!(
                "{} is seen as mount and inode {seen} here and was {parent_view} when the scope was recorded, so the recorded path may lead somewhere else",
                parent.display()
            ));
        }
        Err(error) => return Membership::Unknown(format!("{} cannot be examined ({error})", parent.display())),
    }
    match fs::metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Membership::Gone,
        Err(error) => return Membership::Unknown(format!("{} cannot be examined ({error})", path.display())),
    }
    return match populated_at(path) {
        Ok(true) => Membership::Alive,
        Ok(false) => Membership::Gone,
        // A scope removed between the two reads is absent.
        Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) => Membership::Gone,
        Err(error) => Membership::Unknown(format!("{} cannot be read ({error})", path.join("cgroup.events").display())),
    };
}
