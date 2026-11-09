//! What:
//!  Everything one audit record states,
//!  and its line as JSON.
//! Why:
//!  One builder for the line keeps every record the same shape,
//!  and one `write` per line keeps
//!       lines from interleaving when several processes append at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // auditLine(record) === '{"schemaVersion":1,"time":"...","level":"info",...}\n'
//! ```

/// The release and the quoting helpers every field uses.
use crate::discard_audit_facts::{AuditFact, facts_object};
use crate::iso_time::format_iso_milliseconds;
use crate::json_record::{quote, string_array};
use crate::rule_linked_worktree::GuardedCommand;
use crate::service_protocol::RELEASE;
/// The outcome word an audit record carries.
use crate::discard_audit::AuditOutcome;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{Path, PathBuf};

/// What:
///  Everything one record states.
///  A `struct` is a record with named fields.
/// Why:
///   The values travel together to the line builder,
/// and a struct keeps the call readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AuditRecord = { timeMillis: bigint; pid: number; parentPid: number; form: string; outcome: string; /* ... */ };
/// ```
pub struct AuditRecord<'record> {
    /// Milliseconds since the Unix epoch.
    pub time_millis: u128,
    /// The wrapper process.
    pub process: u32,
    /// The process that started it.
    pub parent: u32,
    /// The guarded form.
    pub form: GuardedCommand,
    /// What the policy decided.
    pub outcome: AuditOutcome,
    /// The common Git directory and the worktree kind.
    pub place: &'record (PathBuf, &'static str),
    /// The invocation's own Git directory,
    /// which names the worktree and its service instance.
    pub git_dir: &'record Path,
    /// The working directory of the invocation.
    pub directory: &'record Path,
    /// The whole argument list,
    /// global options included.
    pub arguments: &'record [OsString],
    /// Whether uncommitted state existed,
    /// when the policy asked.
    pub dirty: Option<bool>,
    /// Where the copies of conflicted files were saved,
    /// each as `path => place`,
    /// or `path => none` when there was
    ///  nothing to copy;
    /// empty for every other outcome.
    pub copies: &'record [String],
    /// The named facts this decision read,
    ///  such as the variable that exempted a repository;
    /// empty when the decision needed none.
    pub facts: &'record [AuditFact],
}

/// What:
///  One record as a line of JSON,
/// ending in a newline.
/// Why:
///   One `write` call per line keeps lines from interleaving when several processes append at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function auditLine(record: AuditRecord): string;
/// ```
pub fn audit_line(record: &AuditRecord) -> String {
    let mut rendered: Vec<String> = Vec::new();
    for argument in record.arguments {
        rendered.push(argument.to_string_lossy().into_owned());
    }
    let dirty: &str = match record.dirty {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    let copies: String = if record.copies.is_empty() {
        String::new()
    } else {
        format!(",\"copies\":{}", string_array(record.copies))
    };
    // A refusal is what a reader scanning the file looks for; the rest are decisions that let the command run.
    let level: &str = if record.outcome == AuditOutcome::Rejected { "warn" } else { "info" };
    let time: String = format_iso_milliseconds(i64::try_from(record.time_millis).unwrap_or(i64::MAX));
    return format!(
        "{{\"schemaVersion\":1,\"time\":{},\"level\":\"{level}\",\"timeMillis\":{},\"release\":{},\"pid\":{},\"parentPid\":{},\"worktree\":{},\"gitDir\":{},\"form\":{},\
         \"outcome\":{},\"dirty\":{dirty},\"cwd\":{},\"arguments\":{}{copies}{}}}\n",
        quote(&time),
        quote(&record.time_millis.to_string()),
        quote(RELEASE),
        record.process,
        record.parent,
        quote(record.place.1),
        quote(&record.git_dir.to_string_lossy()),
        quote(&format!("{:?}", record.form)),
        quote(record.outcome.word()),
        quote(&record.directory.to_string_lossy()),
        string_array(&rendered),
        facts_object(record.facts),
    );
}
