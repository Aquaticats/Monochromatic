//! What:
//!  Every fact the boot setup's check reads:
//!  one reading per file the setup's plan writes,
//!  with the text this build writes there and what stands there now,
//!  the key-loading unit the machine has,
//!  and each dataset measured under the checkout with the `canmount` value
//!  the kernel's own reply answered for it.
//! Why:
//!  the owner's answer V7 asks `doctor` to print the boot setup,
//!  and that setup is a set of files an earlier setup run wrote plus one
//!  property on every dataset under the checkout,
//!  so the check compares this build's own text with what stands on the
//!  machine and with the value the kernel answered
//!  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
//!  V1 to V9).
//!  The texts are values rather than the plan's own types,
//!  because the plan's folder and this one already sit in one loop the layout
//!  allow list tolerates,
//!  and a tolerated loop may only lose uses
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`;
//!  `AGENTS.md` rule LYL).
//!  The wiring run builds both sides with the plan it also installs from,
//!  so no second copy of a unit text exists.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BootSetupFacts = { gate: PlannedFileReading; others: PlannedFileReading[]; keyUnit?: string; nested: NestedDatasetSetting[] };
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;
use std::path::PathBuf;

/// What:
///  One file the setup's plan writes:
///  where it goes,
///  the whole text this build writes there,
///  and what a read of that path answered.
/// Why:
///  a missing file is a finding a person installs,
///  a file whose text differs is a finding a person replaces,
///  and the two texts have to travel together so no reader compares one
///  path's bytes with another file's text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const one = { path: '/etc/systemd/system/…', plannedText: '…', atThePath: { kind: 'content', text: '…' } };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedFileReading {
    /// Where the plan writes the file.
    pub path: PathBuf,
    /// The whole text this build writes there,
    /// built by the plan the setup run installs from.
    pub planned_text: String,
    /// What a read of that path answered.
    pub at_the_path: FileAtInstallPath,
}

/// What:
///  What a read of one install path answered:
///  the file's text,
///  nothing at the path,
///  something that is not a regular file,
///  or the reason the path could not be read.
/// Why:
///  a missing file is a finding a person installs,
///  a file whose text differs is a finding a person replaces,
///  and a path nobody could read proves nothing and has to stay unknown,
///  because reporting it as absent would claim a file system answer nobody
///  got
///  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
///  step 18,
///  "Open items").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileAtInstallPath = { kind: 'content'; text } | { kind: 'absent' } | { kind: 'wrong-kind'; found } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileAtInstallPath {
    /// The file's whole text,
    /// as the read returned it.
    Content(String),
    /// Nothing stands at the path.
    Absent,
    /// The entry is there and systemd would read no text out of it.
    WrongKind {
        /// What the entry is,
        /// as the caller's own look described it.
        found: String,
    },
    /// The path could not be read.
    Unreadable {
        /// Why the read gave no answer.
        why: String,
    },
}

/// What:
///  One dataset inside the checkout beside the `canmount` value a read of it
///  answered.
/// Why:
///  the plan writes one mount unit per such dataset,
///  and the owner's V4 sets `canmount=noauto` so `zfs mount -a` cannot mount
///  it before the gate proved the checkout,
///  so the value and the unit are judged together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const one = { dataset: 'zpcachyos/nosnap/vm-builder-output', canmount };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedDatasetSetting {
    /// The dataset's full name,
    /// as `zfs list -H -o name` prints it.
    pub dataset: String,
    /// What the reply's `canmount` entry answered,
    /// which is a number for this property.
    pub canmount: PropertyValue,
}

/// What:
///  Everything the boot setup's check reads,
///  gathered once.
/// Why:
///  the gate's lines decide the provider,
///  the key unit and the two bounds,
///  and every other file the plan writes is compared with the bytes on the
///  machine,
///  so one value holds both sides and a control changes exactly one field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = { gate, others, keyUnit, nested };
/// ```
pub struct BootSetupFacts {
    /// The gate unit,
    /// whose lines the provider,
    /// the key unit and the two bounds are read from.
    pub gate: PlannedFileReading,
    /// Every other file the plan writes,
    /// which this check compares and names.
    pub others: Vec<PlannedFileReading>,
    /// The generator's key-loading unit for the checkout's encryption root,
    /// present only on a machine that has one.
    pub key_unit: Option<String>,
    /// Every dataset whose mountpoint lies inside the checkout,
    /// with the `canmount` value its own reply answered.
    pub nested: Vec<NestedDatasetSetting>,
}
