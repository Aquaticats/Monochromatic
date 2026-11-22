//! What:
//!  The root handler's all-namespace holder scan:
//!  group the live tasks by mount namespace,
//!  read one mount table per distinct namespace,
//!  match the ZFS filesystem type and the exact dataset source,
//!  set aside the clone's own managed attach while its intent stands at
//!  `unmount`,
//!  and answer the holders and every read it could not complete,
//!  bound to one nonce and to the dataset's GUID.
//! Why:
//!  "Unmounted" means absent in every mount namespace holding the clone's ZFS
//!  source,
//!  including binds and staging,
//!  so the scan groups by the kernel's own namespace identity and matches the
//!  source rather than a slot's spelling or a working directory
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 9).
//!  The scan never unmounts,
//!  mounts or records anything:
//!  it reads `/proc` and answers a value,
//!  and the steps that act on holders are later ones.
//!  The grouping,
//!  the one table per namespace and the source match are
//!  `crate::mount_namespace_scan`'s,
//!  which the start probe's arm reads as well,
//!  so the two cannot disagree about what a mount of one dataset is.
//!  A clear answer is never a proof:
//!  a namespace held open by a handle and no task is in no file the scan reads,
//!  and a task that mounts the clone after the scan is not in it either,
//!  which is the residual the project's destroy-anyway decision accepts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = scanHolders({ procRoot: '/proc', source: clone, ownNamespace: 'mnt:[…]', dataset, nonce, time });
//! scan.holders; scan.incomplete; scan.isClear();
//! ```

use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::clone_holder_scan::incomplete_reads::IncompleteRead;
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
use crate::clone_holder_scan::standing_unmount::StandingUnmount;
use crate::mount_namespace_scan::{MountNamespaceTasks, TaskTableAttempt, mounts_of_source, read_one_namespace, tasks_by_mount_namespace};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::{Path, PathBuf};

/// How many mount namespaces one scan reads before it stops.
/// Why:
///  The scan's cost follows the namespaces,
///  not the tasks,
///  and a machine that kept making namespaces would otherwise decide how long
///  one root scan runs.
///  The number is adopted with the project's veto open and is far above the
///  measurements this round took in its guest
///  (`package/git-policy/cli/doc/holder-scan-across-mount-namespaces-2026-11-22.md`).
///  A scan that stops here is not clear:
///  the namespaces it did not read are one incomplete read.
pub const MAXIMUM_MOUNT_NAMESPACES: usize = 1024;

/// How many findings one result lists before the scan stops listing and
/// reading.
/// Why:
///  Every finding is one line of the result's text,
///  and the parse's bound is derived from this number,
///  so a host that holds the clone ten thousand times cannot write a result
///  its own reader refuses.
///  A scan that stops here is not clear either,
///  and it says so as one more incomplete read.
pub const MAXIMUM_DETAIL_LINES: usize = 1024;

/// The most bytes of a read failure's own text one finding carries.
/// Why:
///  A reason is the kernel's sentence about one path,
///  so it is short here;
///  the bound is what keeps one peculiar path from deciding the result's size,
///  and a longer reason is cut with the cut said.
pub const MAXIMUM_REASON_BYTES: usize = 256;

/// What:
///  The one dataset a scan reports on,
///  where it reads,
///  which namespace is the reader's own,
///  and the intent that may set the managed attach aside.
/// Why:
///  The dataset is named by its GUID and by the source a mount table carries,
///  and both travel with the nonce,
///  so a result cannot be read as another dataset's or another request's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HolderScanRequest = { procRoot; source; ownNamespace?; standingUnmount?; dataset; nonce; time };
/// ```
pub struct HolderScanRequest<'a> {
    /// The `/proc` root to read,
    /// which a control points at a fixture tree.
    pub proc_root: &'a Path,
    /// The exact dataset name a mount table's source must equal.
    pub source: &'a str,
    /// The reader's own namespace identity,
    /// or nothing when it could not be read,
    /// which reports every namespace as a holder.
    pub own_namespace: Option<&'a str>,
    /// The exclusion a standing `unmount` intent makes,
    /// or nothing when no intent stands there.
    pub standing_unmount: Option<&'a StandingUnmount>,
    /// The dataset's GUID.
    pub dataset: DatasetGuid,
    /// The request nonce the result answers under.
    pub nonce: &'a RequestNonce,
    /// When the scan ran,
    /// as ISO 8601 with offset.
    pub time: String,
}

/// What:
///  What one scan found:
///  its own namespace's mounts,
///  the namespaces that hold the dataset,
///  the managed attaches it set aside,
///  and every read it could not complete,
///  under the dataset's GUID and the request's nonce.
/// Why:
///  A reader decides from this value alone:
///  `is_clear` is the removal-ready report,
///  and a caller that must act on holders reads the holders and the incomplete
///  reads separately,
///  because an incomplete read does not block by itself while a mismatched or
///  missing unmount does
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Mounts held by other namespaces").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HolderScan = { dataset; nonce; source; ownNamespace; namespacesScanned; ownMounts; holders; excluded; incomplete; time };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderScan {
    /// The dataset's GUID.
    pub dataset: DatasetGuid,
    /// The exact dataset name the mounts were matched against.
    pub source: String,
    /// The request nonce the result answers under.
    pub nonce: RequestNonce,
    /// The reader's own namespace identity,
    /// or nothing when it could not be read.
    pub own_namespace: Option<String>,
    /// How many namespaces the scan read a table of.
    pub namespaces_scanned: usize,
    /// The mounts of the dataset in the reader's own namespace,
    /// which is what a local unmount is about.
    pub own_mounts: Vec<HeldMount>,
    /// The namespaces other than the reader's own that hold the dataset.
    pub holders: Vec<NamespaceHolder>,
    /// The managed attaches set aside under a standing `unmount` intent.
    pub excluded: Vec<NamespaceHolder>,
    /// Every read the scan could not complete.
    pub incomplete: Vec<IncompleteRead>,
    /// When the scan ran,
    /// as ISO 8601 with offset.
    pub time: String,
}

impl HolderScan {
    /// Whether the scan found the dataset in no namespace and read everything
    /// it meant to.
    /// Why:
    ///  This is the "removal-ready unmount" the design asks for,
    ///  and it is deliberately the strict answer:
    ///  one incomplete read makes it false,
    ///  even though such a read does not refuse a removal by itself.
    pub fn is_clear(&self) -> bool {
        return self.own_mounts.is_empty() && self.holders.is_empty() && self.incomplete.is_empty();
    }

    /// How many namespaces other than the reader's own hold the dataset.
    pub fn holder_namespaces(&self) -> usize {
        return self.holders.len();
    }

    /// How many mounts of the dataset those namespaces hold between them.
    pub fn holder_mounts(&self) -> usize {
        return self.holders.iter().map(|holder| return holder.mount_count()).sum();
    }

    /// How many reads the scan could not complete.
    pub fn incomplete_reads(&self) -> usize {
        return self.incomplete.len();
    }
}

/// What:
///  The findings one scan may still list,
///  and whether it ran out.
/// Why:
///   Every finding costs one line of the result's text,
///  so the writer and its reader share one budget rather than two guesses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const budget = { left: 1024, exceeded: false };
/// ```
struct DetailBudget {
    /// How many more findings may be listed.
    left: usize,
    /// Whether a finding was refused because none was left.
    exceeded: bool,
}

impl DetailBudget {
    /// A budget of `lines` findings.
    fn new(lines: usize) -> Self {
        return DetailBudget { left: lines, exceeded: false };
    }

    /// Take one line for a finding,
    /// or answer that none is left.
    fn take(&mut self) -> bool {
        if self.left == 0 {
            self.exceeded = true;
            return false;
        }
        self.left -= 1;
        return true;
    }
}

/// What:
///  One read failure's own text,
///  cut to the bound with the cut said.
/// Why:
///  The bound keeps one path from deciding the result's size,
///  and a person reading the result has to know the sentence was cut.
fn bounded_reason(reason: &str) -> String {
    if reason.len() <= MAXIMUM_REASON_BYTES {
        return String::from(reason);
    }
    let mut kept: String = String::with_capacity(MAXIMUM_REASON_BYTES + 24);
    for character in reason.chars() {
        if kept.len() + character.len_utf8() > MAXIMUM_REASON_BYTES {
            break;
        }
        kept.push(character);
    }
    kept.push_str(" (the rest of this reason was cut)");
    return kept;
}

/// What:
///  Read every namespace of one `/proc` root,
///  and answer the dataset's holders,
///  the managed attach a standing `unmount` intent is taking down,
///  and every read that could not be completed.
/// Why:
///  One value answers the whole question a removal asks,
///  so a caller cannot read the holders of one scan with the incompletes of
///  another,
///  and the nonce says which request it answers.
///  A `/proc` root that cannot be listed is one incomplete read and an empty
///  result,
///  never a refusal to answer:
///  the caller still learns that nothing was read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scan = scanHolders(request);
/// ```
pub fn scan_holders(request: &HolderScanRequest<'_>) -> HolderScan {
    let mut scan: HolderScan = HolderScan {
        dataset: request.dataset,
        source: String::from(request.source),
        nonce: request.nonce.clone(),
        own_namespace: request.own_namespace.map(String::from),
        namespaces_scanned: 0,
        own_mounts: Vec::new(),
        holders: Vec::new(),
        excluded: Vec::new(),
        incomplete: Vec::new(),
        time: request.time.clone(),
    };
    let groups: Vec<MountNamespaceTasks> = match tasks_by_mount_namespace(request.proc_root) {
        Ok(groups) => groups,
        Err(refusal) => {
            scan.incomplete.push(IncompleteRead::Listing { proc_root: refusal.path, reason: bounded_reason(refusal.reason.as_str()) });
            return scan;
        }
    };
    let more_namespaces_than_the_bound: bool = groups.len() > MAXIMUM_MOUNT_NAMESPACES;
    let mut budget: DetailBudget = DetailBudget::new(MAXIMUM_DETAIL_LINES);
    for group in groups.into_iter().take(MAXIMUM_MOUNT_NAMESPACES) {
        scan.namespaces_scanned += 1;
        let read = read_one_namespace(&group);
        let namespace: String = read.namespace.clone();
        let causes: Vec<IncompleteRead> = causes_of(&read, namespace.as_str());
        let mut stopped: bool = false;
        for cause in causes {
            if !budget.take() {
                stopped = true;
                break;
            }
            scan.incomplete.push(cause);
        }
        if stopped {
            break;
        }
        let Some((task, mounts)) = read.table() else {
            continue;
        };
        let matching: Vec<&crate::fs_mount_options::MountFacts> = mounts_of_source(mounts, request.source);
        if matching.is_empty() {
            continue;
        }
        let mut kept: Vec<HeldMount> = Vec::with_capacity(matching.len());
        let mut set_aside: Vec<HeldMount> = Vec::new();
        let mut ran_out: bool = false;
        for facts in matching {
            if !budget.take() {
                ran_out = true;
                break;
            }
            let held: HeldMount = HeldMount::of(facts);
            let aside: bool = request.standing_unmount.is_some_and(|standing| return standing.sets_aside(&held));
            if aside {
                set_aside.push(held);
            } else {
                kept.push(held);
            }
        }
        let task_path: PathBuf = PathBuf::from(task);
        let is_own: bool = request.own_namespace == Some(namespace.as_str());
        if is_own {
            scan.own_mounts.extend(kept);
        } else if !kept.is_empty() {
            scan.holders.push(NamespaceHolder { namespace: namespace.clone(), task: task_path.clone(), mounts: kept });
        }
        if !set_aside.is_empty() {
            scan.excluded.push(NamespaceHolder { namespace, task: task_path, mounts: set_aside });
        }
        if ran_out {
            break;
        }
    }
    if more_namespaces_than_the_bound {
        scan.incomplete.push(IncompleteRead::NamespacesPastTheBound { bound: MAXIMUM_MOUNT_NAMESPACES });
    }
    if budget.exceeded {
        scan.incomplete.push(IncompleteRead::DetailsPastTheBound { bound: MAXIMUM_DETAIL_LINES });
    }
    return scan;
}

/// What:
///  The reads one namespace's attempt left incomplete.
/// Why:
///  A namespace that answered a table names every task that ended or could not
///  be read before the answering one,
///  because that churn is what a report shows.
///  A namespace that answered no table names one cause rather than two:
///  the tables it may not read when there were any,
///  and otherwise a single "no live member" entry,
///  so a reader never counts one namespace as two unknowns.
fn causes_of(read: &crate::mount_namespace_scan::NamespaceTableRead, namespace: &str) -> Vec<IncompleteRead> {
    let mut causes: Vec<IncompleteRead> = Vec::with_capacity(read.attempts.len());
    let unreadable: Vec<&TaskTableAttempt> = read.attempts.iter().filter(|attempt| return matches!(attempt, TaskTableAttempt::Unreadable { .. })).collect();
    if read.table().is_none() {
        if !unreadable.is_empty() {
            for attempt in unreadable {
                causes.extend(cause_of(attempt, namespace));
            }
            return causes;
        }
        let reason: &str = read.attempts.last().and_then(|attempt| return attempt.reason()).unwrap_or("no task of it was tried");
        causes.push(IncompleteRead::NoLiveMember { namespace: String::from(namespace), tasks_tried: read.attempts.len(), reason: bounded_reason(reason) });
        return causes;
    }
    for attempt in &read.attempts {
        causes.extend(cause_of(attempt, namespace));
    }
    return causes;
}

/// What:
///  One failed attempt as the cause it names,
///  or nothing when the attempt answered a table.
fn cause_of(attempt: &TaskTableAttempt, namespace: &str) -> Option<IncompleteRead> {
    return match attempt {
        TaskTableAttempt::Read { .. } => None,
        TaskTableAttempt::Ended { task, reason } => Some(IncompleteRead::TaskEnded { namespace: String::from(namespace), task: task.clone(), reason: bounded_reason(reason) }),
        TaskTableAttempt::Unreadable { task, reason } => Some(IncompleteRead::TableUnreadable { namespace: String::from(namespace), task: task.clone(), reason: bounded_reason(reason) }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_tests.rs"]
mod tests;
