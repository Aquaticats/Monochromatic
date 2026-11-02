//! What:
//!  The record of every decision of the push trigger after a command that adds commits:
//! handed over,
//! or declined with the step that declined and why.
//! Why:
//!  A merge that was never pushed left nothing to look at,
//! because the trigger returned silently at eight places
//! and the reason sat only in the code.
//! The record is one JSON line appended to a size-capped file in the common Git directory beside the commit timing records,
//! and a debug line under `MONOCHROMATIC_VERBOSE=true`;
//! the person's terminal shows neither.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await appendFile(join(commonDir, 'cli-git-push-trigger.jsonl'), JSON.stringify({ command, outcome, reason }) + '\n');
//! ```

use super::diagnostic_log::debug;
use super::discard_audit::append_line;
use super::iso_time::now_iso;
use super::json_record::ObjectWriter;
use super::service_protocol::RELEASE;
use std::path::Path;

/// The file inside the common Git directory that collects the trigger records of every worktree of the repository.
pub const TRIGGER_FILE: &str = "cli-git-push-trigger.jsonl";

/// The version of the record's fields;
/// it changes only when a field's meaning changes,
/// never when a field is added.
pub const TRIGGER_SCHEMA_VERSION: i64 = 1;

/// Module tag for diagnostics.
const TAG: &str = "pushTrigger";

/// How the trigger ended for one command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The push was handed to the service.
    HandedOver,
    /// The trigger did nothing,
    /// for the reason the record names.
    Declined,
}

impl Outcome {
    /// The word in the record.
    pub fn word(self) -> &'static str {
        return match self {
            Self::HandedOver => "handed-over",
            Self::Declined => "declined",
        };
    }
}

/// What:
///  The line for one decision.
/// Why:
///   The fields say which command,
/// which release,
/// which branch and tips,
/// and the one reason,
/// so a missing push is diagnosed from the file alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ schemaVersion, time, pid, release, command, outcome, reason, detail, branch }) + '\n'
/// ```
pub fn trigger_line(command: &str, outcome: Outcome, reason: &str, detail: &str, branch: Option<&str>) -> String {
    let mut record: ObjectWriter = ObjectWriter::new();
    record
        .integer("schemaVersion", TRIGGER_SCHEMA_VERSION)
        .string("time", now_iso().as_str())
        .integer("pid", i64::from(std::process::id()))
        .string("release", RELEASE)
        .string("command", command)
        .string("outcome", outcome.word())
        .string("reason", reason)
        .string("detail", detail);
    if let Some(name) = branch {
        record.string("branch", name);
    }
    return format!("{}\n", record.finish());
}

/// What:
///  Records one decision:
/// a debug line always (when verbose),
/// and one appended line when the common Git directory is known.
/// Why:
///   The command's result never depends on the record:
/// a file that cannot be written is only a debug line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordDecision(commonDir, 'merge', 'declined', 'upstream-equals-tip', detail, branch)
/// ```
pub fn record_decision(common_dir: Option<&Path>, command: &str, outcome: Outcome, reason: &str, detail: &str, branch: Option<&str>) {
    debug(TAG, &format!("{command}: {} ({reason}) {detail}", outcome.word()));
    let Some(directory) = common_dir else { return };
    let target = directory.join(TRIGGER_FILE);
    let line: String = trigger_line(command, outcome, reason, detail, branch);
    if let Err(error) = append_line(target.as_path(), line.as_bytes()) {
        debug(TAG, &format!("the trigger record could not be written to {}: {error}", target.display()));
    }
}
