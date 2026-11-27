//! What:
//!  One companion run inside the scratch directory the add made:
//!  the buffers,
//!  the pipe,
//!  the fork with its wrapper-budget retry,
//!  the wait bounded by the deadline,
//!  the kill and the reaping at the far side of it,
//!  and the verdict that run is decided by.
//! Why:
//!  The check runs in the add's own process,
//!  which is the process's main thread and not a thread the runtime polls,
//!  so the fork holds no worker;
//!  the whole wait is one `poll` on the child's pid descriptor,
//!  and a child a `SIGKILL` cannot end is handed to a reaper rather than waited
//!  on.
//!  The steps live apart from the lines the add calls
//!  (`crate::account_companion::companion_parent`) so each file says one thing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pid = forkWithRetry(); const ended = poll(pidfd, deadline) ? readRecords() : killGroup(pid);
//! removeRunDirectory(run); return verdictOf(ended);
//! ```

use super::companion_buffers::{
    CompanionBuffers,
    build,
};
use super::companion_child::{
    ChildStart,
    run as run_the_child,
};
use super::companion_lines::{
    CompanionVerdict,
    Refusal,
};
use super::companion_parent_verdict::the_verdict;
use super::companion_process_calls::{
    Exit,
    effective_ids,
    fork,
    kill_group,
    pause,
    pid_descriptor,
    pipe_pair,
    read_available,
    ready_within,
    reap_now,
};
use super::companion_records::{
    Decoded,
    decode_all,
};
use super::companion_scratch::{
    RunDirectory,
    remove_run_directory,
};
use super::companion_settings::Settings;
use super::companion_test_switch::Switch;
use std::os::fd::{
    AsRawFd,
    OwnedFd,
};
use std::time::{
    Duration,
    Instant,
};
/// How long the fork is retried inside the wrapper's start budget,
/// which is the budget its own child starts spend.
pub const FORK_RETRY_BUDGET: Duration = Duration::from_secs(5);

/// How long one refused fork waits before the next attempt.
pub const FORK_RETRY_PAUSE: Duration = Duration::from_millis(50);

/// The largest number of bytes one status read takes,
/// which is every step's record and room to spare.
const STATUS_BYTES: usize = 256;

/// What:
///  Run the check inside one scratch directory,
///  and answer its verdict.
/// Why:
///  Every ending removes the directory the run made,
///  except the one a child a signal cannot end is still using.
pub(super) fn run_the_check(
    run: &RunDirectory,
    switch: Switch,
    settings: &Settings,
    deadline: Duration,
) -> CompanionVerdict {
    let (user, group): (u32, u32) = effective_ids();
    let buffers: CompanionBuffers = match build(
        run.scratch.as_path(),
        user,
        group,
        switch.fixture(),
        switch.hold(),
    ) {
        Ok(buffers) => buffers,
        Err(refusal) => {
            remove_run_directory(run);
            return CompanionVerdict::Refused {
                refusal: Refusal::Scratch {
                    message: format!(
                        "the scratch path {} cannot be named ({})",
                        refusal.path, refusal.why
                    ),
                },
            };
        }
    };
    let (reading, writing): (OwnedFd, OwnedFd) = match pipe_pair() {
        Ok(ends) => ends,
        Err(error) => {
            remove_run_directory(run);
            return CompanionVerdict::Refused {
                refusal: Refusal::PipeRefused {
                    errno: error.raw_os_error().unwrap_or(libc::EIO),
                },
            };
        }
    };
    let mut mount_table: Vec<u8> =
        vec![0; usize::try_from(crate::plain_file::MOUNT_TABLE_LIMIT).unwrap_or(1 << 22)];
    let child: libc::pid_t =
        match start_the_child(&buffers, &writing, switch, mount_table.as_mut_slice()) {
            Ok(pid) => pid,
            Err(refusal) => {
                remove_run_directory(run);
                return CompanionVerdict::Refused {
                    refusal,
                };
            }
        };
    drop(writing);
    return wait_for_the_child(run, child, &reading, settings, deadline);
}

/// What:
///  Fork the child,
///  retrying a refusal inside the wrapper's start budget,
///  and let it run.
/// Why:
///  A start the system refused started nothing,
///  so the wrapper's own rule is to ask again,
///  and a budget that runs out is the arm failure the design names.
fn start_the_child(
    buffers: &CompanionBuffers,
    writing: &OwnedFd,
    switch: Switch,
    mount_table: &mut [u8],
) -> Result<libc::pid_t, Refusal> {
    let began: Instant = Instant::now();
    let mut refusals: u32 = 0;
    loop {
        match fork() {
            Ok(None) => {
                if let Switch::Signal {
                    number,
                } = switch
                {
                    // SAFETY: `raise` sends this signal to this process and touches no memory of ours.
                    let _killed: libc::c_int = unsafe { libc::raise(number) };
                }
                if switch == Switch::NoStatus {
                    super::companion_file_calls::exit_now(0);
                }
                run_the_child(
                    &ChildStart {
                        buffers,
                        status: writing.as_raw_fd(),
                    },
                    mount_table,
                );
            }
            Ok(Some(pid)) => {
                let _group = super::companion_process_calls::set_process_group(pid);
                return Ok(pid);
            }
            Err(error) => {
                refusals += 1;
                let errno: i32 = error.raw_os_error().unwrap_or(libc::EAGAIN);
                if began.elapsed() >= FORK_RETRY_BUDGET {
                    return Err(Refusal::ForkRefused {
                        refusals,
                        errno,
                    });
                }
                pause(FORK_RETRY_PAUSE);
            }
        }
    }
}

/// What:
///  Wait for the child under the deadline,
///  kill its group at the far side of it,
///  read the records it wrote,
///  and decide.
/// Why:
///  A missed deadline fails closed,
///  and a child a `SIGKILL` cannot end keeps its directory until the next
///  sweep.
fn wait_for_the_child(
    run: &RunDirectory,
    child: libc::pid_t,
    reading: &OwnedFd,
    settings: &Settings,
    deadline: Duration,
) -> CompanionVerdict {
    let bound: Duration = deadline;
    let ready: bool = pid_descriptor(child)
        .and_then(|descriptor| return ready_within(&descriptor, bound))
        .unwrap_or_default();
    if !ready {
        let _killed = kill_group(child);
        let ended: Exit = reap_now(child).unwrap_or(Exit::StillRunning);
        if ended == Exit::StillRunning {
            // The crate's own reaper collects a child the kernel has not released, whenever it does.
            crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon_pid(child);
            return CompanionVerdict::Refused {
                refusal: Refusal::Deadline {
                    deadline: bound,
                },
            };
        }
        remove_run_directory(run);
        return CompanionVerdict::Refused {
            refusal: Refusal::Deadline {
                deadline: bound,
            },
        };
    }
    let mut bytes: [u8; STATUS_BYTES] = [0; STATUS_BYTES];
    let mut read: usize = 0;
    while read < bytes.len() {
        let Some(more) = read_once(reading, &mut bytes[read..]) else {
            break;
        };
        if more == 0 {
            break;
        }
        read += more;
    }
    let ended: Exit = reap_now(child).unwrap_or(Exit::StillRunning);
    remove_run_directory(run);
    let decoded: Decoded = decode_all(&bytes[..read]);
    return the_verdict(decoded, ended, settings);
}

/// One read of the status pipe,
/// or nothing when the call refused.
fn read_once(
    reading: &OwnedFd,
    into: &mut [u8],
) -> Option<usize> {
    return read_available(reading, into).ok();
}
