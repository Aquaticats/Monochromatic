//! What:
//!  The three answers a boot takes from one reading of the covered entry:
//!  protected,
//!  unprotected with the reason,
//!  or unproven,
//!  and the line each prints.
//! Why:
//!  the project's answers make the flag a prerequisite of automated attach,
//!  and an answer that cannot be proved must not read as either a pass or a
//!  failure
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Checkout boot mount protection";
//!  step 18's rule that an unprovable check is reported as unknown).
//!  The order of the checks is what makes each message true:
//!  an entry that is not a directory says nothing about a mount,
//!  a mounted checkout says nothing about the underlay's flag,
//!  and a filesystem that reports no attributes says nothing about the flag
//!  either,
//!  so each is answered before the flag is.
//!  The owner is compared last, because the flag is what refuses the account's
//!  rename and removal and an entry owned by another account is a separate
//!  finding a person repairs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = verify(facts, account); // { protected: true } | { protected: false, reason } | { unproven }
//! ```

use crate::checkout_underlay_entry::underlay_entry_read::EntryFacts;

/// What:
///  Why a covered entry is not protected.
/// Why:
///  each reason has a different repair,
///  and a sentence that said only "not protected" would send a person to the
///  migration for a mount that merely needs unmounting or a flag that needs
///  setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnprotectedReason = 'not-a-directory' | 'mounted-over' | 'immutable-clear' | { kind: 'owner', found, wanted };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnprotectedReason {
    /// The entry is a file or a symlink,
    /// so there is no covered folder to protect.
    NotADirectory,
    /// The path's device is not its parent's,
    /// so the read saw a mount at the path and not the underlay.
    MountedOver,
    /// The filesystem reports the attribute and the entry does not carry it.
    ImmutableClear,
    /// The entry belongs to another account than the one that must own it.
    Owner {
        /// The owner the kernel reports.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
}

impl UnprotectedReason {
    /// The reason in words,
    /// with the repair a person can run.
    pub fn message(&self) -> String {
        return match self {
            UnprotectedReason::NotADirectory => String::from(
                "the checkout path is not a directory, so the underlay entry the protection covers does not exist. Nothing was attached or changed. Ways forward: confirm the path with `stat --format='%F'`; a path that is a file or a link needs the migration's own rollback, not a flag.",
            ),
            UnprotectedReason::MountedOver => String::from(
                "the checkout path is a mount of its own, so this read saw the dataset and not the underlay entry the flag protects. Nothing was attached or changed. Ways forward: verify the entry while the checkout is unmounted, which is what the boot does before it attaches anything; `findmnt --target <checkout path>` says whether it is mounted now.",
            ),
            UnprotectedReason::ImmutableClear => String::from(
                "the checkout's covered entry does not carry the immutable attribute, so the account can rename or remove the path while it is unmounted. Nothing was attached or changed. Ways forward: re-run the setup's protection step (`chattr +i <checkout path>` as root on the home dataset) and verify again; a rollback to a snapshot taken before the flag loses it, which is why the check runs after every reimport and rollback.",
            ),
            UnprotectedReason::Owner { found, wanted } => format!(
                "the checkout's covered entry is owned by user {found}, and the route requires user {wanted}. Nothing was attached or changed. Ways forward: restore the entry's owner from the backup the migration kept (`chown {wanted}: {wanted} <checkout path>` as root on the home dataset) and verify again; do not chown a clone's slot."
            ),
        };
    }
}

/// What:
///  Why the flag could not be proved either way.
/// Why:
///  a filesystem without the attribute answers a mask without it,
///  and that is not the same answer as a clear flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnprovenReason = 'attributes-not-reported';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnprovenReason {
    /// The filesystem reports no immutable attribute at all,
    /// so this check cannot say whether the entry carries one.
    AttributesNotReported,
}

impl UnprovenReason {
    /// The reason in words,
    /// with what to inspect.
    pub fn message(&self) -> String {
        return match self {
            UnprovenReason::AttributesNotReported => String::from(
                "this filesystem reports no immutable attribute, so whether the checkout's covered entry is protected is unknown here. Nothing was attached or changed on this reading. Ways forward: read the entry's attributes on the dataset that holds it (the home dataset, not a mount of the checkout) with `lsattr -d <checkout path>` as root, and report the answer; the setup sets the flag on that dataset, so a filesystem that cannot carry one means the decided protection is not in force.",
            ),
        };
    }
}

/// What:
///  What one reading of the covered entry came to.
/// Why:
///  the caller decides whether an unsafe answer stops an attach,
///  and the three variants are what let it do that without reading the facts
///  again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = { kind: 'protected' } | { kind: 'unprotected', reason } | { kind: 'unproven', reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnderlayVerdict {
    /// The flag is set on the unmounted covered entry of the parent's device.
    Protected,
    /// The entry is there and the proof failed,
    /// with the reason.
    Unprotected(UnprotectedReason),
    /// The proof could not be made,
    /// with the reason.
    Unproven(UnprovenReason),
}

impl UnderlayVerdict {
    /// Whether the reading proved the protection,
    /// which is the only answer an automated attach may act on.
    pub fn is_protected(&self) -> bool {
        return matches!(self, UnderlayVerdict::Protected);
    }

    /// The verdict in words,
    /// naming what was read and what to do next.
    pub fn message(&self, facts: &EntryFacts) -> String {
        let read: String = format!(
            "device {} inode {} owner {} mode {:06o} immutable {} attributes-reported {} same-device-as-parent {}",
            facts.device, facts.inode, facts.owner, facts.mode, facts.immutable, facts.attributes_reported, facts.same_device_as_parent
        );
        return match self {
            UnderlayVerdict::Protected => format!(
                "the checkout's covered entry is protected: it is the unmounted underlay entry on its parent's device and carries the immutable attribute ({read})."
            ),
            UnderlayVerdict::Unprotected(reason) => format!("{}. The reading was: {read}.", reason.message()),
            UnderlayVerdict::Unproven(reason) => format!("{}. The reading was: {read}.", reason.message()),
        };
    }

    /// The one line a boot logs for this reading,
    /// so a journal shows what was compared without the sentence.
    pub fn line(&self, facts: &EntryFacts) -> String {
        let answer: &str = match self {
            UnderlayVerdict::Protected => "protected",
            UnderlayVerdict::Unprotected(_) => "unprotected",
            UnderlayVerdict::Unproven(_) => "unproven",
        };
        return format!(
            "UNDERLAY entry={answer} device={} inode={} owner={} mode={:06o} immutable={} attributes-reported={} same-device-as-parent={}",
            facts.device, facts.inode, facts.owner, facts.mode, facts.immutable, facts.attributes_reported, facts.same_device_as_parent
        );
    }
}

/// What:
///  Judge one reading:
///  is the covered entry the protected, immutable, account-owned underlay entry
///  the boot needs before it attaches anything?
/// Why:
///  the checks run in the order that keeps every message true,
///  so a mounted checkout is named as one rather than as a missing flag,
///  and a filesystem with no attributes is named as unproven.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = verify(facts, account);
/// ```
pub fn verify(facts: &EntryFacts, wanted_owner: u32) -> UnderlayVerdict {
    if !facts.directory {
        return UnderlayVerdict::Unprotected(UnprotectedReason::NotADirectory);
    }
    if !facts.same_device_as_parent {
        return UnderlayVerdict::Unprotected(UnprotectedReason::MountedOver);
    }
    if !facts.attributes_reported {
        return UnderlayVerdict::Unproven(UnprovenReason::AttributesNotReported);
    }
    if !facts.immutable {
        return UnderlayVerdict::Unprotected(UnprotectedReason::ImmutableClear);
    }
    if facts.owner != wanted_owner {
        return UnderlayVerdict::Unprotected(UnprotectedReason::Owner { found: facts.owner, wanted: wanted_owner });
    }
    return UnderlayVerdict::Protected;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "underlay_entry_verdict_tests.rs"]
mod tests;
