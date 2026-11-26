//! What:
//!  One companion run,
//!  as the add's preflight makes it:
//!  the scratch directory,
//!  the buffers,
//!  the fork with its wrapper-budget retry,
//!  the wait bounded by the deadline,
//!  the kill and the reaping at the far side of it,
//!  and the verdict the add is decided by.
//! Why:
//!  The check runs in the add's own process,
//!  which is the process's main thread and not a thread the runtime polls,
//!  so the fork holds no worker
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion");
//!  the whole wait is one `poll` on the child's pid descriptor,
//!  and a child a `SIGKILL` cannot end is handed to a reaper rather than waited
//!  on.
//!  The parent names every step from the records it read,
//!  so a skip is a host's answer and a missing record is a failure,
//!  never a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! beforeTheAdd(runtime) {
//!   const run = makeRunDirectory(runtime); const pid = forkWithRetry();
//!   const ended = poll(pidfd, deadline) ? readRecords() : killGroup(pid);
//!   removeRunDirectory(run); return verdictOf(ended);
//! }
//! ```

use super::companion_buffers::{
    CompanionBuffers,
    build,
};
use super::companion_calls::CompanionCall;
use super::companion_calls::{
    Failure,
    failed,
};
use super::companion_child::{
    ChildStart,
    run as run_the_child,
};
use super::companion_deadline::the_deadline;
use super::companion_lines::{
    CompanionVerdict,
    Refusal,
    Skip,
};
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
    RuntimeFolder,
    make_run_directory,
    remove_run_directory,
};
use super::companion_settings::{
    self,
    Settings,
};
use super::companion_steps::{
    CompanionStep,
    STEP_COUNT,
    STEPS_IN_ORDER,
    is_skip,
};
use super::companion_test_switch::{
    Switch,
    from_environment,
};
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
///  The check as the add calls it,
///  with the runtime folder the add resolved:
///  prints every line the run carries and answers whether the managed attach
///  may go ahead.
/// Why:
///  Every line goes out on this one path,
///  so no call site decides what a person reads,
///  and a refusal leaves the route at once,
///  which is what puts the slot on the reflink or ordinary Git route.
pub fn the_managed_attach_may_go_ahead(runtime: &RuntimeFolder) -> bool {
    let verdict: CompanionVerdict = before_the_add(runtime);
    verdict.report();
    return verdict.the_managed_attach_goes_ahead();
}

/// What:
///  The account's runtime folder as this process's environment names it,
///  or the line a person reads when it names none.
/// Why:
///  The add route resolves the folder once and hands it to the check as an input,
///  so the check reads no environment and every control passes a folder of its
///  own;
///  the words of a missing folder stay beside the other refusals,
///  where a person's line is written.
pub fn the_accounts_runtime_folder() -> Option<RuntimeFolder> {
    return match RuntimeFolder::from_environment() {
        Ok(folder) => Some(folder),
        Err(refusal) => {
            let refused: CompanionVerdict = CompanionVerdict::Refused {
                refusal: Refusal::Scratch {
                    message: refusal.message,
                },
            };
            refused.report();
            None
        }
    };
}

/// What:
///  Run the companion and answer what the add does with it.
/// Why:
///  The add calls this before its first write,
///  so a refusal leaves no managed object,
///  and a skip lets the managed attach go ahead.
pub fn before_the_add(runtime: &RuntimeFolder) -> CompanionVerdict {
    let switch: Switch = from_environment();
    return run_under(runtime, switch, the_deadline(switch.is_set()));
}

/// What:
///  One run inside the runtime folder the caller resolved,
///  with the deadline named rather than read,
///  which is what the controls use so no test changes another's environment.
pub fn run_under(
    runtime: &RuntimeFolder,
    switch: Switch,
    deadline: Duration,
) -> CompanionVerdict {
    if switch == Switch::Disabled {
        return CompanionVerdict::NotRun;
    }
    let settings: Settings = companion_settings::read();
    let run: RunDirectory = match make_run_directory(runtime) {
        Ok(run) => run,
        Err(refusal) => {
            return CompanionVerdict::Refused {
                refusal: Refusal::Scratch {
                    message: refusal.message,
                },
            };
        }
    };
    let verdict: CompanionVerdict = run_the_check(&run, switch, &settings, deadline);
    return verdict;
}

/// What:
///  Run the check inside one scratch directory,
///  and answer its verdict.
/// Why:
///  Every ending removes the directory the run made,
///  except the one a child a signal cannot end is still using.
fn run_the_check(
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

/// What:
///  The verdict of one ended run.
/// Why:
///  A signal death,
/// a missing record and a failed step each have their own
///  words,
///  and a host refusal on the skip's list is told apart from a failure by the
///  call and the errno the design names.
fn the_verdict(
    decoded: Decoded,
    ended: Exit,
    settings: &Settings,
) -> CompanionVerdict {
    let step_in_flight: CompanionStep = step_after(decoded.records.len());
    if let Exit::Signaled {
        signal,
    } = ended
    {
        return CompanionVerdict::Refused {
            refusal: Refusal::ChildSignaled {
                step_in_flight,
                signal,
            },
        };
    }
    if decoded.unreadable {
        return CompanionVerdict::Refused {
            refusal: Refusal::NoStatusLine {
                step_in_flight,
            },
        };
    }
    if let Some(failed_record) = decoded
        .records
        .iter()
        .find(|record| return record.failure.is_some())
    {
        let failure: Failure = failed_record
            .failure
            .unwrap_or_else(|| return failed(CompanionCall::WriteRecord));
        if is_skip(
            failed_record.step,
            failure.call,
            failure.errno,
            companion_settings::user_namespace_entry_exists(),
        ) {
            return CompanionVerdict::Skipped {
                skip: Skip {
                    call: failure.call,
                    errno: failure.errno,
                    settings: settings.clone(),
                },
            };
        }
        return CompanionVerdict::Refused {
            refusal: Refusal::ArmFailed {
                step: failed_record.step,
                failure,
            },
        };
    }
    if decoded.records.len() != STEP_COUNT {
        return CompanionVerdict::Refused {
            refusal: Refusal::NoStatusLine {
                step_in_flight,
            },
        };
    }
    return CompanionVerdict::Passed;
}

/// The step after the last whole record,
/// which is the step that was in flight.
fn step_after(records: usize) -> CompanionStep {
    return STEPS_IN_ORDER
        .get(records)
        .copied()
        .unwrap_or(CompanionStep::MoveUnmount);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_parent_tests.rs"]
mod tests;

/// The measurement this round's deadline cites,
/// which is no control and runs only when it is asked for.
#[cfg(test)]
#[path = "companion_cost_measurement_tests.rs"]
mod measurement;
