//! What:
//!  The `first-attach` record's words and its text:
//!  the file's kind and format version,
//!  the two states and the two outcomes' words,
//!  the block of machine and build facts both records carry,
//!  and the started and finished records a reader parses.
//! Why:
//!  A reader parses one fixed key per line and refuses free text,
//!  so every word and every key's order live here,
//!  and the process that claims the name writes those words rather than
//!  deciding them
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The first real attach's record").
//!  Every key is fixed and no value is a sentence,
//!  so the reader of this file has one place to look for the file's format.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = startedRecord(fingerprint, time); const outcome = outcomeRecord(fingerprint, time, attached);
//! ```

use super::probe_fingerprint::ProbeFingerprint;
use super::probe_record::ABSENT;

/// The file's kind,
/// which tells it from the probe's `probe-result`.
pub const FIRST_ATTACH_KIND: &str = "operation-outcome";

/// The file's format version.
pub const FIRST_ATTACH_VERSION: &str = "v1";

/// The operation every record of this file names.
pub const FIRST_ATTACH_OPERATION: &str = "attach";

/// The started state's word.
pub const STARTED: &str = "started";

/// The finished state's word.
pub const FINISHED: &str = "finished";

/// The attached outcome's word.
pub const ATTACHED: &str = "attached";

/// The refused outcome's word.
pub const REFUSED: &str = "refused";

/// The word a value this build has none of is written as.
pub const NONE: &str = "none";
/// What:
///  What one first real attach did,
///  as the file's fixed keys hold it.
/// Why:
///  A reader parses one key per line and refuses free text,
///  so the two outcomes are two shapes and neither carries a sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FirstAttachOutcome = { kind: 'attached'; mountId; flags; slot; clone; guid } | { kind: 'refused'; slot; clone; guid; step; errno };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FirstAttachOutcome {
    /// The verified mount is at the slot.
    Attached {
        /// The mount the attach made.
        mount_id: u64,
        /// The flags the kernel lists for that mount,
        /// read back after the move.
        flags: String,
        /// The slot the clone's name gave.
        slot: String,
        /// The clone's dataset name.
        clone: String,
        /// The clone's GUID.
        guid: String,
    },
    /// The attach refused from the walk onward,
    /// and the step names the call or check that refused.
    Refused {
        /// The slot the clone's name gave.
        slot: String,
        /// The clone's dataset name.
        clone: String,
        /// The clone's GUID.
        guid: String,
        /// The handler's own step name.
        step: String,
        /// The errno's name,
        /// or `none` where the refusal was a comparison.
        errno: String,
    },
}

impl FirstAttachOutcome {
    /// The file's outcome block,
    /// one fixed key per line.
    fn lines(&self) -> Vec<String> {
        return match self {
            FirstAttachOutcome::Attached {
                mount_id,
                flags,
                slot,
                clone,
                guid,
            } => vec![
                format!("outcome {ATTACHED}"),
                format!("mount {mount_id}"),
                format!("flags {flags}"),
                format!("slot {slot}"),
                format!("clone {clone}"),
                format!("guid {guid}"),
            ],
            FirstAttachOutcome::Refused {
                slot,
                clone,
                guid,
                step,
                errno,
            } => vec![
                format!("outcome {REFUSED}"),
                format!("slot {slot}"),
                format!("clone {clone}"),
                format!("guid {guid}"),
                format!("step {step}"),
                format!("errno {errno}"),
            ],
        };
    }
}

/// The fingerprint lines both records carry,
/// in the design's fixed order.
fn fingerprint_lines(fingerprint: &ProbeFingerprint) -> Vec<String> {
    return vec![
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
            "handler {}",
            fingerprint.handler.as_deref().unwrap_or(ABSENT)
        ),
    ];
}

/// What:
///  The started record,
///  which a winner that dies before its outcome leaves behind.
/// Why:
///  `doctor` reports the started line as an attach begun and not finished,
///  so a killed handler leaves a boot's first attempt visible rather than
///  looking like a boot whose first attach never ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = startedRecord(fingerprint, time);
/// ```
pub fn started_record(
    fingerprint: &ProbeFingerprint,
    time: &str,
) -> String {
    let mut lines: Vec<String> = vec![
        String::from(FIRST_ATTACH_VERSION),
        format!("kind {FIRST_ATTACH_KIND}"),
        format!("state {STARTED}"),
    ];
    lines.extend(fingerprint_lines(fingerprint));
    lines.push(format!("time {time}"));
    lines.push(format!("operation {FIRST_ATTACH_OPERATION}"));
    return format!("{}\n", lines.join("\n"));
}

/// What:
///  The finished record,
///  the outcome the claiming process publishes over its started one.
/// Why:
///  Every key is fixed and no value is free text,
///  so the reader parses the file rather than interpreting a sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = outcomeRecord(fingerprint, time, outcome);
/// ```
pub fn outcome_record(
    fingerprint: &ProbeFingerprint,
    time: &str,
    outcome: &FirstAttachOutcome,
) -> String {
    let mut lines: Vec<String> = vec![
        String::from(FIRST_ATTACH_VERSION),
        format!("kind {FIRST_ATTACH_KIND}"),
        format!("state {FINISHED}"),
    ];
    lines.extend(fingerprint_lines(fingerprint));
    lines.push(format!("time {time}"));
    lines.push(format!("operation {FIRST_ATTACH_OPERATION}"));
    lines.extend(outcome.lines());
    return format!("{}\n", lines.join("\n"));
}
