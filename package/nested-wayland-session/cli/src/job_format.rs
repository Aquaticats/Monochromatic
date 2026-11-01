//! Wire text for Plasma job answers.
//!
//! Every answer stays on one line and uses only the separators the protocol documents,
//!  so text a hosted app chose cannot break the response framing.

/// Imports the items this module uses from `crate`.
use crate::{
    event_ring::Timed,
    job_store::{JobEvent, JobRecord},
    value_summary::format_summaries,
    wire_text::{encode_text, format_records},
};

/// Formats one job.
///
/// What:
///      `pub fn format_job(job: &JobRecord) -> String`.
///  Fields:
///  `id`,
///  `path`,
///  `entry`,
///  `title`,
///  `info`,
///  `percent`,
///  `suspended`,
///  `capabilities`,
///  `terminated`.
///  A terminated job continues with `code` and `message`.
/// Why:
///       One line carries everything a test compares after `requestView`,
///  `update`,
///  and `terminate`.
#[must_use]
pub fn format_job(job: &JobRecord) -> String {
    let base = format!(
        "id={} path={} entry={} title={} info={} percent={} suspended={} capabilities={}",
        job.id,
        job.path,
        encode_text(&job.desktop_entry),
        encode_text(&job.title),
        encode_text(&job.info_message),
        job.percent,
        job.suspended,
        job.capabilities,
    );
    return match &job.termination {
        None => format!("{base} terminated=false"),
        Some(ended) => format!("{base} terminated=true code={} message={}", ended.code, encode_text(&ended.message)),
    };
}

/// Formats the `jobs` answer payload:
///  a count,
///  then the jobs.
#[must_use]
pub fn format_job_list(jobs: &[JobRecord]) -> String {
    let records: Vec<String> = jobs.iter().map(format_job).collect();
    return format_records(&records);
}

/// Formats one log record:
///  the time,
///  the kind,
///  and the fields of that kind.
///
/// What:
///      `pub fn format_job_event(timed: &Timed<JobEvent>) -> String`.
///  Kinds:
///  `request-view`,
///  `update`,
///  `terminate`,
///  `signal`,
///  and `kuiserver`.
///  `update` and `terminate` show `no-reply`,
///  the flag a Qt caller sets because the methods never answer.
#[must_use]
pub fn format_job_event(timed: &Timed<JobEvent>) -> String {
    let detail = match &timed.event {
        JobEvent::RequestView { id, api, desktop_entry, capabilities, hints } => format!(
            "request-view id={id} api={api} entry={} capabilities={capabilities} hints={}",
            encode_text(desktop_entry),
            format_summaries(hints),
        ),
        JobEvent::Update { id, no_reply, properties } => {
            format!("update id={id} no-reply={no_reply} properties={}", format_summaries(properties))
        }
        JobEvent::Terminate { id, no_reply, code, message, hints } => format!(
            "terminate id={id} no-reply={no_reply} code={code} message={} hints={}",
            encode_text(message),
            format_summaries(hints),
        ),
        JobEvent::Signal { id, name } => format!("signal id={id} name={}", encode_text(name)),
        JobEvent::KuiserverCall { method, detail } => {
            format!("kuiserver method={} detail={}", encode_text(method), encode_text(detail))
        }
    };
    return format!("t={} {detail}", timed.at_ms);
}

/// Formats the `job-log` answer payload:
///  a count,
///  then the records.
#[must_use]
pub fn format_job_event_list(events: &[&Timed<JobEvent>]) -> String {
    let mut records = Vec::with_capacity(events.len());
    for timed in events {
        records.push(format_job_event(timed));
    }
    return format_records(&records);
}

/// Unit tests for the answer text.
#[cfg(test)]
#[path = "job_format_tests.rs"]
mod tests;
