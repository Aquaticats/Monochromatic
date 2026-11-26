//! What:
//!  The add's read of the removal's destruction-plan folder:
//!  it finds the plan that names the clone the add is about to fill,
//!  and answers what that plan records about its own head destroy and its
//!  snapshot attempts.
//! Why:
//!  An add that closed its binding over a standing plan would leave the plan with no clone,
//!  no binding and no command,
//!  and the plan is the record of what a refused removal had already destroyed
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  `V24`;
//!  the project kept the choice on 2026-11-23 and builds the read in step 10).
//!  The person who meets a standing plan reads the one command that ends it,
//!  which the plan's head word and the clone the pool can find decide
//!  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
//!  choice 2C),
//!  so this reader answers those two fields and the add's own stop picks the
//!  command from them (`worktree_clone_add_route::add_plan_stop`).
//!  The plan grammar is the plan design's
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The grammar"),
//!  and this reader asks only the fields the refusal and its command need:
//!  the clone's two identities,
//!  the head word,
//!  and whether every attempt line still says `planned`.
//!  A plan it cannot read refuses too,
//!  because nothing proves that plan does not name this clone.
//!  The folder's file-name rule is this file's,
//!  and the removal's plan reader calls it rather than repeating it,
//!  so one grammar's plans are found one way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = thePlanNamingTheClone(commonDir, cloneGuid, cloneDataset);
//! ```

use crate::worktree_clone_binding_record::add_state_commands::PlanHeadWord;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{Path, PathBuf};

/// The largest one plan file this reader reads.
/// A plan records names and identities rather than history,
/// so this bound is the read's own runaway guard and not a grammar's limit.
const PLAN_READ_MAXIMUM_BYTES: u64 = 262_144;

/// The field a plan records the clone's own GUID in.
const CLONE_GUID_FIELD: &str = "cloneGuid=";

/// The field a plan records the clone's dataset name in.
const CLONE_DATASET_FIELD: &str = "cloneDataset=";

/// The two fields a plan may name the clone by.
const CLONE_FIELDS: [&str; 2] = [CLONE_GUID_FIELD, CLONE_DATASET_FIELD];

/// The field a plan records the state of its own head destroy in.
const HEAD_FIELD: &str = "head=";

/// The field one snapshot attempt line begins with.
const ATTEMPT_FIELD: &str = "tuple=";

/// The outcome word a plan's first write records for every snapshot it planned.
/// A plan whose every attempt line holds it witnesses nothing destroyed,
/// which is the state the cancel command is named beside
/// (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
/// `V24`).
const PLANNED_OUTCOME: &str = "planned";

/// How many whitespace-separated words of an attempt line come before its
/// outcome:
/// `tuple=<kind>` then the attempt number,
/// so the outcome is the third word.
const ATTEMPT_WORDS_BEFORE_THE_OUTCOME: usize = 2;

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
///  One readable plan that names the clone this add would fill,
///  with the parts of its record the add's stop reads.
/// Why:
///  The person's next command follows from the plan's head word and the clone
///  the pool can find,
///  and the cancel command is named beside it only where the plan witnesses
///  nothing destroyed,
///  so the stop needs the recorded clone's two identities,
///  its head word and whether every attempt line still says `planned`
///  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
///  choice 2C).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type APlanThatNamesTheClone = { plan: string; cloneDataset: string; cloneGuid: string; head: PlanHeadWord; everyTuplePlanned: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct APlanThatNamesTheClone {
    /// The plan file.
    pub plan: PathBuf,
    /// The clone's dataset name as the plan records it,
    /// which is the name the pool's copy would answer under.
    pub clone_dataset: String,
    /// The clone's own GUID as the plan records it,
    /// which is the identity a child of the container carries.
    pub clone_guid: DatasetGuid,
    /// The plan's head word,
    /// which is the state of the clone's own destroy.
    pub head: PlanHeadWord,
    /// Whether every attempt line the plan holds still says `planned`,
    /// so the plan witnesses nothing destroyed.
    pub every_tuple_planned: bool,
}

/// What:
///  Why a file in the plan folder could not be read as a plan.
/// Why:
///  A plan that cannot be read may name this clone,
///  and refusing changes nothing,
///  which is the same answer the design gives an unreadable record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanUnreadable = { plan: string; sentence: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanUnreadable {
    /// The plan file,
    /// or the folder when the folder itself could not be listed.
    pub plan: PathBuf,
    /// The sentence a person reads.
    pub sentence: String,
}

/// Whether one plan's bytes name this clone,
/// looking only at the two fields a plan may carry it in.
fn names_the_clone(bytes: &[u8], clone_guid: Option<DatasetGuid>, clone_dataset: &str) -> bool {
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes);
    for line in text.lines() {
        for field in CLONE_FIELDS {
            let Some(found) = line.strip_prefix(field) else { continue };
            let trimmed: &str = found.trim();
            if field == CLONE_DATASET_FIELD && trimmed == clone_dataset {
                return true;
            }
            if field == CLONE_GUID_FIELD && clone_guid.is_some_and(|guid| return trimmed == guid.file_name()) {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  The plan that names the clone this add would fill,
///  or nothing when the folder holds none,
///  or the file that could not be read as a plan.
/// Why:
///  This is the add's only read of another design's folder,
///  made to refuse and for nothing else:
///  it acts on no plan and removes none,
///  and it answers the plan rather than a sentence because the command the
///  person reads needs the pool as well
///  (`worktree_clone_add_route::add_plan_stop`).
///  A folder that is absent holds no plan,
///  and a folder that cannot be listed refuses,
///  because nothing then proves no plan names this clone.
///
/// # Errors
/// The plan that could not be read,
/// with the sentence the add prints before it changes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = thePlanNamingTheClone(commonDir, cloneGuid, cloneDataset); // undefined when none stands
/// ```
pub fn the_plan_naming_the_clone(common_dir: &Path, clone_guid: Option<DatasetGuid>, clone_dataset: &str) -> Result<Option<APlanThatNamesTheClone>, PlanUnreadable> {
    let folder: PathBuf = super::registration_home::plans_folder(common_dir);
    let entries = match std::fs::read_dir(folder.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(PlanUnreadable {
                plan: folder,
                sentence: format!("the plan folder could not be listed ({error}), so nothing proves no plan names this clone. Nothing was changed. Ways forward: fix the folder's access and run the command again."),
            });
        }
    };
    for entry in entries {
        let found_entry = entry.map_err(|error| {
            return PlanUnreadable {
                plan: folder.clone(),
                sentence: format!("the plan folder could not be read past one entry ({error}), so nothing proves no plan names this clone. Nothing was changed. Ways forward: fix the folder's access and run the command again."),
            };
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
        // A plan filed under this clone's own GUID is this clone's whatever its fields say,
        // so the key decides before any field is compared.
        let by_its_key: bool = clone_guid.is_some_and(|guid| return name == guid.file_name());
        let bytes: Vec<u8> = read_a_plan(path.as_path())?;
        if !by_its_key && !names_the_clone(bytes.as_slice(), clone_guid, clone_dataset) {
            continue;
        }
        return Ok(Some(what_the_plan_records(path.as_path(), bytes.as_slice())?));
    }
    return Ok(None);
}

/// What:
///  The parts of one plan's record the add's stop reads,
///  from the bytes already proved to begin with the route's plan header.
/// Why:
///  The command a standing plan leaves the person follows from these four
///  values alone,
///  so the parse stops at them and refuses the lines it cannot read,
///  because a plan this add cannot read is one nothing proves harmless.
///
/// # Errors
/// The sentence naming the field or the line that refused the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = whatThePlanRecords(path, bytes); // throws the sentence for a field it cannot read
/// ```
fn what_the_plan_records(path: &Path, bytes: &[u8]) -> Result<APlanThatNamesTheClone, PlanUnreadable> {
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes);
    let mut recorded_dataset: Option<String> = None;
    let mut recorded_guid: Option<DatasetGuid> = None;
    let mut recorded_head: Option<PlanHeadWord> = None;
    let mut every_tuple_planned: bool = true;
    for line in text.lines() {
        if let Some(found) = line.strip_prefix(CLONE_DATASET_FIELD) {
            recorded_dataset = Some(String::from(found.trim()));
            continue;
        }
        if let Some(found) = line.strip_prefix(CLONE_GUID_FIELD) {
            let word: &str = found.trim();
            let guid: DatasetGuid = DatasetGuid::parse(word).map_err(|refusal| return unreadable(path, format!("its `cloneGuid=` line does not hold a canonical dataset GUID ({refusal:?})").as_str()))?;
            recorded_guid = Some(guid);
            continue;
        }
        if let Some(found) = line.strip_prefix(HEAD_FIELD) {
            let word: &str = found.trim();
            let parsed: PlanHeadWord = PlanHeadWord::parse(word).ok_or_else(|| return unreadable(path, format!("its `head=` line holds `{word}`, which is not one of the five words the plan design's grammar allows").as_str()))?;
            recorded_head = Some(parsed);
            continue;
        }
        if line.starts_with(ATTEMPT_FIELD) && !says_planned(line) {
            every_tuple_planned = false;
        }
    }
    let Some(clone_dataset) = recorded_dataset else {
        return Err(unreadable(path, "it carries no `cloneDataset=` line, so this add cannot tell which clone's record it is"));
    };
    let Some(clone_guid) = recorded_guid else {
        return Err(unreadable(path, "it carries no `cloneGuid=` line, so nothing proves which clone's absence the pool would be asked about"));
    };
    let Some(head) = recorded_head else {
        return Err(unreadable(path, "it carries no `head=` line, so this add cannot tell which command the plan's head gives"));
    };
    // What:     `Ok(APlanThatNamesTheClone { .. })` builds the success variant
    //           of `Result`, wrapping the four values the stop reads.
    // Why:      Hand the add's stop a whole record, or one sentence saying
    //           which field refused it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { plan, cloneDataset: clone_dataset, cloneGuid: clone_guid, head, everyTuplePlanned: every_tuple_planned };
    // ```
    return Ok(APlanThatNamesTheClone { plan: PathBuf::from(path), clone_dataset, clone_guid, head, every_tuple_planned });
}

/// Whether one attempt line records the `planned` outcome,
/// which is where the outcome word sits after the tuple's kind and number.
/// A line with too few words holds no outcome this reader can find,
/// and it is answered as one that is not `planned`:
/// the cancel command is then not named beside the plan,
/// which leaves out a command rather than naming one that may destroy.
fn says_planned(line: &str) -> bool {
    let mut words = line.split_ascii_whitespace();
    for _ in 0..ATTEMPT_WORDS_BEFORE_THE_OUTCOME {
        let Some(_word) = words.next() else { return false };
    }
    return words.next() == Some(PLANNED_OUTCOME);
}

/// The sentence for a plan this reader cannot read,
/// which names the file and what the plan failed to say.
fn unreadable(path: &Path, what: &str) -> PlanUnreadable {
    return PlanUnreadable {
        plan: PathBuf::from(path),
        sentence: format!("the plan at {} was not read as a plan: {what}. Nothing was changed. Ways forward: inspect the plan's file and run the command again.", path.display()),
    };
}

/// One plan's bytes,
/// or the sentence that says why they were not read.
fn read_a_plan(path: &Path) -> Result<Vec<u8>, PlanUnreadable> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).map_err(|error| return unreadable(path, format!("it could not be examined ({error})").as_str()))?;
    if !metadata.is_file() {
        return Err(unreadable(path, "it is not a regular file"));
    }
    if metadata.len() > PLAN_READ_MAXIMUM_BYTES {
        return Err(unreadable(path, format!("it holds more bytes ({}) than this reader reads ({PLAN_READ_MAXIMUM_BYTES})", metadata.len()).as_str()));
    }
    let bytes: Vec<u8> = std::fs::read(path).map_err(|error| return unreadable(path, format!("it could not be read ({error})").as_str()))?;
    if !bytes.starts_with(PLAN_HEADER.as_bytes()) {
        return Err(unreadable(path, "it does not begin with this route's plan header"));
    }
    // What:     `Ok(bytes)` wraps the plan's bytes in the success variant of
    //           `Result`.
    // Why:      The caller parses the bytes this read proved to be one plan.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return bytes;
    // ```
    return Ok(bytes);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_folder_read_tests.rs"]
mod tests;
