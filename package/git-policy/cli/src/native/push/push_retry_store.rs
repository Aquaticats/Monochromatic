//! What:
//!  The retry plan on disk:
//! one small file per branch whose push waits for a new try,
//! in a directory of the service instance's own
//!  journal,
//! written when a try is planned,
//! removed when the plan ends,
//! and read when a service starts.
//! Why:
//!  A service that is stopped (a release,
//! a restart of the unit,
//! a crash) must not forget the pushes it keeps trying:
//! the defect
//!  the retry mends is a stopped push that is never tried again,
//! and a restart that forgot the plan would bring it back.
//! The
//!  person's failure record cannot carry the plan:
//! the person's next command shows it and removes it.
//! The journal is the
//!  instance's own and survives a restart (`%S/monochromatic-git-policy-cli/service/%i`),
//! and one instance is the only writer of its plan,
//! so no lock
//!  is taken.
//! A plan is replaced by publishing the new file whole before the old one is removed,
//! so a kill leaves one of them,
//!  never none;
//! when a kill leaves both,
//! the newer one is the plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await save(dir, plan); const plans = await load(dir); await remove(dir, gitDir, branchRef);
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::push_failure_notice::fnv_hex;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::work_item::WorkItem;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The record format version.
pub const PLAN_VERSION: u64 = 1;

/// The extension of a plan file.
const PLAN_EXTENSION: &str = "plan";

/// Separates the parts of a plan file's name;
/// it is not a character a hexadecimal digit can be.
const NAME_SEPARATOR: char = '~';

/// One plan as it is kept on disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredPlan {
    /// The Git directory whose service pushes.
    pub git_dir: PathBuf,
    /// The newest push item of the branch.
    pub item: WorkItem,
    /// The number of the try that is due.
    pub attempt: u32,
}

/// The part of every file name that belongs to one branch of one Git directory.
/// Why:
///   The key is built from the bytes of the directory,
/// so two directories whose paths differ only in a byte that is not text get a key each;
/// their text is the same,
/// and a shared key would let the plan of the second remove the plan of the first.
fn branch_key(git_dir: &Path, branch_ref: &str) -> String {
    return fnv_hex(format!("{}\n{branch_ref}", path_bytes::hex_of_path(git_dir)).as_str());
}

/// The key a release before the bytes were used wrote,
/// so that its plan files are still found and removed.
/// Why:
///   A plan file an earlier release wrote carries this key,
/// and `remove` after an upgrade would otherwise leave it to be read again at every start.
fn earlier_branch_key(git_dir: &Path, branch_ref: &str) -> String {
    return fnv_hex(format!("{}\n{branch_ref}", git_dir.display()).as_str());
}

/// The file name of one plan:
/// the branch's key,
/// the try number and a hash of the commit.
fn plan_name(plan: &StoredPlan) -> String {
    return format!("{}{NAME_SEPARATOR}{}{NAME_SEPARATOR}{}.{PLAN_EXTENSION}", branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str()), plan.attempt, fnv_hex(plan.item.oid.as_str()));
}

/// The text of a plan file.
fn encode(plan: &StoredPlan) -> String {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("v"), Value::from(PLAN_VERSION));
    fields.insert(String::from("attempt"), Value::from(plan.attempt));
    // The directory goes in as plain text and as its bytes, so a plan is read back as the directory it was saved for.
    path_bytes::write_field(&mut fields, "gitDir", plan.git_dir.as_path());
    fields.insert(String::from("item"), Value::from(plan.item.encode()));
    return format!("{}\n", Value::Object(fields));
}

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

/// The plan a file's text holds,
/// or why it holds none.
fn decode(text: &str) -> Result<StoredPlan, String> {
    let value: Value = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => return Err(format!("the plan is not JSON: {error}")),
    };
    if value.get("v").and_then(Value::as_u64) != Some(PLAN_VERSION) {
        return Err(String::from("the plan has an unsupported version"));
    }
    let Some(number) = value.get("attempt").and_then(Value::as_u64) else {
        return Err(String::from("the plan has no try number"));
    };
    let Ok(attempt) = u32::try_from(number) else {
        return Err(String::from("the plan has no try number"));
    };
    let Some(item_text) = value.get("item").and_then(Value::as_str) else {
        return Err(String::from("the plan holds no push item"));
    };
    let item: WorkItem = WorkItem::decode(item_text)?;
    return Ok(StoredPlan { git_dir: path_bytes::read_field(&value, "gitDir")?, item, attempt });
}

/// What:
///  Writes the plan of a branch and removes the files of its earlier plans.
/// Why:
///   The new file is published whole before the old ones go,
/// so a kill between the two leaves a plan,
/// not nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeExclusive(join(dir, name(plan)), encode(plan)); await removeOthersOfBranch(dir, plan);
/// ```
pub fn save(directory: &Path, plan: &StoredPlan) -> Result<(), String> {
    if let Err(error) = create_private_directory_all(directory) {
        return Err(format!("creating the plan directory failed: {error}"));
    }
    let name: String = plan_name(plan);
    match write_private_file_unsynced(directory.join(name.as_str()).as_path(), encode(plan).as_bytes()) {
        Ok(()) => {}
        // The same try of the same commit is already there.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(format!("writing the plan failed: {error}")),
    }
    remove_files_of(
        directory,
        plan.git_dir.as_path(),
        plan.item.branch_ref.as_str(),
        &[branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str()), earlier_branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str())],
        std::slice::from_ref(&name),
    );
    return Ok(());
}

/// The file name the plan of a branch is kept under,
/// for a caller that must know which files a round is about to write.
pub fn name_of(plan: &StoredPlan) -> String {
    return plan_name(plan);
}

/// What:
///  Removes the files of the plan of a branch;
/// one that cannot be removed stays and is read again by the next start.
/// Why:
///   The worker that decided the branch needs no new try removes the file itself,
/// so a forget travels to the journal in the round that decided it
/// and no later round replays it onto a plan that has been planned again since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeFilesOf(gitDir, branchRef, keys, [])
/// ```
pub fn remove(directory: &Path, git_dir: &Path, branch_ref: &str) {
    remove_files_of(directory, git_dir, branch_ref, &[branch_key(git_dir, branch_ref), earlier_branch_key(git_dir, branch_ref)], &[]);
}

/// Removes the plan files of one branch of `git_dir`,
/// whose keys are `keys`,
/// except the ones named in `keep`,
/// and removes a file only when its record was read and names this directory and this branch.
/// Why:
///   "never discard user data."
/// The key an earlier release wrote was built from the text of the path,
/// and two directories whose names differ only in a byte that is not text have one text,
/// so that key alone can name both;
/// only the record inside the file says which plan it holds.
/// A file that cannot be read,
/// or that holds no plan,
/// says nothing about the push it schedules,
/// which may be neither done nor cancelled,
/// so it stays and `load` judges it and says so,
/// since deleting it needs permission on its directory alone.
///  The names in `keep` are the plans a round is about to write,
/// which a forget of the same round must not take.
fn remove_files_of(directory: &Path, git_dir: &Path, branch_ref: &str, keys: &[String], keep: &[String]) {
    let Ok(listing) = std::fs::read_dir(directory) else { return };
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if keep.contains(&name) || !names_the_branch(keys, name.as_str()) {
            continue;
        }
        let path: PathBuf = entry.path();
        if !records_this_branch(path.as_path(), git_dir, branch_ref) {
            continue;
        }
        // A plan that stays says so, so a try the person was told about is not lost in silence.
        if let Err(error) = std::fs::remove_file(path.as_path()) {
            crate::service_warn!("the retry plan file {} could not be removed, so the next start reads it again: {error}", path.display());
        }
    }
}

/// Whether the plan in `path` was read and names `git_dir` and `branch_ref`,
/// which is the only case a cleanup may remove it.
/// Why:
///   A read or decode that fails is no proof about the plan:
/// an unreadable file and a file that holds no plan both stay,
/// and `load` says what it found and removes what is no plan.
fn records_this_branch(path: &Path, git_dir: &Path, branch_ref: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else { return false };
    let Ok(found) = decode(text.as_str()) else { return false };
    return found.git_dir == git_dir && found.item.branch_ref == branch_ref;
}

/// Whether a plan file's name begins with the key of one of the branch's plans.
fn names_the_branch(keys: &[String], name: &str) -> bool {
    for key in keys {
        if name.starts_with(format!("{key}{NAME_SEPARATOR}").as_str()) {
            return true;
        }
    }
    return false;
}

/// Order two stored plans with the one written later first;
/// equal times keep their order (the sort is stable).
fn newest_first(left: &(SystemTime, StoredPlan), right: &(SystemTime, StoredPlan)) -> std::cmp::Ordering {
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plans = loadReporting(dir) // one per branch, and whether something was unread
/// ```
pub fn load_reporting(directory: &Path) -> LoadedPlans {
    let listing = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        // A journal that never held a plan.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return LoadedPlans::default(),
        Err(error) => {
            crate::service_warn!("the retry plans in {} could not be listed, so they are read again after a pause: {error}", directory.display());
            return LoadedPlans { plans: Vec::new(), unread: true };
        }
    };
    let mut unread: bool = false;
    let mut found: Vec<(SystemTime, StoredPlan)> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(PLAN_EXTENSION) {
            continue;
        }
        let written: SystemTime = match entry.metadata() {
            Ok(meta) => meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            Err(_) => SystemTime::UNIX_EPOCH,
        };
        match std::fs::read_to_string(path.as_path()) {
            Ok(text) => match decode(text.as_str()) {
                Ok(plan) => found.push((written, plan)),
                Err(reason) => {
                    crate::service_warn!("the retry plan file {} holds no plan and is removed: {reason}", path.display());
                    let _removed = std::fs::remove_file(path.as_path());
                }
            },
            // A read that failed is no proof about the plan inside the file, so the file stays for the next read.
            Err(error) => {
                crate::service_warn!("the retry plan file {} could not be read, so it stays and is read again after a pause: {error}", path.display());
                unread = true;
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
    return LoadedPlans { plans, unread };
}

/// Plan file controls stay out of the release executable.
#[cfg(test)]
#[path = "push_retry_store_tests.rs"]
mod tests;
