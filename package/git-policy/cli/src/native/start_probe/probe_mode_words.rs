//! What:
//!  The mode's own words and readings:
//!  its verdict line,
//!  the line it says when it could not start,
//!  the source and the source refusal it read from its children's lines,
//!  a child's namespace identity,
//!  and the record a refused gate writes.
//! Why:
//!  The design fixes what a refusal prints and what the record holds
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "What a refusal prints" and "The result record and its fingerprint"),
//!  and a kernel whose gate refused records a skipped probe that enables
//!  nothing rather than leaving a boot unprobed
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  Q4).
//!  Every word here is written from a value the mode read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! verdictLine(verdict); recordSkip(place, paths, 'member');
//! ```

use super::probe_fingerprint::{FingerprintPaths, ProbeFingerprint};
use super::probe_record::ProbeRecord;
use super::probe_record_files::{RecordFileRefusal, RecordPlace};
use super::probe_errno_names::ErrnoWord;
use super::probe_report::{ProbeReport, ProbeVerdict};

/// The line the mode says about a run it could not start.
pub fn unstarted(what: &str, errno: i32) -> String {
    return format!("start probe could not start its {what} (errno {errno}), so no arm ran and this boot has no probe result");
}

/// The one line the mode says about its verdict.
/// It names the arm,
/// the step and the errno a reader needs,
/// the record that holds the rest,
/// and the route an unused slot takes instead.
pub fn verdict_line(verdict: ProbeVerdict, report: &ProbeReport) -> String {
    let record: &str = super::probe_record_files::RECORD_DIRECTORY;
    return match verdict {
        ProbeVerdict::Passed => format!("start probe passed: every arm passed, so managed attaches are enabled for this boot and module identity and the result is in {record}"),
        ProbeVerdict::Failed { step } => {
            let errno: String = first_errno(report).text();
            format!(
                "start probe failed at arm `{}`'s `{}` step (errno {errno}), so managed attaches are refused, an unused slot takes the reflink or ordinary Git route, and the result is in {record}",
                step.arm().name(),
                step.name()
            )
        }
        ProbeVerdict::Blocked { arm } => format!(
            "start probe did not run its `{}` arm, so managed attaches stay blocked, an unused slot takes the reflink or ordinary Git route, and which arm did not run and why is in {record}.{}",
            arm.name(),
            blocked_hint(report)
        ),
        ProbeVerdict::TimedOut { in_flight, last_finished } => format!(
            "start probe missed its deadline: the step in flight was `{}` and the last one that finished was `{}`, so the record in {record} is not current and managed attaches stay blocked",
            in_flight.map_or("none", |step| return step.name()),
            last_finished.map_or("none", |step| return step.name())
        ),
        ProbeVerdict::Skipped { gate_step } => format!("start probe skipped: the compatibility gate refused at `{gate_step}`, so nothing was mounted and no probe verdict exists in {record}"),
    };
}

/// What:
///  The command a blocked record's reader needs,
///  where the arm that did not run was asking for a source the pool does not
///  hold.
/// Why:
///  The design requires the words a person reads to say which setup command
///  creates the probe dataset
///  (`doc/planning/cli-git-zfs-start-probe-design.md`,
///  "The source:
/// the probe dataset"),
///  and the record's own reason word is a closed word rather than a command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hint = blockedHint(report); // ' The probe dataset ... `zfs create` ...' or ''
/// ```
fn blocked_hint(report: &ProbeReport) -> String {
    let mut hint: String = String::new();
    for line in report.lines() {
        let super::probe_status_line::StepAnswer::NotExercised { reason, .. } = &line.answer else {
            continue;
        };
        if *reason == "missing-source" {
            let dataset: String = format!("<pool>/{}", crate::zfs_mounted_dataset::the_designs_datasets_in_the_mount_table::DESIGN_PROBE_DATASET_UNDER_THE_POOL);
            hint.push_str(&format!(
                " The source that step asked for does not exist in this pool: plan step 19's setup creates it with `zfs create -o mountpoint=legacy -o canmount=off -o com.sun:auto-snapshot=false {dataset}` and its empty snapshot with `zfs snapshot {dataset}@ready`, and `doctor` prints both commands with this pool's own name."
            ));
        }
    }
    return hint;
}

/// The first failed step's errno field,
/// or `none` when no step failed.
fn first_errno(report: &ProbeReport) -> ErrnoWord {
    return report.lines().iter().find_map(|line| {
        return match &line.answer {
            super::probe_status_line::StepAnswer::Failed { errno } => Some(*errno),
            super::probe_status_line::StepAnswer::Passed | super::probe_status_line::StepAnswer::NotExercised { .. } => None,
        };
    }).unwrap_or(ErrnoWord::None);
}

/// The source the worker's read-source-name line carried.
pub fn source_of(report: &ProbeReport) -> Option<String> {
    return report.lines().iter().find_map(|line| return line.source.clone());
}

/// The word saying why the source arm did not run.
pub fn source_refusal_word(report: &ProbeReport) -> Option<&'static str> {
    return report.lines().iter().find_map(|line| {
        return match (&line.step, &line.answer) {
            (super::probe_arms::ProbeStep::ReadSourceName, super::probe_status_line::StepAnswer::NotExercised { reason, .. }) => Some(*reason),
            _ => None,
        };
    });
}

/// The mount-namespace identity one live process is in.
pub fn namespace_of(pid: libc::pid_t) -> Option<String> {
    return std::fs::read_link(format!("/proc/{pid}/ns/mnt")).ok().map(|path| return path.to_string_lossy().into_owned());
}

/// What:
///  The record a gate refusal writes:
///  the probe skipped,
///  with the gate's own failing step,
///  and no arms.
/// Why:
///  The design records a gate refusal as a skipped probe that enables nothing,
///  so a boot with a refused gate reads as skipped rather than as an unprobed
///  or passed one
///  (`doc/planning/cli-git-zfs-start-probe-design.md`,
///  "Who runs it,
/// and where").
///
/// # Errors
/// The fingerprint could not be read or the record could not be written.
pub fn skip_after_gate_refusal(place: &RecordPlace, paths: &FingerprintPaths, gate_step: &'static str) -> Result<String, String> {
    let fingerprint: ProbeFingerprint = ProbeFingerprint::read(paths).map_err(|refusal| return refusal.message())?;
    place.prepare().map_err(|refusal: RecordFileRefusal| return refusal.message())?;
    let time: String = crate::iso_time::now_iso();
    let report: ProbeReport = ProbeReport::new();
    let verdict: ProbeVerdict = ProbeVerdict::Skipped { gate_step };
    let record = ProbeRecord {
        fingerprint: &fingerprint,
        time: time.as_str(),
        source: None,
        source_refusal: None,
        report: &report,
        verdict,
        worker_namespace: None,
        holder_namespace: None,
        skip_reason: Some(gate_step),
    };
    place.publish(record.text().as_str()).map_err(|refusal| return refusal.message())?;
    return Ok(verdict_line(verdict, &report));
}


/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_mode_words_tests.rs"]
mod tests;
