//! What:
//!  The propagation record one root run publishes:
//!  the fields it carries,
//!  and the text its file holds.
//! Why:
//!  One record answers for one checkout and names the instance it answers
//!  about,
//!  so a reader compares it with the live mount instead of trusting it
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The propagation record").
//!  A refusal still writes a record,
//!  with `none` for every field it never reached and `unread` for a count of
//!  mounts beneath it never looked at,
//!  so `doctor` can tell "the step never looked" from "nothing was found".
//!  The writer proves its own parser,
//! which sits beside this file,
//! reads the
//!  text back before a byte is written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = record.text(); // 'v1\nkind checkout-propagation\n...'
//! const parsed = parseRecord(text); // throws PropagationParseRefusal
//! ```

use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::checkout_propagation::propagation_words::{
    BENEATH_PATH_KEY,
    NONE_WORD,
    PROPAGATION_KIND,
    PROPAGATION_VERSION,
    PropagationOutcome,
    PropagationStep,
};
use crate::checkout_propagation::shared_mounts_beneath::SharedMountsBeneath;

/// What:
///  One run's answer about one checkout.
/// Why:
///  The step,
/// its refusal and its reader all read these fields,
///  and the file is the same values as one text,
///  so one type holds both and no reader reassembles the fields itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationRecord = { boot; mountId?; mountIdOld?; mountNamespaceId?; checkout?; source?; before?; after?; master?; beneath; outcome; step; errno?; time };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationRecord {
    /// The boot this record belongs to,
    /// as the kernel's own identifier.
    pub boot: String,
    /// The mount's unique number,
    /// which decides whether the record still describes the live instance.
    pub mount_id: Option<u64>,
    /// The number the mount table prints,
    /// which only finds that table's row.
    pub mount_id_old: Option<u64>,
    /// The mount namespace's number,
    /// where the kernel reports one.
    pub mount_namespace_id: Option<u64>,
    /// The checkout the step was about.
    pub checkout: Option<String>,
    /// The source the descriptor's mount reported,
    /// where the kernel reports one.
    pub source: Option<String>,
    /// The set the mount carried before the step.
    pub before: Option<MountPropagation>,
    /// The set it carried after,
    /// which equals `before` for every outcome but `applied`.
    pub after: Option<MountPropagation>,
    /// The master the set detached,
    /// which is the cost a shared and slave mount pays.
    pub master: Option<u64>,
    /// What the run found beneath the checkout.
    pub beneath: SharedMountsBeneath,
    /// What the step did.
    pub outcome: PropagationOutcome,
    /// The act that refused.
    pub step: PropagationStep,
    /// The kernel's error number,
    /// where a call answered.
    pub errno: Option<i32>,
    /// When the record was published,
    /// as ISO 8601 with its offset.
    pub time: String,
}

impl PropagationRecord {
    /// What:
    ///  The file's whole text,
    ///  its final newline included.
    /// Why:
    ///  The writer proves this text readable by this file's own parser before
    ///  it writes a byte,
    ///  so the text and the grammar are one subject and cannot drift.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = (record: PropagationRecord): string => [version, `kind ${kind}`, ...].join('\n') + '\n';
    /// ```
    pub fn text(&self) -> String {
        let mut lines: Vec<String> = vec![
            String::from(PROPAGATION_VERSION),
            format!("kind {PROPAGATION_KIND}"),
            format!("boot {}", self.boot),
            format!("mount-id {}", number_or_none(self.mount_id)),
            format!("mount-id-old {}", number_or_none(self.mount_id_old)),
            format!("mount-ns-id {}", number_or_none(self.mount_namespace_id)),
            format!("checkout {}", self.checkout.as_deref().unwrap_or(NONE_WORD)),
            format!("source {}", self.source.as_deref().unwrap_or(NONE_WORD)),
            format!("before {}", set_or_none(self.before)),
            format!("after {}", set_or_none(self.after)),
            format!("master {}", number_or_none(self.master)),
            format!("beneath {}", self.beneath.word()),
        ];
        for path in self.beneath.paths() {
            lines.push(format!("{BENEATH_PATH_KEY} {path}"));
        }
        lines.push(format!("outcome {}", self.outcome.word()));
        lines.push(format!("step {}", self.step.word()));
        lines.push(format!("errno {}", errno_or_none(self.errno)));
        lines.push(format!("time {}", self.time));
        return format!("{}\n", lines.join("\n"));
    }
}

/// One optional number as the record's field,
/// or the word a refusal leaves there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numberOrNone = (n?: number): string => n === undefined ? 'none' : String(n);
/// ```
fn number_or_none(number: Option<u64>) -> String {
    return match number {
        Some(value) => value.to_string(),
        None => String::from(NONE_WORD),
    };
}

/// One optional error number as the record's field,
/// or the word a refusal leaves there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const errnoOrNone = (n?: number): string => n === undefined ? 'none' : String(n);
/// ```
fn errno_or_none(errno: Option<i32>) -> String {
    return match errno {
        Some(value) => value.to_string(),
        None => String::from(NONE_WORD),
    };
}

/// One optional propagation set as the record's field,
/// or the word a refusal leaves there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const setOrNone = (set?: MountPropagation): string => set === undefined ? 'none' : word(set);
/// ```
fn set_or_none(set: Option<MountPropagation>) -> String {
    return match set {
        Some(value) => String::from(value.word()),
        None => String::from(NONE_WORD),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_record_tests.rs"]
mod tests;
