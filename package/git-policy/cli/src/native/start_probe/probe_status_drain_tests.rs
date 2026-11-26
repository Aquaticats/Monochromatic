//! What:
//!  Controls that one drain keeps every whole line,
//!  reports half a line as unfinished,
//!  marks a line it cannot read,
//!  and tells a pipe's end from an empty pipe.
//! Why:
//!  The mode writes the record from these lines,
//!  and a half line read as one would fail a child that answered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! drainStatus(fd, feed, report) // 'lines' then 'end-of-file'
//! ```

use super::super::probe_arms::ProbeStep;
use super::super::probe_processes::{
    pipe_pair,
    raw,
};
use super::super::probe_report::ProbeReport;
use super::super::probe_status_line::{
    LineFeed,
    StepLine,
};
use super::{
    Drained,
    drain_status,
};
use std::os::fd::AsRawFd;

/// A whole line is kept,
/// half a line is not,
/// and the pipe's end is told from an empty pipe.
#[test]
fn whole_lines_are_kept_and_the_end_is_named() {
    let ends = pipe_pair().expect("a pipe is made");
    super::super::probe_pipe_lines::set_nonblocking(raw(&ends.read))
        .expect("the read end is non-blocking");
    let mut feed: LineFeed = LineFeed::new();
    let mut report: ProbeReport = ProbeReport::new();
    assert_eq!(
        drain_status(raw(&ends.read), &mut feed, &mut report),
        Drained::Nothing
    );
    let line: String = StepLine::passed(ProbeStep::Unshare).encode();
    super::super::probe_pipe_lines::write_all(raw(&ends.write), &line.as_bytes()[..8])
        .expect("half a line is written");
    assert_eq!(
        drain_status(raw(&ends.read), &mut feed, &mut report),
        Drained::Lines
    );
    assert!(report.lines().is_empty(), "half a line is no line");
    super::super::probe_pipe_lines::write_all(raw(&ends.write), &line.as_bytes()[8..])
        .expect("the rest is written");
    assert_eq!(
        drain_status(raw(&ends.read), &mut feed, &mut report),
        Drained::Lines
    );
    assert_eq!(report.lines(), &[StepLine::passed(ProbeStep::Unshare)]);
    drop(ends.write);
    assert_eq!(
        drain_status(raw(&ends.read), &mut feed, &mut report),
        Drained::EndOfFile
    );
    let _ = ends.read.as_raw_fd();
}

/// A foreign line is the report's unreadable ending,
/// not a step.
#[test]
fn a_foreign_line_is_marked_unreadable() {
    let ends = pipe_pair().expect("a pipe is made");
    super::super::probe_pipe_lines::set_nonblocking(raw(&ends.read))
        .expect("the read end is non-blocking");
    let mut feed: LineFeed = LineFeed::new();
    let mut report: ProbeReport = ProbeReport::new();
    super::super::probe_pipe_lines::write_all(raw(&ends.write), b"not a status line\n")
        .expect("the line is written");
    assert_eq!(
        drain_status(raw(&ends.read), &mut feed, &mut report),
        Drained::Lines
    );
    assert!(report.lines().is_empty());
    assert!(report.ending().is_some());
}
