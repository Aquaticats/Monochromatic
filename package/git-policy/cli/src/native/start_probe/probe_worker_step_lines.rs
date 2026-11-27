//! What:
//!  The lines the worker says on its status pipe:
//!  one step's line,
//!  and the line an attach failure becomes,
//!  which is the design's not-exercised answer for the exclusive create's
//!  special errnos.
//! Why:
//!  The mode can name the step that was in flight only from these lines,
//!  so the worker writes each step's line as that step ends,
//!  and the worker and its unmount child both say theirs through one
//!  function.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! say(status, StepLine.passed(ProbeStep.Unshare)); const line = attachLine(failure);
//! ```

use super::probe_arms::ProbeStep;
use super::probe_attach_arm::AttachFailure;
use super::probe_pipe_lines::write_all;
use super::probe_status_line::StepLine;
use std::os::fd::RawFd;

/// One step's line on the status pipe.
pub(super) fn say(
    status: RawFd,
    line: StepLine,
) {
    let _written = write_all(status, line.encode().as_bytes());
}

/// The step an attach failure happened at,
/// or the not-exercised answer the design gives the exclusive create's two
/// special errnos.
pub(super) fn attach_line(failure: AttachFailure) -> StepLine {
    return match (failure.step, failure.errno) {
        (ProbeStep::AttachExclusiveCreate, Some(libc::EOPNOTSUPP)) => StepLine::not_exercised(
            failure.step,
            "exclusive-create-unsupported",
            Some("EOPNOTSUPP"),
        ),
        (ProbeStep::AttachExclusiveCreate, Some(libc::EBUSY)) => {
            StepLine::not_exercised(failure.step, "source-held", Some("EBUSY"))
        }
        (ProbeStep::AttachExclusiveCreate, Some(libc::ENOENT)) => {
            StepLine::not_exercised(failure.step, "missing-source", Some("ENOENT"))
        }
        _ => StepLine::failed_number(failure.step, failure.errno),
    };
}
