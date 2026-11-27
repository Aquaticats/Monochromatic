//! What:
//!  The mode's answers to the worker's two holder requests:
//!  the holder it starts on the first,
//!  the namespace it read from that holder,
//!  and the answer line the worker reads for each request.
//! Why:
//!  The worker asks for the holder with one byte and waits for one line,
//!  so the request in flight,
//!  the holder that runs and the lines it has written are one state the mode
//!  keeps across the turns of its wait,
//!  and holding it here leaves the wait itself to name only what it watches
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms" and "The pipes and parent-death signals each child owns").
//!  A start answers the holder's namespace after a space,
//!  so the worker's first scan compares the exact set of namespaces rather
//!  than a count,
//!  and a start or a stop the holder could not perform answers `F`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answers = HolderAnswers.of(replyPipe); answers.takeRequest(requestPipe); answers.answer(ready, place, report); answers.namespace();
//! ```

use super::probe_holder_control::{
    Arrival,
    Holder,
    reports_of,
};
use super::probe_mode_words::source_of;
use super::probe_pipe_lines::{
    ReadSome,
    read_available,
    write_all,
};
use super::probe_processes::{
    Exit,
    raw,
};
use super::probe_record_files::RecordPlace;
use super::probe_report::ProbeReport;
use super::probe_status_line::LineFeed;
use std::os::fd::{
    OwnedFd,
    RawFd,
};

/// The byte the worker sends to ask for the holder to start.
pub(super) const START_HOLDER: u8 = b'H';
/// The byte the worker sends to ask for the holder to stop.
pub(super) const STOP_HOLDER: u8 = b'S';
/// The reply that says the holder performed the request.
/// A start carries the holder's namespace after a space,
/// so the worker's first pass compares the exact set.
pub(super) const HOLDER_DONE: &str = "P";
/// The reply that says the holder could not perform the request.
pub(super) const HOLDER_FAILED: &str = "F";

/// What:
///  The mode's side of the holder across the turns of one wait:
///  the reply pipe it answers over,
///  the holder it started,
///  the lines that holder wrote and no turn has read yet,
///  and the request it has not answered.
/// Why:
///  The worker sends one request at a time and waits for its line,
///  so the state has one holder,
///  one request in flight and one namespace,
///  and every turn of the mode's loop reads that state in one order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HolderAnswers = { reply: number; holder?: Holder; feed: LineFeed; awaiting?: number; namespace?: string };
/// ```
pub(super) struct HolderAnswers {
    /// The mode's write end of the reply pipe the worker reads.
    reply: OwnedFd,
    /// The holder the mode started,
    /// while it runs.
    holder: Option<Holder>,
    /// The step lines the holder wrote that no turn has read yet.
    feed: LineFeed,
    /// The request byte this mode has not answered yet.
    awaiting: Option<u8>,
    /// The namespace the holder's attach named,
    /// read while the holder lived.
    namespace: Option<String>,
}

impl HolderAnswers {
    /// The answers over one reply pipe,
    /// with no holder started and no request in flight.
    pub(super) fn new(reply: OwnedFd) -> Self {
        return HolderAnswers {
            reply,
            holder: None,
            feed: LineFeed::new(),
            awaiting: None,
            namespace: None,
        };
    }

    /// The descriptor of the running holder's own report pipe,
    /// which the mode watches while a holder runs.
    pub(super) fn watched_descriptor(&self) -> Option<RawFd> {
        return self.holder.as_ref().map(reports_of);
    }

    /// The namespace the holder's attach named,
    /// which the record carries.
    pub(super) fn namespace(&self) -> Option<&str> {
        return self.namespace.as_deref();
    }

    /// Read the request byte the worker wrote,
    /// where the mode saw the request pipe ready and has no request in flight.
    pub(super) fn take_request(
        &mut self,
        request: RawFd,
    ) {
        if self.awaiting.is_some() {
            return;
        }
        let mut byte: [u8; 1] = [0];
        if let Ok(ReadSome::Bytes(_)) = read_available(request, byte.as_mut_slice()) {
            self.awaiting = Some(byte[0]);
        }
    }

    /// What:
    ///  Answer every holder event of one turn of the mode's wait:
    ///  the step lines the holder wrote,
    ///  a start the worker asked for,
    ///  and then a stop it asked for.
    /// Why:
    ///  The mode answers the request it has in flight and no other,
    ///  and a request it cannot perform answers `F` rather than leaving the
    ///  worker waiting for its line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answers.answer(holderReady, place, report);
    /// ```
    pub(super) fn answer(
        &mut self,
        holder_ready: bool,
        place: &RecordPlace,
        report: &mut ProbeReport,
    ) {
        if holder_ready && let Some(held) = self.holder.as_mut() {
            match held.drain(&mut self.feed, report) {
                Arrival::Attach {
                    passed,
                } => {
                    self.namespace = held.namespace();
                    if self.awaiting == Some(START_HOLDER) {
                        let answer: String = match (passed, self.namespace.as_deref()) {
                            (true, Some(namespace)) => format!("{HOLDER_DONE} {namespace}\n"),
                            _ => format!("{HOLDER_FAILED}\n"),
                        };
                        let _ = write_all(raw(&self.reply), answer.as_bytes());
                        self.awaiting = None;
                    }
                }
                Arrival::Gone => {
                    if self.awaiting == Some(START_HOLDER) {
                        let _ =
                            write_all(raw(&self.reply), format!("{HOLDER_FAILED}\n").as_bytes());
                        self.awaiting = None;
                    }
                    self.holder = None;
                }
                Arrival::Nothing => {}
            }
        }
        if self.awaiting == Some(START_HOLDER) && self.holder.is_none() {
            let source: String = source_of(report).unwrap_or_default();
            self.holder = Holder::start(place, source.as_str());
            if self.holder.is_none() {
                let _ = write_all(raw(&self.reply), format!("{HOLDER_FAILED}\n").as_bytes());
                self.awaiting = None;
            }
        }
        if self.awaiting == Some(STOP_HOLDER) {
            if let Some(held) = self.holder.as_mut() {
                held.stop();
                if held.reap() != Exit::StillRunning {
                    let _ = held.drain(&mut self.feed, report);
                    let _ = write_all(raw(&self.reply), format!("{HOLDER_DONE}\n").as_bytes());
                    self.awaiting = None;
                    self.holder = None;
                }
            } else {
                let _ = write_all(raw(&self.reply), format!("{HOLDER_DONE}\n").as_bytes());
                self.awaiting = None;
            }
        }
    }

    /// End the holder that is still running,
    /// which the mode does before it writes the record.
    pub(super) fn end(&mut self) {
        if let Some(held) = self.holder.as_mut() {
            held.end();
        }
    }
}
