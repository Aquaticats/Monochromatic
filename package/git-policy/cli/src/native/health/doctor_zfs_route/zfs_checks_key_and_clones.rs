//! What:
//!  The encryption key of each dataset,
//!  and every clone against the properties this route attaches it with.
//! Why:
//!  A clone made while the pool's key was unloaded can exist with its data
//!  unreadable,
//!  and a dataset this caller could not read leaves the key answer unknown
//!  rather than passed,
//!  while a clone that lost `mountpoint=legacy` or gained `setuid` or
//!  `devices` widens what its mount exposes,
//!  so each one names the dataset and the property a person fixes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const keys = checkKey(datasets); // pass | finding('keystatus') | unknown
//! const clones = checkCloneProperties(clones); // pass | finding(property) | unknown
//! ```

/// One check's answer,
/// and the three constructors every check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
/// The properties a clone's facts are judged by.
use super::zfs_checks_dataset_properties::clone_failure;
/// One dataset's facts,
/// and what a read of it answered.
use super::zfs_checks_dataset_roles::DatasetFacts;
/// The values the design fixes for a key and a clone's mount.
use super::zfs_checks_limits::{
    ENCRYPTION_OFF,
    KEY_UNAVAILABLE,
    LEGACY_MOUNTPOINT,
};
/// What a read of one dataset answered.
use super::zfs_checks_reading::Reading;
/// What:
///  The dataset's encryption key,
///  where the dataset has one.
/// Why:
///  A clone made while the pool's key was unloaded can exist with its data
///  unreadable,
///  and the design's preflight asks for key availability before an attach
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Add and Git registration",
///  step 1).
///  A dataset that is not encrypted has no key to load and passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkKey(datasets); // pass | finding('keystatus') | unknown
/// ```
pub fn check_key(datasets: &[DatasetFacts]) -> CheckOutcome {
    let mut unreadable: Vec<String> = Vec::new();
    let mut encrypted: usize = 0;
    for dataset in datasets {
        let Reading::Read(facts) = &dataset.reading else {
            unreadable.push(dataset.name.clone());
            continue;
        };
        if facts.encryption.text() == Some(ENCRYPTION_OFF) {
            continue;
        }
        encrypted += 1;
        if facts.key_status.number() == Some(KEY_UNAVAILABLE) {
            let root: String = facts
                .encryption_root
                .text()
                .unwrap_or("an unread encryption root")
                .to_string();
            return finding(
                "keystatus",
                format!(
                    "`{}`: `keystatus` is unavailable, so the key of {root} is not loaded and the dataset's data cannot be read",
                    facts.name
                ),
            );
        }
    }
    if !unreadable.is_empty() {
        return unknown(format!(
            "the key state of {} could not be read, so no key answer holds for the route",
            unreadable.join(", ")
        ));
    }
    return pass(format!(
        "every key is loaded ({encrypted} encrypted dataset(s) read)"
    ));
}

/// What:
///  Every clone against the properties this route attaches it with.
/// Why:
///  A clone that lost `mountpoint=legacy` could be mounted by the system's own
///  generator,
///  and one that gained `setuid` or `devices` widens what its mount exposes,
///  so a person must see the clone and the property rather than a count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCloneProperties(clones); // pass | finding(property) | unknown
/// ```
pub fn check_clone_properties(clones: &[DatasetFacts]) -> CheckOutcome {
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    let mut read: usize = 0;
    for clone in clones {
        match &clone.reading {
            Reading::Read(facts) => {
                read += 1;
                if let Some(failure) = clone_failure(facts) {
                    failures.push(failure);
                }
            }
            Reading::Missing {
                ..
            }
            | Reading::Unreadable {
                ..
            } => unreadable.push(clone.name.clone()),
        }
    }
    if let Some((field, first)) = failures.first() {
        let mut detail: String = first.clone();
        for (_, other) in failures.iter().skip(1) {
            detail.push_str("; ");
            detail.push_str(other);
        }
        return finding(field.clone(), detail);
    }
    if !unreadable.is_empty() {
        return unknown(format!(
            "the properties of {} could not be read, so no property answer holds for every clone",
            unreadable.join(", ")
        ));
    }
    return pass(format!(
        "{read} clone(s) carry `mountpoint={LEGACY_MOUNTPOINT}`, `setuid=off` and `devices=off`"
    ));
}
