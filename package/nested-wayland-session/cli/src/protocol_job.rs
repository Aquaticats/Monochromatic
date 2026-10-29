//! Parse the Plasma job control verbs.
//!
//! `jobs`,
//!  `job-log [N]`,
//!  `job-clear`,
//!  `job-suspend ID`,
//!  `job-resume ID`,
//!  `job-cancel ID`,
//!  `job-refresh ID`.

/// Imports `no_arguments`,
///  `notification::{parse_count, parse_id}`,
///  `Command` from `super`.
use super::{
    no_arguments,
    notification::{parse_count, parse_id},
    Command,
};

/// A parsed job verb.
///
/// What:
///      `pub enum JobCommand { List, Log, Clear, Suspend, Resume, Cancel, Refresh }`.
/// Why:
///       The executor matches on one typed value,
///  and the parser never touches the bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobCommand {
    /// List the jobs.
    List,
    /// List the newest log records,
    ///  oldest first.
    Log {
        /// How many of the newest records to list.
        count: usize,
    },
    /// Forget the log and the ended jobs.
    Clear,
    /// Emit `suspendRequested` on a job.
    Suspend {
        /// The job number.
        id: u32,
    },
    /// Emit `resumeRequested` on a job.
    Resume {
        /// The job number.
        id: u32,
    },
    /// Emit `cancelRequested` on a job.
    Cancel {
        /// The job number.
        id: u32,
    },
    /// Emit `updateRequested` on a job.
    Refresh {
        /// The job number.
        id: u32,
    },
}

/// Parse a job verb and its arguments.
///
/// What:
///      `pub(super) fn parse(verb, tokens) -> Result<Command, String>`.
///  The four per-job verbs take exactly one id.
pub(super) fn parse(verb: &str, tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let command = match verb {
        "jobs" => return no_arguments(tokens, verb, Command::Job(JobCommand::List)),
        "job-clear" => return no_arguments(tokens, verb, Command::Job(JobCommand::Clear)),
        "job-log" => JobCommand::Log { count: parse_count(tokens, verb)? },
        "job-suspend" => JobCommand::Suspend { id: parse_job_id(tokens, verb)? },
        "job-resume" => JobCommand::Resume { id: parse_job_id(tokens, verb)? },
        "job-cancel" => JobCommand::Cancel { id: parse_job_id(tokens, verb)? },
        "job-refresh" => JobCommand::Refresh { id: parse_job_id(tokens, verb)? },
        other => return Err(format!("unknown command: {other}")),
    };
    return Ok(Command::Job(command));
}

/// Parse the single id of a per-job verb and reject anything after it.
fn parse_job_id(tokens: &mut std::str::SplitWhitespace, verb: &str) -> Result<u32, String> {
    let Some(text) = tokens.next() else {
        return Err(format!("{verb} requires a job id"));
    };
    let id = parse_id(text, verb)?;
    if tokens.next().is_some() {
        return Err(format!("{verb} takes exactly one job id"));
    }
    return Ok(id);
}
