//! What:
//!  The probe's worker:
//!  what the fork hands it,
//!  and the sequence of its arms:
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
use super::probe_child_prelude::{
    exit_now,
    join_own_process_group,
};
use super::probe_configuration::{
    ProbeCheckout,
    checkout_path,
    read_configuration,
    source_name,
};
use super::probe_configuration_refusal::CheckoutRefusal;
use super::probe_mount_calls::{
    make_mounts_private,
    mount_scratch_tmpfs,
    unshare_mount_namespace,
};
use super::probe_open_beneath::open_path_handle;
use super::probe_processes::wait_blocking;
use super::probe_record_files::RecordPlace;
use super::probe_status_line::StepLine;
use super::probe_walk_arm::{
    make_scratch_tree,
    open_checkout,
    refusal_controls,
    walk_to_slot,
};
use super::probe_worker_holder_scans::{
    ask_holder,
    own_namespace,
    scan_expecting,
};
use super::probe_worker_step_lines::{
    attach_line,
    say,
};
use super::probe_worker_unmount_child::fork_unmount_child;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::os::fd::{
    OwnedFd,
    RawFd,
};
use std::path::{
    Path,
    PathBuf,
};

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
            // Twice the shipped bound keeps the step long enough for the
            // deadline to fire,
            // and lets M3's raised-deadline control see the step finish.
            let mut watched: [libc::pollfd; 1] = [libc::pollfd {
                fd: start.reply,
                events: libc::POLLIN,
                revents: 0,
            }];
            let _ = super::probe_processes::poll_descriptors(
                watched.as_mut_slice(),
                Some(super::probe_deadline::PROBE_DEADLINE * 2),
            );
        }
    };
    let _ = join_own_process_group();
    if let Err(error) = unshare_mount_namespace() {
        ending(StepLine::failed_number(
            ProbeStep::Unshare,
            Some(error.errno),
        ));
    }
    say(start.status, StepLine::passed(ProbeStep::Unshare));
    hold_after(ProbeStep::Unshare);
    if let Err(error) = make_mounts_private() {
        ending(StepLine::failed_number(
            ProbeStep::PrivatePropagation,
            Some(error.errno),
        ));
    }
    say(
        start.status,
        StepLine::passed(ProbeStep::PrivatePropagation),
    );
    hold_after(ProbeStep::PrivatePropagation);
    let scratch: PathBuf = start.place.scratch();
    let slot: PathBuf = scratch.join(SLOT_NAME);
    let move_target: PathBuf = scratch.join(MOVE_TARGET_NAME);
    if let Err(error) = mount_scratch_tmpfs(scratch.as_path()) {
        ending(StepLine::failed_number(
            ProbeStep::ScratchTmpfs,
            Some(error.errno),
        ));
    }
    say(start.status, StepLine::passed(ProbeStep::ScratchTmpfs));
    hold_after(ProbeStep::ScratchTmpfs);
    if let Err(failure) =
        make_scratch_tree(scratch.as_path(), slot.as_path(), move_target.as_path())
    {
        ending(StepLine::failed_number(
            ProbeStep::ScratchTree,
            failure.errno_number(),
        ));
    }
    say(start.status, StepLine::passed(ProbeStep::ScratchTree));
    hold_after(ProbeStep::ScratchTree);
    if let Err(failure) = walk_to_slot(scratch.as_path(), slot.as_path()) {
        ending(StepLine::failed_number(
            ProbeStep::WalkSlot,
            failure.errno_number(),
        ));
    }
    say(start.status, StepLine::passed(ProbeStep::WalkSlot));
    hold_after(ProbeStep::WalkSlot);
    if let Err(failure) = refusal_controls(scratch.as_path(), slot.as_path()) {
        ending(StepLine::failed_number(
            ProbeStep::WalkRefusalControls,
            failure.errno_number(),
        ));
    }
    say(
        start.status,
        StepLine::passed(ProbeStep::WalkRefusalControls),
    );
    hold_after(ProbeStep::WalkRefusalControls);
    // The file is read once, and the same bytes answer both the checkout's finding and the source's.
    let document: Option<String> = match read_configuration(start.configuration) {
        Ok(document) => document,
        Err(refusal) => ending(StepLine::not_exercised(
            ProbeStep::ReadSourceName,
            refusal.word(),
            None,
        )),
    };
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    // A checkout this run cannot tell leaves its own arm not exercised and runs the source arm anyway,
    // so a file that names the probe dataset still lets the attach arm report.
    let told: Result<ProbeCheckout, CheckoutRefusal> = checkout_path(document.as_deref(), &mounts);
    let checkout: Option<&ProbeCheckout> = told.as_ref().ok();
    match checkout.and_then(|found| return open_checkout(found.path.as_path()).ok()) {
        Some(_identity) => say(start.status, StepLine::passed(ProbeStep::CheckoutRoot)),
        None => say(
            start.status,
            StepLine::not_exercised(
                ProbeStep::CheckoutRoot,
                told.as_ref()
                    .err()
                    .map_or("no-checkout-path", |refusal| return refusal.word()),
                None,
            ),
        ),
    }
    let source: String =
        match source_name(document.as_deref(), start.configuration, &mounts, checkout) {
            Ok(source) => source.snapshot_name(),
            Err(refusal) => ending(StepLine::not_exercised(
                ProbeStep::ReadSourceName,
                refusal.word(),
                None,
            )),
        };
    let mut read = StepLine::passed(ProbeStep::ReadSourceName);
    read.source = Some(source.clone());
    say(start.status, read);
    let slot_handle: OwnedFd = match open_path_handle(slot.as_path()) {
        Ok(handle) => handle,
        Err(error) => ending(StepLine::failed_number(
            ProbeStep::AttachPlace,
            Some(error.errno),
        )),
    };
    let evidence = match super::probe_attach_arm::attach_snapshot(
        &slot_handle,
        slot.as_path(),
        source.as_str(),
        start.keep_descriptor,
        |step| {
            say(start.status, StepLine::passed(step));
            hold_after(step);
        },
    ) {
        Ok(evidence) => evidence,
        Err(failure) => ending(attach_line(failure)),
    };
    drop(slot_handle);
    let Some(holder_namespace) = ask_holder(start.request, start.reply, b'H') else {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    };
    if holder_namespace.is_empty() {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    }
    let worker_own: String = match own_namespace() {
        Ok(namespace) => namespace,
        Err(()) => ending(StepLine::failed(ProbeStep::ScanWhileHeld, None)),
    };
    if scan_expecting(
        source.as_str(),
        &[worker_own.as_str(), holder_namespace.as_str()],
    )
    .is_err()
    {
        ending(StepLine::failed(ProbeStep::ScanWhileHeld, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ScanWhileHeld));
    hold_after(ProbeStep::ScanWhileHeld);
    if ask_holder(start.request, start.reply, b'S').is_none() {
        ending(StepLine::failed(ProbeStep::ScanAfterHolder, None));
    }
    let worker_own_after: String = match own_namespace() {
        Ok(namespace) => namespace,
        Err(()) => ending(StepLine::failed(ProbeStep::ScanAfterHolder, None)),
    };
    if scan_expecting(source.as_str(), &[worker_own_after.as_str()]).is_err() {
        ending(StepLine::failed(ProbeStep::ScanAfterHolder, None));
    }
    say(start.status, StepLine::passed(ProbeStep::ScanAfterHolder));
    hold_after(ProbeStep::ScanAfterHolder);
    let mut child = match fork_unmount_child(
        slot.as_path(),
        move_target.as_path(),
        evidence.mount.mount_id,
        start.hold,
    ) {
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
