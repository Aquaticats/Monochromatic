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
use super::probe_namespace_scan::scan_namespaces;
use super::probe_open_beneath::open_path_handle;
use super::probe_pipe_lines::{ReadSome, read_available, read_one_blocking, write_all};
use super::probe_processes::{fork, pipe_pair, raw, set_process_group, wait_blocking};
use super::probe_record_files::RecordPlace;
use super::probe_status_line::{StepAnswer, StepLine};
use super::probe_walk_arm::{make_scratch_tree, open_checkout, refusal_controls, walk_to_slot};
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
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
        (ProbeStep::AttachExclusiveCreate, Some(libc::EOPNOTSUPP)) => StepLine::not_exercised(failure.step, "exclusive-create-unsupported"),
        (ProbeStep::AttachExclusiveCreate, Some(libc::EBUSY)) => StepLine::not_exercised(failure.step, "source-held"),
        _ => StepLine::failed(failure.step, failure.errno_name()),
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
    let _ = join_own_process_group();
    if let Err(error) = unshare_mount_namespace() {
        ending(StepLine::failed(ProbeStep::Unshare, error.errno_name()));
    }
    say(start.status, StepLine::passed(ProbeStep::Unshare));
    if let Err(error) = make_mounts_private() {
        ending(StepLine::failed(ProbeStep::PrivatePropagation, error.errno_name()));
    }
    say(start.status, StepLine::passed(ProbeStep::PrivatePropagation));
    let scratch: PathBuf = start.place.scratch();
    let slot: PathBuf = scratch.join(SLOT_NAME);
    let move_target: PathBuf = scratch.join(MOVE_TARGET_NAME);
    if let Err(error) = mount_scratch_tmpfs(scratch.as_path()) {
        ending(StepLine::failed(ProbeStep::ScratchTmpfs, error.errno_name()));
    }
    say(start.status, StepLine::passed(ProbeStep::ScratchTmpfs));
    if let Err(failure) = make_scratch_tree(scratch.as_path(), slot.as_path(), move_target.as_path()) {
        ending(StepLine::failed(ProbeStep::ScratchTree, failure.errno_name()));
    }
    say(start.status, StepLine::passed(ProbeStep::ScratchTree));
    if let Err(failure) = walk_to_slot(scratch.as_path(), slot.as_path()) {
        ending(StepLine::failed(ProbeStep::WalkSlot, failure.errno_name()));
    }
    say(start.status, StepLine::passed(ProbeStep::WalkSlot));
    if let Err(failure) = refusal_controls(scratch.as_path(), slot.as_path()) {
        ending(StepLine::failed(ProbeStep::WalkRefusalControls, failure.errno_name()));
    }
    say(start.status, StepLine::passed(ProbeStep::WalkRefusalControls));
    let configuration = read_configuration(start.configuration);
    let checkout = configuration.as_ref().ok().and_then(|text| return checkout_from_configuration(text.as_str(), start.configuration).ok());
    match checkout.and_then(|path| return open_checkout(path.as_path()).ok()) {
        Some(_identity) => say(start.status, StepLine::passed(ProbeStep::CheckoutRoot)),
        None => say(start.status, StepLine::not_exercised(ProbeStep::CheckoutRoot, "no-checkout-path")),
    }
    let text: String = match configuration {
        Ok(text) => text,
        Err(refusal) => ending(StepLine::not_exercised(ProbeStep::ReadSourceName, refusal.word())),
    };
    let source: String = match source_from_configuration(text.as_str(), start.configuration) {
        Ok(source) => source.snapshot_name(),
        Err(refusal) => ending(StepLine::not_exercised(ProbeStep::ReadSourceName, refusal.word())),
    };
    let mut read = StepLine::passed(ProbeStep::ReadSourceName);
    read.source = Some(source.clone());
    say(start.status, read);
    let slot_handle: OwnedFd = match open_path_handle(slot.as_path()) {
        Ok(handle) => handle,
        Err(error) => ending(StepLine::failed(ProbeStep::AttachPlace, error.errno_name())),
    };
    let evidence = match super::probe_attach_arm::attach_snapshot(&slot_handle, slot.as_path(), source.as_str()) {
        Ok(evidence) => evidence,
        Err(failure) => ending(attach_line(failure)),
    };
    drop(slot_handle);
    for step in [ProbeStep::AttachContext, ProbeStep::AttachSource, ProbeStep::AttachOptions, ProbeStep::AttachExclusiveCreate, ProbeStep::AttachMountDescriptor, ProbeStep::AttachPlace, ProbeStep::VerifyMount] {
        say(start.status, StepLine::passed(step));
    }
    if !ask_holder(start, b'H') {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    }
    if scan_expecting(source.as_str(), 2).is_err() {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ScanWhileHeld));
    if !ask_holder(start, b'S') {
        ending(StepLine::failed(ProbeStep::ScanAfterHolder, None));
    }
    if scan_expecting(source.as_str(), 1).is_err() {
        ending(StepLine::failed(ProbeStep::ScanAfterHolder, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ScanAfterHolder));
    let mut child = match fork_unmount_child(slot.as_path(), move_target.as_path(), evidence.mount.mount_id) {
        Ok(child) => child,
        Err(errno) => ending(StepLine::failed(ProbeStep::Move, errno)),
    };
    let _ended = wait_blocking(child.pid);
    let failed: bool = child.relay(start.status);
    if failed {
        exit_now(1);
    }
    if scan_expecting(source.as_str(), 0).is_err() {
        ending(StepLine::failed(ProbeStep::ClosingScan, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ClosingScan));
    say(start.status, StepLine::passed(ProbeStep::Teardown));
    exit_now(0);
}

/// What:
///  Ask the mode to start or stop the holder,
///  and answer whether it did.
/// Why:
///  Requests are single bytes and the reply is one byte,
///  so neither process waits on a pipe the other is waiting to fill.
fn ask_holder(start: &WorkerStart<'_>, request: u8) -> bool {
    if write_all(start.request, &[request]).is_err() {
        return false;
    }
    return read_one_blocking(start.reply).is_ok_and(|answer| return answer == b'P');
}

/// Scan this PID namespace for the source and require the number of
///  namespaces that mention it,
///  with the worker's own among them.
fn scan_expecting(source: &str, expected: usize) -> Result<(), ()> {
    let own: String = match std::fs::read_link("/proc/self/ns/mnt") {
        Ok(namespace) => namespace.to_string_lossy().into_owned(),
        Err(_) => return Err(()),
    };
    let mentioned: Vec<String> = match scan_namespaces(Path::new("/proc"), source) {
        Ok(mentioned) => mentioned,
        Err(_) => return Err(()),
    };
    if !mentioned.iter().any(|namespace| return namespace == &own) {
        return Err(());
    }
    return if mentioned.len() == expected { Ok(()) } else { Err(()) };
}

/// One forked unmount child and its status pipe's read end.
struct UnmountChild {
    /// The child's pid.
    pid: libc::pid_t,
    /// The read end of the child's status pipe.
    reports: OwnedFd,
}

impl UnmountChild {
    /// Read the child's lines after it ended,
    /// relay each one to the mode,
    /// and answer whether one of them failed.
    fn relay(&mut self, status: RawFd) -> bool {
        let _ = super::probe_pipe_lines::set_nonblocking(raw(&self.reports));
        let mut pending: Vec<u8> = Vec::new();
        let mut failed: bool = false;
        loop {
            let mut buffer: [u8; 512] = [0; 512];
            match read_available(raw(&self.reports), buffer.as_mut_slice()).unwrap_or(ReadSome::NothingYet) {
                ReadSome::Bytes(count) => pending.extend_from_slice(&buffer[..count]),
                ReadSome::NothingYet | ReadSome::End => break,
            }
        }
        for line in pending.split(|byte| return *byte == b'\n') {
            if line.is_empty() {
                continue;
            }
            let Ok(text) = std::str::from_utf8(line) else {
                continue;
            };
            let Ok(parsed) = StepLine::parse(text) else {
                continue;
            };
            if matches!(parsed.answer, StepAnswer::Failed { .. }) {
                failed = true;
            }
            say(status, parsed);
        }
        return failed;
    }
}

/// Fork the unmount child with its two pipes.
fn fork_unmount_child(slot: &Path, move_target: &Path, mount_id: u64) -> Result<UnmountChild, Option<&'static str>> {
    let control = pipe_pair().map_err(|error| return error.errno_name())?;
    let reports = pipe_pair().map_err(|error| return error.errno_name())?;
    match fork() {
        Err(error) => return Err(error.errno_name()),
        Ok(None) => {
            let _ = move_descriptor(raw(&control.read), CHILD_CONTROL_DESCRIPTOR);
            let _ = move_descriptor(raw(&reports.write), CHILD_STATUS_DESCRIPTOR);
            let _ = close_from(5);
            super::probe_unmount_child::run(CHILD_STATUS_DESCRIPTOR, slot, move_target, mount_id);
        }
        Ok(Some(pid)) => {
            let _ = set_process_group(pid);
            drop(control.read);
            drop(reports.write);
            let _ = control.write.as_raw_fd();
            return Ok(UnmountChild { pid, reports: reports.read });
        }
    }
}
