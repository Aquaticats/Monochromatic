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
/// The bytes of a path as the hexadecimal digits a record holds beside its text.
use crate::path_bytes_in_records::hex_of_path;
use crate::rule_linked_worktree::GuardedCommand;
use crate::service_protocol::RELEASE;
/// The outcome word an audit record carries.
use crate::discard_audit::AuditOutcome;
/// The form the wrapper records without refusing it.
use crate::rule_linked_worktree::AuditedCommand;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{Path, PathBuf};

/// What:
///  The form one record is about:
///  a guarded one,
///  or one the wrapper only records.
///  An `enum` is a closed set of named
///       alternatives.
/// Why:
///   Both land in the same file and a reader sortes them by this word,
///  so one type covers both and
///       nothing has to guess which list a word came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DecidedForm = { kind: 'guarded'; command: GuardedCommand } | { kind: 'audited'; command: AuditedCommand }
///   | { kind: 'shared-remote' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecidedForm {
    /// A form the policy decides.
    Guarded(GuardedCommand),
    /// A form the policy only records.
    Audited(AuditedCommand),
    /// A write to the remote settings every worktree of the repository shares,
    ///  which only the hatch of its refusal lets through.
    ///  The arguments of the line name the setting,
    /// so this word says the kind of change and no more.
    SharedRemote,
}

impl DecidedForm {
    /// The word written to the record.
    pub fn word(&self) -> String {
        return match self {
            DecidedForm::Guarded(command) => format!("{command:?}"),
            DecidedForm::Audited(command) => format!("{command:?}"),
            DecidedForm::SharedRemote => String::from("SharedRemote"),
        };
    }
}

/// What:
///  A guarded command as the form of a record.
/// Why:
///   Every record of a decided form is written by the same function,
///  so the conversion lives here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// DecidedForm.fromGuarded(command)
/// ```
impl From<GuardedCommand> for DecidedForm {
    fn from(command: GuardedCommand) -> Self {
        return DecidedForm::Guarded(command);
    }
}

/// What:
///  An audited command as the form of a record.
/// Why:
///   A recorded form that is never refused lands in the same file,
///  so it converts the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// DecidedForm.fromAudited(command)
/// ```
impl From<AuditedCommand> for DecidedForm {
    fn from(command: AuditedCommand) -> Self {
        return DecidedForm::Audited(command);
    }
}

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
    /// The form the decision was about.
    pub form: DecidedForm,
    /// What the policy decided.
    pub outcome: AuditOutcome,
    /// The common Git directory and the worktree kind.
    pub place: &'record (PathBuf, &'static str),
    /// The invocation's own Git directory,
    /// which names the worktree and its service instance,
    /// written as its text and under `gitDirHex` as its bytes.
    pub git_dir: &'record Path,
    /// The working directory of the invocation,
    /// written as its text and under `cwdHex` as its bytes.
    pub directory: &'record Path,
    /// The whole argument list,
    /// global options included,
    /// written as text and under `argumentsHex` as each one's bytes.
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
///   A command's working directory,
///  Git directory and arguments need not be text,
///  and this line exists to attribute a loss to the exact command,
/// so each of them is written beside its text as the bytes it was given as;
/// a reader that takes the text reads what an earlier release wrote.
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
    // The record exists to attribute a lost worktree to the exact command, and a working directory or an argument
    //  need not be text, so each of them is written beside its text as the bytes it was given as (`path_bytes_in_records`).
    let mut arguments_as_bytes: Vec<String> = Vec::with_capacity(record.arguments.len());
    for argument in record.arguments {
        arguments_as_bytes.push(hex_of_path(Path::new(argument)));
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
    // A recorded form is written before Git runs it, so this line cannot know what Git did with the command: the field
    //  says so in words a reader cannot take for the result. Every other outcome is a decision of the wrapper itself,
    //  which is complete before Git is asked to run.
    let before_git: &str = if record.outcome == AuditOutcome::Logged {
        ",\"writtenBeforeGitRuns\":true"
    } else {
        ""
    };
    return format!(
        "{{\"schemaVersion\":1,\"time\":{},\"level\":\"{level}\",\"timeMillis\":{},\"release\":{},\"pid\":{},\"parentPid\":{},\"worktree\":{},\"gitDir\":{},\"gitDirHex\":{},\"form\":{},\
         \"outcome\":{}{before_git},\"dirty\":{dirty},\"cwd\":{},\"cwdHex\":{},\"arguments\":{},\"argumentsHex\":{}{copies}{}}}\n",
        quote(&time),
        quote(&record.time_millis.to_string()),
        quote(RELEASE),
        record.process,
        record.parent,
        quote(record.place.1),
        quote(&record.git_dir.to_string_lossy()),
        quote(&hex_of_path(record.git_dir)),
        quote(&record.form.word()),
        quote(record.outcome.word()),
        quote(&record.directory.to_string_lossy()),
        quote(&hex_of_path(record.directory)),
        string_array(&rendered),
        string_array(&arguments_as_bytes),
        facts_object(record.facts),
    );
}
