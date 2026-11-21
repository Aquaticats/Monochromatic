//! What:
//!  The mode's own holder child:
//!  starting it,
//!  reading its two step lines,
//!  stopping it and reaping it.
//! Why:
//!  The mode forks the holder rather than the worker,
//!  so the holder inherits no descriptor on the worker's mount and no mount it
//!  did not make,
//!  and the mode is what stops and reaps it between the scan's two passes
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms" and "The pipes and parent-death signals each child owns").
//!  Its two pipes are created immediately before its fork,
//!  its process group is set at once,
//!  and its namespace identity is read while it lives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holder = Holder.start(place, source); holder.drain(feed, report); holder.stop(); holder.reap();
//! ```

use super::probe_arms::ProbeStep;
use super::probe_child_prelude::{CHILD_CONTROL_DESCRIPTOR, CHILD_STATUS_DESCRIPTOR, close_from, move_descriptor};
use super::probe_pipe_lines::{ReadSome, read_available, set_nonblocking, write_all};
use super::probe_processes::{Exit, fork, kill_process, pipe_pair, raw, reap_now, set_process_group};
use super::probe_record_files::RecordPlace;
use super::probe_report::ProbeReport;
use super::probe_status_line::{LineFeed, StepAnswer};
use std::os::fd::{AsRawFd, OwnedFd, RawFd};

/// What:
///  One holder the mode owns.
/// Why:
///  Its pid stops it,
///  its control pipe carries the stop byte,
///  its status pipe carries the two step lines,
///  and the two flags say what the mode has already answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holder = { pid: number; control: OwnedFd; reports: OwnedFd; attach?: boolean; stopped: boolean };
/// ```
pub struct Holder {
    /// The holder's pid.
    pub pid: libc::pid_t,
    /// The write end of its control pipe.
    pub control: OwnedFd,
    /// The read end of its status pipe.
    pub reports: OwnedFd,
    /// What the holder's attach line said,
    /// once it arrived.
    pub attach: Option<bool>,
    /// Whether the mode already wrote the stop byte.
    pub stopped: bool,
}

/// What:
///  What one drain of the holder's status pipe answered.
/// Why:
///  The mode answers the worker's request once,
///  on the line that settles it,
///  and not on every read that found bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Arrival = { kind: 'nothing' } | { kind: 'attach'; passed: boolean } | { kind: 'gone' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arrival {
    /// No whole line arrived.
    Nothing,
    /// The attach line arrived,
    /// with whether it passed.
    Attach {
        /// Whether the attach passed.
        passed: bool,
    },
    /// The holder's pipe ended without another line.
    Gone,
}

impl Holder {
    /// What:
    ///  Fork the holder with its own two pipes.
    /// Why:
    ///  A pipe that cannot be made starts nothing,
    ///  which the worker's request then answers as a failed holder arm.
    pub fn start(place: &RecordPlace, source: &str) -> Option<Holder> {
        let control = pipe_pair().ok()?;
        let reports = pipe_pair().ok()?;
        let holder_path = place.holder();
        match fork().ok()? {
            None => {
                let _ = move_descriptor(raw(&control.read), CHILD_CONTROL_DESCRIPTOR);
                let _ = move_descriptor(raw(&reports.write), CHILD_STATUS_DESCRIPTOR);
                let _ = close_from(5);
                super::probe_holder::run(CHILD_CONTROL_DESCRIPTOR, CHILD_STATUS_DESCRIPTOR, source, holder_path.as_path());
            }
            Some(pid) => {
                let _ = set_process_group(pid);
                drop(control.read);
                drop(reports.write);
                let _ = set_nonblocking(raw(&reports.read));
                return Some(Holder { pid, control: control.write, reports: reports.read, attach: None, stopped: false });
            }
        }
    }

    /// Read what the holder wrote and keep its lines.
    pub fn drain(&mut self, feed: &mut LineFeed, report: &mut ProbeReport) -> Arrival {
        let mut arrival: Arrival = Arrival::Nothing;
        loop {
            let mut buffer: [u8; 512] = [0; 512];
            let read: ReadSome = read_available(raw(&self.reports), buffer.as_mut_slice()).unwrap_or(ReadSome::NothingYet);
            let count: usize = match read {
                ReadSome::Bytes(count) => count,
                ReadSome::NothingYet => break,
                ReadSome::End => {
                    if self.attach.is_none() {
                        arrival = Arrival::Gone;
                    }
                    break;
                }
            };
            for parsed in feed.push(&buffer[..count]) {
                match parsed {
                    Ok(step_line) => {
                        if step_line.step == ProbeStep::HolderAttach {
                            let passed: bool = matches!(step_line.answer, StepAnswer::Passed);
                            self.attach = Some(passed);
                            arrival = Arrival::Attach { passed };
                        }
                        report.record(step_line);
                    }
                    Err(refusal) => report.finish_with(super::probe_report::ChildEnding::UnparsedLine { text: refusal.message() }),
                }
            }
        }
        return arrival;
    }

    /// Write the stop byte once.
    pub fn stop(&mut self) {
        if !self.stopped {
            let _ = write_all(raw(&self.control), b"S");
            self.stopped = true;
        }
    }

    /// Take the holder's ending,
    /// without waiting when it is still running.
    pub fn reap(&mut self) -> Exit {
        return reap_now(self.pid).unwrap_or(Exit::StillRunning);
    }

    /// The holder's mount-namespace identity,
    /// read while it lives.
    pub fn namespace(&self) -> Option<String> {
        return std::fs::read_link(format!("/proc/{}/ns/mnt", self.pid)).ok().map(|path| return path.to_string_lossy().into_owned());
    }

    /// End the holder,
    /// which the deadline does and which a finished run does when the worker
    ///  died before asking for the stop.
    pub fn end(&mut self) {
        let _ = kill_process(self.pid);
        let _ = reap_now(self.pid);
        let _ = self.control.as_raw_fd();
    }
}

/// The descriptor of the holder's status pipe,
/// for the mode's watch list.
pub fn reports_of(held: &Holder) -> RawFd {
    return raw(&held.reports);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_holder_control_tests.rs"]
mod tests;
