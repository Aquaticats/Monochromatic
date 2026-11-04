//! What:
//!  The retry plan on disk: one small file per branch whose push waits for a new try, in a directory of the service instance's own
//!  journal, written when a try is planned, removed when the plan ends, and read when a service starts.
//! Why:
//!  A service that is stopped (a release, a restart of the unit, a crash) must not forget the pushes it keeps trying: the defect
//!  the retry mends is a stopped push that is never tried again, and a restart that forgot the plan would bring it back. The
//!  person's failure record cannot carry the plan: the person's next command shows it and removes it. The journal is the
//!  instance's own and survives a restart (`%S/cli-git/service/%i`), and one instance is the only writer of its plan, so no lock
//!  is taken. A plan is replaced by publishing the new file whole before the old one is removed, so a kill leaves one of them,
//!  never none; when a kill leaves both, the newer one is the plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await save(dir, plan); const plans = await load(dir); await remove(dir, gitDir, branchRef);
//! ```

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

/// Separates the parts of a plan file's name; it is not a character a hexadecimal digit can be.
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
fn branch_key(git_dir: &Path, branch_ref: &str) -> String {
    return fnv_hex(format!("{}\n{branch_ref}", git_dir.display()).as_str());
}

/// The file name of one plan: the branch's key, the try number and a hash of the commit.
fn plan_name(plan: &StoredPlan) -> String {
    return format!("{}{NAME_SEPARATOR}{}{NAME_SEPARATOR}{}.{PLAN_EXTENSION}", branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str()), plan.attempt, fnv_hex(plan.item.oid.as_str()));
}

/// The text of a plan file.
fn encode(plan: &StoredPlan) -> String {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("v"), Value::from(PLAN_VERSION));
    fields.insert(String::from("attempt"), Value::from(plan.attempt));
    fields.insert(String::from("gitDir"), Value::from(plan.git_dir.display().to_string()));
    fields.insert(String::from("item"), Value::from(plan.item.encode()));
    return format!("{}\n", Value::Object(fields));
}

/// The plan a file's text holds, or why it holds none.
fn decode(text: &str) -> Result<StoredPlan, String> {
    let value: Value = serde_json::from_str(text).map_err(|error| { return format!("the plan is not JSON: {error}"); })?;
    if value.get("v").and_then(Value::as_u64) != Some(PLAN_VERSION) {
        return Err(String::from("the plan has an unsupported version"));
    }
    let attempt: u32 = value.get("attempt").and_then(Value::as_u64).and_then(|number| { return u32::try_from(number).ok(); }).ok_or_else(|| { return String::from("the plan has no try number"); })?;
    let git_dir: &str = value.get("gitDir").and_then(Value::as_str).ok_or_else(|| { return String::from("the plan names no Git directory"); })?;
    let item_text: &str = value.get("item").and_then(Value::as_str).ok_or_else(|| { return String::from("the plan holds no push item"); })?;
    let item: WorkItem = WorkItem::decode(item_text)?;
    return Ok(StoredPlan { git_dir: PathBuf::from(git_dir), item, attempt });
}

/// What:
///  Writes the plan of a branch and removes the files of its earlier plans.
/// Why:
///   The new file is published whole before the old ones go, so a kill between the two leaves a plan, not nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeExclusive(join(dir, name(plan)), encode(plan)); await removeOthersOfBranch(dir, plan);
/// ```
pub fn save(directory: &Path, plan: &StoredPlan) -> Result<(), String> {
    create_private_directory_all(directory).map_err(|error| { return format!("creating the plan directory failed: {error}"); })?;
    let name: String = plan_name(plan);
    match write_private_file_unsynced(directory.join(name.as_str()).as_path(), encode(plan).as_bytes()) {
        Ok(()) => {}
        // The same try of the same commit is already there.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(format!("writing the plan failed: {error}")),
    }
    remove_files_of(directory, branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str()).as_str(), Some(name.as_str()));
    return Ok(());
}

/// Removes the files of the plan of a branch; one that cannot be removed stays and is read again by the next start.
pub fn remove(directory: &Path, git_dir: &Path, branch_ref: &str) {
    remove_files_of(directory, branch_key(git_dir, branch_ref).as_str(), None);
}

/// Removes the plan files of one branch key, except the one named `keep`.
fn remove_files_of(directory: &Path, key: &str, keep: Option<&str>) {
    let Ok(listing) = std::fs::read_dir(directory) else { return };
    let prefix: String = format!("{key}{NAME_SEPARATOR}");
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix.as_str()) && keep != Some(name.as_str()) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

/// What:
///  Reads every plan in the directory, one per branch, the newest file winning; a file that holds no plan is said and removed.
/// Why:
///   A restarted service rebuilds its plan from these. Older files of a branch are what a kill between a save and its cleanup
///  leaves, and a file that cannot be read would otherwise be said at every start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plans = await load(dir) // one per branch
/// ```
pub fn load(directory: &Path) -> Vec<StoredPlan> {
    let Ok(listing) = std::fs::read_dir(directory) else { return Vec::new() };
    let mut found: Vec<(SystemTime, StoredPlan)> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.extension().and_then(|extension| { return extension.to_str(); }) != Some(PLAN_EXTENSION) {
            continue;
        }
        let written: SystemTime = entry.metadata().and_then(|meta| { return meta.modified(); }).unwrap_or(SystemTime::UNIX_EPOCH);
        match std::fs::read_to_string(path.as_path()).map_err(|error| { return error.to_string(); }).and_then(|text| { return decode(text.as_str()); }) {
            Ok(plan) => found.push((written, plan)),
            Err(reason) => {
                crate::service_warn!("the retry plan file {} holds no plan and is removed: {reason}", path.display());
                let _removed = std::fs::remove_file(path.as_path());
            }
        }
    }
    found.sort_by_key(|(written, _)| { return std::cmp::Reverse(*written); });
    let mut plans: Vec<StoredPlan> = Vec::new();
    for (_, plan) in found {
        let same_branch = |kept: &StoredPlan| { return kept.git_dir == plan.git_dir && kept.item.branch_ref == plan.item.branch_ref; };
        if !plans.iter().any(same_branch) {
            plans.push(plan);
        }
    }
    return plans;
}

/// Plan file controls stay out of the release executable.
#[cfg(test)]
#[path = "push_retry_store_tests.rs"]
mod tests;
