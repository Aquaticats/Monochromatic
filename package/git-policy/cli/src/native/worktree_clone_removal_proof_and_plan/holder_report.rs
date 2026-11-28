//! What:
//!  One all-namespace scan's answer as a report that names holders reads it:
//!  the namespaces other than the reader's own that hold the dataset,
//!  with one live task and one mount point each,
//!  the mounts of the reader's own namespace,
//!  and every read the scan could not complete.
//! Why:
//!  A scan's own value keeps the reader's table apart from the other
//!  namespaces' and counts in the scan's words,
//!  and `doctor` names a holder by its namespace and a representative task
//!  (`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`;
//!  plan step 18,
//!  "the holders of a clone by namespace ID and representative PID,
//!  read from step 9's all-namespace scan result").
//!  The mapping lives here rather than at each report because the scan's own
//!  types are this crate's removal-side ones:
//!  a caller that names them would name this folder's dependence on the
//!  namespace scan,
//!  and the removal's own refusal and the report must name one holder in one
//!  way.
//!  A task that ended between the scan's listing and its table read leaves no
//!  task to name,
//!  so the scan's own incomplete list carries the namespace instead and no
//!  holder is invented for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theHoldersOfOneClone(clone, guid);   // throws the scan's own refusal
//! answer.holders.forEach((held) => console.log(`${held.namespace} holds it at ${held.mountPoint} (task ${held.task})`));
//! ```

/// The scan's own value and the live scan a removal and a report share.
use crate::clone_holder_scan::holder_scan::HolderScan;
/// One namespace the scan saw holding the dataset,
/// and one live task in it.
use crate::clone_holder_scan::incomplete_reads::IncompleteRead;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::{
    FreshHolderScan,
    TheLiveHolderScan,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  One namespace that holds the dataset and one task a person can look at.
/// Why:
///  A namespace a person can name is the place the mount lives,
///  and a task is what another run can look at,
///  so both travel with the mount point the scan matched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = { namespace: 'mnt:[4026532001]', task: 4242, mountPoint: '/home/user/.cache/cli-git/staged-7' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AProvenHolder {
    /// The mount namespace's own identity,
    /// as the kernel prints it.
    pub namespace: String,
    /// One live task in that namespace.
    pub task: u32,
    /// Where the mount stands in that namespace.
    pub mount_point: PathBuf,
}

/// What:
///  One scan's answer,
///  as the lines that name holders read it.
/// Why:
///  The reader's own mounts are not a namespace a removal must unmount
///  elsewhere,
///  and they are the first thing a person acts on,
///  so they are kept apart from the other namespaces' holders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHoldersOfOneClone = { holders; ownMounts; ownNamespace; incomplete };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheHoldersOfOneClone {
    /// Every namespace other than the reader's own that holds the dataset.
    pub holders: Vec<AProvenHolder>,
    /// The mounts of the dataset in the reader's own namespace.
    pub own_mounts: Vec<PathBuf>,
    /// The reader's own namespace identity,
    /// and nothing when it could not be read.
    pub own_namespace: Option<String>,
    /// Every read the scan could not complete,
    /// in the scan's own words.
    pub incomplete: Vec<String>,
}

/// What:
///  One fresh all-namespace scan of one clone,
///  as the lines that name holders read it.
/// Why:
///  The report and a removal ask the same question of the same machine,
///  so both take the scan from `TheLiveHolderScan` and only the shape the
///  answer is printed in differs.
///
/// # Errors
///
/// The scan's own run nonce could not be made,
/// which the reader's sentence says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = theHoldersOfOneClone(clone, guid);
/// ```
pub fn the_holders_of_one_clone(
    clone: &str,
    clone_guid: DatasetGuid,
) -> Result<TheHoldersOfOneClone, String> {
    let scan: HolderScan = TheLiveHolderScan
        .holders_of(clone, clone_guid)
        .map_err(|stop| return stop.message())?;
    return Ok(holders_of(&scan));
}

/// What:
///  One scan's answer in the shape a report prints it.
/// Why:
///  A control drives this function from a value it wrote,
///  so the mapping a live scan takes is the one a control proves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = holdersOf(scan);
/// ```
pub fn holders_of(scan: &HolderScan) -> TheHoldersOfOneClone {
    let mut holders: Vec<AProvenHolder> = Vec::new();
    for holder in &scan.holders {
        for mount in &holder.mounts {
            holders.push(AProvenHolder {
                namespace: String::from(holder.namespace()),
                task: a_task_number(holder.task.as_path()),
                mount_point: mount.mount_point.clone(),
            });
        }
    }
    // A task that ended between the scan's listing and its table read leaves
    // no namespace whose mounts are proved, and the scan's own incomplete list
    // already names it, so no holder stands in for it here.
    return TheHoldersOfOneClone {
        holders,
        own_mounts: scan
            .own_mounts
            .iter()
            .map(|mount| return mount.mount_point.clone())
            .collect(),
        own_namespace: scan.own_namespace.clone(),
        incomplete: scan.incomplete.iter().map(the_scans_own_words).collect(),
    };
}

/// What:
///  The task number one scan row names,
///  and zero when the row's task name is not a number.
/// Why:
///  The scan prints a task as the path it read,
///  so the last component is the kernel's own number,
///  and a name that is not one is a task no person can look at:
///  the namespace is still named and the row is not dropped,
///  because a holder whose task nobody can read still blocks a removal.
fn a_task_number(task: &Path) -> u32 {
    return task
        .file_name()
        .and_then(|name| return name.to_str())
        .and_then(|text| return text.parse().ok())
        .unwrap_or(0);
}

/// What:
///  One incomplete read in the scan's own words.
/// Why:
///  The report says which cause kept it from proving an absence,
///  and the reason a reader of the scan's own result reads is the same one,
///  so the cause's own word is printed with the namespace and the task it
///  belongs to.
fn the_scans_own_words(read: &IncompleteRead) -> String {
    let mut words: String = String::from(read.kind());
    if let Some(namespace) = read.namespace() {
        words.push_str(format!(" in the namespace {namespace}").as_str());
    }
    if let Some(task) = read.task() {
        words.push_str(format!(" at {}", task.display()).as_str());
    }
    return words;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_report_tests.rs"]
mod tests;
