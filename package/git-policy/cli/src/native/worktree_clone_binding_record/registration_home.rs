//! What:
//!  Where the add's claim and binding records live,
//!  the four name forms that folder holds,
//!  and the plan folder the add reads to refuse.
//! Why:
//!  The record's home is the wrapper's own folder in the Git common directory,
//!  under one folder per family and a version the grammar compares
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The home" and "The route's record convention");
//!  the design's choice 1 of 2026-11-23 keeps that home.
//!  The name is keyed by the slot,
//!  the destination's last path component,
//!  because every reader knows the slot first,
//!  and the clone's GUID is a value and never part of a name.
//!  The folder is the account's,
//!  mode `0700`,
//!  and each record mode `0600`,
//!  because a record whose owner is another account is not this account's to act on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const home = join(commonDir, 'monochromatic-git-policy-cli', 'worktree-clones', 'v1');
//! const record = join(home, `${slot}.binding`);
//! ```

use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// The folder under the wrapper's own Git-directory folder that holds this record family.
pub const WORKTREE_CLONES_FOLDER: &str = "worktree-clones";

/// The grammar's version,
/// a segment of its own so a later grammar cannot be read as this one.
pub const GRAMMAR_VERSION_FOLDER: &str = "v1";

/// The suffix of the record itself.
pub const BINDING_SUFFIX: &str = ".binding";

/// The suffix of the origin-side inventory.
pub const ORIGIN_INVENTORY_SUFFIX: &str = ".origin-inventory";

/// The suffix of the clone's initial inventory.
pub const CLONE_INVENTORY_SUFFIX: &str = ".clone-inventory";

/// The generation file's prefix inside the folder,
/// joined to its number by a dot;
/// `take_claim` completes it with `<slot>.claim.<n>`.
pub const CLAIM_PREFIX_SEPARATOR: char = '.';

/// The claim generation's word in a generation file's name.
pub const CLAIM_WORD: &str = "claim";

/// The suffix of a record a killed writer left beside its record name.
pub const STAGING_SUFFIX: &str = ".partial";

/// The folder of the removal's destruction plans,
/// which the add reads to refuse while a plan names its clone.
///
/// Why:
///  The plan design's own home is under the wrapper's folder,
///  and step 10 builds only the read:
///  no plan can stand before step 14,
///  so the read finds nothing until then (the project's choice 18 and its "Build it in step 10").
pub const CLONE_DESTRUCTION_PLANS_FOLDER: &str = "clone-destruction-plans";

/// The bytes of a staging name's random part,
/// written as lowercase hexadecimal,
/// which is the same width the slot store's staging names hold.
const STAGING_RANDOM_BYTES: usize = 16;

/// The wrapper's own folder inside a Git common directory.
pub fn wrapper_folder(common_dir: &Path) -> PathBuf {
    return common_dir.join(DATA_FOLDER.current);
}

/// The folder this record family lives in.
pub fn home_folder(common_dir: &Path) -> PathBuf {
    return wrapper_folder(common_dir)
        .join(WORKTREE_CLONES_FOLDER)
        .join(GRAMMAR_VERSION_FOLDER);
}

/// The folder of the removal's destruction plans.
pub fn plans_folder(common_dir: &Path) -> PathBuf {
    return wrapper_folder(common_dir).join(CLONE_DESTRUCTION_PLANS_FOLDER);
}

/// What:
///  The slot one destination names:
///  its last path component,
///  refused when it is no plain folder name.
/// Why:
///  The record's key is the slot from its first byte to its last,
///  and the route fixes the destination at `.claude/worktrees/<name>`,
///  so one rule reads a path as a key.
///
/// # Errors
/// The path has no final component,
/// the component is not UTF-8,
/// or `slot_name_refusal` refuses it.
pub fn slot_of(destination: &Path) -> Result<String, String> {
    let name: OsString = destination
        .file_name()
        .ok_or_else(|| {
            return String::from(
                "the destination names no folder, so no slot name can key its record",
            );
        })?
        .to_os_string();
    let slot: String = name.into_string().map_err(|_| return String::from("the destination's last path component is not UTF-8, and the record's key is a name a person types"))?;
    if let Some(reason) = crate::worktree_clone_mount::slot_name_refusal(&slot) {
        return Err(format!("`{slot}` is not a plain slot name ({reason})"));
    }
    return Ok(slot);
}

/// The record's own name for one slot.
pub fn binding_name(slot: &str) -> String {
    return format!("{slot}{BINDING_SUFFIX}");
}

/// What:
///  The record's own file for one slot,
///  under one Git common directory.
/// Why:
///  The writer and every reader of one binding spell the same two names,
///  so a sentence that names the file a person must delete and the reader that
///  finds it cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bindingPath('/repo/.git', 'topic'); // '/repo/.git/monochromatic-git-policy-cli/worktree-clones/v1/topic.binding'
/// ```
pub fn binding_path(
    common_dir: &Path,
    slot: &str,
) -> PathBuf {
    return home_folder(common_dir).join(binding_name(slot));
}

/// The origin inventory's name for one slot.
pub fn origin_inventory_name(slot: &str) -> String {
    return format!("{slot}{ORIGIN_INVENTORY_SUFFIX}");
}

/// The clone inventory's name for one slot.
pub fn clone_inventory_name(slot: &str) -> String {
    return format!("{slot}{CLONE_INVENTORY_SUFFIX}");
}

/// The prefix `take_claim` completes with a generation number.
pub fn claim_prefix(slot: &str) -> OsString {
    return OsString::from(format!("{slot}{CLAIM_PREFIX_SEPARATOR}{CLAIM_WORD}"));
}

/// What:
///  Which record a file in the home folder is,
///  read from its name.
/// Why:
///  One listing serves `doctor` and the finder's ownership exclusion,
///  and a name that is none of the four forms is reported rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordName = { kind: 'binding' | 'originInventory' | 'cloneInventory'; slot: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordName {
    /// Which of the three record forms the name holds.
    pub kind: RecordKind,
    /// The slot the name is keyed by.
    pub slot: String,
}

/// Which of the family's record forms one name holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordKind {
    /// The binding itself.
    Binding,
    /// The origin-side inventory.
    OriginInventory,
    /// The clone's initial inventory.
    CloneInventory,
}

/// The record one name holds,
/// or nothing when the name is none of the bearer forms.
pub fn record_name_of(name: &str) -> Option<RecordName> {
    let forms: [(&str, RecordKind); 3] = [
        (BINDING_SUFFIX, RecordKind::Binding),
        (ORIGIN_INVENTORY_SUFFIX, RecordKind::OriginInventory),
        (CLONE_INVENTORY_SUFFIX, RecordKind::CloneInventory),
    ];
    for (suffix, kind) in forms {
        if let Some(slot) = name.strip_suffix(suffix) {
            if slot.is_empty() {
                return None;
            }
            return Some(RecordName {
                kind,
                slot: String::from(slot),
            });
        }
    }
    return None;
}

/// Whether one name is a staging name this family's writers leave.
///
/// Why:
///  `doctor` removes the known forms only when no live claim stands,
///  so it must tell them from every other entry.
pub fn is_known_staging_name(name: &str) -> bool {
    return is_record_staging_name(name) || is_claim_staging_name(name);
}

/// What:
///  Whether the record's home holds a record this family writes,
///  or a record a writer is still putting there under its staging name.
/// Why:
///  The unknown-global refusal protects a slot that has a record,
///  so a folder the route made on an earlier command and holds nothing
///  endangers no slot,
///  and `SPEC.md` says the refusal applies "while the repository holds slot
///  records".
///  A record being written counts:
///  its staging name is the same home's record mid-write,
/// and the refusal's
///  readers look in that home.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdsARecord(commonDir: string): boolean;
/// ```
pub fn holds_a_record(common_dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(home_folder(common_dir)) else {
        return false;
    };
    for listed in entries.flatten() {
        let file_name: OsString = listed.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        if record_name_of(name).is_some() || is_known_staging_name(name) {
            return true;
        }
    }
    return false;
}

/// Whether one name is `take_claim`\x27s private temporary file,
/// `.cli-git-claim.tmp.<pid>.<nanos>.<counter>`,
/// which a kill between its bytes and its link leaves.
pub fn is_claim_staging_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix(".cli-git-claim.tmp.") else {
        return false;
    };
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    return parts.iter().all(|part| {
        return !part.is_empty()
            && part.len() <= 20
            && part.bytes().all(|byte| return byte.is_ascii_digit());
    });
}

/// Whether one name is a record's `.partial` staging name.
fn is_record_staging_name(name: &str) -> bool {
    let Some(body) = name.strip_prefix('.') else {
        return false;
    };
    let Some(stem) = body.strip_suffix(STAGING_SUFFIX) else {
        return false;
    };
    let Some((record, random)) = stem.rsplit_once('.') else {
        return false;
    };
    if record.is_empty() || random.len() != STAGING_RANDOM_BYTES * 2 {
        return false;
    }
    return random
        .bytes()
        .all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
}

/// The staging name one record write fills before it takes the record's name.
pub fn record_staging_name(
    record_name: &str,
    random: &[u8; STAGING_RANDOM_BYTES],
) -> String {
    let random_text: String = random
        .iter()
        .map(|byte| return format!("{byte:02x}"))
        .collect();
    return format!(".{record_name}.{random_text}{STAGING_SUFFIX}");
}

/// The claim generation one name holds for `slot`,
/// or nothing when the name is another slot's or another form.
pub fn claim_generation_of(
    name: &str,
    slot: &str,
) -> Option<u64> {
    let prefix: String = format!("{slot}{CLAIM_PREFIX_SEPARATOR}{CLAIM_WORD}.");
    let number: &str = name.strip_prefix(prefix.as_str())?;
    if number.is_empty()
        || number.len() > 20
        || !number.bytes().all(|byte| return byte.is_ascii_digit())
    {
        return None;
    }
    let generation: u64 = number.parse().ok()?;
    return if generation == 0 {
        None
    } else {
        Some(generation)
    };
}

/// What:
///  Open the record's home folder,
///  making the wrapper's folder,
///  the family folder and the version folder with mode `0700` when they are absent.
/// Why:
///  The first command that needs the folder makes it,
///  and the two parents are the wrapper's own,
///  so one call gives every reader a descriptor to work from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(home, { recursive: true, mode: 0o700 }); const folder = open(home, O_DIRECTORY);
/// ```
///
/// # Errors
/// A parent cannot be made or the home folder cannot be opened.
pub fn open_or_create_home(common_dir: &Path) -> std::io::Result<std::fs::File> {
    let home: PathBuf = home_folder(common_dir);
    crate::private_storage::create_private_directory_all(&home)?;
    return std::fs::File::open(home);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_home_tests.rs"]
mod tests;
