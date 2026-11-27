//! What:
//! Stops the delegated control group of one run and waits for the kernel to say it is empty,
//! and reads the `populated` line and the `pids.peak` count of any group.
//! Why:
//! The stop is the one proof the contract needs,
//! that every writer a command started has stopped,
//! so it is read on its own:
//! `cgroup.kill` first,
//! then `SIGKILL` through a pidfd for each listed process,
//! then `populated 0` or a verified absence,
//! all inside the caller's bound,
//! and then,
//! only when the wait was spent,
//! a listing of every member that was left,
//! inside the listing's own bound.
//! `control_group.rs` keeps the type and the launcher that start a run in the group.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-stop.ts
//! ControlGroup.prototype.stop = (boundMs: number): boolean
//! export function populatedAt(directory: string): boolean
//! export function pidsPeakAt(directory: string): number
//! ```

/// What:
/// `use std::fs;` imports the file system calls.
/// Why:
/// `write_text` opens a control file,
/// and the reads here open `cgroup.events` and read a directory's file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { openSync, readFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io::{self, Write};` imports the input/output error type and the write trait.
/// Why:
/// The stop answers an `io::Result`,
/// and a control file's command is written with `write_all`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::{
    self,
    Write,
};
/// What:
/// `use std::os::fd::OwnedFd;` imports the owning wrapper of an open file descriptor.
/// Why:
/// A pidfd opened for every listed process must be closed when it goes out of scope,
/// and `OwnedFd` is what closes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has no equivalent: a number is a descriptor and nothing closes it for you.
/// ```
use std::os::fd::OwnedFd;
/// What:
/// `use std::os::unix::fs::FileExt;` imports the positional read that does not move the file's position.
/// Why:
/// `populated_through` re-reads `cgroup.events` from its start on every pass,
/// which is also what arms the file for `poll`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readSync } from 'node:fs';
/// ```
use std::os::unix::fs::FileExt;
/// What:
/// `use std::path::Path;` imports the borrowed view of a path (its sibling `PathBuf` owns one).
/// Why:
/// A group's directory is lent to the reads here and never stored;
/// the walk that keeps owned paths lives in `control_group_stop_listing.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;
/// What:
/// `use std::time::{Duration, Instant};` imports a length of time and a point on the monotonic clock.
/// Why:
/// The stop's bound is a `Duration` and its deadline is an `Instant`,
/// so a slow host cannot stretch the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() and a millisecond count do the same job.
/// ```
use std::time::{
    Duration,
    Instant,
};

/// What:
/// `use rustix::event::{PollFd, PollFlags, Timespec, poll};` imports the wait on `cgroup.events` for the next change.
/// Why:
/// The stop sleeps on the kernel's own notification rather than polling in a loop,
/// so it costs nothing while it waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { poll, PollFd, PollFlags, Timespec } from 'rustix/event';
/// ```
use rustix::event::{
    PollFd,
    PollFlags,
    Timespec,
    poll,
};
/// What:
/// `use rustix::io::Errno;` imports the error code of a removed group and of an interrupted wait.
/// Why:
/// A removed group reads as unpopulated (`ENODEV`),
/// and `EINTR` ends one wait without ending the stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Errno } from 'rustix/io';
/// ```
use rustix::io::Errno;
/// What:
/// `use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};` imports the pidfd route to signalling one process.
/// Why:
/// A process id read from a list can be reused,
/// so the signal is sent through a descriptor that cannot name another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { pidfdOpen, pidfdSendSignal, Pid, PidfdFlags, Signal } from 'rustix/process';
/// ```
use rustix::process::{
    Pid,
    PidfdFlags,
    Signal,
    pidfd_open,
    pidfd_send_signal,
};

/// What:
/// `mod listing;` declares the listing a stop makes when its bound passed:
/// the members of the scope's own group and of every group below it.
/// Why:
/// The walk and its two reads are a subject of their own,
/// and a file that held both the stop and the walk would pass the code-line limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the listing of the members, beside the stop
/// ```
#[path = "control_group_stop_listing.rs"]
pub(crate) mod listing;
/// What:
/// `mod report;` declares what a stop reports:
/// how long it waited,
/// whether the kernel still reported the scope populated,
/// and the members it listed with the state of their threads.
/// Why:
/// The report is built from the stop's own reading and handed to the refusal of the command,
/// so it lives beside the stop;
/// `#[path]` keeps its file beside this one and named for what it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { StopReport, standingOf } from './control_group_stop_report.ts';
/// ```
#[path = "control_group_stop_report.rs"]
pub(crate) mod report;

/// What:
/// `mod report_tests;` declares the controls of what a stop reports (issue 6528).
/// Why:
/// A unit test belongs in a sidecar file,
/// and declaring it here keeps the crate root's list of modules out of the change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a sidecar test file next to the module it tests
/// ```
#[cfg(test)]
#[path = "control_group_stop_report_tests.rs"]
mod report_tests;

/// What:
/// `use self::report::{Listing, StopReport};` imports what a stop reports and the listing it found.
/// Why:
/// The stop answers a report,
/// and the report must be able to say whether a member it waited for had exited or was alive;
/// the listing itself is made in `control_group_stop_listing.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Listing, StopReport } from './control_group_stop_report.ts';
/// ```
use self::report::{
    Listing,
    StopReport,
};
/// What:
/// `use crate::control_group::ControlGroup;` imports the scope this stop acts on.
/// Why:
/// `stop`,
/// `signal_members` and `absence_is_verified` are methods of that one type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ControlGroup } from 'crate/control_group';
/// ```
use crate::control_group::ControlGroup;
/// What:
/// `use crate::control_group_route_discovery::{STRAGGLER_INTERVAL, View, namespace_id, view_of};` imports how long one wait lasts and how a directory is seen.
/// Why:
/// A wait is at most `STRAGGLER_INTERVAL` long,
/// and the absence of a scope counts only in the `View` (mount and inode) and the `namespace_id` it was planned in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { STRAGGLER_INTERVAL, View, namespaceId, viewOf } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::{
    STRAGGLER_INTERVAL,
    View,
    namespace_id,
    view_of,
};

/// How long reading the members of a scope that was left may take.
/// Why:
/// The listing runs after the stop's own bound,
/// so it is the one wait a refusal adds;
/// without a bound of its own a scope with many threads would hold the refusal for as long as the reads take.
/// What the value rests on (measured 2026-11-27 on this host,
/// release build,
/// a delegated scope of its own for each round,
/// a C fixture that starts N threads that sleep,
/// the stop reporting a member it could not stop so that the listing ran,
/// three rounds):
/// 90 µs to 136 µs with no thread beside the program,
/// 12.5 ms to 15.6 ms with 1000,
/// 37.5 ms to 45.6 ms with 3000.
/// Read linearly,
/// 500 ms covers a scope of about 33000 threads,
/// far more than a build leaves,
/// and it still bounds what a refusal adds after its own bound.
pub(crate) const LISTING_BOUND: Duration = Duration::from_millis(500);

/// The methods that stop the scope,
/// and the reads they wait on.
impl ControlGroup {
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
    /// `pub(crate) fn absence_is_verified(&self) -> io::Result<()>` checks that the absence of the scope can be read as emptiness.
    /// Why:
    /// The manager removes a scope only when it is empty or after stopping its members (probed),
    /// so a scope that is absent from the directory that held it had no member.
    /// That holds only when the directory is seen as it was when the scope was planned (contract assumption c),
    /// and the end of a run reads an absence the same way the stop does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// absenceIsVerified(): void  // throws when the view differs
    /// ```
    pub(crate) fn absence_is_verified(&self) -> io::Result<()> {
        // A test makes a scope's absence unreadable without changing this process's cgroup view, which no run can do.
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::AbsenceUnverifiable) {
            return Err(io::Error::other(
                "injected: the absence of the scope cannot be verified",
            ));
        }
        let namespace: u64 = namespace_id()?;
        if namespace != self.namespace {
            return Err(io::Error::other(
                "this process is in another cgroup namespace than the one the scope was made in",
            ));
        }
        let seen: View = view_of(&self.parent)?;
        if seen != self.parent_view {
            return Err(io::Error::other(format!(
                "{} is seen as {seen} now and was {} when the scope was planned",
                self.parent.display(),
                self.parent_view
            )));
        }
        return Ok(());
    }

    /// What:
    /// `pub(crate) fn stop(&self, bound: Duration) -> io::Result<StopReport>` kills every member and answers a report whose
    /// `empty` is true once the kernel reports the scope unpopulated or its absence is verified,
    /// and whose `empty` is false when `bound` passes first.
    /// Why:
    /// `cgroup.kill` is recursive and asynchronous,
    /// so the answer is the wait for `populated 0`,
    /// which can only be earned.
    /// A member in an uninterruptible wait cannot be killed and that is a report that is not empty;
    /// the report carries how long the wait was and which members were left,
    /// because a bound that passed on a loaded host and one that passed on a member that cannot be killed look the same
    /// without it (issue 6528).
    /// Every other failure is an error,
    /// and the caller reads both as "not quiescent".
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stop(boundMs: number): StopReport
    /// ```
    pub(crate) fn stop(
        &self,
        bound: Duration,
    ) -> io::Result<StopReport> {
        let started: Instant = Instant::now();
        #[cfg(test)]
        {
            if crate::failpoints::fires(crate::failpoints::Point::ContainmentStopError) {
                return Err(io::Error::other("injected failure to stop the scope"));
            }
            if crate::failpoints::fires(crate::failpoints::Point::ContainmentStopStuck) {
                return Ok(self.report_now(started));
            }
        }
        let deadline: Instant = started + bound;
        // A test makes a member survive every signal, so the loop below runs to its deadline as it would for a member in an
        // uninterruptible wait (round 16, V6: an unsuccessful shutdown has a deadline).
        #[cfg(test)]
        let survivor: bool =
            crate::failpoints::fires(crate::failpoints::Point::ContainmentSurvivor);
        #[cfg(not(test))]
        let survivor: bool = false;
        // A test makes the kill and every signal miss the members, so a real member is alive when the bound passes: this is what a
        // member in an uninterruptible wait does on a loaded host (issue 6528).
        #[cfg(test)]
        let unkillable: bool =
            crate::failpoints::fires(crate::failpoints::Point::ContainmentUnkillableMember);
        #[cfg(not(test))]
        let unkillable: bool = false;
        if !unkillable {
            match write_text(&self.directory.join("cgroup.kill"), "1") {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    self.absence_is_verified()?;
                    #[cfg(test)]
                    crate::bounded_run::trace(format_args!("scope proved absent"));
                    return Ok(StopReport::absent(started.elapsed()));
                }
                // The signal pass below does the stopping when the kill file cannot be written.
                Err(error) => eprintln!(
                    "cargo-shadow: warning: cgroup.kill of {} could not be written ({error}); signalling the members one by one",
                    self.directory.display()
                ),
            }
        }
        let events = match fs::File::open(self.directory.join("cgroup.events")) {
            Ok(found) => found,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.absence_is_verified()?;
                #[cfg(test)]
                crate::bounded_run::trace(format_args!("scope proved absent"));
                return Ok(StopReport::absent(started.elapsed()));
            }
            Err(error) => return Err(error),
        };
        loop {
            let populated: bool = populated_through(&events)?;
            if !populated && !survivor && !unkillable {
                // The authoritative proof: the kernel reports the scope unpopulated after the kill (a test build traces it, so a
                // test sees that a successful stop went through this check and not through a snapshot of the members).
                #[cfg(test)]
                crate::bounded_run::trace(format_args!("scope proved empty"));
                // The scope was proved unpopulated, so a listing of it would name no member; none is made here.
                return Ok(StopReport::new(
                    started.elapsed(),
                    true,
                    Some(false),
                    Listing::Read(Vec::new()),
                ));
            }
            if !unkillable {
                self.signal_members()?;
            }
            let now: Instant = Instant::now();
            if now >= deadline {
                // The wait is the time up to the deadline; the listing that follows is its own bounded step and is not part of it.
                let waited: Duration = started.elapsed();
                let listing: Listing = self.listings();
                return Ok(StopReport::new(waited, false, Some(populated), listing));
            }
            let wait: Duration = STRAGGLER_INTERVAL.min(deadline - now);
            let timeout: Timespec = Timespec {
                tv_sec: i64::try_from(wait.as_secs()).unwrap_or(0),
                tv_nsec: i64::from(wait.subsec_nanos()),
            };
            let mut watched = [PollFd::new(&events, PollFlags::PRI)];
            match poll(&mut watched, Some(&timeout)) {
                Ok(_) | Err(Errno::INTR) => {}
                Err(error) => return Err(io::Error::from(error)),
            }
        }
    }

    /// What:
    /// `fn report_now(&self, started: Instant) -> StopReport` is the report of a stop that answered without waiting.
    /// Why:
    /// A test makes the stop report that a member could not be stopped (round 16),
    /// and the answer must describe the scope as it really is,
    /// so the refusal that follows carries true facts about what was left.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reportNow(started: number): StopReport
    /// ```
    #[cfg(test)]
    fn report_now(
        &self,
        started: Instant,
    ) -> StopReport {
        let listing: Listing = self.listings();
        return StopReport::new(started.elapsed(), false, self.population(), listing);
    }

    /// What:
    /// `fn population(&self) -> Option<bool>` is the kernel's `populated` word,
    /// or `None` when the group is gone.
    /// Why:
    /// The sentence must not claim the kernel reported a group that no longer exists.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// population(): boolean | undefined
    /// ```
    #[cfg(test)]
    fn population(&self) -> Option<bool> {
        return match populated_at(&self.directory) {
            Ok(found) => Some(found),
            Err(_) => None,
        };
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
fn write_text(
    path: &Path,
    text: &str,
) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    return file.write_all(text.as_bytes());
}

/// What:
/// `fn parse_populated(text: &str) -> io::Result<bool>` reads the `populated` line of `cgroup.events`.
/// Why:
/// Anything but `populated 0` or `populated 1` is an error,
/// never "empty".
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
    return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("cgroup.events has no populated line: {text:?}"),
    ));
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
        Err(error) if error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) => {
            return Ok(false);
        }
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

/// What:
/// `pub(crate) fn pids_peak_at(directory: &Path) -> io::Result<u64>` reads how many processes have ever been counted in a
/// group,
/// from the `pids.peak` file of its pids controller.
/// Why:
/// The manager watches a unit's `cgroup.events` from the moment it realizes the group,
/// and ends the unit on the empty change
/// (`unit_watch_cgroup` at `src/core/cgroup.c:2011` and `scope_notify_cgroup_empty_event` at `src/core/scope.c:566` of
/// systemd 262.2,
/// the version on this host),
/// so a group that has held a process has already told the manager that it emptied.
/// A group that has held none never changed its `populated` line,
/// which is the state issue 6578 leaves behind;
/// and this is the one kernel statement that tells the two apart without starting a program.
/// The kernel documents the file as "the maximum value that the number of processes in the cgroup and its descendants has
/// ever reached"
/// (`Documentation/admin-guide/cgroup-v2.rst`,
/// "pids.peak"),
/// and it read 1 for a scope whose program ran and 0 for a scope the manager attached an already exiting process to
/// (probed 2026-11-27,
/// kernel 7.3.5-2-cachyos).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pidsPeakAt(directory: string): number
/// ```
pub(crate) fn pids_peak_at(directory: &Path) -> io::Result<u64> {
    let text: String = fs::read_to_string(directory.join("pids.peak"))?;
    return text.trim().parse::<u64>().map_err(|error| {
        return io::Error::new(
            io::ErrorKind::InvalidData,
            format!("pids.peak holds {text:?} ({error})"),
        );
    });
}
