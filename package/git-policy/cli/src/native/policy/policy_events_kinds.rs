//! What:
//!  The kinds of event one policy pass produces:
//!  a finding and the byte range it names,
//!  a policy that `warn` does not protect configured as `warn`,
//!  a fixed transform's rejection,
//!  an engine failure and the hint that follows it,
//!  a fix summary,
//!  a landed commit whose automatic backup was stopped,
//!  and a replay that merged parts of a file by word.
//! Why:
//!  The closed set of events and the fields each carries is one subject,
//!  and every reader in the wrapper matches on one of these variants;
//!  the renderer beside this file
//!       owns the field order of the lines they become.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type PolicyEvent = FindingEvent | WarnUnsafe | CoreFinding | EngineFailure | FixSummary | CommitLanded | WordMerged;
//! ```

/// Import the engine-failure codes a failure event carries.
use crate::diagnostics::EngineFailureCode;
/// Import the event form of a pathname,
///  which a finding may carry.
use crate::event_path::EventPath;
/// Import the policy identity and severity an event names.
use crate::policy_registry::{
    PolicyId,
    Severity,
};
/// Import the lifecycle point an event belongs to.
use crate::policy_trigger::Trigger;
/// The code of the warning for a `warn` severity on a policy whose protection `warn` removes.
pub const WARN_UNSAFE_CODE: &str = "warn-unsafe";

/// What:
///  A byte range inside a candidate.
///  A `struct` is a record with named fields;
///  `u64`
///       is an unsigned 64-bit integer (siblings `u32`,
///  `usize`).
/// Why:
///   Content policies point at the exact bytes of a finding.
///  `u64` holds any file
///       offset on every platform,
///  where `usize` would be 32 bits on a 32-bit system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FindingLocation = { byteStart: number; byteEnd: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FindingLocation {
    /// Offset of the first byte.
    pub byte_start: u64,
    /// Offset one past the last byte.
    pub byte_end: u64,
}

/// What:
///  One policy finding as it is reported.
///  `String` is owned UTF-8 text (sibling `&str`
///       borrows);
///  `Option<T>` is "a value or nothing";
///  `&'static str` is text baked into
///       the program.
/// Why:
///   The record owns its message and path because events outlive the check that made
///       them.
///  The code is compiled-in text,
///  so a shipped policy cannot emit an empty or
///       misspelled code at run time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FindingEvent = { trigger; policyId; severity: 'warn' | 'error'; code; message; path?; pathBytes?; location?; fix: 'none' | 'available' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindingEvent {
    /// The lifecycle point that ran the policy.
    pub trigger: Trigger,
    /// The policy that reported.
    pub policy: PolicyId,
    /// The effective severity,
    ///  `Warn` or `Error`.
    pub severity: Severity,
    /// The policy-local code;
    ///  rendering prefixes the policy name.
    pub code: &'static str,
    /// The explanation for the person who ran the command.
    pub message: String,
    /// The repository path the finding is about,
    ///  when it has one.
    pub path: Option<EventPath>,
    /// The byte range the finding is about,
    ///  when it has one.
    pub location: Option<FindingLocation>,
    /// Whether the engine holds a correction for this finding.
    pub fix_available: bool,
}

/// What:
///  Which hint follows an engine failure's message.
/// Why:
///   The code's hint is a guess that holds for most failures of that code;
/// a failure that knows its cause chooses its own
///  hint,
/// or none,
/// so the hint is never one that is untrue for the case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EngineFailureHint = { kind: 'of-code' } | { kind: 'chosen'; hint?: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineFailureHint {
    /// The hint of the failure's code.
    OfCode,
    /// The failure's own hint,
    /// or none.
    Chosen(Option<&'static str>),
}

/// What:
///  The hint of a read the wrapper ended at its bound.
/// Why:
///   The code's own hint speaks of content that is not in this clone and of `git fetch`,
///  which answers a
///       missing object and nothing about a Git that stopped answering;
///  here nothing changed,
///  so the remedy is
///       one more run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STOPPED_READ_HINT = 'Hint: the wrapper ended a Git read that showed no sign of work, so no policy decided anything; the index, the worktree and the staged changes are as they were, so run the command again.';
/// ```
pub const STOPPED_READ_HINT: &str = "Hint: the wrapper ended a Git read that showed no sign of work, so no policy decided anything; the index, the worktree and the staged changes are as they were, so run the command again. If it stops again, check what that Git process waits for with `ps`, and raise the window with CLI_GIT_CHILD_WAIT_SECONDS.";

/// What:
///  The hint of a check that left some work alone rather than overwrite changes it
///       did not write.
/// Why:
///   The code's own hint speaks of a policy that could not read its input,
///  and of retrying;
///  here the check finished and the command goes on,
///  so the remedy is to commit the file the
///       message names or to restore it,
///  and nothing is lost meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LEFT_ALONE_HINT = 'Hint: this command is not stopped and nothing was discarded; ...';
/// ```
pub const LEFT_ALONE_HINT: &str = "Hint: this command is not stopped and nothing was discarded. The message names a file a policy left as it is, because changing it would overwrite the changes it holds; commit that file too, or restore it to match HEAD with `git restore --staged --worktree -- <path>`, and write the command again.";

/// What:
///  The hint of a note whose dependent was left out because a repository read the check
///       needed failed.
/// Why:
///   Nothing of the person's was held or overwritten:
///  the commit keeps that
///       dependent at its version because Git could not answer,
///  so the remedy is to make the
///       read answer and to write the command again,
///  never to commit a file or to restore one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNPROVEN_RIPPLE_HINT = 'Hint: this command is not stopped and nothing was discarded; the message names a dependent that keeps its version because a Git read failed, so correct what Git reported and write the command again.';
/// ```
pub const UNPROVEN_RIPPLE_HINT: &str = "Hint: this command is not stopped and nothing was discarded. The message names a dependent that keeps its version \
     because a Git read this policy needs failed, so this commit lands without that version bump; correct what Git \
     reported, then write the command again.";

/// What:
///  Every event a policy pass can produce.
///  An `enum` is a closed set of named
///       alternatives;
///  each variant carries its own fields.
///  `Vec<EventPath>` is an owned
///       list of pathnames in their event form.
/// Why:
///   One list of events is rendered in order,
///  whatever mixture of kinds it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyEvent = FindingEvent | ConfigurationWarningEvent | CoreFindingEvent | EngineFailureEvent | FixSummaryEvent | CommitLandedEvent | WordMergedEvent;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyEvent {
    /// A policy reported something.
    Finding(FindingEvent),
    /// A policy that `warn` does not protect is configured as `warn`.
    WarnUnsafe {
        /// The lifecycle point that ran the policy.
        trigger: Trigger,
        /// The policy configured as `warn`.
        policy: PolicyId,
    },
    /// A fixed transform rejected the command;
    ///  it has no severity and cannot be configured.
    CoreFinding {
        /// The transform,
        ///  for example `commit-only`.
        core_id: &'static str,
        /// The transform-local code;
        ///  rendering prefixes the transform name.
        code: &'static str,
        /// The explanation for the person who ran the command.
        message: String,
    },
    /// The pass could not complete.
    EngineFailure {
        /// The stable cause.
        code: EngineFailureCode,
        /// Which hint follows the message.
        hint: EngineFailureHint,
        /// The explanation for the person who ran the command.
        message: String,
        /// The lifecycle point,
        ///  when the failure belongs to one.
        trigger: Option<Trigger>,
        /// The policy that could not complete,
        ///  when one is responsible.
        policy: Option<PolicyId>,
        /// The repository path responsible,
        ///  when the failure is about one path.
        path: Option<EventPath>,
    },
    /// Corrections were applied.
    FixSummary {
        /// The fixing lifecycle point.
        trigger: Trigger,
        /// How many passes changed candidate content before it settled.
        passes: u64,
        /// The changed paths,
        ///  each once,
        ///  in Git's byte order.
        changed_paths: Vec<EventPath>,
    },
    /// The landed commit whose automatic backup the post-commit gate stopped.
    ///  The commit stays local;
    /// the event names it so the user can push it deliberately.
    CommitLanded {
        /// The exact landed commit object ID.
        oid: String,
    },
    /// A commit replayed over a branch that moved had files merged by word where Git's line merge reported a conflict.
    ///  The event lists each merged part,
    /// so the person sees what was combined.
    WordMerged {
        /// One line per merged part,
        /// each naming the file and the part.
        lines: Vec<String>,
    },
}

/// What:
///  Whether an event blocks the command.
///  `&PolicyEvent` borrows the event read-only.
/// Why:
///   An error finding and a fixed-transform rejection keep Git from running;
///  a
///       warning finding and a configuration warning do not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blocks = event.type === 'core-finding' || (event.type === 'finding' && event.severity === 'error');
/// ```
pub fn event_blocks(event: &PolicyEvent) -> bool {
    // `match` picks one arm per variant; `{ .. }` ignores the fields a variant carries.
    match event {
        PolicyEvent::Finding(finding) => return finding.severity == Severity::Error,
        PolicyEvent::CoreFinding {
            ..
        } => return true,
        PolicyEvent::WarnUnsafe {
            ..
        }
        | PolicyEvent::EngineFailure {
            ..
        }
        | PolicyEvent::FixSummary {
            ..
        }
        | PolicyEvent::CommitLanded {
            ..
        }
        | PolicyEvent::WordMerged {
            ..
        } => return false,
    }
}
