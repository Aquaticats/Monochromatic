//! What:
//!  The single-threaded child that enters one mount namespace and runs one
//!  piece of the act there:
//!  the pipe, the fork, the entry, the wait, and the report it reads back.
//! Why:
//!  `setns(CLONE_NEWNS)` from a thread of a multithreaded process is refused
//!  `EINVAL`,
//!  and the same call from a single-threaded process succeeds
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  F15,
//!  and "The preflight"),
//!  so the act forks one single-threaded child per namespace,
//!  enters it there,
//!  and reads or changes that namespace's mounts from inside.
//!  A child that outlived its handler would hold a namespace and every mount in
//!  it,
//!  so it asks the kernel to end it with its parent and rechecks that parent
//!  before it acts,
//!  and the handler waits for it on every path.
//!  What the child hands back is `the_childs_report.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = runInNamespace(namespace, () => takeTheCopyOut(...));  // the child's own answer
//! ```

// What:     every word of the report a caller names through this module,
//           re-exported where they were declared before the split, so a caller
//           that names the child's module still finds its answer there.
// Why:      the act's stages name ActReport, ActOutcome and ChildRefusal through
//           this module's path, and a split that moved them would break each.
//
// In TS you'd write (pseudocode):
// ```ts
// export { ActOutcome, ActReport, ChildRefusal, MAXIMUM_HEADER_BYTES } from './theChildsReport.js';
// ```
use super::the_childs_report::MAXIMUM_TABLE_BYTES;
pub use super::the_childs_report::{
    ActOutcome,
    ActReport,
    ChildRefusal,
    MAXIMUM_HEADER_BYTES,
};
use std::io::Read;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    RawFd,
};

/// What:
///  Enter one mount namespace in a child of this process and run one piece of
///  the act there.
/// Why:
///  The child is single-threaded,
///  which is the only state the kernel lets `setns` change the mount namespace
///  in,
///  and the handler waits for it on every path so no namespace outlives the
///  act.
///
/// # Errors
/// The pipe or the fork was refused,
/// the child's wait failed,
/// the child died before it reported,
/// or its report is not one this build writes.
pub fn run_in<Work>(
    namespace: &std::fs::File,
    work: Work,
) -> Result<ActReport, ChildRefusal>
where
    Work: Fn() -> ActReport,
{
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: `ends` is a live two-element array the call fills with the two descriptors it makes.
    let piped: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    if piped != 0 {
        return Err(ChildRefusal {
            calling: "pipe2",
            errno: std::io::Error::last_os_error().raw_os_error(),
            said: String::from("a report pipe"),
        });
    }
    let read_end: RawFd = ends[0];
    let write_end: RawFd = ends[1];
    // SAFETY: `fork` takes no argument and this call makes the child below run on its own.
    let child: libc::pid_t = unsafe { libc::fork() };
    if child < 0 {
        close(read_end);
        close(write_end);
        return Err(ChildRefusal {
            calling: "fork",
            errno: std::io::Error::last_os_error().raw_os_error(),
            said: String::from("one child per namespace"),
        });
    }
    if child == 0 {
        close(read_end);
        enters_the_namespace(namespace.as_raw_fd(), write_end);
        let report: ActReport = work();
        let bytes: Vec<u8> = report.encode();
        write_all(write_end, bytes.as_slice());
        close(write_end);
        end_now(0);
    }
    close(write_end);
    let mut read: Vec<u8> = Vec::new();
    let bound: u64 =
        u64::try_from(MAXIMUM_HEADER_BYTES + MAXIMUM_TABLE_BYTES + 1).unwrap_or(u64::MAX);
    // SAFETY: the descriptor is this call's own read end and nothing else reads it.
    let mut pipe: std::fs::File = unsafe { std::fs::File::from_raw_fd(read_end) };
    let read_result: std::io::Result<usize> =
        Read::by_ref(&mut pipe).take(bound).read_to_end(&mut read);
    drop(pipe);
    let waited: Result<(), ChildRefusal> = wait_for(child);
    read_result.map_err(|error| {
        return ChildRefusal {
            calling: "read",
            errno: error.raw_os_error(),
            said: String::from("the child's report"),
        };
    })?;
    waited?;
    if read.len() > MAXIMUM_HEADER_BYTES + MAXIMUM_TABLE_BYTES {
        return Err(ChildRefusal {
            calling: "read",
            errno: None,
            said: String::from("the child's report is longer than this build writes"),
        });
    }
    return ActReport::decode(read.as_slice()).map_err(|said| {
        return ChildRefusal {
            calling: "report",
            errno: None,
            said,
        };
    });
}

/// What:
///  Make this forked child tied to its parent's life,
///  and enter the namespace,
///  reporting the refusal through the pipe and ending when the kernel refuses.
/// Why:
///  A child that survived its handler would hold the namespace it entered and
///  every mount in there,
///  so the kernel ends it with its parent and the recheck catches a parent that
///  died before the request was made,
///  and a `setns` the kernel refuses must reach the handler as a report rather
///  than as a silent child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prctl(PR_SET_PDEATHSIG, SIGKILL); if (getppid() === 1) _exit(1);
/// if (setns(namespaceFd, CLONE_NEWNS) !== 0) { write(report, failure(errno)); _exit(1); }
/// ```
fn enters_the_namespace(
    namespace: RawFd,
    report: RawFd,
) {
    // SAFETY: `prctl` reads its four words and touches no memory of ours.
    let _set: libc::c_int = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };
    // SAFETY: `getppid` takes no argument and reads one number.
    if unsafe { libc::getppid() } == 1 {
        end_now(1);
    }
    // SAFETY: the descriptor is inherited from the handler and stays open in this child; no memory of ours is read.
    let entered: libc::c_int = unsafe { libc::setns(namespace, libc::CLONE_NEWNS) };
    if entered == 0 {
        return;
    }
    let errno: i32 = std::io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or(libc::EINVAL);
    let refused: ActReport = ActReport::failed(
        format!("setns into the mount namespace answered errno {errno}"),
        errno,
        String::new(),
    );
    let bytes: Vec<u8> = refused.encode();
    write_all(report, bytes.as_slice());
    // The report is the answer, so the child ends as one that finished; a child that dies without a report is what
    // the handler reads as a child that never answered.
    end_now(0);
}

/// What:
///  Close one descriptor this call owns.
/// Why:
///  Each end of the pipe is closed on the side that does not read or write it,
///  so the reader sees the writer's end of file.
fn close(descriptor: RawFd) {
    // SAFETY: the descriptor is this call's own and is closed exactly once.
    let _closed: libc::c_int = unsafe { libc::close(descriptor) };
}

/// What:
///  Write every byte of one report to the pipe,
///  and say nothing when the reader has gone.
/// Why:
///  A handler that died closes its read end,
///  and the child's own end is then the one thing it must not wait on.
fn write_all(
    descriptor: RawFd,
    mut bytes: &[u8],
) {
    while !bytes.is_empty() {
        // SAFETY: the bytes are this call's own and the descriptor is the child's write end.
        let written: libc::ssize_t = unsafe {
            libc::write(
                descriptor,
                bytes.as_ptr().cast::<libc::c_void>(),
                bytes.len(),
            )
        };
        if written <= 0 {
            return;
        }
        bytes = &bytes[usize::try_from(written).unwrap_or(0)..];
    }
}

/// What:
///  End this child at once,
///  never through the handler's frames.
/// Why:
///  A child that returned would run a second copy of the handler,
///  and `_exit` makes no call the child may not make after a fork.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exit(status) // without running any cleanup of the parent's
/// ```
fn end_now(status: libc::c_int) -> ! {
    // SAFETY: `_exit` ends this process at once and takes no argument but its status.
    unsafe { libc::_exit(status) }
}

/// What:
///  Wait for the child and refuse when it did not end cleanly.
/// Why:
///  A child killed by a signal or exiting nonzero has left a mount half changed,
///  and its report must not be read as an act that finished.
fn wait_for(child: libc::pid_t) -> Result<(), ChildRefusal> {
    let mut status: libc::c_int = 0;
    loop {
        // SAFETY: the status word is this call's own and `waitpid` writes it.
        let waited: libc::pid_t = unsafe { libc::waitpid(child, &mut status, 0) };
        if waited < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(ChildRefusal {
                calling: "waitpid",
                errno: error.raw_os_error(),
                said: String::from("the child that entered a namespace"),
            });
        }
        break;
    }
    let exited: bool = libc::WIFEXITED(status);
    let status_of_exit: libc::c_int = libc::WEXITSTATUS(status);
    if exited && status_of_exit == 0 {
        return Ok(());
    }
    return Err(ChildRefusal {
        calling: "waitpid",
        errno: None,
        said: format!(
            "the child that entered a namespace ended with status {status} (exited {exited}, status {status_of_exit})"
        ),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_namespace_child_tests.rs"]
mod tests;
