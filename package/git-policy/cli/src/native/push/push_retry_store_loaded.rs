//! What:
//!  Reading every plan a service instance's journal holds:
//!  one plan per branch,
//!  the newest file winning,
//!  and whether something could not be read.
//! Why:
//!  A restarted service rebuilds its plan from these,
//!  and a file it could not read says nothing about the push inside it,
//!  so the caller is told the journal is not fully read
//!  and reads it again after a pause instead of starting with what it happened to read.
//!  Every record is read through `private_storage`,
//!  so a name holding a pipe is answered at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const loaded = loadReporting(dir); // { plans, unread }
//! ```

/// Import the plan's record,
/// its extension,
/// and what a plan file may hold.
use super::push_retry_store_plan_record::{
    PLAN_EXTENSION,
    PLAN_FILE_LIMIT,
    StoredPlan,
    decode,
};
/// Import the storage reader that answers a pipe,
/// a link and a size bound.
use crate::private_storage::{
    ReadRefusal,
    read_regular_file,
};
/// Import the borrowed and owned path types,
/// and the two times a plan is ordered by.
use std::path::{
    Path,
    PathBuf,
};
use std::time::SystemTime;

/// What:
///  What a plan directory held,
/// and whether something in it could not be read.
/// Why:
///   A start that met a file it could not read knows less than the journal holds,
/// so it reads the journal again after a pause instead of starting with what it happened to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LoadedPlans = { plans: StoredPlan[]; unread: boolean };
/// ```
#[derive(Debug, Default)]
pub struct LoadedPlans {
    /// One plan per branch,
    /// the newest file winning.
    pub plans: Vec<StoredPlan>,
    /// Whether a file or the directory itself could not be read,
    /// so the journal may hold a plan this read did not see.
    pub unread: bool,
}

/// Order two stored plans with the one written later first;
/// equal times keep their order (the sort is stable).
fn newest_first(
    left: &(SystemTime, StoredPlan),
    right: &(SystemTime, StoredPlan),
) -> std::cmp::Ordering {
    return right.0.cmp(&left.0);
}

/// What:
///  The plans in the directory,
/// for a caller that only needs them.
/// Why:
///   `load_reporting` says more,
/// and a caller that has a pause to try again reads that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plans = loadReporting(dir).plans;
/// ```
pub fn load(directory: &Path) -> Vec<StoredPlan> {
    return load_reporting(directory).plans;
}

/// What:
///  Reads every plan in the directory,
/// one per branch,
/// the newest file winning;
/// a file that holds no plan is said and removed,
/// and a file that could not be read is said and kept.
/// Why:
///   A restarted service rebuilds its plan from these.
/// Older files of a branch are what a kill between a save and its cleanup
///  leaves,
/// and a file that cannot be read would otherwise be said at every start.
///  "never discard user data":
/// a read that failed says nothing about the plan inside the file,
/// which may schedule a push that is neither done nor cancelled,
/// so the file stays and the caller is told the journal is not fully read,
/// and the next read either finds the plan or says so again.
/// Only a file that was read and holds no plan is certainly of no use.
/// Each file is read the way every record of this wrapper is read
/// (`private_storage::read_regular_file`),
/// so a name holding a pipe is answered at once instead of waiting inside `open(2)`
/// for a writer that may never come,
/// and a link at a plan's name is refused rather than read through.
/// A name that holds a pipe,
///  a link,
///  a directory or a file larger than a plan can be
/// is no plan this store wrote:
/// it stays,
/// and it is not read again after a pause,
/// because a second read of it answers the same.
/// The listing itself answers twice:
/// the entries it names,
///  and an error that ended it before it named them all.
/// An error part way through is what a directory read on a mount that stopped answering does,
/// and a plan it never named would otherwise wait for the life of the service,
/// so that error leaves the journal unread and it is listed again after a pause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plans = loadReporting(dir) // one per branch, and whether something was unread
/// ```
pub fn load_reporting(directory: &Path) -> LoadedPlans {
    let listing = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        // A journal that never held a plan.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return LoadedPlans::default();
        }
        Err(error) => {
            crate::service_warn!(
                "the retry plans in {} could not be listed, so they are read again after a pause: {error}",
                directory.display()
            );
            return LoadedPlans {
                plans: Vec::new(),
                unread: true,
            };
        }
    };
    let listed = listing.map(|listed| {
        let entry: std::fs::DirEntry = listed?;
        let written: SystemTime = match entry.metadata() {
            Ok(meta) => meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            Err(_) => SystemTime::UNIX_EPOCH,
        };
        return Ok(ListedPlan {
            path: entry.path(),
            written,
        });
    });
    return read_the_plans(directory, listed);
}

/// One file a listing named,
/// and when it was written.
/// Why:
///   A listing answers two things apart:
/// the entries it names and the error that ended it.
///   Holding them apart is what lets a reader judge each,
/// and what a control can hand to one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListedPlan = { path: string; written: SystemTime };
/// ```
pub(super) struct ListedPlan {
    /// The file the listing named.
    pub(super) path: PathBuf,
    /// When it was written,
    /// which decides the plan of a branch when a kill left two.
    pub(super) written: SystemTime,
}

/// What:
///  Reads every plan `listed` names,
///  one per branch,
///  the newest file winning,
///  and judges each file by what was read of it;
///  an entry the listing could not read leaves the journal `unread`.
/// Why:
///   `load_reporting` lists the directory;
///  this is what the entries a listing named and the error that ended it become.
///  An error the listing hands back says the directory holds entries this read never saw,
///  and a plan among them is what a restarted service tries a push from:
///  nothing else scans the item that waits for it,
///  so reading the failure as the end of the listing leaves that push waiting for the life of the service.
///  The directory is read again after a pause instead
/// (`PushRetries::restore`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readThePlans(dir, listed) // an entry error sets unread
/// ```
pub(super) fn read_the_plans(
    directory: &Path,
    listed: impl Iterator<Item = std::io::Result<ListedPlan>>,
) -> LoadedPlans {
    let mut unread: bool = false;
    let mut found: Vec<(SystemTime, StoredPlan)> = Vec::new();
    for listed_plan in listed {
        let (path, written): (PathBuf, SystemTime) = match listed_plan {
            Ok(plan) => (plan.path, plan.written),
            // An entry the listing could not read may be a plan, and no wake comes for the push it schedules.
            Err(error) => {
                crate::service_warn!(
                    "the retry plans in {} could not all be listed, so they are read again after a pause: {error}",
                    directory.display()
                );
                unread = true;
                continue;
            }
        };
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(PLAN_EXTENSION) {
            continue;
        }
        let bytes: Vec<u8> = match read_regular_file(path.as_path(), PLAN_FILE_LIMIT) {
            Ok(bytes) => bytes,
            // Removed between the listing and the read, which leaves no plan to schedule.
            Err(ReadRefusal::Missing) => continue,
            // Nothing this store writes: a pipe, a link, a directory or a file no plan fits in.
            Err(ReadRefusal::NotRegular) => {
                crate::service_warn!(
                    "the retry plan file {} is not a regular file, so it holds no plan and stays",
                    path.display()
                );
                continue;
            }
            Err(ReadRefusal::TooLarge) => {
                crate::service_warn!(
                    "the retry plan file {} holds more than the {PLAN_FILE_LIMIT} bytes a plan can, so it holds no plan and stays",
                    path.display()
                );
                continue;
            }
            // A read that failed is no proof about the plan inside the file, so the file stays for the next read.
            Err(ReadRefusal::Io(error)) => {
                crate::service_warn!(
                    "the retry plan file {} could not be read, so it stays and is read again after a pause: {error}",
                    path.display()
                );
                unread = true;
                continue;
            }
        };
        let text: &str = match std::str::from_utf8(bytes.as_slice()) {
            Ok(text) => text,
            Err(_not_text) => {
                crate::service_warn!(
                    "the retry plan file {} holds no plan and is removed: the plan is not text",
                    path.display()
                );
                let _removed = std::fs::remove_file(path.as_path());
                continue;
            }
        };
        match decode(text) {
            Ok(plan) => found.push((written, plan)),
            Err(reason) => {
                crate::service_warn!(
                    "the retry plan file {} holds no plan and is removed: {reason}",
                    path.display()
                );
                let _removed = std::fs::remove_file(path.as_path());
            }
        }
    }
    found.sort_by(newest_first);
    let mut plans: Vec<StoredPlan> = Vec::new();
    for (_, plan) in found {
        let mut already: bool = false;
        for kept in &plans {
            if kept.git_dir == plan.git_dir && kept.item.branch_ref == plan.item.branch_ref {
                already = true;
                break;
            }
        }
        if !already {
            plans.push(plan);
        }
    }
    return LoadedPlans {
        plans,
        unread,
    };
}
