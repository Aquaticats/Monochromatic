//! What:
//!  The probe's worker:
//!  arms 1 to 5,
//!  the two scan passes around the holder,
//!  arm 8's fork and its relayed lines,
//!  the closing scan and the teardown,
//!  each step reported on its own status pipe.
//! Why:
//!  The worker is the only process that makes the probe's mounts,
//!  and every mount lives in the private namespace it unshares first,
//!  so a failure at any step leaves nothing in the host mount table
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms").
//!  It stops at its first failed arm and writes that arm's step and errno,
//!  a source another namespace holds is recorded as not exercised rather than
//!  as a failure,
//!  and the exclusive create's `EOPNOTSUPP` is the record's "could not run"
//!  rather than a fallback.
//!  It sets no parent-death signal:
//!  it must finish its unmount and report even when the mode dies.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! run(): never { arm1(); arm2(); arm3(); arm4(); arm5(); askHolder('H'); scan(2); askHolder('S'); scan(1); moveAndUnmount(); scan(0); report(); _exit(0); }
//! ```

use super::probe_arms::ProbeStep;
use super::probe_attach_arm::AttachFailure;
use super::probe_child_prelude::{CHILD_CONTROL_DESCRIPTOR, CHILD_STATUS_DESCRIPTOR, close_from, exit_now, join_own_process_group, move_descriptor};
use super::probe_configuration::{checkout_from_configuration, read_configuration, source_from_configuration};
use super::probe_mount_calls::{make_mounts_private, mount_scratch_tmpfs, unshare_mount_namespace};
use super::probe_namespace_scan::{compare_namespaces, scan_namespaces};
use super::probe_open_beneath::open_path_handle;
use super::probe_pipe_lines::{read_line_blocking, read_one_blocking, write_all};
use super::probe_processes::{fork, pipe_pair, raw, wait_blocking};
use super::probe_record_files::RecordPlace;
use super::probe_status_line::{LineFeed, StepAnswer, StepLine};
use super::probe_walk_arm::{make_scratch_tree, open_checkout, refusal_controls, walk_to_slot};
use std::os::fd::{OwnedFd, RawFd};
use std::path::{Path, PathBuf};

/// The directory inside the scratch tmpfs the attach lands on.
pub const SLOT_NAME: &str = "slot";
/// The directory the mount is moved to before its unmount.
pub const MOVE_TARGET_NAME: &str = "move-target";

/// What:
///  What the fork handed the worker:
///  the record's place,
///  the configuration,
///  and its two ends of the pipes it shares with the mode.
/// Why:
///  The worker names every directory itself and reads no argument,
///  so a mode's caller cannot aim the probe at another dataset or checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkerStart = { place: RecordPlace; configuration: string; request: number; reply: number; status: number };
/// ```
#[derive(Clone, Copy)]
pub struct WorkerStart<'a> {
    /// Where the record and the two mount points live.
    pub place: &'a RecordPlace,
    /// The root-owned configuration the source and the checkout come from.
    pub configuration: &'a Path,
    /// The worker's write end of the request pipe.
    pub request: RawFd,
    /// The worker's read end of the mode's reply pipe.
    pub reply: RawFd,
    /// The worker's write end of the status pipe.
    pub status: RawFd,
    /// The step the worker is held after,
    /// which only a test arm sets.
    pub hold: Option<ProbeStep>,
    /// Whether the worker keeps one descriptor on the attached mount,
    /// which only M9's fixture asks for.
    pub keep_descriptor: bool,
}

/// One step's line on the status pipe.
fn say(status: RawFd, line: StepLine) {
    let _written = write_all(status, line.encode().as_bytes());
}

/// The step an attach failure happened at,
/// or the not-exercised answer the design gives the exclusive create's two
/// special errnos.
fn attach_line(failure: AttachFailure) -> StepLine {
    return match (failure.step, failure.errno) {
        (ProbeStep::AttachExclusiveCreate, Some(libc::EOPNOTSUPP)) => StepLine::not_exercised(failure.step, "exclusive-create-unsupported", Some("EOPNOTSUPP")),
        (ProbeStep::AttachExclusiveCreate, Some(libc::EBUSY)) => StepLine::not_exercised(failure.step, "source-held", Some("EBUSY")),
        (ProbeStep::AttachExclusiveCreate, Some(libc::ENOENT)) => StepLine::not_exercised(failure.step, "missing-source", Some("ENOENT")),
        _ => StepLine::failed_number(failure.step, failure.errno),
    };
}

/// What:
///  Run the worker's arms,
///  reporting each step,
///  and end the process.
/// Why:
///  The mode never returns from its fork's child side,
///  and a failure's line is the record's whole account of the arm.
///
/// Never returns:
///  the worker ends itself after its teardown or its first failed arm.
pub fn run(start: &WorkerStart<'_>) -> ! {
    let ending = |line: StepLine| -> ! {
        say(start.status, line);
        exit_now(1);
    };
    let hold_after = |step: ProbeStep| {
        if start.hold == Some(step) {
            let _waited = read_one_blocking(start.reply);
        }
    };
    let _ = join_own_process_group();
    if let Err(error) = unshare_mount_namespace() {
        ending(StepLine::failed_number(ProbeStep::Unshare, Some(error.errno)));
    }
    say(start.status, StepLine::passed(ProbeStep::Unshare));
    hold_after(ProbeStep::Unshare);
    if let Err(error) = make_mounts_private() {
        ending(StepLine::failed_number(ProbeStep::PrivatePropagation, Some(error.errno)));
    }
    say(start.status, StepLine::passed(ProbeStep::PrivatePropagation));
    hold_after(ProbeStep::PrivatePropagation);
    let scratch: PathBuf = start.place.scratch();
    let slot: PathBuf = scratch.join(SLOT_NAME);
    let move_target: PathBuf = scratch.join(MOVE_TARGET_NAME);
    if let Err(error) = mount_scratch_tmpfs(scratch.as_path()) {
        ending(StepLine::failed_number(ProbeStep::ScratchTmpfs, Some(error.errno)));
    }
    say(start.status, StepLine::passed(ProbeStep::ScratchTmpfs));
    hold_after(ProbeStep::ScratchTmpfs);
    if let Err(failure) = make_scratch_tree(scratch.as_path(), slot.as_path(), move_target.as_path()) {
        ending(StepLine::failed_number(ProbeStep::ScratchTree, failure.errno_number()));
    }
    say(start.status, StepLine::passed(ProbeStep::ScratchTree));
    hold_after(ProbeStep::ScratchTree);
    if let Err(failure) = walk_to_slot(scratch.as_path(), slot.as_path()) {
        ending(StepLine::failed_number(ProbeStep::WalkSlot, failure.errno_number()));
    }
    say(start.status, StepLine::passed(ProbeStep::WalkSlot));
    hold_after(ProbeStep::WalkSlot);
    if let Err(failure) = refusal_controls(scratch.as_path(), slot.as_path()) {
        ending(StepLine::failed_number(ProbeStep::WalkRefusalControls, failure.errno_number()));
    }
    say(start.status, StepLine::passed(ProbeStep::WalkRefusalControls));
    hold_after(ProbeStep::WalkRefusalControls);
    let configuration = read_configuration(start.configuration);
    let checkout = configuration.as_ref().ok().and_then(|text| return checkout_from_configuration(text.as_str(), start.configuration).ok());
    match checkout.and_then(|path| return open_checkout(path.as_path()).ok()) {
        Some(_identity) => say(start.status, StepLine::passed(ProbeStep::CheckoutRoot)),
        None => say(start.status, StepLine::not_exercised(ProbeStep::CheckoutRoot, "no-checkout-path", None)),
    }
    let text: String = match configuration {
        Ok(text) => text,
        Err(refusal) => ending(StepLine::not_exercised(ProbeStep::ReadSourceName, refusal.word(), None)),
    };
    let source: String = match source_from_configuration(text.as_str(), start.configuration) {
        Ok(source) => source.snapshot_name(),
        Err(refusal) => ending(StepLine::not_exercised(ProbeStep::ReadSourceName, refusal.word(), None)),
    };
    let mut read = StepLine::passed(ProbeStep::ReadSourceName);
    read.source = Some(source.clone());
    say(start.status, read);
    let slot_handle: OwnedFd = match open_path_handle(slot.as_path()) {
        Ok(handle) => handle,
        Err(error) => ending(StepLine::failed_number(ProbeStep::AttachPlace, Some(error.errno))),
    };
    let evidence = match super::probe_attach_arm::attach_snapshot(&slot_handle, slot.as_path(), source.as_str(), start.keep_descriptor, |step| {
        say(start.status, StepLine::passed(step));
        hold_after(step);
    }) {
        Ok(evidence) => evidence,
        Err(failure) => ending(attach_line(failure)),
    };
    drop(slot_handle);
    let Some(holder_namespace) = ask_holder(start, b'H') else {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    };
    if holder_namespace.is_empty() {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    }
    let own: String = match own_namespace() {
        Ok(namespace) => namespace,
        Err(()) => ending(StepLine::failed(ProbeStep::ScanWhileHeld, None)),
    };
    if scan_expecting(source.as_str(), &[own.as_str(), holder_namespace.as_str()]).is_err() {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ScanWhileHeld));
    hold_after(ProbeStep::ScanWhileHeld);
    if ask_holder(start, b'S').is_none() {
        ending(StepLine::failed(ProbeStep::ScanAfterHolder, None));
    }
    let own: String = match own_namespace() {
        Ok(namespace) => namespace,
        Err(()) => ending(StepLine::failed(ProbeStep::ScanAfterHolder, None)),
    };
    if scan_expecting(source.as_str(), &[own.as_str()]).is_err() {
        ending(StepLine::failed(ProbeStep::ScanAfterHolder, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ScanAfterHolder));
    hold_after(ProbeStep::ScanAfterHolder);
    let mut child = match fork_unmount_child(slot.as_path(), move_target.as_path(), evidence.mount.mount_id, start.hold) {
        Ok(child) => child,
        Err(errno) => ending(StepLine::failed(ProbeStep::Move, errno)),
    };
    let failed: bool = child.relay_until_eof(start.status);
    let _ended = wait_blocking(child.pid);
    if failed {
        exit_now(1);
    }
    if scan_expecting(source.as_str(), &[]).is_err() {
        ending(StepLine::failed(ProbeStep::ClosingScan, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ClosingScan));
    say(start.status, StepLine::passed(ProbeStep::Teardown));
    exit_now(0);
}

/// What:
///  Ask the mode to start or stop the holder,
///  and answer the holder's namespace when it started.
/// Why:
///  Requests are single bytes,
///  and the reply is one line so the first pass can compare the exact set
///  of namespaces with the worker's and the holder's rather than a count.
fn ask_holder(start: &WorkerStart<'_>, request: u8) -> Option<String> {
    if write_all(start.request, &[request]).is_err() {
        return None;
    }
    let line: String = read_line_blocking(start.reply, 128).ok()?;
    if line == "P" {
        return Some(String::new());
    }
    return line.strip_prefix("P ").map(String::from);
}

/// This process's own mount-namespace identity.
fn own_namespace() -> Result<String, ()> {
    return std::fs::read_link("/proc/self/ns/mnt").map(|path| return path.to_string_lossy().into_owned()).map_err(|_| return ());
}

/// Scan this PID namespace for the source and require exactly the namespaces
///  the pass expects,
///  naming a leaked or missing namespace rather than comparing a count.
fn scan_expecting(source: &str, expected: &[&str]) -> Result<(), ()> {
    let mentioned: Vec<String> = match scan_namespaces(Path::new("/proc"), source) {
        Ok(mentioned) => mentioned,
        Err(_) => return Err(()),
    };
    return match compare_namespaces(mentioned.as_slice(), expected) {
        None => Ok(()),
        Some(_difference) => Err(()),
    };
}

/// One forked unmount child and its status pipe's read end.
struct UnmountChild {
    /// The child's pid.
    pid: libc::pid_t,
    /// The read end of the child's status pipe.
    reports: OwnedFd,
    /// The write end of the child's control pipe,
    /// kept open so a hold in the child waits rather than reading its end.
    _control: OwnedFd,
}

impl UnmountChild {
    /// What:
    ///  Relay every line the child writes,
    ///  as it writes it,
    ///  and answer whether one of them failed.
    /// Why:
    ///  The mode can name which of arm 8's steps was in flight only if the
    ///  step's line reaches it before the next step starts,
    ///  so the worker relays while the child lives rather than after it ends.
    ///  The child holds its status pipe open until `_exit`,
    ///  so the end of the pipe is the child's own end.
    fn relay_until_eof(&mut self, status: RawFd) -> bool {
        let mut feed: LineFeed = LineFeed::new();
        let mut failed: bool = false;
        loop {
            let byte: u8 = match read_one_blocking(raw(&self.reports)) {
                Ok(byte) => byte,
                Err(_) => break,
            };
            for parsed in feed.push(&[byte]) {
                match parsed {
                    Ok(step_line) => {
                        if matches!(step_line.answer, StepAnswer::Failed { .. }) {
                            failed = true;
                        }
                        say(status, step_line);
                    }
                    Err(_refusal) => say(status, StepLine::failed(ProbeStep::Move, None)),
                }
            }
        }
        return failed;
    }
}

/// Fork the unmount child with its two pipes.
fn fork_unmount_child(slot: &Path, move_target: &Path, mount_id: u64, hold: Option<ProbeStep>) -> Result<UnmountChild, Option<&'static str>> {
    let control = pipe_pair().map_err(|error| return error.errno_name())?;
    let reports = pipe_pair().map_err(|error| return error.errno_name())?;
    match fork() {
        Err(error) => return Err(error.errno_name()),
        Ok(None) => {
            let _ = move_descriptor(raw(&control.read), CHILD_CONTROL_DESCRIPTOR);
            let _ = move_descriptor(raw(&reports.write), CHILD_STATUS_DESCRIPTOR);
            let _ = close_from(5);
            super::probe_unmount_child::run(CHILD_STATUS_DESCRIPTOR, slot, move_target, mount_id, hold);
        }
        Ok(Some(pid)) => {
            // The child stays in the worker's own process group,
            // so the mode's group kill at the deadline reaches it too.
            drop(control.read);
            drop(reports.write);
            return Ok(UnmountChild { pid, reports: reports.read, _control: control.write });
        }
    }
}
