//! What:
//!  The status pipes' bytes folded into the report:
//!  every whole line a child wrote,
//!  the first line the mode cannot read,
//!  and the end of a pipe with no line at all.
//! Why:
//!   A pipe hands whatever bytes the kernel has,
//!  which is half a line as often as two,
//!  so the reader keeps its tail and reports only whole lines
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The pipes and parent-death signals each child owns").
//!  A line the mode cannot read is an arm failure naming the parse,
//!  never a guess at what the child meant.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! drainStatus(fd, feed, report); // keeps every whole line, marks an unreadable one
//! ```

use super::probe_pipe_lines::{
    ReadSome,
    read_available,
};
use super::probe_report::ProbeReport;
use super::probe_status_line::LineFeed;
use std::os::fd::RawFd;

/// What:
///  How much one drain read,
///  so a caller knows whether a pipe is now at end of file.
/// Why:
///  A pipe that ended says a child wrote nothing more,
///  which is the arm failure the record names rather than an absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Drained = { kind: 'lines' } | { kind: 'end-of-file' } | { kind: 'nothing' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Drained {
    /// One or more whole lines arrived.
    Lines,
    /// The pipe is at its end.
    EndOfFile,
    /// Nothing was there to read.
    Nothing,
}

/// Keep every whole line one status pipe answered,
/// and answer what the read found.
///
/// # Errors
/// Never:
///  a line the mode cannot read is kept as the report's own unreadable-line
///  ending.
pub fn drain_status(
    fd: RawFd,
    feed: &mut LineFeed,
    report: &mut ProbeReport,
) -> Drained {
    let mut drained: Drained = Drained::Nothing;
    loop {
        let mut buffer: [u8; 512] = [0; 512];
        match read_available(fd, buffer.as_mut_slice()).unwrap_or(ReadSome::NothingYet) {
            ReadSome::End => {
                if drained == Drained::Nothing {
                    drained = Drained::EndOfFile;
                }
                break;
            }
            ReadSome::NothingYet => break,
            ReadSome::Bytes(count) => {
                drained = Drained::Lines;
                keep(feed, report, &buffer[..count]);
            }
        }
    }
    return drained;
}

/// Keep the whole lines one read completed.
fn keep(
    feed: &mut LineFeed,
    report: &mut ProbeReport,
    bytes: &[u8],
) {
    for parsed in feed.push(bytes) {
        match parsed {
            Ok(step_line) => report.record(step_line),
            Err(refusal) => report.finish_with(super::probe_report::ChildEnding::UnparsedLine {
                text: refusal.message(),
            }),
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_status_drain_tests.rs"]
mod tests;
