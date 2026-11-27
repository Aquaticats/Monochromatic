//! What:
//!  The `start-probe` mode:
//!  the fingerprint,
//!  the started record,
//!  the worker and the holder,
//!  the one deadline the mode enforces,
//!  and the finished record it writes from the lines its children sent.
//! Why:
//!  The mode is the only process that writes the record,
//!  and it writes every field from a value it read itself
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "Who runs it,
//! and where").
//!  It enforces the deadline rather than the worker,
//!  because a worker wedged in a kernel wait never reaches its own check,
//!  and on expiry it kills the worker's process group and the holder,
//!  reaps with `WNOHANG`,
//!  writes a timed-out record naming the step in flight and the last one that
//!  finished,
//!  and exits successfully.
//!  A failed or timed-out probe is a verdict in the record,
//!  never a failed unit.
//!  The command line refuses a caller that is not root before the gate is asked
//!  and before this mode starts,
//!  because every write here is root's
//!  (`handler_mode_dispatch`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = new ProbeMode(place, configuration).run(); // writes result and returns one line
//! ```

use super::probe_deadline::{
    deadline_from_environment,
    deadline_passed,
    remaining,
};
use super::probe_fingerprint::{
    FingerprintPaths,
    ProbeFingerprint,
};
use super::probe_mode_holder_answers::HolderAnswers;
use super::probe_mode_words::{
    namespace_of,
    refusal_sentence,
    refusal_word,
    skip_after_gate_refusal,
    source_of,
    source_refusal_word,
    unstarted,
    verdict_line,
};
use super::probe_pipe_lines::set_nonblocking;
use super::probe_processes::{
    Exit,
    fork,
    kill_process,
    kill_process_group,
    pid_descriptor,
    pipe_pair,
    poll_descriptors,
    raw,
    reap_now,
    set_process_group,
};
use super::probe_record::{
    ProbeRecord,
    started_text,
};
use super::probe_record_files::{
    RecordFileRefusal,
    RecordPlace,
};
use super::probe_report::{
    ChildEnding,
    ProbeReport,
    ProbeVerdict,
};
use super::probe_status_drain::drain_status;
use super::probe_status_line::LineFeed;
use super::probe_test_switch::{
    TestSwitch,
    keep_descriptor_from_environment,
    test_switch_from_environment,
};
use std::path::Path;
use std::time::Instant;

/// What:
///  The mode's one run:
///  where the record lives and which configuration names the source.
/// Why:
///  Both are the caller's to name,
///  so a control runs the mode against a fixture directory and a fixture
///  configuration instead of `/run` and this machine's handler config.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mode = new ProbeMode(RecordPlace.at(dir), configPath); mode.runWith(FingerprintPaths.live());
/// ```
pub struct ProbeMode<'a> {
    /// Where the record and the two mount points live.
    place: &'a RecordPlace,
    /// The root-owned configuration.
    configuration: &'a Path,
}

impl<'a> ProbeMode<'a> {
    /// The mode over one place and one configuration.
    pub fn new(
        place: &'a RecordPlace,
        configuration: &'a Path,
    ) -> Self {
        return ProbeMode {
            place,
            configuration,
        };
    }

    /// Run the probe against this machine's own kernel files.
    ///
    /// # Errors
    /// The fingerprint could not be read or the record could not be written.
    pub fn run(&self) -> Result<String, String> {
        return self.run_with(&FingerprintPaths::live());
    }

    /// Run the probe against the given fingerprint paths.
    ///
    /// # Errors
    /// The fingerprint could not be read or the record could not be written.
    pub fn run_with(
        &self,
        paths: &FingerprintPaths,
    ) -> Result<String, String> {
        let switch: TestSwitch = test_switch_from_environment();
        if switch == TestSwitch::Disabled {
            return skip_after_gate_refusal(self.place, paths, "test-switch");
        }
        let fingerprint: ProbeFingerprint =
            ProbeFingerprint::read(paths).map_err(|refusal| return refusal.message())?;
        self.place
            .prepare()
            .map_err(|refusal: RecordFileRefusal| return refusal.message())?;
        let time: String = crate::iso_time::now_iso();
        self.place
            .publish(started_text(&fingerprint, time.as_str()).as_str())
            .map_err(|refusal| return refusal.message())?;
        let hold: Option<super::probe_arms::ProbeStep> = match switch {
            TestSwitch::Hold {
                step,
            } => Some(step),
            TestSwitch::None | TestSwitch::Disabled => None,
        };
        return self.supervise(&fingerprint, time.as_str(), hold);
    }

    /// Fork the worker and the holder,
    /// wait for the worker within the deadline,
    /// and write the finished record.
    fn supervise(
        &self,
        fingerprint: &ProbeFingerprint,
        time: &str,
        hold: Option<super::probe_arms::ProbeStep>,
    ) -> Result<String, String> {
        let deadline: std::time::Duration = deadline_from_environment();
        let keep_descriptor: bool = keep_descriptor_from_environment();
        let request = pipe_pair().map_err(|error| return unstarted("request pipe", error.errno))?;
        let reply = pipe_pair().map_err(|error| return unstarted("reply pipe", error.errno))?;
        let status = pipe_pair().map_err(|error| return unstarted("status pipe", error.errno))?;
        let started: Instant = Instant::now();
        let worker: libc::pid_t =
            match fork().map_err(|error| return unstarted("worker", error.errno))? {
                Some(pid) => pid,
                None => {
                    let start = super::probe_worker::WorkerStart {
                        place: self.place,
                        configuration: self.configuration,
                        request: raw(&request.write),
                        reply: raw(&reply.read),
                        status: raw(&status.write),
                        hold,
                        keep_descriptor,
                    };
                    drop(request.read);
                    drop(reply.write);
                    drop(status.read);
                    super::probe_worker::run(&start);
                }
            };
        let _ = set_process_group(worker);
        let mut worker_namespace: Option<String> = None;
        drop(request.write);
        drop(reply.read);
        drop(status.write);
        let mut answers: HolderAnswers = HolderAnswers::new(reply.write);
        let mut report: ProbeReport = ProbeReport::new();
        let mut feed: LineFeed = LineFeed::new();
        let _ = set_nonblocking(raw(&status.read));
        let _ = set_nonblocking(raw(&request.read));
        let pidfd = pid_descriptor(worker)
            .map_err(|error| return unstarted("worker watch", error.errno))?;
        let mut timed_out: bool = false;
        loop {
            if deadline_passed(started, Instant::now(), deadline) {
                timed_out = true;
                break;
            }
            let mut watched: Vec<libc::pollfd> = vec![
                libc::pollfd {
                    fd: raw(&status.read),
                    events: libc::POLLIN,
                    revents: 0,
                },
                libc::pollfd {
                    fd: raw(&request.read),
                    events: libc::POLLIN,
                    revents: 0,
                },
                libc::pollfd {
                    fd: raw(&pidfd),
                    events: libc::POLLIN,
                    revents: 0,
                },
            ];
            if let Some(holder_reports) = answers.watched_descriptor() {
                watched.push(libc::pollfd {
                    fd: holder_reports,
                    events: libc::POLLIN,
                    revents: 0,
                });
            }
            match poll_descriptors(
                watched.as_mut_slice(),
                remaining(started, Instant::now(), deadline),
            ) {
                Ok(0) => continue,
                Ok(_) => {}
                Err(_) => continue,
            }
            if watched
                .first()
                .is_some_and(|entry| return entry.revents != 0)
            {
                let _ = drain_status(raw(&status.read), &mut feed, &mut report);
            }
            if worker_namespace.is_none()
                && report
                    .lines()
                    .iter()
                    .any(|line| return line.step == super::probe_arms::ProbeStep::Unshare)
            {
                worker_namespace = namespace_of(worker);
            }
            if watched
                .get(1)
                .is_some_and(|entry| return entry.revents != 0)
            {
                answers.take_request(raw(&request.read));
            }
            let holder_ready: bool = watched
                .get(3)
                .is_some_and(|entry| return entry.revents != 0);
            answers.answer(holder_ready, self.place, &mut report);
            if watched
                .get(2)
                .is_some_and(|entry| return entry.revents != 0)
            {
                break;
            }
        }
        let ending: Exit = reap_now(worker).unwrap_or(Exit::StillRunning);
        if timed_out {
            let _ = kill_process_group(worker);
            let _ = kill_process(worker);
        }
        answers.end();
        let _ = drain_status(raw(&status.read), &mut feed, &mut report);
        let verdict: ProbeVerdict = if timed_out {
            ProbeVerdict::TimedOut {
                in_flight: report.in_flight(),
                last_finished: report.last_finished(),
            }
        } else {
            if !report.stopped_on_purpose()
                && report
                    .lines()
                    .iter()
                    .all(|line| return line.step != super::probe_arms::ProbeStep::Teardown)
            {
                report.finish_with(match ending {
                    Exit::Signaled {
                        signal,
                    } => ChildEnding::Signal {
                        number: signal,
                    },
                    _ => ChildEnding::NoStatusLine,
                });
            }
            report.verdict()
        };
        let source: Option<String> = source_of(&report);
        let record = ProbeRecord {
            fingerprint,
            time,
            source: source.as_deref(),
            source_refusal: source_refusal_word(&report),
            report: &report,
            verdict,
            worker_namespace: worker_namespace.as_deref(),
            holder_namespace: answers.namespace(),
            skip_reason: None,
        };
        self.place
            .publish(record.text().as_str())
            .map_err(|refusal| return refusal.message())?;
        let line: String = verdict_line(verdict, &report);
        // A refusal about the configuration file gets its sentence in the mode as well: the record keeps closed words,
        // and a person reads what the file, the table or the setup command needs (the project's second and fourth
        // answers of 2026-11-22).
        if let Some(word) = refusal_word(&report)
            && let Some(sentence) = refusal_sentence(
                word,
                self.configuration,
                &crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets,
            )
        {
            return Ok(format!("{line}\n{sentence}"));
        }
        return Ok(line);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_mode_tests.rs"]
mod tests;
