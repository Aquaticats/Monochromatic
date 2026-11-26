//! What:
//!  The text of the probe's record:
//!  `v1` and one fixed key per line,
//!  the fingerprint,
//!  the time,
//!  the source,
//!  the two namespace identities,
//!  each arm's outcome,
//!  each step's line,
//!  and the verdict.
//! Why:
//!  `doctor` and the add preflight read this file rather than a second
//!  observer's answer,
//!  so its keys are fixed and no value is free text
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The result record and its fingerprint").
//!  The started record is the same shape with `state started` and no verdict,
//!  which is how a killed probe leaves a boot whose adds refuse rather than a
//!  boot that looks unprobed or passed
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  Q5).
//!  `first-attach` is a second file with its own writer,
//!  so nothing here writes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = record.text(); // 'v1\nkind probe-result\nstate finished\n...'
//! ```

use super::probe_arms::ProbeStep;
use super::probe_errno_names::ErrnoWord;
use super::probe_fingerprint::ProbeFingerprint;
use super::probe_report::{
    ArmAnswer,
    ProbeReport,
    ProbeVerdict,
};
use super::probe_status_line::StepAnswer;

/// The record's format version,
/// which every reader checks first.
pub const RECORD_VERSION: &str = "v1";

/// The record's kind,
/// which tells it from the operation-outcome file beside it.
pub const RECORD_KIND: &str = "probe-result";

/// The value an absent attribute is written as.
pub const ABSENT: &str = "absent";

/// What:
///  Whether the source the record names was there,
///  as the attach arm observed it.
/// Why:
///   `ENOENT` from the exclusive create is the kernel's only word on a missing
///  dataset or snapshot,
///  and a held source answers `EBUSY`,
///  so the record says which of the three it knows instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourcePresence = 'yes' | 'no' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourcePresence {
    /// The attach passed,
    /// or the source was held,
    /// so the kernel found it.
    Yes,
    /// The exclusive create answered `ENOENT`,
    /// so the kernel found no such dataset or snapshot.
    No,
    /// The attach never ran,
    /// so nothing was learned.
    Unknown,
}

impl SourcePresence {
    /// The word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            SourcePresence::Yes => "yes",
            SourcePresence::No => "no",
            SourcePresence::Unknown => "unknown",
        };
    }
}

/// What:
///  Everything one finished record says.
/// Why:
///   The mode holds these values and nothing else,
///  so every field of the file is a value the mode read itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProbeRecord = { fingerprint; time; source?; sourceRefusal?; report; verdict; workerNamespace?; holderNamespace? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeRecord<'a> {
    /// What the record pins about the machine and the build.
    pub fingerprint: &'a ProbeFingerprint,
    /// When the record was written,
    /// as ISO 8601 with offset.
    pub time: &'a str,
    /// The source the probe attached,
    /// where it read one.
    pub source: Option<&'a str>,
    /// The word saying why the source arm could not name a source,
    /// where it could not.
    pub source_refusal: Option<&'a str>,
    /// The lines and the ending the mode read.
    pub report: &'a ProbeReport,
    /// The verdict,
    /// which the mode may have set to `timed-out` or `skipped` itself.
    pub verdict: ProbeVerdict,
    /// The worker's mount-namespace identity.
    pub worker_namespace: Option<&'a str>,
    /// The holder's mount-namespace identity.
    pub holder_namespace: Option<&'a str>,
    /// The gate's failing step,
    /// where a refused gate skipped the probe.
    pub skip_reason: Option<&'a str>,
}

impl ProbeRecord<'_> {
    /// The record's whole text,
    /// its final newline included.
    pub fn text(&self) -> String {
        let mut lines: Vec<String> = self.fingerprint_lines();
        lines.insert(2, String::from("state finished"));
        lines.push(format!("time {}", self.time));
        lines.push(format!("verdict {}", self.verdict.word()));
        lines.push(format!(
            "verdict-step {}",
            self.verdict
                .named_step()
                .map_or("none", |step| return step.name())
        ));
        lines.push(format!(
            "last-finished {}",
            self.report
                .last_finished()
                .map_or("none", |step| return step.name())
        ));
        lines.push(format!("in-flight {}", self.in_flight_word()));
        lines.push(format!(
            "worker-namespace {}",
            self.worker_namespace.unwrap_or("none")
        ));
        lines.push(format!(
            "holder-namespace {}",
            self.holder_namespace.unwrap_or("none")
        ));
        lines.push(format!("source {}", self.source.unwrap_or("none")));
        lines.push(format!(
            "source-refusal {}",
            self.source_refusal.unwrap_or("none")
        ));
        lines.push(format!("source-existed {}", self.source_presence().word()));
        lines.push(format!(
            "skip-reason {}",
            self.skip_reason.unwrap_or("none")
        ));
        for outcome in self.report.arms() {
            let answer: &str = match outcome.answer {
                ArmAnswer::Passed => "passed",
                ArmAnswer::Failed => "failed",
                ArmAnswer::NotExercised => "not-exercised",
            };
            lines.push(format!(
                "arm {} {answer} errno {}",
                outcome.arm.name(),
                outcome.errno.text()
            ));
        }
        for line in self.report.lines() {
            let errno: String = match &line.answer {
                StepAnswer::Failed {
                    errno,
                } => errno.text(),
                StepAnswer::NotExercised {
                    errno,
                    ..
                } => errno.text(),
                StepAnswer::Passed => String::from("none"),
            };
            let reason: String = match &line.answer {
                StepAnswer::NotExercised {
                    reason,
                    ..
                } => format!(" reason {reason}"),
                StepAnswer::Passed
                | StepAnswer::Failed {
                    ..
                } => String::new(),
            };
            lines.push(format!(
                "step {} {} errno {errno}{reason}",
                line.step.name(),
                line.answer.word()
            ));
        }
        return format!("{}\n", lines.join("\n"));
    }

    /// The fingerprint's lines,
    /// shared by the started record and the finished one.
    fn fingerprint_lines(&self) -> Vec<String> {
        let fingerprint: &ProbeFingerprint = self.fingerprint;
        return vec![
            String::from(RECORD_VERSION),
            format!("kind {RECORD_KIND}"),
            format!("boot {}", fingerprint.boot_id),
            format!("kernel {}", fingerprint.kernel_release),
            format!(
                "module {}",
                fingerprint.module_version.as_deref().unwrap_or(ABSENT)
            ),
            format!(
                "srcversion {}",
                fingerprint.module_srcversion.as_deref().unwrap_or(ABSENT)
            ),
            format!(
                "build-id {}",
                fingerprint.module_build_id.as_deref().unwrap_or(ABSENT)
            ),
            format!(
                "handler {}",
                fingerprint.handler.as_deref().unwrap_or(ABSENT)
            ),
        ];
    }

    /// The step a refused or timed-out record names,
    /// in the record's words.
    fn in_flight_word(&self) -> &'static str {
        return match self.verdict {
            ProbeVerdict::TimedOut {
                in_flight,
                ..
            } => in_flight.map_or("none", |step| return step.name()),
            ProbeVerdict::Failed {
                step,
            } => step.name(),
            ProbeVerdict::Passed
            | ProbeVerdict::Blocked {
                ..
            }
            | ProbeVerdict::Skipped {
                ..
            } => "none",
        };
    }

    /// Whether the kernel found the source,
    /// as the attach arm answered.
    fn source_presence(&self) -> SourcePresence {
        let Some(line) = self
            .report
            .lines()
            .iter()
            .find(|line| return line.step == ProbeStep::AttachExclusiveCreate)
        else {
            return SourcePresence::Unknown;
        };
        return match &line.answer {
            StepAnswer::Passed => SourcePresence::Yes,
            StepAnswer::Failed {
                errno: ErrnoWord::Named("ENOENT"),
            }
            | StepAnswer::NotExercised {
                errno: ErrnoWord::Named("ENOENT"),
                ..
            } => SourcePresence::No,
            StepAnswer::Failed {
                errno: ErrnoWord::Named("EBUSY"),
            }
            | StepAnswer::NotExercised {
                errno: ErrnoWord::Named("EBUSY"),
                ..
            } => SourcePresence::Yes,
            StepAnswer::Failed {
                ..
            }
            | StepAnswer::NotExercised {
                ..
            } => SourcePresence::Unknown,
        };
    }
}

/// The started record's own text,
/// which needs no report and no verdict.
pub fn started_text(
    fingerprint: &ProbeFingerprint,
    time: &str,
) -> String {
    let record: ProbeRecord = ProbeRecord {
        fingerprint,
        time,
        source: None,
        source_refusal: None,
        report: &ProbeReport::new(),
        verdict: ProbeVerdict::Blocked {
            arm: super::probe_arms::ProbeArm::Namespace,
        },
        worker_namespace: None,
        holder_namespace: None,
        skip_reason: None,
    };
    let mut lines: Vec<String> = record.fingerprint_lines();
    lines.insert(2, String::from("state started"));
    lines.push(format!("time {time}"));
    return format!("{}\n", lines.join("\n"));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_record_tests.rs"]
mod tests;
