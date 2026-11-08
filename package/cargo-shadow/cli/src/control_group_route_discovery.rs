//! What:
//! How the route to a delegated control group is discovered:
//! the cgroup mount,
//! the user manager,
//! the slice,
//! and the limits the scope inherits.
//! Why:
//! Split out of `control_group.rs`,
//! which holds the containment of a run in a control group,
//! to keep that file under the code-line limit.
//! The items are re-exported from `control_group`,
//! so their paths there are unchanged.

/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::{ExplainedError};
/// Imports `std::fmt` for this file.
use std::fmt;

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;


/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::os::unix::fs::{FileExt, MetadataExt}` for this file.
use std::os::unix::fs::{MetadataExt};

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


/// Imports `rustix::fs::{AtFlags, CWD, StatxFlags, statx}` for this file.
use rustix::fs::{AtFlags, CWD, StatxFlags, statx};



/// Where the unified control group hierarchy is mounted.
pub(crate) const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// The file that names the control group of this process.
pub(crate) const OWN_CGROUP_FILE: &str = "/proc/self/cgroup";

/// The file system type number of a cgroup v2 mount (`CGROUP2_SUPER_MAGIC`,
/// ASCII "cgrp"),
/// from `statfs(2)`.
pub(crate) const CGROUP2_MAGIC: u64 = 0x6367_7270;

/// `STATX_MNT_ID_UNIQUE` of `linux/stat.h` (Linux 6.8):
/// the mount id that is never reused while the system runs.
/// `rustix` 1.1.6 has the field and not the flag,
/// so the bit is spelled here.
pub(crate) const STATX_MNT_ID_UNIQUE: u32 = 0x4000;

/// How long the stop waits for a change of `cgroup.events` before it looks for members to signal again.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// measurement beside it:
/// on 2026-11-04 `populated 0` followed `cgroup.kill` within 5 ms for ordinary members and 1 ms after a signal to a zombie leader,
/// so a wake-up every 50 ms only matters for a member that appears or survives the first signals.
pub(crate) const STRAGGLER_INTERVAL: Duration = Duration::from_millis(50);

/// How long the discovery of the manager's slice may take.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// measurement beside it:
/// `systemctl --user show app.slice` answered in a few milliseconds on 2026-11-04,
/// so ten seconds only ends a manager that does
/// not answer.
pub(crate) const DISCOVERY_BOUND: Duration = Duration::from_secs(10);

/// How often the discovery asks whether `systemctl` has ended.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
pub(crate) const DISCOVERY_POLL: Duration = Duration::from_millis(5);

/// How many random bytes make a scope name unique.
/// 16 bytes are 128 bits,
/// so a name repeated by chance is not a case to plan for.
pub(crate) const TOKEN_BYTES: usize = 16;

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

/// How a path is seen by this process:
/// the mount it is on and its inode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct View {
    /// The mount id (`STATX_MNT_ID`),
    /// which differs in another mount namespace.
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
/// A kernel without `STATX_MNT_ID` (before Linux 5.8) is an error,
/// never a view of zero.
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
    /// `systemd-run`,
    /// `systemctl`,
    /// `/sys/fs/cgroup` and `app.slice`.
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

/// What a host can give:
/// the way to a delegated scope.
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
    /// Shared,
    /// not inline,
    /// so a `Route` stays small enough to sit in the run's containment value (`clippy::large_enum_variant`).
    pub(crate) limits: Arc<InheritedLimits>,
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
pub(crate) fn is_cgroup2(path: &Path) -> io::Result<bool> {
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
pub(crate) fn namespace_id() -> io::Result<u64> {
    return Ok(fs::metadata("/proc/self/ns/cgroup")?.ino());
}

/// What:
/// `fn on_path(program: &str) -> bool` says whether an executable of that name is on `PATH` (or is itself a path to one).
/// Why:
/// A missing `systemd-run` is a missing capability to name,
/// found without starting anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onPath(program: string): boolean
/// ```
pub(crate) fn on_path(program: &str) -> bool {
    if program.contains('/') {
        return is_executable_file(Path::new(program));
    }
    let search: String = std::env::var("PATH").unwrap_or_default();
    for directory in search.split(':') {
        if !directory.is_empty() && is_executable_file(&Path::new(directory).join(program)) {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn is_executable_file(candidate: &Path) -> bool` says whether a path is a regular file with an execute bit.
/// Why:
/// A function replaces the closure that `on_path` called for each search directory.
pub(crate) fn is_executable_file(candidate: &Path) -> bool {
    /// The permission bits that tell an executable file from a plain one.
    use std::os::unix::fs::PermissionsExt;
    match fs::metadata(candidate) {
        Ok(found) => return found.is_file() && found.permissions().mode() & 0o111 != 0,
        Err(_) => return false,
    }
}

/// What:
/// `fn run_plain(program, arguments, environment) -> Result<String, String>` runs a short program under a deadline and returns its
/// standard output,
/// or the reason it failed.
/// Why:
/// The question to the manager must not wait without end (contract V6);
/// the program is not a run of the user's,
/// so it needs no containment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runPlain(program: string, args: string[], env: [string, string | undefined][]): string  // throws the reason
/// ```
pub(crate) fn run_plain(program: &str, arguments: &[&str], environment: &[(String, Option<String>)], bound: Duration) -> Result<String, String> {
    let mut command: Command = Command::new(program);
    command.args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    for (name, value) in environment {
        match value {
            Some(set) => command.env(name, set),
            None => command.env_remove(name),
        };
    }
    let mut child = command.spawn().explained(format_args!("{program} could not be started"))?;
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
    let status = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => {
                let note: &str = end_child(&mut child, &cancel, deadline);
                return Err(format!("{program} could not be waited for ({error}){note}"));
            }
        }
        if Instant::now() >= deadline {
            let note: &str = end_child(&mut child, &cancel, deadline);
            return Err(format!("{program} did not answer within {bound:?}{note}"));
        }
        std::thread::sleep(DISCOVERY_POLL);
    };
    // The direct child ended; a descendant may still hold the pipes, so the draining has the same deadline.
    let out_wait: Duration = deadline.saturating_duration_since(Instant::now());
    let out: Vec<u8> = match out_reader.recv_timeout(out_wait) {
        Ok(Ok(bytes)) => bytes,
        _ => {
            let note: &str = end_child(&mut child, &cancel, deadline);
            return Err(format!("{program} did not answer within {bound:?} (its output was not closed){note}"));
        }
    };
    let err_wait: Duration = deadline.saturating_duration_since(Instant::now());
    let err: Vec<u8> = match err_reader.recv_timeout(err_wait) {
        Ok(Ok(bytes)) => bytes,
        _ => {
            let note: &str = end_child(&mut child, &cancel, deadline);
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
/// `fn end_child(running: &mut std::process::Child, readers: &Arc<AtomicBool>, deadline: Instant) -> &'static str` ends the readers,
/// kills a child that is still running,
/// and reaps it within what is left of the deadline.
/// Why:
/// Every exit from the wait ends the child this way,
/// and a function replaces the closure that captured the deadline.
/// It returns the text that says so when the child was not reaped in time.
pub(crate) fn end_child(running: &mut std::process::Child, readers: &Arc<AtomicBool>, deadline: Instant) -> &'static str {
    readers.store(true, Ordering::Relaxed);
    let _ = running.kill();
    return unreaped_note(reap_within(running, deadline));
}

/// What:
/// `fn unreaped_note(reaped: bool) -> &'static str` is the text that a discovery adds to its failure when its killed program was
/// not reaped before the deadline.
/// Why:
/// The failure is conservative:
/// the capability is missing,
/// and the message says that a process may still exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreapedNote(reaped: boolean): string
/// ```
pub(crate) fn unreaped_note(reaped: bool) -> &'static str {
    return if reaped { "" } else { "; the killed program could not be reaped before the deadline" };
}

/// What:
/// `fn reap_within(child: &mut std::process::Child, deadline: Instant) -> bool` waits for a killed child until `deadline` and says
/// whether it was reaped.
/// Why:
/// `Child::wait` has no deadline (contract V6),
/// and the discovery's one deadline covers its reading,
/// its killing and its reaping:
/// a reaping with a budget of its own would end after the promised time (round 10,
/// 9a D3).
/// One non-blocking attempt is made even when the deadline has passed,
/// so a child that is already reapable is reaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reapWithin(child: ChildProcess, deadline: number): boolean
/// ```
pub(crate) fn reap_within(child: &mut std::process::Child, deadline: Instant) -> bool {
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
/// The file exists since Linux 5.14;
/// a kernel without it takes the weaker path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function killFilePresent(directory: string): boolean
/// ```
pub(crate) fn kill_file_present(directory: &Path) -> bool {
    return fs::metadata(directory.join("cgroup.kill")).is_ok();
}

/// What:
/// `fn unavailable(reason: String) -> GroupUnavailable` wraps a reason in the refusal that route discovery returns.
/// Why:
/// A function replaces the closure that `detect_route` called at every refusal.
pub(crate) fn unavailable(reason: String) -> GroupUnavailable {
    return GroupUnavailable { reason };
}

/// What:
/// `pub(crate) fn detect_route(settings: &RouteSettings) -> Result<Route, GroupUnavailable>` finds out whether this host can give a
/// delegated scope,
/// and where the manager keeps its scopes.
/// Why:
/// The manager's real hierarchy is asked for,
/// never assumed from this process's own path (contract,
/// "The spawn route").
/// Every failure names the missing capability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function detectRoute(settings: RouteSettings): Route  // throws GroupUnavailable
/// ```
pub(crate) fn detect_route(settings: &RouteSettings) -> Result<Route, GroupUnavailable> {
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
        let started: Result<String, String> = run_plain(&settings.systemctl, &["--user", "start", &settings.slice], &settings.environment, settings.discovery_bound);
        if let Err(reason) = started {
            return Err(unavailable(format!("the user manager cannot start {} ({reason})", settings.slice)));
        }
    }
    let ordinary: Vec<String> = vec!["--user".to_string(), "show".to_string(), settings.slice.clone(), "--property=ControlGroup".to_string(), "--value".to_string()];
    let question: Vec<&str> = settings.question.as_ref().unwrap_or(&ordinary).iter().map(String::as_str).collect();
    let asked: Result<String, String> = run_plain(&settings.systemctl, &question, &settings.environment, settings.discovery_bound);
    let answer: String = match asked {
        Ok(text) => text,
        Err(reason) => return Err(unavailable(format!("the user manager cannot be asked where {} is ({reason})", settings.slice))),
    };
    let reported: &str = answer.trim();
    if reported.is_empty() || !reported.starts_with('/') || reported.contains('\n') {
        return Err(unavailable(format!("the user manager reports no control group for {}", settings.slice)));
    }
    let slice_dir: PathBuf = settings.cgroup_root.join(reported.trim_start_matches('/'));
    if slice_dir.to_str().is_none() {
        return Err(unavailable(format!("{} is not UTF-8, which the journal record cannot hold", slice_dir.display())));
    }
    let slice_is_directory: bool = match fs::metadata(&slice_dir) {
        Ok(found) => found.is_dir(),
        Err(_) => false,
    };
    if !slice_is_directory {
        return Err(unavailable(format!("{} does not exist in the cgroup file system", slice_dir.display())));
    }
    if !kill_file_present(&slice_dir) {
        return Err(unavailable(format!("{} has no cgroup.kill; Linux 5.14 or later is needed", slice_dir.display())));
    }
    let namespace: u64 = match namespace_id() {
        Ok(found) => found,
        Err(error) => return Err(unavailable(format!("the cgroup namespace cannot be identified ({error})"))),
    };
    let slice_view: View = match view_of(&slice_dir) {
        Ok(found) => found,
        Err(error) => return Err(unavailable(format!("{} cannot be examined ({error})", slice_dir.display()))),
    };
    let limits: InheritedLimits = match caller_limits(&settings.own_cgroup_file, &settings.cgroup_root, &slice_dir) {
        Ok(found) => found,
        Err(error) => return Err(unavailable(format!("the resource limits of the caller's control group cannot be read ({error}), and a scope without them would loosen the run"))),
    };
    return Ok(Route { systemd_run: settings.systemd_run.clone(), slice: settings.slice.clone(), slice_dir, namespace, slice_view, limits: Arc::new(limits) });
}
