//! What:
//!  The one name a removal's staging folder,
//! its record and its claim carry,
//!  all derived from the clone's canonical numeric dataset GUID.
//! Why:
//!  No request carries a path,
//!  so both the wrapper and the root handler derive the folder's component from
//!  data both already hold
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging target and who creates it";
//!  `doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision").
//!  One clone has one component,
//!  so two removals of one clone cannot share a folder and two clones cannot
//!  collide.
//!  The record's `.json` suffix and the claim's `.staged` suffix name shapes no
//!  other reader of the wrapper picks up,
//!  and the temporary name beside the record is the only stray an interrupted
//!  write leaves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = stagingFolderName(guid);      // 'monochromatic-git-policy-cli-staging-<guid>'
//! const record = stagingRecordName(guid);      // `${folder}.json`
//! const claim = stagingClaimName(guid);        // `<guid>.staged`
//! ```

use crate::worktree_clone_mount_protocol::DatasetGuid;

/// The component every staging folder begins with.
pub const STAGING_FOLDER_PREFIX: &str = "monochromatic-git-policy-cli-staging-";

/// The suffix of the record file beside a folder.
pub const RECORD_SUFFIX: &str = ".json";

/// The suffix of the temporary name a record is written under.
pub const TEMPORARY_SUFFIX: &str = ".json.tmp";

/// The suffix of a staging claim.
pub const CLAIM_SUFFIX: &str = ".staged";

/// What:
///  Why a name is not one this route derives.
/// Why:
///  A name read from the account's folder or from a claim is untrusted,
///  and every reader compares it with the component it derived rather than
///  resolving whatever the name says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingNameRefusal = { kind: 'prefix' | 'suffix' | 'guid'; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StagingNameRefusal {
    /// The name does not begin with the staging prefix.
    Prefix {
        /// The name as read.
        name: String,
    },
    /// The name carries a suffix this shape does not have.
    Suffix {
        /// The name as read.
        name: String,
    },
    /// The component between the prefix and the suffix is not a dataset GUID.
    Guid {
        /// The component as read.
        component: String,
        /// Why the GUID rule refused it.
        said: String,
    },
}

impl StagingNameRefusal {
    /// The refusal in words.
    pub fn message(&self) -> String {
        return match self {
            StagingNameRefusal::Prefix { name } => format!(
                "`{name}` does not begin with `{STAGING_FOLDER_PREFIX}`, so it is not a staging name this route derives. Nothing was read, bound or removed under it."
            ),
            StagingNameRefusal::Suffix { name } => format!(
                "`{name}` does not carry the suffix this shape has (`{RECORD_SUFFIX}`, `{TEMPORARY_SUFFIX}` or `{CLAIM_SUFFIX}`). Nothing was read, bound or removed under it."
            ),
            StagingNameRefusal::Guid { component, said } => format!(
                "`{component}` is not the canonical numeric dataset GUID this route derives: {said}. Nothing was read, bound or removed under it."
            ),
        };
    }
}

/// The folder's component for one clone.
pub fn staging_folder_name(guid: DatasetGuid) -> String {
    return format!("{STAGING_FOLDER_PREFIX}{}", guid.file_name());
}

/// The record's name beside the folder.
pub fn staging_record_name(guid: DatasetGuid) -> String {
    return format!("{}{RECORD_SUFFIX}", staging_folder_name(guid));
}

/// The temporary name the record is written under before its rename.
pub fn staging_temporary_name(guid: DatasetGuid) -> String {
    return format!("{}{TEMPORARY_SUFFIX}", staging_folder_name(guid));
}

/// The claim's name in the repository's staging folder.
pub fn staging_claim_name(guid: DatasetGuid) -> String {
    return format!("{}{CLAIM_SUFFIX}", guid.file_name());
}

/// What:
///  The GUID one whole staging component names,
///  when the name begins with the prefix and holds nothing else.
/// Why:
///  The wrapper lists `~/.cache` by descriptor and must tell a folder it made
///  from a record and from anything else,
///  and `doctor` prints an entry it cannot match rather than removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guid = guidOfFolderName('monochromatic-git-policy-cli-staging-123');
/// ```
pub fn guid_of_folder_name(name: &str) -> Result<DatasetGuid, StagingNameRefusal> {
    let Some(component) = name.strip_prefix(STAGING_FOLDER_PREFIX) else {
        return Err(StagingNameRefusal::Prefix { name: String::from(name) });
    };
    if component.ends_with(RECORD_SUFFIX) || component.ends_with(CLAIM_SUFFIX) {
        return Err(StagingNameRefusal::Suffix { name: String::from(name) });
    }
    return DatasetGuid::parse(component).map_err(|refusal| return StagingNameRefusal::Guid { component: String::from(component), said: refusal.message() });
}

/// What:
///  The GUID a record's name describes,
///  when the whole name is one record name.
/// Why:
///  A reader lists records and folders apart by suffix,
///  and a name with both suffixes is neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guid = guidOfRecordName('monochromatic-git-policy-cli-staging-123.json');
/// ```
pub fn guid_of_record_name(name: &str) -> Result<DatasetGuid, StagingNameRefusal> {
    let Some(component) = name.strip_suffix(RECORD_SUFFIX) else {
        return Err(StagingNameRefusal::Suffix { name: String::from(name) });
    };
    return guid_of_folder_name(component);
}

/// What:
///  The GUID a claim's name describes,
///  when the whole name is one claim name.
/// Why:
///  A claim is read only for the repository a removal holds,
///  and its name is the GUID its folder's component carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guid = guidOfClaimName('123.staged');
/// ```
pub fn guid_of_claim_name(name: &str) -> Result<DatasetGuid, StagingNameRefusal> {
    let Some(component) = name.strip_suffix(CLAIM_SUFFIX) else {
        return Err(StagingNameRefusal::Suffix { name: String::from(name) });
    };
    if component.starts_with(STAGING_FOLDER_PREFIX) {
        return Err(StagingNameRefusal::Prefix { name: String::from(name) });
    }
    return DatasetGuid::parse(component).map_err(|refusal| return StagingNameRefusal::Guid { component: String::from(component), said: refusal.message() });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staging_folder_name_tests.rs"]
mod tests;
