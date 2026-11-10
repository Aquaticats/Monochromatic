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
//! await appendFile(join(commonDir, 'monochromatic-git-policy-cli-push-trigger.jsonl'), JSON.stringify({ command, outcome, reason }) + '\n');
//! ```

use crate::wrapper_names_old_and_new::PUSH_TRIGGER_LOG;
use crate::diagnostic_log::debug;
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_line;
use crate::iso_time::now_iso;
use crate::json_record::ObjectWriter;
use crate::lfs_output_mask::mask_userinfo_text;
use crate::service_instance::instance_of;
use crate::service_protocol::RELEASE;
use std::path::Path;

/// The file inside the common Git directory that collects the trigger records of every worktree of the repository.
pub const TRIGGER_FILE: &str = PUSH_TRIGGER_LOG.current;

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

/// The worktree a decision was made in;
/// the file is shared by every worktree of the repository,
/// so a line must say which.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Origin<'place> {
    /// The worktree's top level.
    pub worktree: &'place Path,
    /// The worktree's own Git directory,
    /// which names its service instance.
    pub git_dir: &'place Path,
}

/// Everything one line states.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Decision<'record> {
    /// The common Git directory that holds the file;
    /// nothing is written without it.
    pub common_dir: Option<&'record Path>,
    /// The worktree the command ran in.
    pub origin: Option<Origin<'record>>,
    /// The command word.
    pub command: &'record str,
    /// How the trigger ended.
    pub outcome: Outcome,
    /// The step,
    /// in a stable word.
    pub reason: &'record str,
    /// What the step read,
    /// as text.
    pub detail: &'record str,
    /// The branch the command was on,
    /// when it was known.
    pub branch: Option<&'record str>,
}

/// The reasons that mean the wrapper could not decide or could not hand the push over,
/// so a reader scanning the file
/// finds them;
/// every other reason is a decision working as intended (a branch that did not grow,
/// a detached `HEAD`).
const WARN_REASONS: [&str; 9] = [
    "layout-declined",
    "head-unreadable",
    "branch-has-no-tip",
    "object-database-unopened",
    "upstream-config-unreadable",
    "walk-undecided-growth",
    "walk-undecided-ahead",
    "no-route",
    "handover-failed",
];

/// The level a line states:
/// `warn` for a decision the wrapper could not make or a hand-over that failed,
/// else `info`.
pub fn level_of(outcome: Outcome, reason: &str) -> &'static str {
    if outcome == Outcome::Declined && WARN_REASONS.contains(&reason) {
        return "warn";
    }
    return "info";
}

/// What:
///  The line for one decision.
/// Why:
///   The fields say which worktree (`worktree`,
/// `gitDir`,
/// `instance`),
/// which command,
/// which release,
/// which branch and the
///  one reason with what it read,
/// so a missing push is diagnosed from the file alone;
/// the userinfo of a URL in the
///  detail is masked,
/// because a failure the service reports can name a remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ schemaVersion, time, level, pid, release, worktree, gitDir, instance, command, outcome, reason, detail, branch }) + '\n'
/// ```
pub fn trigger_line(decision: &Decision<'_>) -> String {
    let mut record: ObjectWriter = ObjectWriter::new();
    record
        .integer("schemaVersion", TRIGGER_SCHEMA_VERSION)
        .string("time", now_iso().as_str())
        .string("level", level_of(decision.outcome, decision.reason))
        .integer("pid", i64::from(std::process::id()))
        .string("release", RELEASE);
    if let Some(origin) = decision.origin {
        record
            .string("worktree", &origin.worktree.to_string_lossy())
            .string("gitDir", &origin.git_dir.to_string_lossy())
            .string("instance", instance_of(origin.git_dir).as_str());
    }
    record
        .string("command", decision.command)
        .string("outcome", decision.outcome.word())
        .string("reason", decision.reason)
        .string("detail", mask_userinfo_text(decision.detail).as_str());
    if let Some(name) = decision.branch {
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
/// recordDecision({ commonDir, origin, command: 'merge', outcome: 'declined', reason: 'upstream-equals-tip', detail, branch })
/// ```
pub fn record_decision(decision: &Decision<'_>) {
    debug(TAG, &format!("{}: {} ({}) {}", decision.command, decision.outcome.word(), decision.reason, mask_userinfo_text(decision.detail)));
    let Some(directory) = decision.common_dir else { return };
    let target = directory.join(TRIGGER_FILE);
    let line: String = trigger_line(decision);
    if let Err(error) = append_line(target.as_path(), line.as_bytes(), &UNPINNED_LOG) {
        debug(TAG, &format!("the trigger record could not be written to {}: {error}", target.display()));
    }
}

/// Who the record names,
/// its level and its masking.
#[cfg(test)]
#[path = "push_trigger_log_tests.rs"]
mod tests;
