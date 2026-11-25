//! What:
//!  The add's read of the removal's destruction-plan folder:
//!  it refuses while a plan names the clone the add is about to fill.
//! Why:
//!  An add that closed its binding over a standing plan would leave the plan with no clone,
//!  no binding and no command,
//!  and the plan is the record of what a refused removal had already destroyed
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`, `V24`;
//!  the project kept the choice on 2026-11-23 and builds the read in step 10).
//!  No plan can stand before step 14,
//!  so this read finds nothing until then,
//!  and its control plants one.
//!  The plan grammar is not accepted yet,
//!  so this reader asks only what the refusal needs:
//!  a `cloneGuid=` or `cloneDataset=` line naming this clone.
//!  A plan it cannot read refuses too,
//!  because nothing proves that plan does not name this clone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = refuseWhileAPlanNamesTheClone(commonDir, cloneGuid, cloneDataset);
//! ```

use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{Path, PathBuf};

/// The largest one plan file this reader reads.
/// A plan records names and identities rather than history,
/// so this bound is the read's own runaway guard and not a grammar's limit.
const PLAN_READ_MAXIMUM_BYTES: u64 = 262_144;

/// The two fields a plan may name the clone by.
const CLONE_FIELDS: [&str; 2] = ["cloneGuid=", "cloneDataset="];

/// The first line every plan of this route carries,
/// which names its grammar and its version.
///
/// Why:
///  A file under a clone's GUID that does not carry it is not a plan this add can read,
///  and nothing proves such a file does not name this clone.
pub const PLAN_HEADER: &str = "# monochromatic-git-policy-cli zfs worktree clone destruction plan, version 1";

/// What:
///  Whether one entry's name is a clone's canonical numeric dataset GUID,
///  which is the plan's key and file name.
/// Why:
///  The plan's grammar is one grammar across the add,
///  the removal and the later steps that read a plan,
///  so this is the route's only GUID-name check:
///  the name reader in
///  `worktree_clone_removal_proof_and_plan::plan_name` calls it rather than repeating it
///  (plan step 10's binding record;
///  `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "The key,
///  and why it is the clone's GUID").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isCanonicalGuidName = (name: string) => /^(0|[1-9][0-9]{0,19})$/.test(name);
/// ```
pub fn is_canonical_guid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 20 || (name.len() > 1 && name.starts_with('0')) {
        return false;
    }
    return name.bytes().all(|byte| return byte.is_ascii_digit());
}

/// What:
///  Why the add must not go on while a plan stands.
/// Why:
///  The refusal names the plan and the removal that ends it,
///  so the person is not left to guess which of two commands is true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { plan: string; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanNamesTheClone {
    /// The plan file.
    pub plan: PathBuf,
    /// The sentence a person reads.
    pub sentence: String,
}

/// What:
///  Why the plan folder could not be read.
/// Why:
///  A plan that cannot be read may name this clone,
///  and refusing changes nothing,
///  which is the same answer the design gives an unreadable record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { plan: string; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanUnreadable {
    /// The plan file.
    pub plan: PathBuf,
    /// The sentence a person reads.
    pub sentence: String,
}

/// What:
///  Why the add must stop before its first mutation.
/// Why:
///  One answer serves the caller,
///  and each arm names its file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stop = { kind: 'named'; plan: string } | { kind: 'unreadable'; plan: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanStop {
    /// A readable plan names this clone.
    NamesTheClone(PlanNamesTheClone),
    /// A file in the folder could not be read as a plan.
    Unreadable(PlanUnreadable),
}

impl PlanStop {
    /// The sentence a person reads.
    pub fn sentence(&self) -> &str {
        return match self {
            PlanStop::NamesTheClone(stop) => stop.sentence.as_str(),
            PlanStop::Unreadable(stop) => stop.sentence.as_str(),
        };
    }

    /// The plan file the stop names.
    pub fn plan(&self) -> &Path {
        return match self {
            PlanStop::NamesTheClone(stop) => stop.plan.as_path(),
            PlanStop::Unreadable(stop) => stop.plan.as_path(),
        };
    }
}

/// Whether one plan's bytes name this clone,
/// looking only at the two fields a plan may carry it in.
fn names_the_clone(bytes: &[u8], clone_guid: Option<DatasetGuid>, clone_dataset: &str) -> bool {
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes);
    for line in text.lines() {
        for field in CLONE_FIELDS {
            let Some(found) = line.strip_prefix(field) else { continue };
            let trimmed: &str = found.trim();
            if field == "cloneDataset=" && trimmed == clone_dataset {
                return true;
            }
            if field == "cloneGuid=" && clone_guid.is_some_and(|guid| return trimmed == guid.file_name()) {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  Refuse while a destruction plan in the folder names this clone,
///  and while a file there cannot be read.
/// Why:
///  This is the add's only read of another design's folder,
///  made to refuse and for nothing else:
///  it acts on no plan and removes none.
///
/// # Errors
/// The plan that names the clone or the file that could not be read,
/// each with the sentence the add prints before it changes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stop = refuseWhileAPlanNamesTheClone(commonDir, cloneGuid, cloneDataset);
/// ```
pub fn refuse_while_a_plan_names_the_clone(common_dir: &Path, clone_guid: Option<DatasetGuid>, clone_dataset: &str) -> Result<(), PlanStop> {
    let folder: PathBuf = super::registration_home::plans_folder(common_dir);
    let entries = match std::fs::read_dir(folder.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(PlanStop::Unreadable(PlanUnreadable {
                plan: folder,
                sentence: format!("the plan folder could not be listed ({error}), so nothing proves no plan names this clone. Nothing was changed. Ways forward: fix the folder's access and run the command again."),
            }));
        }
    };
    for entry in entries {
        let found_entry = entry.map_err(|error| {
            return PlanStop::Unreadable(PlanUnreadable {
                plan: folder.clone(),
                sentence: format!("the plan folder could not be read past one entry ({error}), so nothing proves no plan names this clone. Nothing was changed. Ways forward: fix the folder's access and run the command again."),
            });
        })?;
        let path: PathBuf = found_entry.path();
        let Some(file_name) = path.file_name() else { continue };
        if file_name.as_encoded_bytes().starts_with(b".") {
            continue;
        }
        let Some(name) = file_name.to_str() else { continue };
        // The plan's key is the clone's GUID, and its name is that key:
        // every other entry is foreign, reported by `doctor` and read by nobody here.
        if !is_canonical_guid_name(name) {
            continue;
        }
        if !found_entry.file_type().map(|kind| return kind.is_file()).unwrap_or(false) {
            continue;
        }
        if clone_guid.is_some_and(|guid| return name == guid.file_name()) {
            return Err(PlanStop::NamesTheClone(PlanNamesTheClone {
                plan: path.clone(),
                sentence: names_the_clone_sentence(path.as_path()),
            }));
        }
        let bytes: Vec<u8> = match read_a_plan(path.as_path()) {
            Ok(found) => found,
            Err(sentence) => return Err(PlanStop::Unreadable(PlanUnreadable { plan: path, sentence })),
        };
        if names_the_clone(bytes.as_slice(), clone_guid, clone_dataset) {
            return Err(PlanStop::NamesTheClone(PlanNamesTheClone {
                plan: path.clone(),
                sentence: names_the_clone_sentence(path.as_path()),
            }));
        }
    }
    return Ok(());
}

/// The sentence a plan that names this clone prints.
fn names_the_clone_sentence(path: &Path) -> String {
    return format!(
        "a removal's plan at {} names the clone this add would fill, so the add stops before its first mutation and changes nothing. Ways forward: run the removal this plan belongs to, which resumes from the plan and ends it; then run the add again at the slot.",
        path.display()
    );
}

/// One plan's bytes,
/// or the sentence that says why they were not read.
fn read_a_plan(path: &Path) -> Result<Vec<u8>, String> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).map_err(|error| return format!("the plan at {} could not be examined ({error}). Nothing was changed. Ways forward: inspect the file and run the command again.", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("the plan at {} is not a regular file, so it was not read. Nothing was changed. Ways forward: inspect the name and run the command again.", path.display()));
    }
    if metadata.len() > PLAN_READ_MAXIMUM_BYTES {
        return Err(format!(
            "the plan at {} holds more bytes ({}) than this reader reads ({PLAN_READ_MAXIMUM_BYTES}). Nothing was changed. Ways forward: inspect the file and run the command again.",
            path.display(),
            metadata.len()
        ));
    }
    let bytes: Vec<u8> = std::fs::read(path).map_err(|error| return format!("the plan at {} could not be read ({error}). Nothing was changed. Ways forward: inspect the file and run the command again.", path.display()))?;
    if !bytes.starts_with(PLAN_HEADER.as_bytes()) {
        return Err(format!(
            "the plan at {} does not begin with this route's plan header, so it was not read as a plan. Nothing was changed. Ways forward: inspect the file and run the command again.",
            path.display()
        ));
    }
    return Ok(bytes);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_folder_read_tests.rs"]
mod tests;
