//! What:
//!  The two-stage read of one destruction plan:
//!  the header,
//!  then the file's own length and the attempt lines it declares,
//!  with the folder's mode,
//!  the record's owner and the name it is filed under checked first.
//! Why:
//!  The reader holds no more than the record's own bytes with no count ceiling:
//!  stage one reads at most the header bound and requires the declared count,
//!  stage two refuses a file longer than `10,130 + 361 * count` after the length read and before an
//!  attempt byte,
//!  then reads at most `361` bytes for each line the record declares
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The size bound and its basis").
//!  A record that does not read,
//!  a foreign entry,
//!  a wrongly owned one and a folder that is not `0700` each refuse and change nothing,
//!  which is the project's answer to an unreadable record
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "A record that cannot be read").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = readPlan(commonDir, cloneGuid);
//! ```

use crate::foundation::plain_file::open_plain_file;
use crate::worktree_clone_binding_record::plan_folder_read::is_canonical_guid_name;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_bounds::{
    ATTEMPT_LINE_MAXIMUM_BYTES,
    PLAN_HEADER_MAXIMUM_BYTES,
};
use crate::worktree_clone_removal_proof_and_plan::plan_parse::{
    PlanHeader,
    parse_plan,
    parse_plan_header,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::PLAN_KEYS_IN_ORDER;
use std::io::Read;
use std::path::{
    Path,
    PathBuf,
};

/// The lines the header region holds:
/// the version line and the plan's eighteen fields.
pub const HEADER_LINE_COUNT: usize = 1 + PLAN_KEYS_IN_ORDER.len();

/// What:
///  One plan file's bytes read in the two stages,
///  or the refusal that says which stage refused.
/// Why:
///  A caller that holds a path and no common directory still reads a plan;
///  the ownership checks belong to the reader that knows whose folder this is.
///
/// # Throws
/// The stage,
/// the field or the line that refused the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = readPlanAt(path, fileName);
/// ```
pub fn read_the_plan_at(
    path: &Path,
    filed_under: Option<&str>,
) -> Result<DestructionPlan, PlanRefusal> {
    let mut file: std::fs::File = open_plain_file(path).map_err(|error| {
        return match error.kind() {
            std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::NotFound => {
                PlanRefusal::at(PlanRefusalCause::Unreadable, "file", error.to_string())
            }
            _ => PlanRefusal::of(PlanRefusalCause::NotARegularFile, error.to_string()),
        };
    })?;
    let mut stage_one: Vec<u8> = Vec::new();
    (&mut file)
        .take(PLAN_HEADER_MAXIMUM_BYTES)
        .read_to_end(&mut stage_one)
        .map_err(|error| return PlanRefusal::of(PlanRefusalCause::Unreadable, error.to_string()))?;
    let header_bytes: usize = header_region_bytes(stage_one.as_slice())?;
    let parsed_header: PlanHeader = parse_plan_header(stage_one[..header_bytes].as_ref())?;
    let count: u64 = parsed_header.count;
    drop(parsed_header);
    // The design's bound is two bounds: the header, which stage one refused past its own cap, and the attempt
    // region, which holds at most one widest line for each line the record declares. Their sum is
    // `plan_maximum_bytes(count)`, and this is where the second half of it is refused, after the length read and
    // before an attempt byte.
    let length: u64 = file
        .metadata()
        .map_err(|error| return PlanRefusal::of(PlanRefusalCause::Unreadable, error.to_string()))?
        .len();
    let held_after_the_header: u64 = length - header_bytes as u64;
    let already_read: u64 = stage_one.len() as u64 - header_bytes as u64;
    if already_read > held_after_the_header {
        return Err(PlanRefusal::of(
            PlanRefusalCause::Oversized,
            format!(
                "{already_read} bytes past the header where the file holds {held_after_the_header}"
            ),
        ));
    }
    let widest_attempts: u64 = ATTEMPT_LINE_MAXIMUM_BYTES * count;
    if held_after_the_header > widest_attempts {
        return Err(PlanRefusal::at(
            PlanRefusalCause::Oversized,
            "attempts",
            format!(
                "{held_after_the_header} bytes for {count} declared lines, and {widest_attempts} is the most they allow"
            ),
        ));
    }
    let mut stage_two: Vec<u8> = stage_one[header_bytes..].to_vec();
    let remaining: u64 = held_after_the_header - already_read;
    (&mut file)
        .take(remaining)
        .read_to_end(&mut stage_two)
        .map_err(|error| return PlanRefusal::of(PlanRefusalCause::Unreadable, error.to_string()))?;
    if stage_two.len() as u64 != held_after_the_header {
        return Err(PlanRefusal::of(
            PlanRefusalCause::MissingAttemptLines,
            format!(
                "{} bytes where the file holds {held_after_the_header}",
                stage_two.len()
            ),
        ));
    }
    // A byte past what the length promised means the file changed between the two stages, and a
    // record that changed while it was read is not the record either stage saw.
    let mut past: [u8; 1] = [0; 1];
    if file
        .read(&mut past)
        .map_err(|error| return PlanRefusal::of(PlanRefusalCause::Unreadable, error.to_string()))?
        > 0
    {
        return Err(PlanRefusal::of(
            PlanRefusalCause::Oversized,
            "the file grew while the reader held it",
        ));
    }
    return parse_plan(
        stage_one[..header_bytes].as_ref(),
        stage_two.as_slice(),
        filed_under,
    );
}

/// What:
///  The plan one clone's removal left in the common directory's plan folder,
///  with the folder's mode and the record's owner checked first.
/// Why:
///  Every reader of a plan finds it by the clone's GUID and no other record,
///  and a folder whose mode is not `0700` or a record another account owns is refused rather
///  than read.
///
/// # Throws
/// The folder,
/// the record or the grammar's refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = readPlan(commonDir, cloneGuid);
/// ```
pub fn read_the_plan(
    common_dir: &Path,
    clone_guid: DatasetGuid,
) -> Result<DestructionPlan, PlanRefusal> {
    let folder: PathBuf = plans_folder(common_dir);
    let folder_owner: u32 = common_directory_owner(common_dir)?;
    check_the_folder(folder.as_path(), folder_owner)?;
    let name: String = clone_guid.file_name();
    let path: PathBuf = folder.join(name.as_str());
    check_the_record_owner(path.as_path(), folder_owner)?;
    return read_the_plan_at(path.as_path(), Some(name.as_str()));
}

/// What:
///  Refuse a plan folder that is not the common directory's owner's `0700` folder.
/// Why:
///  A folder another account owns,
///  or one whose mode has group or other bits,
///  is a person's mistake or a restored backup,
///  and the project's threat model makes this a mistake-catcher rather than a boundary.
///
/// # Throws
/// The mode or the owner that differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkFolder(folderPath, commonDirOwner);
/// ```
pub fn check_the_folder(
    folder: &Path,
    owner: u32,
) -> Result<(), PlanRefusal> {
    let metadata: std::fs::Metadata = std::fs::metadata(folder).map_err(|error| {
        return PlanRefusal::at(PlanRefusalCause::Unreadable, "folder", error.to_string());
    })?;
    if !metadata.is_dir() {
        return Err(PlanRefusal::at(
            PlanRefusalCause::NotARegularFile,
            "folder",
            "not a directory",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let mode: u32 = metadata.mode() & 0o777;
        if mode != 0o700 {
            return Err(PlanRefusal::at(
                PlanRefusalCause::FolderMode,
                "folder",
                format!("{mode:04o}"),
            ));
        }
        if metadata.uid() != owner {
            return Err(PlanRefusal::at(
                PlanRefusalCause::WrongOwner,
                "folder",
                metadata.uid().to_string(),
            ));
        }
    }
    #[cfg(not(unix))]
    drop(owner);
    return Ok(());
}

/// What:
///  Refuse a plan file that the folder's owner does not own.
/// Why:
///  The account that writes a plan is the one that owns the common directory,
///  so a record another account owns is not this removal's.
///
/// # Throws
/// The owner that differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkRecordOwner(planPath, folderOwner);
/// ```
pub fn check_the_record_owner(
    path: &Path,
    owner: u32,
) -> Result<(), PlanRefusal> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).map_err(|error| {
        return PlanRefusal::at(PlanRefusalCause::Unreadable, "file", error.to_string());
    })?;
    if !metadata.is_file() {
        return Err(PlanRefusal::at(
            PlanRefusalCause::NotARegularFile,
            "file",
            path.display().to_string(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != owner {
            return Err(PlanRefusal::at(
                PlanRefusalCause::WrongOwner,
                "file",
                metadata.uid().to_string(),
            ));
        }
    }
    #[cfg(not(unix))]
    drop(owner);
    return Ok(());
}

/// The account that owns the common Git directory.
fn common_directory_owner(common_dir: &Path) -> Result<u32, PlanRefusal> {
    let metadata: std::fs::Metadata = std::fs::metadata(common_dir).map_err(|error| {
        return PlanRefusal::at(
            PlanRefusalCause::Unreadable,
            "common directory",
            error.to_string(),
        );
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return Ok(metadata.uid());
    }
    #[cfg(not(unix))]
    {
        drop(metadata);
        return Ok(0);
    }
}

/// How many bytes the header region takes,
/// which is everything through the newline that ends the last declared field.
fn header_region_bytes(bytes: &[u8]) -> Result<usize, PlanRefusal> {
    let mut newlines: usize = 0;
    for (place, byte) in bytes.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        newlines += 1;
        if newlines == HEADER_LINE_COUNT {
            return Ok(place + 1);
        }
    }
    return Err(PlanRefusal::of(
        PlanRefusalCause::EndsEarly,
        format!("{newlines} of {HEADER_LINE_COUNT} header lines"),
    ));
}

/// What:
///  The plan standing for one clone,
///  or nothing when no plan file is there.
/// Why:
///  A repository with no plan folder and a clone with no plan are the healthy state,
///  and a caller that had to branch on `NotFound` itself would turn a permission error into an
///  absence sooner or later.
///  A plan that is there and does not read is a refusal,
///  never an absence:
///  it may hold the only record of a snapshot an earlier run destroyed.
///
/// # Throws
/// The plan whose bytes refuse,
/// and never an absent file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = readAnExistingPlan(commonDir, cloneGuid); // undefined when none stands
/// ```
pub fn read_an_existing_plan(
    common_dir: &Path,
    clone_guid: DatasetGuid,
) -> Result<Option<DestructionPlan>, PlanRefusal> {
    let folder: PathBuf = plans_folder(common_dir);
    let name: String = clone_guid.file_name();
    let path: PathBuf = folder.join(name.as_str());
    match std::fs::symlink_metadata(path.as_path()) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(PlanRefusal::at(
                PlanRefusalCause::Unreadable,
                "file",
                error.to_string(),
            ));
        }
        Ok(_) => {}
    }
    return read_the_plan(common_dir, clone_guid).map(Some);
}

/// What:
///  One encoded record split where its header region ends,
///  so a writer can hand the two regions to the same parse its reader calls.
/// Why:
///  The writer's read-back must be evidence about the bytes it staged and not about the value it
///  started from,
///  and the split is the one rule both halves of the reader agree on.
///
/// # Throws
/// The header region that never completes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [header, attempts] = splitHeader(bytes);
/// ```
pub fn split_the_header(bytes: &[u8]) -> Result<(&[u8], &[u8]), PlanRefusal> {
    let at: usize = header_region_bytes(bytes)?;
    return Ok((&bytes[..at], &bytes[at..]));
}

/// Whether one folder entry's name is a plan this reader reads,
/// which is the clone's canonical GUID and nothing else.
pub fn is_a_plan_entry(name: &str) -> bool {
    return is_canonical_guid_name(name);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_reading_tests.rs"]
mod tests;
