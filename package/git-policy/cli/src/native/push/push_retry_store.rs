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
//! A plan records the instant it was written,
//! and a forget removes only the plans written before the round that asked for it began:
//! the job that applies a round's changes is never cancelled,
//! so it can outlive the round,
//! and its late forget must not take a statement a later round wrote.
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

/// The instant a plan is written,
/// in microseconds since the epoch,
/// which is how a removal tells a statement newer than a round from an older one.
/// Why:
///   The record's own instant is read back with the plan,
/// so a forget a round applies is judged against the round it belongs to and never against the clock of its removal;
/// a filesystem's timestamps also depend on the mount, while this field is the writer's.
///
pub fn micros_now() -> u64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_or(0, |elapsed| { return u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX); });
}

/// The text of a plan file,
/// written at `saved`.
fn encode_at(plan: &StoredPlan, saved: u64) -> String {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("v"), Value::from(PLAN_VERSION));
    fields.insert(String::from("attempt"), Value::from(plan.attempt));
    fields.insert(String::from("saved"), Value::from(saved));
    // The directory goes in as plain text and as its bytes, so a plan is read back as the directory it was saved for.
    path_bytes::write_field(&mut fields, "gitDir", plan.git_dir.as_path());
    fields.insert(String::from("item"), Value::from(plan.item.encode()));
    return format!("{}\n", Value::Object(fields));
}

/// The text of a plan file written now.
fn encode(plan: &StoredPlan) -> String {
    return encode_at(plan, micros_now());
}

/// The plan a file's text holds and the instant it records,
/// which is nothing for a file an earlier release wrote.
fn decode_at(text: &str) -> Result<(StoredPlan, u64), String> {
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
    // A file an earlier release wrote records no instant, and it is older than any statement this release writes.
    let saved: u64 = value.get("saved").and_then(Value::as_u64).unwrap_or(0);
    let item: WorkItem = WorkItem::decode(item_text)?;
    return Ok((StoredPlan { git_dir: path_bytes::read_field(&value, "gitDir")?, item, attempt }, saved));
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
    match write_private_file_unsynced(directory.join(name.as_str()).as_path(), encode_at(plan, micros_now()).as_bytes()) {
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
        // A save is the newest statement of its branch, so every other file of it goes.
        None,
    );
    // A file a save could not remove is read again by the next start, which is what `load` says.
    return Ok(());
}

/// The file name the plan of a branch is kept under,
/// for a caller that must know which files a round is about to write.
pub fn name_of(plan: &StoredPlan) -> String {
    return plan_name(plan);
}

/// Removes the files of the plan of a branch;
/// one that cannot be removed stays and is read again by the next start.
pub fn remove(directory: &Path, git_dir: &Path, branch_ref: &str) {
    let _left: bool = remove_files_of(directory, git_dir, branch_ref, &[branch_key(git_dir, branch_ref), earlier_branch_key(git_dir, branch_ref)], &[], None);
}

/// What:
///  Removes the files of the plan of a branch except the ones named in `keep`
///  and the ones written at or after `written_before`.
/// Why:
///   A forget a worker asked for is applied by the service loop after the round,
/// so a plan another worker kept meanwhile is newer than the forget and is the statement that still counts;
/// its file name is known before it is written,
/// and is not removed.
/// The job that applies a round's changes is not cancelled,
/// so a round's forget can be written long after that round ended,
/// and a later round may have written the newer statement of the same branch meanwhile;
/// `written_before` is the instant the forgetting round began,
/// so a plan written since is newer than what that round read and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeFilesOf(gitDir, branchRef, keys, keep, roundBeganAt)
/// ```
pub fn remove_except(directory: &Path, git_dir: &Path, branch_ref: &str, keep: &[String], written_before: u64) -> bool {
    return remove_files_of(
        directory,
        git_dir,
        branch_ref,
        &[branch_key(git_dir, branch_ref), earlier_branch_key(git_dir, branch_ref)],
        keep,
        Some(written_before),
    );
}

/// Removes the plan files of one branch of `git_dir`,
/// whose keys are `keys`,
/// except the ones named in `keep` and the ones written at or after `written_before`,
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
/// which a forget of the same round must not take,
///  and `written_before` is the instant the forget was asked,
/// so a statement written since is not taken either.
/// Answers whether a plan of the branch that this removal covers is still there,
/// which is what a caller that tries the removal again reads:
/// a file it left because it is newer, unreadable or another branch's is not one it covers.
fn remove_files_of(directory: &Path, git_dir: &Path, branch_ref: &str, keys: &[String], keep: &[String], written_before: Option<u64>) -> bool {
    let Ok(listing) = std::fs::read_dir(directory) else { return false };
    let mut left: bool = false;
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if keep.contains(&name) || !names_the_branch(keys, name.as_str()) {
            continue;
        }
        let path: PathBuf = entry.path();
        let Some(saved): Option<u64> = saved_of_the_branch(path.as_path(), git_dir, branch_ref) else { continue };
        // A plan written at or since the forget was asked is a newer statement than the one that forget read.
        if written_before.is_some_and(|asked: u64| saved >= asked) {
            continue;
        }
        if std::fs::remove_file(path).is_err() {
            left = true;
        }
    }
    return left;
}

/// The instant the plan in `path` records,
/// when that plan was read and names `git_dir` and `branch_ref`,
/// which is the only case a cleanup may remove it.
/// Why:
///   A read or decode that fails is no proof about the plan:
/// an unreadable file and a file that holds no plan both stay,
/// and `load` says what it found and removes what is no plan.
/// The instant is what tells a statement a later round wrote from one the forgetting round read.
fn saved_of_the_branch(path: &Path, git_dir: &Path, branch_ref: &str) -> Option<u64> {
    let text: String = std::fs::read_to_string(path).ok()?;
    let (found, saved): (StoredPlan, u64) = decode_at(text.as_str()).ok()?;
    if found.git_dir == git_dir && found.item.branch_ref == branch_ref {
        return Some(saved);
    }
    return None;
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
///  Reads every plan in the directory,
/// one per branch,
/// the newest file winning;
/// a file that holds no plan is said and removed.
/// Why:
///   A restarted service rebuilds its plan from these.
/// Older files of a branch are what a kill between a save and its cleanup
///  leaves,
/// and a file that cannot be read would otherwise be said at every start.
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
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(PLAN_EXTENSION) {
            continue;
        }
        let written: SystemTime = match entry.metadata() {
            Ok(meta) => meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            Err(_) => SystemTime::UNIX_EPOCH,
        };
        let decoded: Result<(StoredPlan, u64), String> = match std::fs::read_to_string(path.as_path()) {
            Ok(text) => decode_at(text.as_str()),
            Err(error) => Err(error.to_string()),
        };
        match decoded {
            Ok(plan) => found.push((written, plan.0)),
            Err(reason) => {
                crate::service_warn!("the retry plan file {} holds no plan and is removed: {reason}", path.display());
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
    return plans;
}

/// Plan file controls stay out of the release executable.
#[cfg(test)]
#[path = "push_retry_store_tests.rs"]
mod tests;
